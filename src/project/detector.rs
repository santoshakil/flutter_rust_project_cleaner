use super::*;
use crate::error::Result;
use std::path::Path;

pub struct ProjectDetector;

impl ProjectDetector {
    pub fn detect(path: &Path) -> Result<Option<ProjectType>> {
        let has_pubspec = path.join("pubspec.yaml").exists();
        let has_cargo = path.join("Cargo.toml").exists();
        
        match (has_pubspec, has_cargo) {
            (true, true) => Ok(Some(ProjectType::Mixed)),
            (true, false) => Ok(Some(ProjectType::Flutter)),
            (false, true) => Ok(Some(ProjectType::Rust)),
            (false, false) => Ok(None),
        }
    }
    
    pub fn is_project_root(path: &Path) -> bool {
        path.join("pubspec.yaml").exists() || path.join("Cargo.toml").exists()
    }
}