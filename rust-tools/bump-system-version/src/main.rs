//! bump-system-version v8.0.0
//! The Automated Release Master - Interactive Edition
//! 
//! Handles ALL release tasks:
//! - Interactive content collection
//! - Auto-detection of statistics
//! - README v2.0 updates (title, badges, Recent Changes, footer)
//! - CHANGELOG.md generation
//! - VERSION, Cargo.toml, .zshrc updates
//! - Git commit + tag + push

use anyhow::{Context, Result};
use chrono::Local;
use colored::*;
use faelight_core::paths;
use std::env;
use std::fs;
use std::io::{self, Write, BufRead};
use std::path::PathBuf;
use std::process::{Command, exit};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// ═══════════════════════════════════════════════════════════
// 🏗️ DATA STRUCTURES
// ═══════════════════════════════════════════════════════════

#[derive(Debug)]
struct ReleaseContent {
    version: String,
    theme: String,
    features: Vec<String>,
    manual_stats: Vec<String>,
    quote: Option<String>,
}

#[derive(Debug)]
struct AutoStats {
    commits_count: usize,
    files_changed: usize,
    system_health: u32,
    path_resilience: String,
    tools_updated: Vec<String>,
}

// ═══════════════════════════════════════════════════════════
// 🎯 MAIN ENTRY POINT
// ═══════════════════════════════════════════════════════════

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Handle flags
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
                println!("✅ bump-system-version: The Automated Release Master operational");
                return;
            }
            _ => {}
        }
    }
    
    // Require version argument
    if args.len() < 2 {
        eprintln!("Usage: bump-system-version <version>");
        eprintln!("       bump-system-version --help");
        exit(1);
    }
    
    let new_version = args[1].strip_prefix("v").unwrap_or(&args[1]);
    
    // Validate version format
    if !is_valid_version(new_version) {
        eprintln!("❌ Invalid version format: {}", new_version);
        eprintln!("   Expected: X.Y.Z (e.g., 9.3.0)");
        exit(1);
    }
    
    // Run the interactive release flow
    if let Err(e) = run_interactive_release(new_version) {
        eprintln!("\n{} Release failed: {}", "❌".red(), e);
        exit(1);
    }
}

// ═══════════════════════════════════════════════════════════
// 🎨 INTERACTIVE RELEASE FLOW
// ═══════════════════════════════════════════════════════════

fn run_interactive_release(new_version: &str) -> Result<()> {
    let core_dir = paths::core_dir();
    let old_version = get_current_version()?;
    
    print_banner(&old_version, new_version);
    
    // Pre-flight checks
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "🔍 PRE-FLIGHT CHECKS".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    
    if !check_system_health() {
        eprintln!("\n{} System health check failed!", "❌".red());
        eprintln!("Run 'doctor' to see issues.");
        if !prompt_yes_no("Continue anyway?", false) {
            return Ok(());
        }
    }
    
    if !is_git_clean(&core_dir) {
        eprintln!("\n{} Git working tree not clean!", "⚠️".yellow());
        eprintln!("Uncommitted changes detected.");
        if !prompt_yes_no("Continue anyway?", false) {
            return Ok(());
        }
    }
    
    println!("{} Pre-flight checks passed", "✅".green());
    
    // Collect release content interactively
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "📝 RELEASE CONTENT".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    
    let content = collect_release_content(new_version)?;
    
    // Auto-detect statistics
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "📊 AUTO-DETECTING STATISTICS".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    
    let auto_stats = detect_auto_stats(&old_version)?;
    print_auto_stats(&auto_stats);
    
    // Preview changes
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "👁️  PREVIEW CHANGES".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    
    preview_changes(&content, &auto_stats, &old_version, new_version)?;
    
    if !prompt_yes_no("\nProceed with release?", true) {
        println!("\n{} Release cancelled", "ℹ️".blue());
        return Ok(());
    }
    
    // Execute release
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "🚀 EXECUTING RELEASE".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    
    execute_release(&content, &auto_stats, &old_version, new_version)?;
    
    // Success!
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".green());
    println!("{} {} {}", "🎊".green(), format!("RELEASE v{} COMPLETE!", new_version).green().bold(), "🎊".green());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".green());
    
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// 📝 INTERACTIVE CONTENT COLLECTION
// ═══════════════════════════════════════════════════════════

fn collect_release_content(version: &str) -> Result<ReleaseContent> {
    let theme = prompt_string("\n> Release theme (e.g., 'Tool Harmony'):", true)?;
    
    println!("\n> Major features (one per line, empty to finish):");
    let features = collect_multiline_input()?;
    
    println!("\n> Additional statistics (empty to finish):");
    println!("  Examples: 'Tools migrated: 5', 'New paths: 3'");
    let manual_stats = collect_multiline_input()?;
    
    let quote = prompt_string("\n> Philosophy quote [optional, empty to skip]:", false).ok();
    
    Ok(ReleaseContent {
        version: version.to_string(),
        theme,
        features,
        manual_stats,
        quote,
    })
}

fn collect_multiline_input() -> Result<Vec<String>> {
    let mut lines = Vec::new();
    let stdin = io::stdin();
    
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            break;
        }
        lines.push(line);
    }
    
    Ok(lines)
}

