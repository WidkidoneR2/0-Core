//! Path constants for faelight-bar
use std::env;

pub fn core_dir() -> String {
    let home = env::var("HOME").unwrap_or_default();
    format!("{}/0-core", home)
}

pub fn current_profile_path() -> String {
    let home = env::var("HOME").unwrap_or_default();
    format!("{}/.local/state/0-core/current-profile", home)
}
