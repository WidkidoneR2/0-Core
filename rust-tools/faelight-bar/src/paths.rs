#![allow(dead_code)]
//! Path functions for faelight-bar - now using faelight-core
use faelight_core::paths;

pub fn current_profile_path() -> String {
    paths::faelight_state_dir()
        .join("current-profile")
        .display()
        .to_string()
}
