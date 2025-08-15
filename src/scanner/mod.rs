use crate::error::Result;
use crate::project::{Project, ProjectType, detector::ProjectDetector, metadata::MetadataCollector};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use walkdir::WalkDir;

pub struct Scanner {
    max_depth: Option<usize>,
    exclude_patterns: Vec<String>,
    include_patterns: Vec<String>,
    project_type_filter: Vec<ProjectType>,
    show_progress: bool,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            max_depth: None,
            exclude_patterns: Vec::new(),
            include_patterns: Vec::new(),
            project_type_filter: Vec::new(),
            show_progress: true,
        }
    }
    
    pub fn with_max_depth(mut self, depth: Option<usize>) -> Self {
        self.max_depth = depth;
        self
    }
    
    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }
    
    pub fn with_include_patterns(mut self, patterns: Vec<String>) -> Self {
        self.include_patterns = patterns;
        self
    }
    
    pub fn with_project_type_filter(mut self, types: Vec<ProjectType>) -> Self {
        self.project_type_filter = types;
        self
    }
    
    pub fn with_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }
    
    pub fn scan(&self, root_path: &Path) -> Result<Vec<Project>> {
        self.scan_with_interrupt(root_path, Arc::new(AtomicBool::new(false)))
    }
    
    pub fn scan_with_interrupt(&self, root_path: &Path, interrupted: Arc<AtomicBool>) -> Result<Vec<Project>> {
        if !root_path.exists() {
            return Err(crate::error::CleanerError::PathAccess {
                path: root_path.to_path_buf(),
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "Path does not exist"),
            });
        }
        
        if !root_path.is_dir() {
            return Err(crate::error::CleanerError::PathAccess {
                path: root_path.to_path_buf(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "Path is not a directory"),
            });
        }
        
        let walker = WalkDir::new(root_path)
            .follow_links(false)
            .max_depth(self.max_depth.unwrap_or(usize::MAX));
            
        let progress = if self.show_progress {
            let bar = ProgressBar::new_spinner();
            bar.set_style(ProgressStyle::default_spinner()
                .template("[{elapsed_precise}] {spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]));
            bar.set_message("Scanning for projects...");
            Some(bar)
        } else {
            None
        };
        
        let mut project_paths = Vec::new();
        let mut scanned_count = 0;
        
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if interrupted.load(Ordering::SeqCst) {
                if let Some(bar) = &progress {
                    bar.finish_with_message("Scanning interrupted");
                }
                return Err(crate::error::CleanerError::Interrupted);
            }
            
            let path = entry.path();
            
            if self.should_exclude(path) {
                continue;
            }
            
            scanned_count += 1;
            if let Some(ref bar) = progress {
                if scanned_count % 100 == 0 {
                    bar.set_message(format!("Scanned {} directories...", scanned_count));
                }
            }
            
            if ProjectDetector::is_project_root(path) {
                if let Some(project_type) = ProjectDetector::detect(path)? {
                    if self.matches_filter(project_type) {
                        project_paths.push(path.to_path_buf());
                    }
                }
            }
        }
        
        if let Some(bar) = progress {
            bar.finish_with_message(format!("Found {} projects", project_paths.len()));
        }
        
        let projects: Vec<Project> = project_paths
            .par_iter()
            .filter_map(|path| self.create_project(path).ok())
            .collect();
            
        Ok(projects)
    }
    
    fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        if !self.include_patterns.is_empty() {
            let included = self.include_patterns.iter()
                .any(|pattern| glob_match::glob_match(pattern, &path_str));
            if !included {
                return true;
            }
        }
        
        self.exclude_patterns.iter()
            .any(|pattern| glob_match::glob_match(pattern, &path_str))
    }
    
    fn matches_filter(&self, project_type: ProjectType) -> bool {
        self.project_type_filter.is_empty() || 
        self.project_type_filter.contains(&project_type)
    }
    
    fn create_project(&self, path: &PathBuf) -> Result<Project> {
        let project_type = ProjectDetector::detect(path)?
            .ok_or_else(|| crate::error::CleanerError::ConfigError("Not a project".to_string()))?;
            
        let mut project = Project::new(path.clone(), project_type);
        MetadataCollector::collect(&mut project)?;
        Ok(project)
    }
}