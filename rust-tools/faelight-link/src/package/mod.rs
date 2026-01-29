use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Discover all packages in the stow directory
pub fn discover_packages(stow_dir: &Path) -> Result<Vec<String>> {
    let mut packages = Vec::new();
    
    if !stow_dir.exists() {
        return Ok(packages);
    }
    
    for entry in fs::read_dir(stow_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Skip hidden directories and .dotmeta
                if !name.starts_with('.') {
                    packages.push(name.to_string());
                }
            }
        }
    }
    
    packages.sort();
    Ok(packages)
}

/// Get the stow directory path
pub fn get_stow_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join("0-core/stow"))
}

/// List all available packages
pub fn list() -> Result<()> {
    let stow_dir = get_stow_dir()?;
    let packages = discover_packages(&stow_dir)?;
    
    if packages.is_empty() {
        println!("  {}", "No packages found".bright_black());
        return Ok(());
    }
    
    for (i, package) in packages.iter().enumerate() {
        let pkg_path = stow_dir.join(package);
        let file_count = count_files(&pkg_path)?;
        println!(
            "  {} {} {} files",
            format!("{:2}.", i + 1).bright_black(),
            package.bright_green(),
            format!("({} files)", file_count).bright_black()
        );
    }
    
    println!("\n  Total: {} packages", packages.len());
    Ok(())
}

/// Count files in a package directory
fn count_files(dir: &Path) -> Result<usize> {
    let count = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count();
    Ok(count)
}

/// Stow a package
pub fn stow(package: &str, force: bool) -> Result<()> {
    let stow_dir = get_stow_dir()?;
    let pkg_path = stow_dir.join(package);
    
    if !pkg_path.exists() {
        anyhow::bail!("Package '{}' not found in {}", package, stow_dir.display());
    }
    
    println!("  Package: {}", pkg_path.display().to_string().bright_black());
    
    // Discover files to link
    let files = discover_package_files(&pkg_path)?;
    println!("  Files to link: {}", files.len());
    
    if files.is_empty() {
        println!("  {}", "No files to link".bright_yellow());
        return Ok(());
    }
    
    // Show first few files
    for (_i, file) in files.iter().take(5).enumerate() {
        println!("    {} {}", "→".bright_black(), file.display().to_string().bright_black());
    }
    if files.len() > 5 {
        println!("    {} {} more...", "→".bright_black(), (files.len() - 5).to_string().bright_black());
    }
    
    // Ask for confirmation
    if !force {
        use dialoguer::Confirm;
        if !Confirm::new()
            .with_prompt("Proceed with linking?")
            .default(true)
            .interact()?
        {
            println!("  {}", "Cancelled".bright_yellow());
            return Ok(());
        }
    }
    
    println!("\n  {} Creating links...", "🔗".bright_blue());
    
    // Create links (next step)
    crate::link::create_links(&pkg_path, &files)?;
    
    println!("\n  {} Package stowed successfully!", "✅".bright_green());
    Ok(())
}

/// Discover all files in a package
fn discover_package_files(pkg_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new(pkg_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        files.push(entry.path().to_path_buf());
    }
    
    Ok(files)
}

/// Unstow a package (remove symlinks)
pub fn unstow(package: &str) -> Result<()> {
    let stow_dir = get_stow_dir()?;
    let pkg_path = stow_dir.join(package);
    
    if !pkg_path.exists() {
        anyhow::bail!("Package '{}' not found in {}", package, stow_dir.display());
    }
    
    println!("  Package: {}", pkg_path.display().to_string().bright_black());
    
    // Find all symlinks for this package
    let home = std::env::var("HOME").context("HOME not set")?;
    let links = find_package_links(&home, package)?;
    
    if links.is_empty() {
        println!("  {} No symlinks found for this package", "⚠️".bright_yellow());
        return Ok(());
    }
    
    println!("  Found {} symlinks to remove:", links.len());
    for link in &links {
        println!("    {} {}", "→".bright_black(), link.display().to_string().bright_black());
    }
    
    // Ask for confirmation
    use dialoguer::Confirm;
    if !Confirm::new()
        .with_prompt("Remove these symlinks?")
        .default(true)
        .interact()?
    {
        println!("  {}", "Cancelled".bright_yellow());
        return Ok(());
    }
    
    println!("\n  {} Removing symlinks...", "🗑️".bright_blue());
    
    let mut removed = 0;
    let mut errors = 0;
    
    for link in &links {
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
    
    println!("\n  {} Package unstowed successfully!", "✅".bright_green());
    Ok(())
}

/// Find all symlinks for a package
fn find_package_links(home: &str, package: &str) -> Result<Vec<PathBuf>> {
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
        
        find_links_recursive(&search_path, package, &mut links)?;
    }
    
    Ok(links)
}

/// Recursively find symlinks
fn find_links_recursive(dir: &Path, package: &str, links: &mut Vec<PathBuf>) -> Result<()> {
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
            find_links_recursive(&path, package, links)?;
        }
    }
    
    Ok(())
}

