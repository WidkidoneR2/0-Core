use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: bump-system-version <new-version>");
        eprintln!("Example: bump-system-version 6.3.0");
        process::exit(1);
    }
    
    let new_version = &args[1];
    let core_dir = get_core_dir();
    
    // Get current version
    let version_file = core_dir.join("VERSION");
    let old_version = fs::read_to_string(&version_file)
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();
    
    println!("🔄 Bumping system version: v{} → v{}", old_version, new_version);
    println!();
    
    // Update VERSION file
    if let Err(e) = fs::write(&version_file, format!("{}\n", new_version)) {
        eprintln!("❌ Failed to update VERSION file: {}", e);
        process::exit(1);
    }
    println!("✅ Updated VERSION file");
    
    // Update shell-zsh config (specific patterns only)
    let zshrc_path = core_dir.join("shell-zsh/.config/zsh/.zshrc");
    if let Err(e) = update_zshrc(&zshrc_path, &old_version, new_version) {
        eprintln!("❌ Failed to update shell-zsh config: {}", e);
        process::exit(1);
    }
    println!("✅ Updated shell-zsh config");
    
    // Update README.md (specific patterns only)
    let readme_path = core_dir.join("README.md");
    if let Err(e) = update_readme(&readme_path, &old_version, new_version) {
        eprintln!("❌ Failed to update README.md: {}", e);
        process::exit(1);
    }
    println!("✅ Updated README.md");
    
    println!();
    println!("🎉 System version bumped to v{}", new_version);
    println!();
    println!("Next steps:");
    println!("  1. Restow shell: stow -R shell-zsh");
    println!("  2. Review changes: git diff");
    println!("  3. Commit: git commit -am 'Bump version to v{}'", new_version);
    println!("  4. Push: git push");
    println!();
}

fn get_core_dir() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("0-core")
}

fn update_zshrc(path: &PathBuf, old_version: &str, new_version: &str) -> Result<(), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read: {}", e))?;
    
    // Only replace in specific contexts
    let new_content = content
        // "# Version X.Y.Z" comment at top
        .replace(&format!("# Version {}", old_version), &format!("# Version {}", new_version))
        // "Welcome to Faelight Forest vX.Y.Z"
        .replace(&format!("Faelight Forest v{}", old_version), &format!("Faelight Forest v{}", new_version))
        // "DANGEROUS COMMAND HIGHLIGHTING (vX.Y.Z)"
        .replace(&format!("HIGHLIGHTING (v{})", old_version), &format!("HIGHLIGHTING (v{})", new_version));
    
    fs::write(path, new_content)
        .map_err(|e| format!("Failed to write: {}", e))?;
    
    Ok(())
}

fn update_readme(path: &PathBuf, old_version: &str, new_version: &str) -> Result<(), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read: {}", e))?;
    
    // Replace version patterns in README
    let new_content = content
        // Badge: "v6.2.0-faelight"
        .replace(&format!("v{}-faelight", old_version), &format!("v{}-faelight", new_version))
        // "**Current Version**: 6.2.0"
        .replace(&format!("**Current Version**: {}", old_version), &format!("**Current Version**: {}", new_version))
        // "Version X.Y.Z"
        .replace(&format!("Version {}", old_version), &format!("Version {}", new_version))
        // Generic "vX.Y.Z" but only after "0-Core" or at start of badge
        .replace(&format!("0-Core v{}", old_version), &format!("0-Core v{}", new_version));
    
    fs::write(path, new_content)
        .map_err(|e| format!("Failed to write: {}", e))?;
    
    Ok(())
}
