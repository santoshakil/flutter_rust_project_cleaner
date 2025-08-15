use flutter_rust_project_cleaner::{
    project::{Project, ProjectType, detector::ProjectDetector},
    scanner::Scanner,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_project_detection() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    assert_eq!(ProjectDetector::detect(path).unwrap(), None);

    fs::write(path.join("pubspec.yaml"), "name: test").unwrap();
    assert_eq!(ProjectDetector::detect(path).unwrap(), Some(ProjectType::Flutter));

    fs::write(path.join("Cargo.toml"), "[package]").unwrap();
    assert_eq!(ProjectDetector::detect(path).unwrap(), Some(ProjectType::Mixed));
}

#[test]
fn test_scanner_finds_projects() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let flutter_dir = root.join("flutter_project");
    fs::create_dir(&flutter_dir).unwrap();
    fs::write(flutter_dir.join("pubspec.yaml"), "name: flutter_test").unwrap();

    let rust_dir = root.join("rust_project");
    fs::create_dir(&rust_dir).unwrap();
    fs::write(rust_dir.join("Cargo.toml"), "[package]\nname = \"rust_test\"").unwrap();

    let scanner = Scanner::new();
    let projects = scanner.scan(root).unwrap();

    assert_eq!(projects.len(), 2);
    
    let flutter_project = projects.iter()
        .find(|p| p.project_type == ProjectType::Flutter)
        .expect("Flutter project not found");
    assert_eq!(flutter_project.metadata.name, Some("flutter_test".to_string()));

    let rust_project = projects.iter()
        .find(|p| p.project_type == ProjectType::Rust)
        .expect("Rust project not found");
    assert_eq!(rust_project.metadata.name, Some("rust_test".to_string()));
}

#[test]
fn test_scanner_excludes() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let included_dir = root.join("included");
    fs::create_dir(&included_dir).unwrap();
    fs::write(included_dir.join("Cargo.toml"), "[package]").unwrap();

    let excluded_dir = root.join("excluded");
    fs::create_dir(&excluded_dir).unwrap();
    fs::write(excluded_dir.join("Cargo.toml"), "[package]").unwrap();

    let scanner = Scanner::new()
        .with_exclude_patterns(vec!["**/excluded".to_string()]);
    let projects = scanner.scan(root).unwrap();

    assert_eq!(projects.len(), 1);
    assert!(projects[0].path.ends_with("included"));
}