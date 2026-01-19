//! bump-system-version v3.0.0
//! Complete 0-Core Release Automation - Stow-Aware
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, exit};
use chrono::Local;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Validate arguments
    if args.len() != 2 {
        print_usage();
        exit(1);
    }
    
    let new_version = &args[1];
    
    // Check for help flags
    if new_version.starts_with('-') {
        print_usage();
        exit(1);
    }
    
    // Validate version format
    if !is_valid_version(new_version) {
        eprintln!("❌ Invalid version format: {}", new_version);
        eprintln!("   Must be X.Y.Z (e.g., 7.6.3)");
        eprintln!("   Example: bump-system-version 7.7.0");
        exit(1);
    }
    
    let core_dir = get_core_dir();
    let old_version = get_current_version(&core_dir);
    
    println!("🌲 0-Core Release Process v{} → v{}", old_version, new_version);
    println!();
    
    // Pre-flight checks
    println!("📋 Step 1: Pre-flight Checks");
    
    if !check_system_health() {
        eprintln!("❌ System health check failed");
        eprintln!("   Run: doctor");
        exit(1);
    }
    println!("  ✅ System health: OK");
    
    if !is_git_clean(&core_dir) {
        eprintln!("❌ Git has uncommitted changes");
        eprintln!("   Commit or stash changes first");
        exit(1);
    }
    println!("  ✅ Git status: clean");
    
    println!();
    
    // Create snapshot
    println!("📸 Step 2: Creating Snapshot");
    
    let snapshot_id = create_snapshot(&format!("Before 0-Core v{}", new_version));
    match snapshot_id {
        Some(id) => {
            println!("  ✅ Snapshot #{} created", id);
            println!("  🔄 Rollback: sudo snapper rollback {}", id);
        }
        None => {
            eprintln!("  ⚠️  Snapshot creation failed - continuing anyway");
        }
    }
    
    println!();
    
    // Update all version references
    println!("🔄 Step 3: Updating Version Numbers");
    
    let mut errors = Vec::new();
    
    match update_version_file(&core_dir, new_version) {
        Ok(_) => println!("  ✅ VERSION file"),
        Err(e) => errors.push(format!("VERSION: {}", e)),
    }
    
    match update_cargo_toml(&core_dir, &old_version, new_version) {
        Ok(_) => println!("  ✅ Cargo.toml"),
        Err(e) => errors.push(format!("Cargo.toml: {}", e)),
    }
    
    match update_zshrc(&core_dir, &old_version, new_version) {
        Ok(_) => println!("  ✅ stow/shell-zsh/.zshrc"),
        Err(e) => errors.push(format!(".zshrc: {}", e)),
    }
    
    match update_readme(&core_dir, &old_version, new_version) {
        Ok(count) => println!("  ✅ README.md ({} updates)", count),
        Err(e) => errors.push(format!("README.md: {}", e)),
    }
    
    println!();
    
    // Generate CHANGELOG template
    println!("📝 Step 4: Generating CHANGELOG.md Template");
    
    match generate_changelog_template(&core_dir, new_version) {
        Ok(_) => println!("  ✅ Template added to CHANGELOG.md"),
        Err(e) => {
            eprintln!("  ⚠️  CHANGELOG: {}", e);
            errors.push(format!("CHANGELOG: {}", e));
        }
    }
    
    println!();
    
    // Report results
    if errors.is_empty() {
        println!("✅ Release v{} Prepared Successfully!", new_version);
    } else {
        println!("⚠️  Release v{} prepared with {} errors:", new_version, errors.len());
        for error in &errors {
            println!("   - {}", error);
        }
    }
    
    println!();
    println!("🔍 Next Steps:");
    println!("   1. Review changes: git diff");
    println!("   2. Edit CHANGELOG.md (fill in template)");
    println!("   3. Edit README.md milestone description");
    println!("   4. Mark intents complete (if any)");
    println!("   5. Verify: doctor");
    println!("   6. Commit: git add -A && git commit -m '🌲 Release v{}'", new_version);
    println!("   7. Tag: git tag -a v{} -m 'v{} - Description'", new_version, new_version);
    println!("   8. Push: git push && git push --tags");
    
    if let Some(id) = snapshot_id {
        println!();
        println!("🔄 Rollback Available:");
        println!("   sudo snapper rollback {}", id);
    }
    
    println!();
    
    if !errors.is_empty() {
        exit(1);
    }
}

fn print_usage() {
    eprintln!("Usage: bump-system-version <new-version>");
    eprintln!();
    eprintln!("Example:");
    eprintln!("  bump-system-version 7.7.0");
    eprintln!();
    eprintln!("Version format: X.Y.Z (e.g., 7.6.3)");
}

fn get_core_dir() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("0-core")
}

fn is_valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u32>().is_ok())
}