// ═══════════════════════════════════════════════════════════
// 📊 AUTO-DETECTION
// ═══════════════════════════════════════════════════════════

fn detect_auto_stats(old_version: &str) -> Result<AutoStats> {
    // Count commits since last version
    let commits_output = Command::new("git")
        .args(&["rev-list", &format!("v{}..HEAD", old_version), "--count"])
        .output()
        .context("Failed to count commits")?;
    
    let commits_count = String::from_utf8_lossy(&commits_output.stdout)
        .trim()
        .parse::<usize>()
        .unwrap_or(0);
    
    // Files changed
    let files_output = Command::new("git")
        .args(&["diff", &format!("v{}", old_version), "--name-only"])
        .output()
        .context("Failed to get changed files")?;
    
    let files_changed = String::from_utf8_lossy(&files_output.stdout)
        .lines()
        .count();
    
    // System health from doctor
    let health = get_system_health();
    
    // Path resilience from doctor
    let path_resilience = get_path_resilience();
    
    // Tools updated (check Cargo.toml changes)
    let tools_updated = detect_updated_tools(old_version)?;
    
    Ok(AutoStats {
        commits_count,
        files_changed,
        system_health: health,
        path_resilience,
        tools_updated,
    })
}

fn detect_updated_tools(old_version: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(&["diff", &format!("v{}", old_version), "--name-only", "--", "rust-tools/*/Cargo.toml"])
        .output()?;
    
    let tools: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            line.split('/').nth(1).map(|s| s.to_string())
        })
        .collect();
    
    Ok(tools)
}

