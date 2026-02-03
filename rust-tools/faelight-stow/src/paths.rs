use faelight_core::paths;

/// Get the stow packages directory
pub fn stow_dir() -> String {
    paths::stow_dir().display().to_string()
}
