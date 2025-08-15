use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[clap(
    name = "frpc",
    version = env!("CARGO_PKG_VERSION"),
    about = "Enterprise-grade CLI tool for cleaning Flutter and Rust projects",
    long_about = None
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
    
    #[clap(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
    
    #[clap(long, global = true)]
    pub quiet: bool,
    
    #[clap(long, global = true)]
    pub no_color: bool,
    
    #[clap(long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(about = "Clean Flutter and Rust projects in a directory")]
    Clean {
        #[clap(help = "Target directory to clean projects in")]
        path: PathBuf,
        
        #[clap(long, short = 'n', help = "Show what would be cleaned without doing it")]
        dry_run: bool,
        
        #[clap(long, short = 't', help = "Types of projects to clean")]
        project_type: Vec<ProjectTypeFilter>,
        
        #[clap(long, short = 'j', help = "Number of parallel jobs")]
        jobs: Option<usize>,
        
        #[clap(long, help = "Clean even if directory is excluded")]
        force: bool,
        
        #[clap(long, help = "Exclude directories matching pattern")]
        exclude: Vec<String>,
        
        #[clap(long, help = "Include only directories matching pattern")]
        include: Vec<String>,
        
        #[clap(long, help = "Interactive mode")]
        interactive: bool,
        
        #[clap(long, help = "Maximum depth to search")]
        max_depth: Option<usize>,
        
        #[clap(long, help = "Output results as JSON")]
        json: bool,
    },
    
    #[clap(about = "List projects without cleaning them")]
    List {
        #[clap(help = "Target directory to scan")]
        path: PathBuf,
        
        #[clap(long, short = 't', help = "Filter by project type")]
        project_type: Vec<ProjectTypeFilter>,
        
        #[clap(long, help = "Output as JSON")]
        json: bool,
    },
    
    #[clap(about = "Manage configuration")]
    Config {
        #[clap(subcommand)]
        command: ConfigCommands,
    },
    
    #[clap(name = "completions", about = "Generate shell completions")]
    GenerateCompletions {
        #[clap(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    #[clap(about = "Initialize configuration file with defaults")]
    Init,
    
    #[clap(about = "Show current configuration")]
    Show,
    
    #[clap(about = "Edit configuration file")]
    Edit,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ProjectTypeFilter {
    Flutter,
    Rust,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_exclude: Vec<String>,
    pub flutter_clean_args: Vec<String>,
    pub cargo_clean_args: Vec<String>,
    pub max_parallel_jobs: Option<usize>,
    pub interactive_by_default: bool,
    pub show_progress: bool,
    pub confirm_before_clean: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_exclude: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
                "build".to_string(),
            ],
            flutter_clean_args: vec!["clean".to_string()],
            cargo_clean_args: vec!["clean".to_string()],
            max_parallel_jobs: None,
            interactive_by_default: false,
            show_progress: true,
            confirm_before_clean: true,
        }
    }
}

impl Config {
    pub fn load(path: Option<&Path>) -> crate::error::Result<Self> {
        let config_path = match path {
            Some(p) => p.to_path_buf(),
            None => Self::default_config_path()?,
        };
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
    
    pub fn save(&self, path: Option<&Path>) -> crate::error::Result<()> {
        let config_path = match path {
            Some(p) => p.to_path_buf(),
            None => Self::default_config_path()?,
        };
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
    
    fn default_config_path() -> crate::error::Result<PathBuf> {
        let home_dir = home::home_dir()
            .ok_or_else(|| crate::error::CleanerError::ConfigError("Cannot find home directory".to_string()))?;
        Ok(home_dir.join(".config").join("frpc").join("config.toml"))
    }
}