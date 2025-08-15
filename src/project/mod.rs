use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

pub mod detector;
pub mod metadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    Flutter,
    Rust,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub path: PathBuf,
    pub project_type: ProjectType,
    pub metadata: ProjectMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: Option<String>,
    pub version: Option<String>,
    pub last_modified: Option<std::time::SystemTime>,
    pub estimated_size: Option<u64>,
}

impl Project {
    pub fn new(path: PathBuf, project_type: ProjectType) -> Self {
        Self {
            path,
            project_type,
            metadata: ProjectMetadata::default(),
        }
    }
    
    pub fn name(&self) -> String {
        self.metadata.name.clone().unwrap_or_else(|| {
            self.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        })
    }
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            name: None,
            version: None,
            last_modified: None,
            estimated_size: None,
        }
    }
}