fn get_current_version(core_dir: &PathBuf) -> String {
    fs::read_to_string(core_dir.join("VERSION"))
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string()
}

fn check_system_health() -> bool {
    let output = Command::new("dot-doctor")
        .output();
    
    match output {
        Ok(out) => out.status.success(),
        _ => false
    }
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

fn update_version_file(core_dir: &PathBuf, new_version: &str) -> Result<(), String> {
    let version_file = core_dir.join("VERSION");
    // Write WITHOUT 'v' prefix - doctor adds it
    fs::write(&version_file, format!("{}\n", new_version))
        .map_err(|e| e.to_string())
}

fn update_cargo_toml(core_dir: &PathBuf, old_version: &str, new_version: &str) -> Result<(), String> {
    let cargo_path = core_dir.join("Cargo.toml");
    
    let content = fs::read_to_string(&cargo_path)
        .map_err(|e| format!("read failed: {}", e))?;
    
    // Update workspace version (near top of file)
    let updated = content.replace(
        &format!("version = \"{}\"", old_version),
        &format!("version = \"{}\"", new_version)
    );
    
    fs::write(&cargo_path, updated)
        .map_err(|e| format!("write failed: {}", e))
}

fn update_zshrc(core_dir: &PathBuf, old_version: &str, new_version: &str) -> Result<(), String> {
    // CORRECT PATH: stow/shell-zsh/.zshrc (root level, not in .config)
    let zshrc_path = core_dir.join("stow/shell-zsh/.zshrc");
    
    let content = fs::read_to_string(&zshrc_path)
        .map_err(|e| format!("read failed: {}", e))?;
    
    let updated = content.replace(
        &format!("Faelight Forest v{}", old_version),
        &format!("Faelight Forest v{}", new_version)
    );
    
    fs::write(&zshrc_path, updated)
        .map_err(|e| format!("write failed: {}", e))
}

fn update_readme(core_dir: &PathBuf, old_version: &str, new_version: &str) -> Result<usize, String> {
    let readme_path = core_dir.join("README.md");
    
    let content = fs::read_to_string(&readme_path)
        .map_err(|e| format!("read failed: {}", e))?;
    
    let mut updated = content.clone();
    let mut update_count = 0;
    
    // 1. Update header: 🌲 Faelight Forest vX.Y.Z - Sway Edition
    if updated.contains(&format!("Faelight Forest v{} - Sway Edition", old_version)) {
        updated = updated.replace(
            &format!("Faelight Forest v{} - Sway Edition", old_version),
            &format!("Faelight Forest v{} - Sway Edition", new_version)
        );
        update_count += 1;
    }
    
    // 2. Update badges: shields.io URLs
    updated = updated.replace(
        &format!("Version-v{}-brightgreen", old_version),
        &format!("Version-v{}-brightgreen", new_version)
    );
    updated = updated.replace(
        &format!("tag/v{}", old_version),
        &format!("tag/v{}", new_version)
    );
    update_count += 2;
    
    // 3. Update milestone line
    if updated.contains(&format!("**v{} Milestone:", old_version)) {
        updated = updated.replace(
            &format!("**v{} Milestone:", old_version),
            &format!("**v{} Milestone:", new_version)
        );
        update_count += 1;
    }
    
    // 4. Insert new row in version history table
    let today = Local::now().format("%Y-%m-%d").to_string();
    let lines: Vec<&str> = updated.lines().collect();
    let mut new_content = String::new();
    let mut inserted = false;
    
    for line in lines.iter() {
        new_content.push_str(line);
        new_content.push('\n');
        
        // Insert after table header separator
        if !inserted && line.starts_with("|---") {
            new_content.push_str(&format!("| v{} | {} | [Edit description] |\n", new_version, today));
            inserted = true;
            update_count += 1;
        }
    }
    
    fs::write(&readme_path, new_content)
        .map_err(|e| format!("write failed: {}", e))?;
    
    Ok(update_count)
}

fn generate_changelog_template(core_dir: &PathBuf, new_version: &str) -> Result<(), String> {
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
    
    let content = fs::read_to_string(&changelog_path)
        .map_err(|e| format!("read failed: {}", e))?;
    
    let lines: Vec<&str> = content.lines().collect();
    let mut new_content = String::new();
    let mut inserted = false;
    
    for (i, line) in lines.iter().enumerate() {
        new_content.push_str(line);
        new_content.push('\n');
        
        // Insert after "# Changelog" header and blank line
        if !inserted && i > 0 && lines[i-1].contains("Changelog") && line.trim().is_empty() {
            new_content.push_str(&template);
            inserted = true;
        }
    }
    
    if !inserted {
        return Err("Could not find insertion point in CHANGELOG.md".to_string());
    }
    
    fs::write(&changelog_path, new_content)
        .map_err(|e| format!("write failed: {}", e))
}
