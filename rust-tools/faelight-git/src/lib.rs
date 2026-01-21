//! faelight-git library - Git governance core

pub mod git;
pub mod risk;
pub mod commands;

pub use git::repo::GitRepo;
pub use risk::engine::RiskScore;

/// Check if 0-core is locked
pub fn is_core_locked() -> bool {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".0-core-locked")
        .exists()
}

/// Get 0-core directory
pub fn get_core_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join("0-core")
}