fn print_auto_stats(stats: &AutoStats) {
    println!("\n  {} Commits since last version", "📝".cyan());
    println!("  {} Files changed", "📁".cyan());
    println!("  {} System Health: {}%", "🏥".cyan(), stats.system_health);
    println!("  {} Path Resilience: {}", "💎".cyan(), stats.path_resilience);
    
    if !stats.tools_updated.is_empty() {
        println!("  {} Tools updated: {}", "🔧".cyan(), stats.tools_updated.len());
        for tool in stats.tools_updated.iter().take(5) {
            println!("     - {}", tool);
        }
        if stats.tools_updated.len() > 5 {
            println!("     ... and {} more", stats.tools_updated.len() - 5);
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 👁️  PREVIEW
// ═══════════════════════════════════════════════════════════

fn preview_changes(content: &ReleaseContent, stats: &AutoStats, old_version: &str, new_version: &str) -> Result<()> {
    println!("\nFiles that will be updated:");
    println!("  • VERSION: {} → {}", old_version, new_version);
    println!("  • README.md (title, badges, Recent Changes, footer)");
    println!("  • CHANGELOG.md (new entry)");
    println!("  • Cargo.toml (workspace version)");
    println!("  • .zshrc (welcome message)");
    
    println!("\nREADME Recent Changes entry preview:");
    println!("{}", "━".repeat(60));
    print_readme_entry_preview(content, stats, new_version);
    println!("{}", "━".repeat(60));
    
    Ok(())
}

fn print_readme_entry_preview(content: &ReleaseContent, stats: &AutoStats, version: &str) {
    let date = Local::now().format("%Y-%m-%d");
    
    println!("### v{} - {} 🎊 ({})", version, content.theme, date);
    println!();
    
    for feature in &content.features {
        println!("- {}", feature);
    }
    
    if !content.manual_stats.is_empty() || stats.commits_count > 0 {
        println!();
        println!("**Statistics:**");
        for stat in &content.manual_stats {
            println!("- {}", stat);
        }
        if stats.commits_count > 0 {
            println!("- {} commits, {} files changed", stats.commits_count, stats.files_changed);
        }
    }
    
    if let Some(quote) = &content.quote {
        println!();
        println!("*\"{}\"* 🌲", quote);
    }
}

// ═══════════════════════════════════════════════════════════
// 🚀 EXECUTE RELEASE
// ═══════════════════════════════════════════════════════════

fn execute_release(content: &ReleaseContent, stats: &AutoStats, old_version: &str, new_version: &str) -> Result<()> {
    println!("\n1️⃣ Updating VERSION file...");
    update_version_file(new_version)?;
    println!("   ✅ VERSION updated");
    
    println!("\n2️⃣ Updating README.md...");
    update_readme(content, stats, old_version, new_version)?;
    println!("   ✅ README updated");
    
    println!("\n3️⃣ Updating CHANGELOG.md...");
    update_changelog(content, stats, new_version)?;
    println!("   ✅ CHANGELOG updated");
    
    println!("\n4️⃣ Updating Cargo.toml...");
    update_cargo_toml(old_version, new_version)?;
    println!("   ✅ Cargo.toml updated");
    
    println!("\n5️⃣ Updating .zshrc...");
    update_zshrc(old_version, new_version)?;
    println!("   ✅ .zshrc updated");
    
    println!("\n6️⃣ Creating git commit...");
    let commit_msg = create_commit_message(content, stats, new_version);
    git_commit_release(&commit_msg)?;
    println!("   ✅ Commit created");
    
    println!("\n7️⃣ Creating git tag...");
    git_create_tag(new_version, &content.theme)?;
    println!("   ✅ Tag created");
    
    if prompt_yes_no("\n8️⃣ Push to origin?", true) {
        git_push()?;
        println!("   ✅ Pushed to origin");
    } else {
        println!("   ⏭️  Skipped push (run 'git push && git push --tags' manually)");
    }
    
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// 📝 FILE UPDATES
// ═══════════════════════════════════════════════════════════

fn update_version_file(new_version: &str) -> Result<()> {
    fs::write(paths::version_file(), format!("{}\n", new_version))?;
    Ok(())
}

fn update_readme(content: &ReleaseContent, stats: &AutoStats, old_version: &str, new_version: &str) -> Result<()> {
    let readme_path = paths::readme_file();
    let mut readme = fs::read_to_string(&readme_path)?;
    
    // Update title
    readme = readme.replace(
        &format!("# 🌲 Faelight Forest v{}", old_version),
        &format!("# 🌲 Faelight Forest v{}", new_version)
    );
    
    // Update badges (all 5)
    readme = readme.replace(
        &format!("Version-v{}-", old_version),
        &format!("Version-v{}-", new_version)
    );
    
    // Add new entry to Recent Changes
    let new_entry = generate_readme_recent_changes_entry(content, stats, new_version);
    readme = readme.replace(
        "## 📋 Recent Changes\n",
        &format!("## 📋 Recent Changes\n\n{}\n", new_entry)
    );
    
    // Update footer
    let date = Local::now().format("%Y-%m-%d");
    readme = readme.replace(
        &format!("**System Version**: v{}", old_version),
        &format!("**System Version**: v{}", new_version)
    );
    readme = readme.replace(
        "**Last Updated**:",
        &format!("**Last Updated**: {}\n**Health**:", date)
    );
    
    fs::write(&readme_path, readme)?;
    Ok(())
}

fn generate_readme_recent_changes_entry(content: &ReleaseContent, stats: &AutoStats, version: &str) -> String {
    let date = Local::now().format("%Y-%m-%d");
    let mut entry = format!("### v{} - {} 🎊 ({})\n\n", version, content.theme, date);
    
    for feature in &content.features {
        entry.push_str(&format!("- {}\n", feature));
    }
    
    if !content.manual_stats.is_empty() || stats.commits_count > 0 {
        entry.push_str("\n**Statistics:**\n");
        for stat in &content.manual_stats {
            entry.push_str(&format!("- {}\n", stat));
        }
        if stats.commits_count > 0 {
            entry.push_str(&format!("- {} commits, {} files changed\n", stats.commits_count, stats.files_changed));
        }
    }
    
    if let Some(quote) = &content.quote {
        entry.push_str(&format!("\n*\"{}\"* 🌲\n", quote));
    }
    
    entry.push_str("\n[Full Changelog →](CHANGELOG.md)\n");
    
    entry
}

fn update_changelog(content: &ReleaseContent, stats: &AutoStats, version: &str) -> Result<()> {
    let changelog_path = paths::changelog_file();
    let changelog = fs::read_to_string(&changelog_path)?;
    
    let new_entry = generate_changelog_entry(content, stats, version);
    
    let updated = changelog.replace(
        "# Changelog\n",
        &format!("# Changelog\n\n{}\n", new_entry)
    );
    
    fs::write(&changelog_path, updated)?;
    Ok(())
}

fn generate_changelog_entry(content: &ReleaseContent, stats: &AutoStats, version: &str) -> String {
    let date = Local::now().format("%Y-%m-%d");
    let mut entry = format!("## [{}] - {}\n\n", version, date);
    
    entry.push_str(&format!("### 🎊 {}\n\n", content.theme));
    
    if !content.features.is_empty() {
        entry.push_str("### 🚀 Features\n\n");
        for feature in &content.features {
            entry.push_str(&format!("- {}\n", feature));
        }
        entry.push_str("\n");
    }
    
    if !content.manual_stats.is_empty() || stats.commits_count > 0 {
        entry.push_str("### 📊 Statistics\n\n");
        for stat in &content.manual_stats {
            entry.push_str(&format!("- {}\n", stat));
        }
        if stats.commits_count > 0 {
            entry.push_str(&format!("- Commits: {}\n", stats.commits_count));
            entry.push_str(&format!("- Files changed: {}\n", stats.files_changed));
        }
        if !stats.tools_updated.is_empty() {
            entry.push_str(&format!("- Tools updated: {}\n", stats.tools_updated.join(", ")));
        }
        entry.push_str("\n");
    }
    
    if let Some(quote) = &content.quote {
        entry.push_str(&format!("### 💎 Philosophy\n\n*\"{}\"* 🌲\n\n", quote));
    }
    
    entry.push_str("---\n");
    
    entry
}

fn update_cargo_toml(old_version: &str, new_version: &str) -> Result<()> {
    let cargo_path = paths::core_dir().join("Cargo.toml");
    let cargo = fs::read_to_string(&cargo_path)?;
    
    let updated = cargo.replace(
        &format!("version = \"{}\"", old_version),
        &format!("version = \"{}\"", new_version)
    );
    
    fs::write(&cargo_path, updated)?;
    Ok(())
}

fn update_zshrc(old_version: &str, new_version: &str) -> Result<()> {
    let zshrc_path = paths::core_dir().join("03-interfaces/stow/shell-zsh/.zshrc");
    let zshrc = fs::read_to_string(&zshrc_path)?;
    
    let updated = zshrc.replace(
        &format!("Faelight Forest v{}", old_version),
        &format!("Faelight Forest v{}", new_version)
    );
    
    fs::write(&zshrc_path, updated)?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// 📝 GIT OPERATIONS
// ═══════════════════════════════════════════════════════════

fn create_commit_message(content: &ReleaseContent, stats: &AutoStats, version: &str) -> String {
    let mut msg = format!("release: v{} - {} 🎊\n\n", version, content.theme);
    
    for feature in &content.features {
        msg.push_str(&format!("- {}\n", feature));
    }
    
    if !content.manual_stats.is_empty() {
        msg.push_str("\nStatistics:\n");
        for stat in &content.manual_stats {
            msg.push_str(&format!("- {}\n", stat));
        }
    }
    
    if stats.commits_count > 0 {
        msg.push_str(&format!("\n{} commits, {} files changed", stats.commits_count, stats.files_changed));
    }
    
    if let Some(quote) = &content.quote {
        msg.push_str(&format!("\n\n\"{}\" 🌲", quote));
    }
    
    msg
}

fn git_commit_release(message: &str) -> Result<()> {
    Command::new("git")
        .args(&["add", "-A"])
        .status()?;
    
    Command::new("git")
        .args(&["commit", "-m", message])
        .status()?;
    
    Ok(())
}

fn git_create_tag(version: &str, theme: &str) -> Result<()> {
    let tag_msg = format!("v{} - {}", version, theme);
    
    Command::new("git")
        .args(&["tag", "-a", &format!("v{}", version), "-m", &tag_msg])
        .status()?;
    
    Ok(())
}

fn git_push() -> Result<()> {
    Command::new("git")
        .args(&["push", "origin", "main"])
        .status()?;
    
    Command::new("git")
        .args(&["push", "--tags"])
        .status()?;
    
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// 🔧 UTILITY FUNCTIONS
// ═══════════════════════════════════════════════════════════

fn get_current_version() -> Result<String> {
    let version = fs::read_to_string(paths::version_file())?;
    Ok(version.trim().to_string())
}

fn is_valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    parts.len() == 3 && parts.iter().all(|p| p.parse::<u32>().is_ok())
}

fn check_system_health() -> bool {
    let output = Command::new("doctor")
        .output()
        .ok();
    
    if let Some(out) = output {
        String::from_utf8_lossy(&out.stdout).contains("100%")
    } else {
        false
    }
}

fn get_system_health() -> u32 {
    let output = Command::new("doctor")
        .output()
        .ok();
    
    if let Some(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            if line.contains("Health:") {
                if let Some(pct) = line.split_whitespace().find(|s| s.ends_with('%')) {
                    if let Ok(num) = pct.trim_end_matches('%').parse::<u32>() {
                        return num;
                    }
                }
            }
        }
    }
    0
}

fn get_path_resilience() -> String {
    let output = Command::new("doctor")
        .output()
        .ok();
    
    if let Some(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            if line.contains("Path Resilience:") {
                if let Some(part) = line.split(':').nth(1) {
                    return part.trim().to_string();
                }
            }
        }
    }
    "Unknown".to_string()
}

fn is_git_clean(core_dir: &PathBuf) -> bool {
    let output = Command::new("git")
        .current_dir(core_dir)
        .args(&["status", "--porcelain"])
        .output()
        .ok();
    
    if let Some(out) = output {
        out.stdout.is_empty()
    } else {
        false
    }
}

fn prompt_string(prompt: &str, required: bool) -> Result<String> {
    loop {
        print!("{} ", prompt);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_string();
        
        if !trimmed.is_empty() || !required {
            return Ok(trimmed);
        }
        
        if required {
            println!("  {} This field is required!", "⚠️".yellow());
        }
    }
}

fn prompt_yes_no(prompt: &str, default: bool) -> bool {
    let default_str = if default { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", prompt, default_str);
    io::stdout().flush().ok();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    
    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        "" => default,
        _ => default,
    }
}

fn print_banner(old_version: &str, new_version: &str) {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "🚀 AUTOMATED RELEASE MASTER v8.0.0".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();
    println!("  {} {}", "Current:".bold(), old_version);
    println!("  {} {}", "Target:".bold(), new_version.green());
}

fn print_help() {
    println!("bump-system-version v{}", VERSION);
    println!("The Automated Release Master - Interactive Edition");
    println!();
    println!("USAGE:");
    println!("    bump-system-version <version>");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Show this help");
    println!("    -v, --version    Show version");
    println!("    --health         Check tool health");
    println!();
    println!("EXAMPLES:");
    println!("    bump-system-version 9.3.0    # Interactive release for v9.3.0");
    println!();
    println!("WHAT IT DOES:");
    println!("    1. Collects release content interactively");
    println!("    2. Auto-detects statistics from git/system");
    println!("    3. Updates VERSION, README, CHANGELOG, Cargo.toml, .zshrc");
    println!("    4. Creates commit + tag");
    println!("    5. Pushes to origin");
}
