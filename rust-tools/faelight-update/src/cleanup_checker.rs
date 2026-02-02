use std::process::Command;
use std::path::Path;
use walkdir::WalkDir;

pub fn find_pacnew_files() -> Vec<String> {
    WalkDir::new("/etc")
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "pacnew")
                .unwrap_or(false)
        })
        .map(|e| e.path().display().to_string())
        .collect()
}

pub fn cleanup_cargo_cache() -> std::io::Result<()> {
    Command::new("cargo-cache")
        .arg("-a")
        .status()?;
    Ok(())
}

pub fn cleanup_pacman_cache() -> std::io::Result<()> {
    Command::new("sudo")
        .args(["pacman", "-Scc", "--noconfirm"])
        .status()?;
    Ok(())
}
