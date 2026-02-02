use std::env;

/// Get the stow packages directory
pub fn stow_dir() -> String {
    let home = env::var("HOME").unwrap_or_default();
    format!("{}/0-core/03-interfaces/stow", home)
}
