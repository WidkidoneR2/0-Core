use regex::Regex;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: bump-system-version <new-version>");
        eprintln!("");
        eprintln!("Example: bump-system-version 4.3.0");
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
    
    // Update shell-zsh config
    let zshrc_path = core_dir.join("shell-zsh/.config/zsh/.zshrc");
    if let Err(e) = replace_version_in_file(&zshrc_path, new_version) {
        eprintln!("❌ Failed to update shell-zsh config: {}", e);
        process::exit(1);
    }
    println!("✅ Updated shell-zsh config");
    
    // Update README.md
    let readme_path = core_dir.join("README.md");
    if let Err(e) = replace_version_in_file(&readme_path, new_version) {
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
    println!("  3. Commit: git commit -am 'chore: Bump version to v{}'", new_version);
    println!("  4. Push: git push");
    println!("  5. Lock: lock-core");
    println!();
    println!("⚠️  Remember to lock-core when finished!");
}

fn get_core_dir() -> PathBuf {
    let home = env::var("HOME").expect("HOME environment variable not set");
    PathBuf::from(home).join("0-core")
}

fn replace_version_in_file(path: &PathBuf, new_version: &str) -> Result<(), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Match version patterns like v1.2.3 or 1.2.3
    let re = Regex::new(r"v?\d+\.\d+\.\d+").unwrap();
    let new_content = re.replace_all(&content, format!("v{}", new_version));
    
    fs::write(path, new_content.as_ref())
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(())
}
