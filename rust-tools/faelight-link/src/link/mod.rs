use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use crate::conflict::{Conflict, ConflictAction, resolve_conflict, backup_file};

/// Create symlinks for package files
pub fn create_links(pkg_dir: &Path, files: &[PathBuf]) -> Result<()> {
    let home = std::env::var("HOME").context("HOME not set")?;
    let home_path = PathBuf::from(&home);
    
    let mut created = 0;
    let mut skipped = 0;
    let mut backed_up = 0;
    let mut errors = 0;
    
    for file in files {
        // Calculate target path (where symlink should be)
        let relative = file.strip_prefix(pkg_dir)?;
        let target = home_path.join(relative);
        
        // Create parent directories if needed
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Check if target already exists (CONFLICT!)
        if target.exists() || target.is_symlink() {
            // Create conflict
            let conflict = Conflict::new(target.clone(), file.clone());
            
            // Ask user what to do
            match resolve_conflict(&conflict)? {
                ConflictAction::Backup => {
                    // Backup existing file
                    match backup_file(&target) {
                        Ok(backup_path) => {
                            println!("    {} {} (backed up to {})", 
                                "💾".bright_blue(), 
                                relative.display(),
                                backup_path.file_name().unwrap().to_string_lossy().bright_black()
                            );
                            backed_up += 1;
                            
                            // Now create the symlink
                            if let Err(e) = symlink(file, &target) {
                                println!("    {} {} ({})", "✗".bright_red(), relative.display(), e);
                                errors += 1;
                            } else {
                                created += 1;
                            }
                        }
                        Err(e) => {
                            println!("    {} {} (backup failed: {})", "✗".bright_red(), relative.display(), e);
                            errors += 1;
                        }
                    }
                }
                ConflictAction::Skip => {
                    println!("    {} {}", "⊘".bright_yellow(), relative.display());
                    skipped += 1;
                }
                ConflictAction::Overwrite => {
                    // Remove existing
                    if target.is_symlink() || target.is_file() {
                        fs::remove_file(&target)?;
                    }
                    
                    // Create symlink
                    match symlink(file, &target) {
                        Ok(_) => {
                            println!("    {} {} (overwritten)", "✓".bright_green(), relative.display());
                            created += 1;
                        }
                        Err(e) => {
                            println!("    {} {} ({})", "✗".bright_red(), relative.display(), e);
                            errors += 1;
                        }
                    }
                }
                ConflictAction::Quit => {
                    println!("\n  {} Operation cancelled by user", "⚠️".bright_yellow());
                    break;
                }
            }
            
            continue;
        }
        
        // No conflict - create symlink
        match symlink(file, &target) {
            Ok(_) => {
                println!("    {} {}", "✓".bright_green(), relative.display());
                created += 1;
            }
            Err(e) => {
                println!("    {} {} ({})", "✗".bright_red(), relative.display(), e);
                errors += 1;
            }
        }
    }
    
    // Summary
    println!();
    if created > 0 {
        println!("  {} Created: {}", "✅".bright_green(), created);
    }
    if backed_up > 0 {
        println!("  {} Backed up: {}", "💾".bright_blue(), backed_up);
    }
    if skipped > 0 {
        println!("  {} Skipped: {}", "⊘".bright_yellow(), skipped);
    }
    if errors > 0 {
        println!("  {} Errors: {}", "✗".bright_red(), errors);
    }
    
    Ok(())
}

/// Show status of all links
pub fn status() -> Result<()> {
    let home = std::env::var("HOME").context("HOME not set")?;
    let stow_dir = PathBuf::from(&home).join("0-core/stow");
    
    if !stow_dir.exists() {
        println!("  {} Stow directory not found", "⚠️".bright_yellow());
        return Ok(());
    }
    
    let packages = crate::package::discover_packages(&stow_dir)?;
    
    if packages.is_empty() {
        println!("  {} No packages found", "⚠️".bright_yellow());
        return Ok(());
    }
    
    let mut total_links = 0;
    let mut total_broken = 0;
    
    for package in &packages {
        let links = count_package_links(&home, package)?;
        let broken = count_broken_links(&home, package)?;
        
        total_links += links;
        total_broken += broken;
        
        if broken > 0 {
            println!(
                "  {} {} ({} links, {} broken)",
                "⚠️".bright_yellow(),
                package.bright_green(),
                links,
                broken.to_string().bright_red()
            );
        } else if links > 0 {
            println!(
                "  {} {} ({} links)",
                "✓".bright_green(),
                package.bright_green(),
                links
            );
        } else {
            println!(
                "  {} {} (not stowed)",
                "○".bright_black(),
                package.bright_black()
            );
        }
    }
    
    println!();
    println!("  Total links: {}", total_links);
    if total_broken > 0 {
        println!("  Broken links: {}", total_broken.to_string().bright_red());
    }
    
    Ok(())
}

/// Count symlinks for a package
fn count_package_links(home: &str, package: &str) -> Result<usize> {
    let home_path = PathBuf::from(home);
    let mut count = 0;
    
    let search_paths = vec![
        home_path.clone(),
        home_path.join(".config"),
    ];
    
    for search_path in search_paths {
        if !search_path.exists() {
            continue;
        }
        
        if let Ok(entries) = fs::read_dir(&search_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_symlink() {
                    if let Ok(target) = fs::read_link(&path) {
                        let target_str = target.to_string_lossy();
                        if target_str.contains(&format!("0-core/stow/{}", package)) {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    
    Ok(count)
}

/// Count broken symlinks for a package
fn count_broken_links(home: &str, package: &str) -> Result<usize> {
    let home_path = PathBuf::from(home);
    let mut count = 0;
    
    let search_paths = vec![
        home_path.clone(),
        home_path.join(".config"),
    ];
    
    for search_path in search_paths {
        if !search_path.exists() {
            continue;
        }
        
        if let Ok(entries) = fs::read_dir(&search_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_symlink() {
                    if let Ok(target) = fs::read_link(&path) {
                        let target_str = target.to_string_lossy();
                        if target_str.contains(&format!("0-core/stow/{}", package)) {
                            if !path.exists() {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(count)
}
