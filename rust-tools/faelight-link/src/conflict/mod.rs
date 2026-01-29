use anyhow::{Context, Result};
use colored::*;
use dialoguer::Select;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum ConflictAction {
    Backup,
    Skip,
    Overwrite,
    Quit,
}

#[derive(Debug)]
pub struct Conflict {
    pub target: PathBuf,
    pub source: PathBuf,
}

impl Conflict {
    pub fn new(target: PathBuf, source: PathBuf) -> Self {
        Self { target, source }
    }
}

/// Ask user what to do with a conflict
pub fn resolve_conflict(conflict: &Conflict) -> Result<ConflictAction> {
    println!("\n  {} Conflict detected!", "⚠️".bright_yellow());
    println!("    Target: {}", conflict.target.display().to_string().bright_white());
    
    // Check what exists
    if conflict.target.is_symlink() {
        let existing = fs::read_link(&conflict.target)?;
        println!("    Existing: {} (symlink)", existing.display().to_string().bright_black());
    } else if conflict.target.is_file() {
        println!("    Existing: {} (regular file)", "file".bright_black());
    } else if conflict.target.is_dir() {
        println!("    Existing: {} (directory)", "directory".bright_black());
    }
    
    let options = vec!["Backup", "Skip", "Overwrite", "Quit"];
    
    let selection = Select::new()
        .with_prompt("How to resolve?")
        .items(&options)
        .default(0)
        .interact()?;
    
    match selection {
        0 => Ok(ConflictAction::Backup),
        1 => Ok(ConflictAction::Skip),
        2 => Ok(ConflictAction::Overwrite),
        3 => Ok(ConflictAction::Quit),
        _ => Ok(ConflictAction::Skip),
    }
}

/// Create backup of existing file
pub fn backup_file(target: &Path) -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    let backup_root = PathBuf::from(home).join(".local/share/faelight-link/backups");
    
    // Create backup directory
    fs::create_dir_all(&backup_root)?;
    
    // Generate backup path with timestamp
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let filename = target.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let backup_path = backup_root.join(format!("{}.{}", filename, timestamp));
    
    // Copy or move
    if target.is_symlink() {
        // For symlinks, just remove (we'll create new one)
        fs::remove_file(target)?;
    } else if target.is_file() {
        // For regular files, copy to backup
        fs::copy(target, &backup_path)?;
        fs::remove_file(target)?;
    } else if target.is_dir() {
        // For directories, this is more complex - skip for now
        anyhow::bail!("Cannot backup directories yet");
    }
    
    Ok(backup_path)
}
