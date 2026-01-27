use std::path::Path;
use std::env;
use faelight_zone::Zone;

/// Detect zone using faelight-zone library
pub fn classify(path: &Path) -> Zone {
    let home = env::var("HOME")
        .map(|h| std::path::PathBuf::from(h))
        .unwrap_or_else(|_| std::path::PathBuf::from("/home"));
    
    let (zone, _display_path) = faelight_zone::current_zone(path, &home);
    zone
}

/// Get root path for a zone
pub fn zone_root(zone: Zone) -> Option<&'static str> {
    match zone {
        Zone::Core => Some("~/0-core"),
        Zone::Workspace => Some("~/0-core/rust-tools"),
        Zone::Src => Some("~/1-src"),
        Zone::Project => Some("~/2-projects"),
        Zone::Archive => Some("~/3-archive"),
        Zone::Scratch => None,
    }
}
