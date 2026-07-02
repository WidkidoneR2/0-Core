//! Shared library functions for faelight-git
//! Using faelight-core for all path management

pub mod commands;
pub mod git;
pub mod risk;

/// Check if 0-core is locked
pub fn is_locked() -> bool {
    faelight_core::paths::is_core_locked()
}

/// Get 0-core directory
pub fn core_dir() -> std::path::PathBuf {
    faelight_core::paths::core_dir()
}
