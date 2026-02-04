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

/// Get path to doctor binary
pub fn doctor_path() -> String {
    paths::scripts_dir()
        .join("dot-doctor")
        .display()
        .to_string()
}
