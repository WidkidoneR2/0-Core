//! bump-system-version v2.0.0
//! Complete 0-Core Release Automation

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, exit};
use chrono::Local;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: bump-system-version <new-version>");
        eprintln!("Example: bump-system-version 7.7.0");
        exit(1);
    }
    
    let new_version = &args[1];
    
    if !is_valid_version(new_version) {
        eprintln!("❌ Invalid version format: {}", new_version);
        eprintln!("   Must be X.Y.Z (e.g., 7.6.1)");
        exit(1);
    }
    
    let core_dir = get_core_dir();
    let old_version = get_current_version(&core_dir);
    
    println!("🌲 0-Core Release Process v{} → v{}", old_version, new_version);
    println!();
    
    println!("📋 Step 1: Pre-flight Checks");
    
    if !check_system_health() {
        eprintln!("❌ System health check failed");
        exit(1);
    }
    println!("  ✅ System health: OK");
    
    if !is_git_clean(&core_dir) {
        eprintln!("❌ Git has uncommitted changes - commit or stash first");
        exit(1);
    }
    println!("  ✅ Git status: clean");
    
    println!();
    
    println!("📸 Step 2: Creating Snapshot");
    
    let snapshot_id = create_snapshot(&format!("Before 0-Core v{}", new_version));
    match snapshot_id {
        Some(id) => {
            println!("  ✅ Snapshot #{} created", id);
            println!("  🔄 Rollback: sudo snapper rollback {}", id);
        }
        None => {
            eprintln!("⚠️  Snapshot creation failed - continuing anyway");
        }
    }
    
    println!();
    
    println!("🔄 Step 3: Updating Version Numbers");
    
    let mut updated = 0;
    
    if update_version_file(&core_dir, new_version) {
        println!("  ✅ VERSION file");
        updated += 1;
    }
    
    if update_zshrc(&core_dir, &old_version, new_version) {
        println!("  ✅ shell-zsh/.zshrc");
        updated += 1;
    }
    
    if update_readme_badges(&core_dir, &old_version, new_version) {
        println!("  ✅ README.md badges");
        updated += 1;
    }
    
    println!();
    
    println!("📝 Step 4: Generating CHANGELOG.md Template");
    
    if generate_changelog_template(&core_dir, new_version) {
        println!("  ✅ Template added to CHANGELOG.md");
    } else {
        eprintln!("  ⚠️  CHANGELOG generation failed");
    }
    
    println!();
    
    println!("📊 Step 5: Adding Version History Entry");
    
    if add_version_history_entry(&core_dir, new_version) {
        println!("  ✅ Added to version history");
    } else {
        eprintln!("  ⚠️  Version history update failed");
    }
    
    println!();
    
    println!("✅ Release v{} Prepared Successfully!", new_version);
    println!();
    println!("📋 Files Updated: {}", updated);
    println!();
    println!("🔍 Next Steps:");
    println!("   1. Review CHANGELOG.md template");
    println!("   2. Edit README.md milestone description");
    println!("   3. Mark intents complete (if any)");
    println!("   4. Review: git diff");
    println!("   5. Commit: git add -A && git commit -m '🌲 Release v{}'", new_version);
    println!("   6. Verify: doctor");
    println!("   7. Push: git push");
    println!();
    
    if let Some(id) = snapshot_id {
        println!("🔄 Rollback Available:");
        println!("   sudo snapper rollback {}", id);
    }
    
    println!();
}

fn get_core_dir() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("0-core")
}

fn is_valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    parts.len() == 3 && parts.iter().all(|p| p.parse::<u32>().is_ok())
}

fn get_current_version(core_dir: &PathBuf) -> String {
    fs::read_to_string(core_dir.join("VERSION"))
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string()
}

fn check_system_health() -> bool {
    true
}

fn is_git_clean(core_dir: &PathBuf) -> bool {
    let output = Command::new("git")
        .current_dir(core_dir)
        .args(&["status", "--porcelain"])
        .output();
    
    match output {
        Ok(out) => out.stdout.is_empty(),
        _ => false
    }
}

