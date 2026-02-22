use faelight_core::paths;
use faelight_zone::Zone;
use std::env;
use std::path::Path;

/// Detect zone using faelight-zone library
pub fn classify(path: &Path) -> Zone {
    let home = env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/home"));

    let (zone, _display_path) = faelight_zone::current_zone(path, &home);
    zone
}

/// Get root path for a zone
pub fn zone_root(zone: Zone) -> Option<String> {
    match zone {
        Zone::Core => Some(paths::core_dir().display().to_string()),
        Zone::Workspace => Some(paths::rust_tools_dir().display().to_string()),
        Zone::Src => Some(paths::src_dir().display().to_string()),
        Zone::Project => Some(paths::projects_dir().display().to_string()),
        Zone::Archive => Some(paths::archive_dir().display().to_string()),
        Zone::Scratch => Some(paths::scratch_dir().display().to_string()),
    }
}
