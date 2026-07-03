//! Shared library functions for faelight-git
//! Using faelight-core for all path management

pub mod commands;
pub mod git;
pub mod risk;


/// Get 0-core directory
pub fn core_dir() -> std::path::PathBuf {
    faelight_core::paths::core_dir()
}
