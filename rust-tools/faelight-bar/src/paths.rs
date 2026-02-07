//! Path functions for faelight-bar - now using faelight-core
use faelight_core::paths;

pub fn core_dir() -> String {
    paths::core_dir().display().to_string()
}

pub fn current_profile_path() -> String {
    paths::faelight_state_dir()
        .join("current-profile")
        .display()
        .to_string()
}


pub fn core_lock_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/christian".to_string());
    std::path::PathBuf::from(home).join(".cache/faelight/core.lock")
}
