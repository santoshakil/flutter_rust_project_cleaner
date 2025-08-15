use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CleanerError {
    #[error("Failed to access path: {path}")]
    PathAccess { path: PathBuf, source: std::io::Error },
    
    #[error("Failed to execute command: {command}")]
    CommandExecution { command: String, source: std::io::Error },
    
    #[error("Command failed with exit code {code}: {command}")]
    CommandFailed { command: String, code: i32 },
    
    #[error("Flutter executable not found in PATH")]
    FlutterNotFound,
    
    #[error("Cargo executable not found in PATH")]
    CargoNotFound,
    
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
    
    #[error("Insufficient permissions for {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("Interrupted by user")]
    Interrupted,
    
    #[error(transparent)]
    Io(#[from] std::io::Error),
    
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),
    
    #[error(transparent)]
    TomlSer(#[from] toml::ser::Error),
    
    #[error(transparent)]
    SerdeYaml(#[from] serde_yaml::Error),
    
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    
    #[error("Failed to open file: {0}")]
    OpenError(String),
}

pub type Result<T> = std::result::Result<T, CleanerError>;