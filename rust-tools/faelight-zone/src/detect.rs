use std::path::Path;
use crate::model::Zone;

pub fn detect_zone(path: &Path, home: &Path) -> (Zone, String) {
    let path = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };

    // Most specific first (workspace before core)
    if path.starts_with(home.join("0-core/rust-tools")) {
        let rel = path.strip_prefix(home.join("0-core")).unwrap_or(path.as_path());
        // Uppercase for critical workspace zone
        (Zone::Workspace, rel.display().to_string().to_uppercase())
    } else if path.starts_with(home.join("0-core")) {
        // Uppercase for critical core zone
        (Zone::Core, "0-CORE".to_string())
    } else if path.starts_with(home.join("1-src")) {
        let rel = path.strip_prefix(home).unwrap_or(path.as_path());
        (Zone::Src, rel.display().to_string())
    } else if path.starts_with(home.join("2-projects")) {
        let rel = path.strip_prefix(home).unwrap_or(path.as_path());
        (Zone::Project, rel.display().to_string())
    } else if path.starts_with(home.join("3-archive")) {
        let rel = path.strip_prefix(home).unwrap_or(path.as_path());
        (Zone::Archive, rel.display().to_string())
    } else {
        (Zone::Scratch, path.display().to_string())
    }
}