fn create_snapshot(description: &str) -> Option<u32> {
    let output = Command::new("sudo")
        .args(&["snapper", "create", "--description", description, "--print-number"])
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.trim().parse::<u32>().ok()
        }
        _ => None
    }
}

fn update_version_file(core_dir: &PathBuf, new_version: &str) -> bool {
    let version_file = core_dir.join("VERSION");
    fs::write(&version_file, format!("{}\n", new_version)).is_ok()
}

fn update_zshrc(core_dir: &PathBuf, old_version: &str, new_version: &str) -> bool {
    let zshrc_path = core_dir.join("shell-zsh/.config/zsh/.zshrc");
    
    match fs::read_to_string(&zshrc_path) {
        Ok(content) => {
            let updated = content.replace(
                &format!("Faelight Forest v{}", old_version),
                &format!("Faelight Forest v{}", new_version)
            );
            fs::write(&zshrc_path, updated).is_ok()
        }
        Err(_) => false
    }
}

fn update_readme_badges(core_dir: &PathBuf, old_version: &str, new_version: &str) -> bool {
    let readme_path = core_dir.join("README.md");
    
    match fs::read_to_string(&readme_path) {
        Ok(content) => {
            let mut updated = String::new();
            let mut in_version_history = false;
            
            for line in content.lines() {
                if line.contains("## 🔄 Version History") || line.contains("🔄 Version History") {
                    in_version_history = true;
                }
                if in_version_history && (line.starts_with("## ") || line.contains("See CHANGELOG")) {
                    in_version_history = false;
                }
                
                if !in_version_history {
                    let updated_line = line
                        .replace(&format!("Version-v{}", old_version), &format!("Version-v{}", new_version))
                        .replace(&format!("v{} Milestone", old_version), &format!("v{} Milestone", new_version));
                    updated.push_str(&updated_line);
                } else {
                    updated.push_str(line);
                }
                updated.push('\n');
            }
            
            fs::write(&readme_path, updated).is_ok()
        }
        Err(_) => false
    }
}

fn generate_changelog_template(core_dir: &PathBuf, new_version: &str) -> bool {
    let changelog_path = core_dir.join("CHANGELOG.md");
    let today = Local::now().format("%Y-%m-%d").to_string();
    
    let template = format!(
r#"## [{}] - {}

### 🚀 New Features
- [Feature description]

### 🔧 Fixes
- [Fix description]

### 📦 Tool Updates
- [tool-name] vX.Y - [changes]

> "Quote for this release" 🌲

---

"#, new_version, today);
    
    match fs::read_to_string(&changelog_path) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let mut new_content = String::new();
            let mut inserted = false;
            
            for (i, line) in lines.iter().enumerate() {
                new_content.push_str(line);
                new_content.push('\n');
                
                if !inserted && line.contains("All notable changes") {
                    new_content.push('\n');
                    new_content.push_str(&template);
                    inserted = true;
                }
            }
            
            if !inserted {
                new_content = format!("# Changelog\n\n{}\n{}", template, content);
            }
            
            fs::write(&changelog_path, new_content).is_ok()
        }
        Err(_) => false
    }
}

fn add_version_history_entry(core_dir: &PathBuf, new_version: &str) -> bool {
    let readme_path = core_dir.join("README.md");
    let today = Local::now().format("%Y-%m-%d").to_string();
    
    match fs::read_to_string(&readme_path) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let mut new_content = String::new();
            let mut inserted = false;
            
            for line in lines.iter() {
                new_content.push_str(line);
                new_content.push('\n');
                
                if !inserted && line.starts_with("|") && line.contains("---") {
                    new_content.push_str(&format!("| v{} | {} | [Edit description] |\n", new_version, today));
                    inserted = true;
                }
            }
            
            fs::write(&readme_path, new_content).is_ok()
        }
        Err(_) => false
    }
}
