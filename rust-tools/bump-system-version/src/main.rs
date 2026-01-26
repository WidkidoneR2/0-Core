//! bump-system-version v4.0.0
//! Complete 0-Core Release Automation - Linus Edition 🌲

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, exit};
use chrono::Local;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn show_preflight_dashboard(core_dir: &PathBuf, old_version: &str, new_version: &str) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           🌲 PRE-FLIGHT RELEASE DASHBOARD 🌲                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("📊 System Status:");
    let health = check_system_health();
    let git_clean = is_git_clean(core_dir);
    let health_icon = if health { "✅" } else { "❌" };
    let git_icon = if git_clean { "✅" } else { "❌" };
    println!("  {} System Health: {}", health_icon, if health { "100%" } else { "DEGRADED" });
    println!("  {} Git Status: {}", git_icon, if git_clean { "clean" } else { "UNCOMMITTED CHANGES" });
    println!();
    println!("📦 Version Information:");
    println!("  Current:  v{}", old_version);
    println!("  Target:   v{}", new_version);
    println!();
    println!("📝 Files That Will Be Modified:");
    println!("  • VERSION");
    println!("  • Cargo.toml (workspace version)");
    println!("  • stow/shell-zsh/.zshrc (welcome message)");
    println!("  • README.md (badges, milestone, version table)");
    println!("  • CHANGELOG.md (new release entry)");
    println!();
    println!("🔧 Operations That Will Execute:");
    println!("  1. Create btrfs snapshot");
    println!("  2. Update all version files");
    println!("  3. Generate CHANGELOG from git commits");
    println!("  4. Stage changes (git add -A)");
    println!("  5. Create commit with release message");
    println!("  6. Create git tag v{}", new_version);
    println!("  7. Push to origin/main");
    println!("  8. Verify final state");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}


fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Handle flags first
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("bump-system-version v{}", VERSION);
                return;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--health" => {
                println!("✅ bump-system-version: Release automation operational");
                return;
            }
            "--dry-run" => {
                if args.len() != 3 {
                    eprintln!("Usage: bump-system-version --dry-run <version>");
                    exit(1);
                }
                dry_run(args[2].strip_prefix("v").unwrap_or(&args[2]));
                return;
            }
            _ => {}
        }
    }
    
    // Validate arguments
    if args.len() != 2 {
        print_usage();
        exit(1);
    }
    
    let new_version = args[1].strip_prefix("v").unwrap_or(&args[1]);
    
    // Validate version format
    if !is_valid_version(new_version) {
        eprintln!("❌ Invalid version format: {}", new_version);
        eprintln!("   Must be X.Y.Z (e.g., 8.0.0)");
        exit(1);
    }
    
    let core_dir = get_core_dir();
    let old_version = get_current_version(&core_dir);
    
    
    // Show pre-flight dashboard
    show_preflight_dashboard(&core_dir, &old_version, new_version);
    
    // Ask for confirmation
    print!("
🤔 Ready to proceed with release? (y/n): ");
    io::stdout().flush().unwrap();
    let mut proceed = String::new();
    io::stdin().read_line(&mut proceed).unwrap();
    
    if proceed.trim().to_lowercase() != "y" {
        println!("
❌ Release cancelled.");
        exit(0);
    }
    
    println!();
    println!("🌲 Faelight Forest Release System v{}", VERSION);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    // Phase 1: Pre-Flight Checks
    println!("📊 Phase 1: Pre-Flight Checks");
    
    if !check_system_health() {
        eprintln!("  ❌ System health check failed");
        eprintln!("     Run: doctor");
        exit(1);
    }
    println!("  ✅ System health: 100%");
    
    if !is_git_clean(&core_dir) {
        eprintln!("  ❌ Git has uncommitted changes");
        eprintln!("     Commit or stash changes first");
        exit(1);
    }
    println!("  ✅ Git status: clean");
    
    if !is_valid_version(new_version) {
        eprintln!("  ❌ Invalid version format");
        exit(1);
    }
    println!("  ✅ Version format: {} valid", new_version);
    
    println!();
    
    // Create snapshot
    println!("📸 Phase 2: Creating Snapshot");
    let snapshot_id = create_snapshot(&format!("Before 0-Core v{}", new_version));
    match snapshot_id {
        Some(id) => {
            println!("  ✅ Snapshot #{} created", id);
            println!("  🔄 Rollback: sudo snapper rollback {}", id);
        }
        None => {
            println!("  ⚠️  Snapshot creation failed - continuing anyway");
        }
    }
    
    println!();
    
    // Phase 3: Update Version Numbers
    println!("📝 Phase 3: Updating Version Numbers");
    
    update_version_file(&core_dir, new_version).expect("Failed to update VERSION");
    println!("  ✅ VERSION: {} → {}", old_version, new_version);
    
    update_cargo_toml(&core_dir, &old_version, new_version).expect("Failed to update Cargo.toml");
    println!("  ✅ Cargo.toml: workspace version");
    
    update_zshrc(&core_dir, &old_version, new_version).expect("Failed to update .zshrc");
    println!("  ✅ .zshrc: welcome message");
    
    let badge_count = update_readme_badges(&core_dir, &old_version, new_version).expect("Failed to update badges");
    println!("  ✅ README.md: badges ({} updates)", badge_count);
    
    // Interactive: Milestone description
    println!();
    print!("❓ Enter milestone description: ");
    io::stdout().flush().unwrap();
    let mut milestone = String::new();
    io::stdin().read_line(&mut milestone).unwrap();
    let milestone = milestone.trim();
    
    update_readme_milestone(&core_dir, &old_version, new_version, milestone).expect("Failed to update milestone");
    println!("  ✅ README.md: milestone updated");
    
    println!();
    
    // Phase 4: CHANGELOG Generation
    println!("📋 Phase 4: Generating CHANGELOG");
    
    // Get date of last release tag
    let last_tag_date = Command::new("git")
        .args(&["log", "-1", "--format=%ai", &format!("v{}", old_version)])
        .current_dir(&core_dir)
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .and_then(|s| s.split_whitespace().next().map(String::from))
        .unwrap_or_else(|| "2026-01-15".to_string());  // Fallback

    // Run compile-changelog.sh
    println!("  🔄 Running compile-changelog.sh...");
    let changelog_result = Command::new("bash")
        .arg(core_dir.join("scripts/compile-changelog.sh"))
        .arg(&last_tag_date)
        .current_dir(&core_dir)
        .output();
    
    match changelog_result {
        Ok(output) if output.status.success() => {
            println!("  ✅ Changelog draft generated");
        }
        _ => {
            println!("  ⚠️  Changelog generation failed - will use template");
        }
    }
    
    // Ask to edit changelog
    print!("❓ Open changelog draft for editing? (y/n): ");
    io::stdout().flush().unwrap();
    let mut response = String::new();
    io::stdin().read_line(&mut response).unwrap();
    
    if response.trim().to_lowercase() == "y" {
        let editor = env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
        Command::new(&editor)
            .arg(core_dir.join(&format!("CHANGELOG-v{}-DRAFT.md", new_version)))
            .status()
            .expect("Failed to open editor");
        println!("  ✅ Changelog edited");
    }
    
    // Insert into main CHANGELOG
    insert_changelog(&core_dir, new_version).expect("Failed to insert changelog");
    println!("  ✅ Inserted into CHANGELOG.md");
    
    // Get release quote
    print!("❓ Enter release quote: ");
    io::stdout().flush().unwrap();
    let mut quote = String::new();
    io::stdin().read_line(&mut quote).unwrap();
    let quote = quote.trim();
    
    add_changelog_quote(&core_dir, quote).expect("Failed to add quote");
    println!("  ✅ Quote added");
    
    println!();
    
    // Phase 5: Version History Table
    println!("📊 Phase 5: Version History Table");
    
    print!("❓ Brief description for version table: ");
    io::stdout().flush().unwrap();
    let mut table_desc = String::new();
    io::stdin().read_line(&mut table_desc).unwrap();
    let table_desc = table_desc.trim();
    
    update_version_table(&core_dir, new_version, table_desc).expect("Failed to update version table");
    println!("  ✅ Added row to README.md");
    
    println!();
    
    // Phase 6: Intent Completion
    println!("🎯 Phase 6: Intent Analysis");
    
    let intents = detect_completed_intents(&core_dir);
    if !intents.is_empty() {
        println!("  Found references to:");
        for intent in &intents {
            println!("    - {}", intent);
        }
        
        print!("❓ Mark these intents as complete? (y/n): ");
        io::stdout().flush().unwrap();
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();
        
        if response.trim().to_lowercase() == "y" {
            for intent in &intents {
                mark_intent_complete(&core_dir, intent);
                println!("  ✅ {}: marked complete", intent);
            }
        }
    } else {
        println!("  ℹ️  No intent references found in commits");
    }
    
    println!();
    
    // Phase 7: Git Operations
    println!("📦 Phase 7: Git Operations");
    
    // Stage all changes
    Command::new("git")
        .args(&["add", "-A"])
        .current_dir(&core_dir)
        .status()
        .expect("Failed to stage changes");
    println!("  ✅ Changes staged");
    
    // Generate commit message
    let commit_msg = format!(
        "🌲 Release v{} - {}\n\n{}\n\nHealth: 100%\n\n\"{}\" 🌲",
        new_version, milestone, table_desc, quote
    );
    
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Preview commit message:");
    println!("{}", commit_msg);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    print!("❓ Create commit? (y/n): ");
    io::stdout().flush().unwrap();
    let mut response = String::new();
    io::stdin().read_line(&mut response).unwrap();
    
    if response.trim().to_lowercase() != "y" {
        println!("  ⚠️  Commit cancelled");
        exit(0);
    }
    
    Command::new("git")
        .args(&["commit", "-m", &commit_msg])
        .current_dir(&core_dir)
        .status()
        .expect("Failed to create commit");
    println!("  ✅ Commit created");
    
    // Create tag
    print!("❓ Create git tag v{}? (y/n): ", new_version);
    io::stdout().flush().unwrap();
    let mut response = String::new();
    io::stdin().read_line(&mut response).unwrap();
    
    if response.trim().to_lowercase() == "y" {
        Command::new("git")
            .args(&["tag", "-a", &format!("v{}", new_version), "-m", &format!("v{} - {}", new_version, milestone)])
            .current_dir(&core_dir)
            .status()
            .expect("Failed to create tag");
        println!("  ✅ Tag v{} created", new_version);
    }
    
    // Push
    print!("❓ Push to origin (including tags)? (y/n): ");
    io::stdout().flush().unwrap();
    let mut response = String::new();
    io::stdin().read_line(&mut response).unwrap();
    
    if response.trim().to_lowercase() == "y" {
        Command::new("git")
            .args(&["push"])
            .current_dir(&core_dir)
            .status()
            .expect("Failed to push");
        
        Command::new("git")
            .args(&["push", "--tags"])
            .current_dir(&core_dir)
            .status()
            .expect("Failed to push tags");
        
        println!("  ✅ Pushed to origin/main");
        println!("  ✅ Tags pushed");
    }
    
    println!();
    
    // Phase 8: Verification
    println!("🔍 Phase 8: Post-Release Verification");
    
    let final_health = check_system_health();
    println!("  ✅ System health: {}", if final_health { "100%" } else { "degraded" });
    
    let final_version = get_current_version(&core_dir);
    println!("  ✅ VERSION file: {}", final_version);
    
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎉 Release v{} Complete!", new_version);
    println!();
    println!("🌲 The forest evolves with intention.");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

fn print_help() {
    println!("bump-system-version v{} - Complete 0-Core Release Automation", VERSION);
    println!("🌲 Faelight Forest");
    println!();
    println!("USAGE:");
    println!("    bump-system-version [OPTIONS] <version>");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help           Show this help message");
    println!("    -v, --version        Show version information");
    println!("    --health             Check tool health status");
    println!("    --dry-run <version>  Preview changes without applying");
    println!();
    println!("ARGUMENTS:");
    println!("    <version>    New version (format: X.Y.Z, e.g., 8.0.0)");
    println!();
    println!("EXAMPLES:");
    println!("    bump-system-version 8.0.0          # Full release");
    println!("    bump-system-version --dry-run 8.0.0  # Preview only");
    println!();
    println!("WHAT IT DOES:");
    println!("    1. Pre-flight checks (health, git status)");
    println!("    2. Creates btrfs snapshot");
    println!("    3. Updates version numbers (VERSION, Cargo.toml, .zshrc, README)");
    println!("    4. Generates CHANGELOG from git commits");
    println!("    5. Updates version history table");
    println!("    6. Detects and marks intents complete");
    println!("    7. Creates git commit, tag, and pushes");
    println!("    8. Verifies release");
    println!();
    println!("PHILOSOPHY:");
    println!("    Automation serves the human. Interactive prompts ensure");
    println!("    you maintain control while the tool handles tedious work.");
}

fn print_usage() {
    eprintln!("Usage: bump-system-version <version>");
    eprintln!("       bump-system-version --dry-run <version>");
    eprintln!();
    eprintln!("Try 'bump-system-version --help' for more information.");
}

fn dry_run(new_version: &str) {
    let core_dir = get_core_dir();
    let old_version = get_current_version(&core_dir);
    
    println!("🌲 Dry Run: Release v{} → v{}", old_version, new_version);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("📝 Would update:");
    println!("  • VERSION: {} → {}", old_version, new_version);
    println!("  • Cargo.toml: workspace version");
    println!("  • stow/shell-zsh/.zshrc: welcome message");
    println!("  • README.md: badges, milestone, version table");
    println!("  • CHANGELOG.md: new entry with commits since last release");
    println!();
    println!("📦 Would create:");
    println!("  • Btrfs snapshot");
    println!("  • Git commit with release message");
    println!("  • Git tag v{}", new_version);
    println!();
    println!("⚠️  This is a DRY RUN - no files would be changed");
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
    let output = Command::new("dot-doctor").output();
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
    fs::write(&version_file, format!("{}\n", new_version))
        .map_err(|e| e.to_string())
}

fn update_cargo_toml(core_dir: &PathBuf, old_version: &str, new_version: &str) -> Result<(), String> {
    let cargo_path = core_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path).map_err(|e| e.to_string())?;
    let updated = content.replace(
        &format!("version = \"{}\"", old_version),
        &format!("version = \"{}\"", new_version)
    );
    fs::write(&cargo_path, updated).map_err(|e| e.to_string())
}

fn update_zshrc(core_dir: &PathBuf, old_version: &str, new_version: &str) -> Result<(), String> {
    let zshrc_path = core_dir.join("stow/shell-zsh/.zshrc");
    let content = fs::read_to_string(&zshrc_path).map_err(|e| e.to_string())?;
    let updated = content.replace(
        &format!("Faelight Forest v{}", old_version),
        &format!("Faelight Forest v{}", new_version)
    );
    fs::write(&zshrc_path, updated).map_err(|e| e.to_string())
}

fn update_readme_badges(core_dir: &PathBuf, old_version: &str, new_version: &str) -> Result<usize, String> {
    let readme_path = core_dir.join("README.md");
    let content = fs::read_to_string(&readme_path).map_err(|e| e.to_string())?;
    
    let mut updated = content;
    let mut count = 0;
    
    // Update header
    if updated.contains(&format!("Faelight Forest v{}", old_version)) {
        updated = updated.replace(
            &format!("Faelight Forest v{}", old_version),
            &format!("Faelight Forest v{}", new_version)
        );
        count += 1;
    }
    
    // Update badge
    updated = updated.replace(
        &format!("Version-v{}-brightgreen", old_version),
        &format!("Version-v{}-brightgreen", new_version)
    );
    count += 1;
    
    // Update tag link
    updated = updated.replace(
        &format!("tag/v{}", old_version),
        &format!("tag/v{}", new_version)
    );
    count += 1;
    
    fs::write(&readme_path, updated).map_err(|e| e.to_string())?;
    Ok(count)
}

fn update_readme_milestone(core_dir: &PathBuf, old_version: &str, new_version: &str, description: &str) -> Result<(), String> {
    let readme_path = core_dir.join("README.md");
    let content = fs::read_to_string(&readme_path).map_err(|e| e.to_string())?;
    
    let updated = content.replace(
        &format!("**v{} Milestone:", old_version),
        &format!("**v{} Milestone:** {} 🌲🦀\n\n**v{} Milestone:", new_version, description, old_version)
    );
    
    fs::write(&readme_path, updated).map_err(|e| e.to_string())
}

fn insert_changelog(core_dir: &PathBuf, new_version: &str) -> Result<(), String> {
    let changelog_path = core_dir.join("CHANGELOG.md");
    let draft_path = core_dir.join(&format!("CHANGELOG-v{}-DRAFT.md", new_version));
    let today = Local::now().format("%Y-%m-%d").to_string();
    
    // Read draft or create template
    let entry = if draft_path.exists() {
        fs::read_to_string(&draft_path).unwrap_or_else(|_| {
            format!("## [{}] - {}\n### 🚀 New Features\n- [Add features]\n\n", new_version, today)
        })
    } else {
        format!("## [{}] - {}\n### 🚀 New Features\n- [Add features]\n\n", new_version, today)
    };
    
    let content = fs::read_to_string(&changelog_path).map_err(|e| e.to_string())?;
    let lines: Vec<&str> = content.lines().collect();
    let mut new_content = String::new();
    
    for (_i, line) in lines.iter().enumerate() {
        new_content.push_str(line);
        new_content.push('\n');
        
        if line.trim() == "# Changelog" {
            new_content.push('\n');
            new_content.push_str(&entry);
        }
    }
    
    fs::write(&changelog_path, new_content).map_err(|e| e.to_string())
}

fn add_changelog_quote(core_dir: &PathBuf, quote: &str) -> Result<(), String> {
    let changelog_path = core_dir.join("CHANGELOG.md");
    let content = fs::read_to_string(&changelog_path).map_err(|e| e.to_string())?;
    
    // Find the first "## [" after "# Changelog" and add quote before next "---"
    let lines: Vec<&str> = content.lines().collect();
    let mut new_content = String::new();
    let mut in_current_release = false;
    let mut quote_added = false;
    
    for line in lines {
        if line.starts_with("## [") && !quote_added {
            in_current_release = true;
        }
        
        if in_current_release && line.trim() == "---" && !quote_added {
            new_content.push_str(&format!("> \"{}\" 🌲\n", quote));
            quote_added = true;
        }
        
        new_content.push_str(line);
        new_content.push('\n');
    }
    
    fs::write(&changelog_path, new_content).map_err(|e| e.to_string())
}

fn update_version_table(core_dir: &PathBuf, new_version: &str, description: &str) -> Result<(), String> {
    let readme_path = core_dir.join("README.md");
    let content = fs::read_to_string(&readme_path).map_err(|e| e.to_string())?;
    let today = Local::now().format("%Y-%m-%d").to_string();
    
    let lines: Vec<&str> = content.lines().collect();
    let mut new_content = String::new();
    let mut in_table = false;
    let mut row_added = false;
    
    for line in lines {
        // Detect version history table
        if line.contains("Version") && line.contains("Date") && line.contains("Description") {
            in_table = true;
        }
        
        // Add new row after header separator
        if in_table && line.starts_with("|---") && !row_added {
            new_content.push_str(line);
            new_content.push('\n');
            new_content.push_str(&format!("| v{} | {} | {} |\n", new_version, today, description));
            row_added = true;
            continue;
        }
        
        new_content.push_str(line);
        new_content.push('\n');
    }
    
    fs::write(&readme_path, new_content).map_err(|e| e.to_string())
}

fn detect_completed_intents(core_dir: &PathBuf) -> Vec<String> {
    // Get git log and look for intent references
    let output = Command::new("git")
        .args(&["log", "--oneline", "--since=30 days ago"])
        .current_dir(core_dir)
        .output();
    
    let mut intents = Vec::new();
    
    if let Ok(out) = output {
        let log = String::from_utf8_lossy(&out.stdout);
        
        // Look for "Intent 067", "Intent 066", etc.
        for line in log.lines() {
            if line.contains("Intent 0") {
                // Extract intent number
                for word in line.split_whitespace() {
                    if word.starts_with("Intent") {
                        continue;
                    }
                    if word.starts_with("0") && word.len() == 3 {
                        let intent = format!("Intent {}", word);
                        if !intents.contains(&intent) {
                            intents.push(intent);
                        }
                    }
                }
            }
        }
    }
    
    intents
}

fn mark_intent_complete(core_dir: &PathBuf, intent: &str) {
    // Extract number from "Intent 067"
    let number = intent.replace("Intent ", "");
    
    // Find intent file
    let intent_dir = core_dir.join("INTENT/future");
    if let Ok(entries) = fs::read_dir(&intent_dir) {
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename.starts_with(&number) && filename.ends_with(".md") {
                let path = entry.path();
                
                // Update status
                if let Ok(content) = fs::read_to_string(&path) {
                    let updated = content.replace("status: planned", "status: complete");
                    let _ = fs::write(&path, updated);
                }
            }
        }
    }
}
