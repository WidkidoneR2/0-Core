use thiserror::Error;

#[derive(Error, Debug)]
pub enum FmError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid path: {0}")]
    #[allow(dead_code)]
    InvalidPath(String),
    
    #[error("Permission denied: {0}")]
    #[allow(dead_code)]
    PermissionDenied(String),
    
    #[error("Zone violation: {0}")]
    #[allow(dead_code)]
    ZoneViolation(String),
}

pub type Result<T> = std::result::Result<T, FmError>;
