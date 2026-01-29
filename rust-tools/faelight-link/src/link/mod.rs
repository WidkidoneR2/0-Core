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

/// Audit all links - comprehensive health check
pub fn audit() -> Result<()> {
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
    
    println!("\n  Scanning all symlinks...\n");
    
    let mut total_valid = 0;
    let mut total_broken = 0;
    let mut broken_links: Vec<(String, PathBuf)> = Vec::new();
    
    for package in &packages {
        let links = find_all_package_links(&home, package)?;
        let mut valid = 0;
        let mut broken = 0;
        
        for link in links {
            if link.exists() {
                valid += 1;
            } else {
                broken += 1;
                broken_links.push((package.clone(), link));
            }
        }
        
        total_valid += valid;
        total_broken += broken;
        
        if broken > 0 {
            println!(
                "  {} {} ({} valid, {} broken)",
                "⚠️".bright_yellow(),
                package.bright_green(),
                valid.to_string().bright_green(),
                broken.to_string().bright_red()
            );
        } else if valid > 0 {
            println!(
                "  {} {} ({} links)",
                "✓".bright_green(),
                package.bright_green(),
                valid
            );
        }
    }
    
    println!("\n  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  {} Summary:", "📊".bright_blue());
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("    Valid links:   {}", total_valid.to_string().bright_green());
    
    if total_broken > 0 {
        println!("    Broken links:  {}", total_broken.to_string().bright_red());
        println!("\n  {} Broken links found:", "⚠️".bright_yellow());
        for (package, link) in &broken_links {
            println!("    {} {} → {}", 
                "✗".bright_red(),
                package.bright_black(),
                link.display().to_string().bright_red()
            );
        }
        println!("\n  {} Run 'faelight-link clean' to remove broken links", "💡".bright_blue());
    } else {
        println!("    Broken links:  {}", "0".bright_green());
        println!("\n  {} All links are healthy!", "✅".bright_green());
    }
    
    let health_pct = if total_valid + total_broken > 0 {
        (total_valid * 100) / (total_valid + total_broken)
    } else {
        100
    };
    
    println!("\n  Overall health: {}%", 
        if health_pct == 100 {
            health_pct.to_string().bright_green()
        } else if health_pct >= 90 {
            health_pct.to_string().bright_yellow()
        } else {
            health_pct.to_string().bright_red()
        }
    );
    
    Ok(())
}

/// Clean up broken and orphaned links
pub fn clean(force: bool) -> Result<()> {
    let home = std::env::var("HOME").context("HOME not set")?;
    let stow_dir = PathBuf::from(&home).join("0-core/stow");
    
    if !stow_dir.exists() {
        println!("  {} Stow directory not found", "⚠️".bright_yellow());
        return Ok(());
    }
    
    let packages = crate::package::discover_packages(&stow_dir)?;
    
    // Find all broken links
    let mut broken_links: Vec<PathBuf> = Vec::new();
    
    for package in &packages {
        let links = find_all_package_links(&home, package)?;
        
        for link in links {
            if !link.exists() {
                broken_links.push(link);
            }
        }
    }
    
    if broken_links.is_empty() {
        println!("  {} No broken links found!", "✅".bright_green());
        return Ok(());
    }
    
    println!("  Found {} broken links:", broken_links.len());
    for link in &broken_links {
        println!("    {} {}", "✗".bright_red(), link.display());
    }
    
    // Ask for confirmation
    if !force {
        use dialoguer::Confirm;
        if !Confirm::new()
            .with_prompt("Remove these broken links?")
            .default(true)
            .interact()?
        {
            println!("  {}", "Cancelled".bright_yellow());
            return Ok(());
        }
    }
    
    println!("\n  {} Removing broken links...", "🗑️".bright_blue());
    
    let mut removed = 0;
    let mut errors = 0;
    
    for link in &broken_links {
        match fs::remove_file(link) {
            Ok(_) => {
                println!("    {} {}", "✓".bright_green(), link.display());
                removed += 1;
            }
            Err(e) => {
                println!("    {} {} ({})", "✗".bright_red(), link.display(), e);
                errors += 1;
            }
        }
    }
    
    println!();
    if removed > 0 {
        println!("  {} Removed: {}", "✅".bright_green(), removed);
    }
    if errors > 0 {
        println!("  {} Errors: {}", "✗".bright_red(), errors);
    }
    
    println!("\n  {} Cleanup complete!", "✅".bright_green());
    Ok(())
}

/// Find all symlinks for a package (recursive)
fn find_all_package_links(home: &str, package: &str) -> Result<Vec<PathBuf>> {
    let home_path = PathBuf::from(home);
    let mut links = Vec::new();
    
    let search_paths = vec![
        home_path.clone(),
        home_path.join(".config"),
        home_path.join(".local"),
    ];
    
    for search_path in search_paths {
        if !search_path.exists() {
            continue;
        }
        
        find_links_in_dir(&search_path, package, &mut links)?;
    }
    
    Ok(links)
}

/// Helper to find links in a directory
fn find_links_in_dir(dir: &Path, package: &str, links: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_symlink() {
            if let Ok(target) = fs::read_link(&path) {
                let target_str = target.to_string_lossy();
                if target_str.contains(&format!("0-core/stow/{}", package)) {
                    links.push(path);
                }
            }
        } else if path.is_dir() {
            // Skip certain directories
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') && name != ".config" && name != ".local" {
                continue;
            }
            find_links_in_dir(&path, package, links)?;
        }
    }
    
    Ok(())
}
