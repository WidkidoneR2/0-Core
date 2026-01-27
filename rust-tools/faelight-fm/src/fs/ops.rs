use std::path::Path;

/// Check if path is readable
#[allow(dead_code)]
pub fn is_readable(path: &Path) -> bool {
    path.exists() && path.metadata().is_ok()
}

/// Get parent directory
#[allow(dead_code)]
pub fn parent(path: &Path) -> Option<&Path> {
    path.parent()
}
