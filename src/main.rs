mod config;
mod error;
mod project;
mod scanner;
mod cleaner;
mod utils;

use clap::{CommandFactory, Parser};
use colored::Colorize;
use error::Result;
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::signal;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}: {}", "Error".red().bold(), e);
        process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = config::Cli::parse();
    
    utils::logging::init_logging(cli.verbose, cli.quiet, cli.no_color);
    
    let config = config::Config::load(cli.config.as_deref())?;
    
    // Set up signal handler
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = interrupted.clone();
    
    tokio::spawn(async move {
        if signal::ctrl_c().await.is_ok() {
            println!("\n{}", "Interrupted! Stopping gracefully...".yellow());
            interrupted_clone.store(true, Ordering::SeqCst);
        }
    });
    
    match cli.command {
        config::Commands::Clean { 
            path, 
            dry_run, 
            project_type, 
            jobs, 
            force: _, 
            exclude, 
            include, 
            interactive,
            max_depth,
            json,
        } => {
            let project_types: Vec<_> = project_type.into_iter()
                .map(|t| match t {
                    config::ProjectTypeFilter::Flutter => project::ProjectType::Flutter,
                    config::ProjectTypeFilter::Rust => project::ProjectType::Rust,
                    config::ProjectTypeFilter::Mixed => project::ProjectType::Mixed,
                })
                .collect();
                
            let scanner = scanner::Scanner::new()
                .with_max_depth(max_depth)
                .with_exclude_patterns(exclude)
                .with_include_patterns(include)
                .with_project_type_filter(project_types)
                .with_progress(config.show_progress && !cli.quiet);
                
            println!("Scanning directory: {}", path.display().to_string().cyan());
            let mut projects = scanner.scan_with_interrupt(&path, interrupted.clone())?;
            
            if projects.is_empty() {
                println!("{}", "No projects found to clean.".yellow());
                return Ok(());
            }
            
            println!("Found {} projects", projects.len().to_string().green());
            
            if interactive || config.interactive_by_default {
                projects = utils::interactive::project_selection(projects)?;
                if projects.is_empty() {
                    println!("{}", "No projects selected.".yellow());
                    return Ok(());
                }
            }
            
            let total_size: u64 = projects.iter()
                .filter_map(|p| p.metadata.estimated_size)
                .sum();
                
            if config.confirm_before_clean && !dry_run {
                if !utils::interactive::confirm_clean(&projects, total_size)? {
                    println!("{}", "Cleaning cancelled.".yellow());
                    return Ok(());
                }
            }
            
            let cleaner = cleaner::Cleaner::new(
                dry_run,
                config.flutter_clean_args.clone(),
                config.cargo_clean_args.clone(),
                jobs.or(config.max_parallel_jobs),
            ).with_progress(config.show_progress && !cli.quiet);
            
            let results = cleaner.clean_projects(projects).await;
            
            if json {
                // Output JSON format
                #[derive(serde::Serialize)]
                struct JsonOutput {
                    success: bool,
                    total_projects: usize,
                    successful: usize,
                    failed: usize,
                    space_freed: u64,
                    results: Vec<JsonResult>,
                }
                
                #[derive(serde::Serialize)]
                struct JsonResult {
                    path: String,
                    name: String,
                    project_type: String,
                    success: bool,
                    space_freed: Option<u64>,
                    error: Option<String>,
                }
                
                let successful = results.iter().filter(|r| r.success).count();
                let failed = results.len() - successful;
                let total_space_freed: u64 = results.iter()
                    .filter_map(|r| r.space_freed)
                    .sum();
                
                let json_results: Vec<JsonResult> = results.iter()
                    .map(|r| JsonResult {
                        path: r.project.path.display().to_string(),
                        name: r.project.name(),
                        project_type: format!("{:?}", r.project.project_type),
                        success: r.success,
                        space_freed: r.space_freed,
                        error: r.error.as_ref().map(|e| e.to_string()),
                    })
                    .collect();
                
                let output = JsonOutput {
                    success: failed == 0,
                    total_projects: results.len(),
                    successful,
                    failed,
                    space_freed: total_space_freed,
                    results: json_results,
                };
                
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                // Display detailed results in normal mode
                let mut successful = 0;
                let mut failed = 0;
                let mut total_space_freed = 0u64;
                
                for result in &results {
                    if result.success {
                        successful += 1;
                        if let Some(space) = result.space_freed {
                            total_space_freed += space;
                        }
                        if cli.verbose > 0 {
                            println!("{} {} - freed {}", 
                                "✓".green(),
                                result.project.name().cyan(),
                                humansize::format_size(result.space_freed.unwrap_or(0), humansize::BINARY).green()
                            );
                        }
                    } else {
                        failed += 1;
                        if let Some(ref error) = result.error {
                            eprintln!("{} {} - {}", 
                                "✗".red(),
                                result.project.name().yellow(),
                                error.to_string().red()
                            );
                        }
                    }
                }
                    
                println!("\n{}", "Cleaning complete!".green().bold());
                println!("  Successful: {}", successful.to_string().green());
                if failed > 0 {
                    println!("  Failed: {}", failed.to_string().red());
                    
                    // Show failed projects in verbose mode
                    if cli.verbose > 0 {
                        println!("\nFailed projects:");
                        for result in &results {
                            if !result.success {
                                println!("  {} - {}", 
                                    result.project.path.display().to_string().yellow(),
                                    result.error.as_ref().map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string())
                                );
                            }
                        }
                    }
                }
                println!("  Space freed: {}", 
                    humansize::format_size(total_space_freed, humansize::BINARY).cyan());
            }
        }
        
        config::Commands::List { path, project_type, json } => {
            let project_types: Vec<_> = project_type.into_iter()
                .map(|t| match t {
                    config::ProjectTypeFilter::Flutter => project::ProjectType::Flutter,
                    config::ProjectTypeFilter::Rust => project::ProjectType::Rust,
                    config::ProjectTypeFilter::Mixed => project::ProjectType::Mixed,
                })
                .collect();
                
            let scanner = scanner::Scanner::new()
                .with_project_type_filter(project_types)
                .with_progress(!json);
                
            let projects = scanner.scan(&path)?;
            
            if json {
                println!("{}", serde_json::to_string_pretty(&projects)?);
            } else {
                for project in &projects {
                    println!("{} [{}] - {}", 
                        project.name().cyan(),
                        format!("{:?}", project.project_type).yellow(),
                        project.path.display()
                    );
                }
                println!("\nTotal: {} projects", projects.len().to_string().green());
            }
        }
        
        config::Commands::Config { command } => {
            match command {
                config::ConfigCommands::Init => {
                    let default_config = config::Config::default();
                    default_config.save(None)?;
                    println!("{}", "Configuration initialized!".green());
                }
                config::ConfigCommands::Show => {
                    println!("{}", toml::to_string_pretty(&config)?);
                }
                config::ConfigCommands::Edit => {
                    let config_path = home::home_dir()
                        .ok_or_else(|| error::CleanerError::ConfigError("Cannot find home directory".to_string()))?
                        .join(".config").join("frpc").join("config.toml");
                    println!("Opening config file: {}", config_path.display());
                    
                    #[cfg(target_os = "macos")]
                    let cmd = "open";
                    #[cfg(target_os = "windows")]
                    let cmd = "start";
                    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                    let cmd = "xdg-open";
                    
                    std::process::Command::new(cmd)
                        .arg(&config_path)
                        .status()
                        .map_err(|e| error::CleanerError::OpenError(
                            format!("Failed to open config file with {}: {}", cmd, e)
                        ))?;
                }
            }
        }
        
        config::Commands::GenerateCompletions { shell } => {
            clap_complete::generate(
                shell,
                &mut config::Cli::command(),
                "frpc",
                &mut std::io::stdout()
            );
        }
    }
    
    Ok(())
}