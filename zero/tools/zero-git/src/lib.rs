//! Shared library functions for zero-git
//! Using zero-core for all path management

pub mod commands;
pub mod git;
pub mod risk;

/// Get 0-core directory
pub fn core_dir() -> std::path::PathBuf {
    zero_core::paths::core_dir()
}
