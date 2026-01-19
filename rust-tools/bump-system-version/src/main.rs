use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: bump-system-version <new-version>");
        eprintln!("Example: bump-system-version 6.4.0");
        process::exit(1);
    }
    
    let new_version = &args[1];
    
    // BUGFIX 1: Validate version format before proceeding
    if !is_valid_version(new_version) {
        eprintln!("❌ Invalid version format: {}", new_version);
        eprintln!("Version must be in format: X.Y.Z (e.g., 7.6.0)");
        process::exit(1);
    }
    
    let core_dir = get_core_dir();
    let home = env::var("HOME").expect("HOME not set");
    
    // Get current version
    let version_file = core_dir.join("VERSION");
    let old_version = fs::read_to_string(&version_file)
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();
    
    println!("🔄 Bumping system version: v{} → v{}", old_version, new_version);
    println!();
    
    let mut updated = 0;
    let mut failed = 0;
    
    // 1. Update VERSION file
    if let Err(e) = fs::write(&version_file, format!("{}\n", new_version)) {
        eprintln!("❌ VERSION file: {}", e);
        failed += 1;
    } else {
        println!("✅ VERSION file");
        updated += 1;
    }
    
    // 2. Update shell-zsh config
    let zshrc_path = core_dir.join("shell-zsh/.config/zsh/.zshrc");
    match update_file(&zshrc_path, &old_version, new_version, false) {
        Ok(count) => {
            println!("✅ shell-zsh/.zshrc ({} replacements)", count);
            updated += 1;
        }
        Err(e) => {
            eprintln!("❌ shell-zsh/.zshrc: {}", e);
            failed += 1;
        }
    }
    
    // 3. Update README.md (BUGFIX 2: Skip version history section)
    let readme_path = core_dir.join("README.md");
    match update_file(&readme_path, &old_version, new_version, true) {
        Ok(count) => {
            println!("✅ README.md ({} replacements)", count);
            updated += 1;
        }
        Err(e) => {
            eprintln!("❌ README.md: {}", e);
            failed += 1;
        }
    }
    
    // 4. Update faelight config.toml
    let config_path = PathBuf::from(&home).join(".config/faelight/config.toml");
    if config_path.exists() {
        match update_file(&config_path, &old_version, new_version, false) {
            Ok(count) => {
                println!("✅ faelight/config.toml ({} replacements)", count);
                updated += 1;
            }
            Err(e) => {
                eprintln!("❌ faelight/config.toml: {}", e);
                failed += 1;
            }
        }
        
        // Also update the stowed version
        let stowed_config = core_dir.join("config-faelight/.config/faelight/config.toml");
        if stowed_config.exists() {
            match update_file(&stowed_config, &old_version, new_version, false) {
                Ok(count) => {
                    println!("✅ config-faelight/config.toml ({} replacements)", count);
                    updated += 1;
                }
                Err(e) => {
                    eprintln!("❌ config-faelight/config.toml: {}", e);
                    failed += 1;
                }
            }
        }
    }
    
    println!();
    
    if failed > 0 {
        println!("⚠️  Version bump completed with {} errors", failed);
    } else {
        println!("🎉 System version bumped to v{}", new_version);
    }
    
    println!();
    println!("Updated {} files", updated);
    println!();
    println!("Next steps:");
    println!("  1. Update CHANGELOG.md manually");
    println!("  2. Review: git diff");
    println!("  3. Commit: git commit -am '🌲 Release v{}'", new_version);
    println!("  4. Push: git push");
}

fn get_core_dir() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("0-core")
}

// BUGFIX 1: Validate version format
fn is_valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    
    // All parts must be valid numbers
    parts.iter().all(|p| p.parse::<u32>().is_ok())
}

// BUGFIX 2: Add skip_version_history parameter
fn update_file(path: &PathBuf, old_version: &str, new_version: &str, skip_version_history: bool) -> Result<usize, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read: {}", e))?;
    
    let mut count = 0;
    
    // BUGFIX 2: For README, process line by line to skip version history section
    let new_content = if skip_version_history && path.file_name().and_then(|n| n.to_str()) == Some("README.md") {
        let mut result = String::new();
        let mut in_version_history = false;
        
        for line in content.lines() {
            // Detect version history section start
            if line.contains("## 🔄 Version History") || line.contains("🔄 Version History") {
                in_version_history = true;
                result.push_str(line);
                result.push('\n');
                continue;
            }
            
            // Detect version history section end (next ## heading or "See CHANGELOG")
            if in_version_history && (line.starts_with("## ") || line.contains("See CHANGELOG") || line.contains("See [CHANGELOG")) {
                in_version_history = false;
            }
            
            // Skip replacing versions in version history section
            if in_version_history {
                result.push_str(line);
                result.push('\n');
                continue;
            }
            
            // Normal replacement for lines outside version history
            let mut updated_line = line.to_string();
            let patterns = get_version_patterns(old_version, new_version);
            
            for (old_pat, new_pat) in &patterns {
                if updated_line.contains(old_pat.as_str()) {
                    updated_line = updated_line.replace(old_pat.as_str(), new_pat.as_str());
                    count += 1;
                }
            }
            
            result.push_str(&updated_line);
            result.push('\n');
        }
        
        result
    } else {
        // Original behavior for non-README files
        let mut new_content = content.clone();
        let patterns = get_version_patterns(old_version, new_version);
        
        for (old_pat, new_pat) in &patterns {
            let matches = new_content.matches(old_pat.as_str()).count();
            if matches > 0 {
                new_content = new_content.replace(old_pat.as_str(), new_pat.as_str());
                count += matches;
            }
        }
        
        new_content
    };
    
    if count > 0 {
        fs::write(path, new_content)
            .map_err(|e| format!("Failed to write: {}", e))?;
    }
    
    Ok(count)
}

fn get_version_patterns(old_version: &str, new_version: &str) -> Vec<(String, String)> {
    vec![
        // Exact version (most common)
        (format!("v{}", old_version), format!("v{}", new_version)),
        // Without v prefix in specific contexts
        (format!("Version {}", old_version), format!("Version {}", new_version)),
        (format!("version = \"{}\"", old_version), format!("version = \"{}\"", new_version)),
        // Badge format
        (format!("Version-v{}", old_version), format!("Version-v{}", new_version)),
        // Faelight Forest specific
        (format!("Faelight Forest v{}", old_version), format!("Faelight Forest v{}", new_version)),
        (format!("Faelight Forest {}", old_version), format!("Faelight Forest {}", new_version)),
        // Highlighting comment
        (format!("HIGHLIGHTING (v{})", old_version), format!("HIGHLIGHTING (v{})", new_version)),
        // 0-Core specific
        (format!("0-Core v{}", old_version), format!("0-Core v{}", new_version)),
    ]
}
