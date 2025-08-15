use crate::error::{CleanerError, Result};
use crate::project::{Project, ProjectType};
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::process::Command;

pub struct Cleaner {
    dry_run: bool,
    flutter_args: Vec<String>,
    cargo_args: Vec<String>,
    parallelism: usize,
    progress: bool,
}

#[derive(Debug)]
pub struct CleanResult {
    pub project: Project,
    pub success: bool,
    pub error: Option<CleanerError>,
    pub space_freed: Option<u64>,
}

impl Cleaner {
    pub fn new(
        dry_run: bool,
        flutter_args: Vec<String>,
        cargo_args: Vec<String>,
        parallelism: Option<usize>,
    ) -> Self {
        let parallelism = parallelism
            .unwrap_or_else(|| num_cpus::get())
            .max(1);
            
        Self {
            dry_run,
            flutter_args,
            cargo_args,
            parallelism,
            progress: true,
        }
    }
    
    pub fn with_progress(mut self, progress: bool) -> Self {
        self.progress = progress;
        self
    }
    
    pub async fn clean_projects(&self, projects: Vec<Project>) -> Vec<CleanResult> {
        let multi_progress = if self.progress {
            Some(MultiProgress::new())
        } else {
            None
        };
        
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.parallelism)
            .build()
            .unwrap();
            
        let results: Vec<CleanResult> = pool.install(|| {
            projects
                .into_par_iter()
                .map(|project| {
                let progress_bar = if let Some(ref mp) = multi_progress {
                    let pb = mp.add(ProgressBar::new_spinner());
                    pb.set_style(ProgressStyle::default_spinner()
                        .template("[{elapsed_precise}] {spinner:.cyan} {msg}")
                        .unwrap());
                    pb.set_message(format!("Cleaning {}...", project.name()));
                    Some(pb)
                } else {
                    None
                };
                
                let result = self.clean_project(&project);
                
                if let Some(pb) = progress_bar {
                    match &result {
                        Ok(res) if res.success => {
                            pb.finish_with_message(format!("{} {}", 
                                "✓".green(),
                                project.name()
                            ));
                        }
                        Ok(_) | Err(_) => {
                            pb.finish_with_message(format!("{} {}", 
                                "✗".red(),
                                project.name()
                            ));
                        }
                    }
                }
                
                result
            })
            .filter_map(|r| r.ok())
            .collect()
        });
            
        results
    }
    
    fn clean_project(&self, project: &Project) -> Result<CleanResult> {
        let initial_size = self.estimate_cleanable_size(project)?;
        
        if self.dry_run {
            println!("{} {} would free ~{}", 
                "[DRY RUN]".yellow(),
                project.name().cyan(),
                humansize::format_size(initial_size, humansize::BINARY)
            );
            return Ok(CleanResult {
                project: project.clone(),
                success: true,
                error: None,
                space_freed: Some(initial_size),
            });
        }
        
        let result = match project.project_type {
            ProjectType::Flutter => {
                match self.clean_flutter(project) {
                    Ok(_) => Ok(CleanResult {
                        project: project.clone(),
                        success: true,
                        error: None,
                        space_freed: Some(initial_size),
                    }),
                    Err(e) => Ok(CleanResult {
                        project: project.clone(),
                        success: false,
                        error: Some(e),
                        space_freed: None,
                    }),
                }
            },
            ProjectType::Rust => {
                match self.clean_rust(project) {
                    Ok(_) => Ok(CleanResult {
                        project: project.clone(),
                        success: true,
                        error: None,
                        space_freed: Some(initial_size),
                    }),
                    Err(e) => Ok(CleanResult {
                        project: project.clone(),
                        success: false,
                        error: Some(e),
                        space_freed: None,
                    }),
                }
            },
            ProjectType::Mixed => {
                let flutter_result = self.clean_flutter(project);
                let rust_result = self.clean_rust(project);
                
                match (flutter_result, rust_result) {
                    (Ok(_), Ok(_)) => Ok(CleanResult {
                        project: project.clone(),
                        success: true,
                        error: None,
                        space_freed: Some(initial_size),
                    }),
                    (Err(e), _) | (_, Err(e)) => Ok(CleanResult {
                        project: project.clone(),
                        success: false,
                        error: Some(e),
                        space_freed: None,
                    }),
                }
            }
        };
        
        result
    }
    
    fn clean_flutter(&self, project: &Project) -> Result<()> {
        self.check_command_available("flutter")?;
        self.check_permissions(&project.path)?;
        
        let mut cmd = Command::new("flutter");
        cmd.current_dir(&project.path);
        for arg in &self.flutter_args {
            cmd.arg(arg);
        }
        
        let output = cmd.output()
            .map_err(|e| CleanerError::CommandExecution {
                command: "flutter clean".to_string(),
                source: e,
            })?;
        
        if !output.status.success() {
            return Err(CleanerError::CommandFailed {
                command: "flutter clean".to_string(),
                code: output.status.code().unwrap_or(-1),
            });
        }
        
        Ok(())
    }
    
    fn clean_rust(&self, project: &Project) -> Result<()> {
        self.check_command_available("cargo")?;
        self.check_permissions(&project.path)?;
        
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&project.path);
        for arg in &self.cargo_args {
            cmd.arg(arg);
        }
        
        let output = cmd.output()
            .map_err(|e| CleanerError::CommandExecution {
                command: "cargo clean".to_string(),
                source: e,
            })?;
        
        if !output.status.success() {
            return Err(CleanerError::CommandFailed {
                command: "cargo clean".to_string(),
                code: output.status.code().unwrap_or(-1),
            });
        }
        
        Ok(())
    }
    
    fn check_command_available(&self, command: &str) -> Result<()> {
        which::which(command).map_err(|_| match command {
            "flutter" => CleanerError::FlutterNotFound,
            "cargo" => CleanerError::CargoNotFound,
            _ => CleanerError::ConfigError(format!("Command {} not found", command)),
        })?;
        Ok(())
    }
    
    fn check_permissions(&self, path: &std::path::Path) -> Result<()> {
        use std::fs;
        
        // Try to read directory contents to check permissions
        match fs::read_dir(path) {
            Ok(_) => Ok(()),
            Err(e) => match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    Err(CleanerError::PermissionDenied {
                        path: path.to_path_buf(),
                    })
                }
                _ => Err(CleanerError::PathAccess {
                    path: path.to_path_buf(),
                    source: e,
                }),
            }
        }
    }
    
    fn estimate_cleanable_size(&self, project: &Project) -> Result<u64> {
        let mut size = 0u64;
        
        match project.project_type {
            ProjectType::Flutter | ProjectType::Mixed => {
                for dir in &[".dart_tool", "build", ".flutter-plugins-dependencies"] {
                    size += self.dir_size(&project.path.join(dir))?;
                }
            }
            _ => {}
        }
        
        match project.project_type {
            ProjectType::Rust | ProjectType::Mixed => {
                size += self.dir_size(&project.path.join("target"))?;
            }
            _ => {}
        }
        
        Ok(size)
    }
    
    fn dir_size(&self, path: &std::path::Path) -> Result<u64> {
        if !path.exists() {
            return Ok(0);
        }
        
        let mut size = 0;
        for entry in walkdir::WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if let Ok(metadata) = entry.metadata() {
                size += metadata.len();
            }
        }
        Ok(size)
    }
}