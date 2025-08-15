use super::*;
use crate::error::Result;
use std::fs;

pub struct MetadataCollector;

impl MetadataCollector {
    pub fn collect(project: &mut Project) -> Result<()> {
        let path = project.path.clone();
        
        if let Ok(metadata) = fs::metadata(&path) {
            project.metadata.last_modified = metadata.modified().ok();
        }
        
        match project.project_type {
            ProjectType::Flutter => Self::collect_flutter_metadata(project)?,
            ProjectType::Rust => Self::collect_rust_metadata(project)?,
            ProjectType::Mixed => {
                Self::collect_flutter_metadata(project)?;
                Self::collect_rust_metadata(project)?;
            }
        }
        
        project.metadata.estimated_size = Self::estimate_size(&path).ok();
        Ok(())
    }
    
    fn collect_flutter_metadata(project: &mut Project) -> Result<()> {
        let pubspec_path = project.path.join("pubspec.yaml");
        if let Ok(content) = fs::read_to_string(&pubspec_path) {
            if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                project.metadata.name = yaml.get("name")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                project.metadata.version = yaml.get("version")
                    .and_then(|v| v.as_str())
                    .map(String::from);
            }
        }
        Ok(())
    }
    
    fn collect_rust_metadata(project: &mut Project) -> Result<()> {
        let cargo_path = project.path.join("Cargo.toml");
        if let Ok(content) = fs::read_to_string(&cargo_path) {
            if let Ok(toml) = toml::from_str::<toml::Value>(&content) {
                if let Some(package) = toml.get("package") {
                    if project.metadata.name.is_none() {
                        project.metadata.name = package.get("name")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                    }
                    if project.metadata.version.is_none() {
                        project.metadata.version = package.get("version")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                    }
                }
            }
        }
        Ok(())
    }
    
    fn estimate_size(path: &Path) -> Result<u64> {
        let mut total_size = 0;
        for entry in walkdir::WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
        Ok(total_size)
    }
}