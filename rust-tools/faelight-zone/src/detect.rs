use std::path::Path;
use crate::model::Zone;

pub fn detect_zone(path: &Path, home: &Path) -> Zone {
    let path = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };

    // Most specific first (workspace before core)
    if path.starts_with(home.join("0-core/rust-tools")) {
        Zone::Workspace
    } else if path.starts_with(home.join("0-core")) {
        Zone::Core
    } else if path.starts_with(home.join("1-src")) {
        Zone::Src
    } else if path.starts_with(home.join("2-projects")) {
        Zone::Project
    } else if path.starts_with(home.join("3-archive")) {
        Zone::Archive
    } else {
        Zone::Scratch
    }
}
