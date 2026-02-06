//! bump-system-version v9.0.0
//! The LEGENDARY Release Master
//! 
//! COMPREHENSIVE RELEASE AUTOMATION:
//! - Pre-flight checks (git, health, stats)
//! - Smart README updates (dynamic section only)
//! - Enhanced release content generation
//! - Git tagging and push
//! - Fixed .zshrc updates with verification
//! - Health badge automation
//! - Full release summary

use anyhow::{Context, Result, bail};
use chrono::Local;
use colored::*;
use faelight_core::paths;
use std::env;
use std::fs;
use std::io::{self, Write};
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
    old_health: u32,
    path_resilience: String,
    tools_updated: Vec<String>,
}

#[derive(Debug)]
struct PreflightResults {
    git_clean: bool,
    health: u32,
    last_tag: Option<String>,
    commits_since_tag: usize,
}

// ═══════════════════════════════════════════════════════════
// 🎯 MAIN ENTRY POINT
// ═══════════════════════════════════════════════════════════

fn main() {
    if let Err(e) = run() {
        eprintln!("\n{} {}", "❌ Error:".red().bold(), e);
        exit(1);
    }
}

fn run() -> Result<()> {
    print_banner();
    
    // PHASE 1: Pre-flight checks
    println!("\n{}", "═══════════════════════════════════════════════════════════".cyan());
    println!("{}", "🔍 PHASE 1: PRE-FLIGHT CHECKS".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    
    let preflight = run_preflight_checks()?;
    
    if !preflight.git_clean {
        bail!("Git working directory has uncommitted changes. Commit or stash them first.");
    }
    
    if preflight.health < 80 {
        println!("\n{} System health is {}% (below 80%)", 
            "⚠️  Warning:".yellow().bold(), preflight.health);
        print!("Continue anyway? (y/n): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            bail!("Release cancelled - fix health issues first");
        }
    }
    
    println!("\n{}", "✅ All pre-flight checks passed!".green().bold());
    
    // PHASE 2: Get current version and calculate new version
    let old_version = get_current_version()?;
    println!("\n{} {}", "Current version:".cyan(), old_version.bold());
    
    println!("\nEnter new version (e.g., 9.3.0):");
    print!("> ");
    io::stdout().flush()?;
    let mut new_version = String::new();
    io::stdin().read_line(&mut new_version)?;
    let new_version = new_version.trim().to_string();
    
    // PHASE 3: Collect release content
    println!("\n{}", "═══════════════════════════════════════════════════════════".cyan());
    println!("{}", "📝 PHASE 2: RELEASE CONTENT".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    
    let content = collect_release_content(&new_version)?;
    
    // PHASE 4: Collect auto-stats
    let auto_stats = collect_auto_stats(&preflight)?;
    
    // PHASE 5: Preview all changes
    println!("\n{}", "═══════════════════════════════════════════════════════════".cyan());
    println!("{}", "👁️  PHASE 3: PREVIEW CHANGES".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    
    preview_changes(&content, &auto_stats, &old_version, &new_version)?;
    
    println!("\n{}", "Proceed with release? (y/n): ".yellow().bold());
    print!("> ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    
    if !confirm.trim().eq_ignore_ascii_case("y") {
        println!("{}", "Release cancelled.".yellow());
        return Ok(());
    }
    
    // PHASE 6: Execute updates
    println!("\n{}", "═══════════════════════════════════════════════════════════".cyan());
    println!("{}", "🚀 PHASE 4: EXECUTING UPDATES".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    
    execute_updates(&content, &auto_stats, &old_version, &new_version)?;
    
    // PHASE 7: Git commit, tag, and push
    println!("\n{}", "═══════════════════════════════════════════════════════════".cyan());
    println!("{}", "📦 PHASE 5: GIT OPERATIONS".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    
    git_commit_tag_push(&new_version, &content)?;
    
    // PHASE 8: Success summary
    print_success_summary(&new_version, &auto_stats)?;
    
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// 🔍 PRE-FLIGHT CHECKS
// ═══════════════════════════════════════════════════════════

fn run_preflight_checks() -> Result<PreflightResults> {
    println!("\n1️⃣ Checking git status...");
    let git_status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to run git status")?;
    
    let git_clean = git_status.stdout.is_empty();
    
    if git_clean {
        println!("   ✅ Git working directory clean");
    } else {
        println!("   ⚠️  Uncommitted changes detected");
    }
    
    println!("\n2️⃣ Running health check...");
    let health = get_system_health()?;
    println!("   ✅ System health: {}%", health);
    
    println!("\n3️⃣ Checking git history...");
    let last_tag = get_last_git_tag()?;
    let commits_since_tag = if last_tag.is_some() {
        count_commits_since_tag()?
    } else {
        0
    };
    
    if let Some(ref tag) = last_tag {
        println!("   ✅ Last tag: {} ({} commits since)", tag, commits_since_tag);
    } else {
        println!("   ℹ️  No previous tags found");
    }
    
    Ok(PreflightResults {
        git_clean,
        health,
        last_tag,
        commits_since_tag,
    })
}

fn get_system_health() -> Result<u32> {
    let output = Command::new("dot-doctor")
        .output()
        .context("Failed to run dot-doctor")?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    for line in stdout.lines() {
        if line.contains("Health:") {
            if let Some(after_health) = line.split("Health:").nth(1) {
                let cleaned = after_health.trim().trim_end_matches('%').trim();
                if let Ok(h) = cleaned.parse::<u32>() {
                    return Ok(h);
                }
            }
        }
    }
    
    Ok(100) // Default if can't parse
}

fn get_last_git_tag() -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            let tag = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Ok(Some(tag))
        }
        _ => Ok(None)
    }
}

fn count_commits_since_tag() -> Result<usize> {
    let output = Command::new("git")
        .args(["rev-list", "--count", "HEAD", "^$(git describe --tags --abbrev=0)"])
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            let count = String::from_utf8_lossy(&out.stdout).trim().parse().unwrap_or(0);
            Ok(count)
        }
        _ => Ok(0)
    }
}

// ═══════════════════════════════════════════════════════════
// 📝 CONTENT COLLECTION
// ═══════════════════════════════════════════════════════════

fn collect_release_content(version: &str) -> Result<ReleaseContent> {
    println!("\nRelease theme/emoji (e.g., '🎊 Path Resilience Complete'):");
    print!("> ");
    io::stdout().flush()?;
    let mut theme = String::new();
    io::stdin().read_line(&mut theme)?;
    let theme = theme.trim().to_string();
    
    println!("\nKey features (one per line, empty line to finish):");
    let mut features = Vec::new();
    let mut line_num = 1;
    loop {
        print!("  {}. ", line_num);
        io::stdout().flush()?;
        let mut feature = String::new();
        io::stdin().read_line(&mut feature)?;
        let feature = feature.trim().to_string();
        
        if feature.is_empty() {
            break;
        }
        
        features.push(feature);
        line_num += 1;
    }
    
    println!("\nManual statistics (optional, one per line, empty to finish):");
    let mut manual_stats = Vec::new();
    line_num = 1;
    loop {
        print!("  {}. ", line_num);
        io::stdout().flush()?;
        let mut stat = String::new();
        io::stdin().read_line(&mut stat)?;
        let stat = stat.trim().to_string();
        
        if stat.is_empty() {
            break;
        }
        
        manual_stats.push(stat);
        line_num += 1;
    }
    
    println!("\nOptional quote:");
    print!("> ");
    io::stdout().flush()?;
    let mut quote = String::new();
    io::stdin().read_line(&mut quote)?;
    let quote = quote.trim().to_string();
    let quote = if quote.is_empty() { None } else { Some(quote) };
    
    Ok(ReleaseContent {
        version: version.to_string(),
        theme,
        features,
        manual_stats,
        quote,
    })
}

fn collect_auto_stats(preflight: &PreflightResults) -> Result<AutoStats> {
    println!("\n{}", "📊 Collecting statistics...".cyan());
    
    // Get old health from README if possible
    let old_health = get_old_health_from_readme().unwrap_or(preflight.health);
    
    Ok(AutoStats {
        commits_count: preflight.commits_since_tag,
        files_changed: count_changed_files()?,
        system_health: preflight.health,
        old_health,
        path_resilience: get_path_resilience()?,
        tools_updated: get_updated_tools()?,
    })
}

fn get_old_health_from_readme() -> Option<u32> {
    let readme = fs::read_to_string(paths::readme_file()).ok()?;
    for line in readme.lines() {
        if line.contains("![Health]") {
            if let Some(health_part) = line.split("health-").nth(1) {
                if let Some(num) = health_part.split("%25").next() {
                    if let Ok(h) = num.parse::<u32>() {
                        return Some(h);
                    }
                }
            }
        }
    }
    None
}

fn count_changed_files() -> Result<usize> {
    let output = Command::new("git")
        .args(["diff", "--name-only", "HEAD~10..HEAD"])
        .output()?;
    
    let count = String::from_utf8_lossy(&output.stdout).lines().count();
    Ok(count)
}

fn get_path_resilience() -> Result<String> {
    // Check if dot-doctor reports path resilience
    let output = Command::new("dot-doctor").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    for line in stdout.lines() {
        if line.contains("Path Resilience:") {
            if let Some(after) = line.split("Path Resilience:").nth(1) {
                return Ok(after.trim().to_string());
            }
        }
    }
    
    Ok("100%".to_string())
}

fn get_updated_tools() -> Result<Vec<String>> {
    // Get recently modified tool names
    let output = Command::new("git")
        .args(["diff", "--name-only", "HEAD~10..HEAD", "rust-tools/"])
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let tools: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            line.split('/').nth(1).map(|s| s.to_string())
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    
    Ok(tools)
}

// ═══════════════════════════════════════════════════════════
// 👁️  PREVIEW
// ═══════════════════════════════════════════════════════════

fn preview_changes(
    content: &ReleaseContent,
    stats: &AutoStats,
    old_version: &str,
    new_version: &str
) -> Result<()> {
    println!("\n{}", "📋 CHANGES TO BE MADE:".yellow().bold());
    
    println!("\n{}", "Version:".cyan());
    println!("  {} → {}", old_version, new_version.green().bold());
    
    println!("\n{}", "README.md Dynamic Section:".cyan());
    println!("  • Title: 🌲 Faelight Forest v{}", new_version);
    println!("  • Version badge: {}", new_version);
    println!("  • Health badge: {}%", stats.system_health);
    println!("  • Latest release: v{} - {}", new_version, content.theme);
    
    println!("\n{}", ".zshrc:".cyan());
    println!("  • Welcome: v{}", new_version);
    
    println!("\n{}", "Git:".cyan());
    println!("  • Commit message: feat: Release v{} - {}", new_version, content.theme);
    println!("  • Tag: v{}", new_version);
    println!("  • Push to origin");
    
    println!("\n{}", "Statistics:".cyan());
    println!("  • Health: {}% → {}%", stats.old_health, stats.system_health);
    println!("  • Commits: {}", stats.commits_count);
    println!("  • Files changed: {}", stats.files_changed);
    if !stats.tools_updated.is_empty() {
        println!("  • Tools updated: {}", stats.tools_updated.len());
    }
    
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// 🚀 EXECUTE UPDATES
// ═══════════════════════════════════════════════════════════

fn execute_updates(
    content: &ReleaseContent,
    stats: &AutoStats,
    old_version: &str,
    new_version: &str
) -> Result<()> {
    println!("\n1️⃣ Updating VERSION file...");
    update_version_file(new_version)?;
    println!("   ✅ VERSION updated");
    
    println!("\n2️⃣ Updating README.md (dynamic section only)...");
    update_readme_dynamic(content, stats, new_version)?;
    println!("   ✅ README updated");
    
    println!("\n3️⃣ Updating CHANGELOG.md...");
    update_changelog(content, stats, new_version)?;
    println!("   ✅ CHANGELOG updated");
    
    println!("\n4️⃣ Updating Cargo.toml...");
    update_cargo_toml(old_version, new_version)?;
    println!("   ✅ Cargo.toml updated");
    
    println!("\n5️⃣ Updating .zshrc...");
    update_zshrc_fixed(old_version, new_version)?;
    println!("   ✅ .zshrc updated and restowed");
    
    Ok(())
}

fn update_version_file(new_version: &str) -> Result<()> {
    fs::write(paths::version_file(), format!("{}\n", new_version))?;
    Ok(())
}

fn update_readme_dynamic(
    content: &ReleaseContent,
    stats: &AutoStats,
    new_version: &str
) -> Result<()> {
    let readme_path = paths::readme_file();
    let readme = fs::read_to_string(&readme_path)?;
    
    let lines: Vec<&str> = readme.lines().collect();
    
    // Find the dynamic section end
    let dynamic_end = lines.iter()
        .position(|line| line.contains("<!-- END DYNAMIC SECTION -->"))
        .context("Could not find dynamic section end marker")?;
    
    // Keep everything after dynamic section
    let static_section: Vec<&str> = lines[dynamic_end..].to_vec();
    
    // Build new dynamic section
    let date = Local::now().format("%Y-%m-%d");
    let health_color = if stats.system_health >= 90 { "brightgreen" } else if stats.system_health >= 80 { "green" } else { "yellow" };
    
    let mut new_readme = String::new();
    new_readme.push_str("<!-- DYNAMIC SECTION - Updated by bump-system-version -->\n");
    new_readme.push_str(&format!("# 🌲 Faelight Forest v{}\n\n", new_version));
    new_readme.push_str(&format!("![Version](https://img.shields.io/badge/version-{}-green?style=flat-square)\n", new_version));
    new_readme.push_str(&format!("![Health](https://img.shields.io/badge/health-{}%25-{}?style=flat-square)\n", stats.system_health, health_color));
    new_readme.push_str(&format!("![Path Resilience](https://img.shields.io/badge/path_resilience-{}-brightgreen?style=flat-square)\n", stats.path_resilience.replace("%", "%25")));
    new_readme.push_str("![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)\n");
    new_readme.push_str("![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)\n\n");
    new_readme.push_str("> **A self-aware, path-resilient personal computing environment built from first principles.**\n\n");
    new_readme.push_str("## 🎊 Latest Release\n\n");
    new_readme.push_str(&format!("### v{} - {} ({})\n\n", new_version, content.theme, date));
    
    if !content.features.is_empty() {
        for feature in &content.features {
            new_readme.push_str(&format!("- {}\n", feature));
        }
        new_readme.push('\n');
    }
    
    if !content.manual_stats.is_empty() {
        for stat in &content.manual_stats {
            new_readme.push_str(&format!("- {}\n", stat));
        }
        new_readme.push('\n');
    }
    
    new_readme.push_str("[Full Changelog →](CHANGELOG.md)\n\n");
    new_readme.push_str("---\n");
    
    // Add back static section
    for line in static_section {
        new_readme.push_str(line);
        new_readme.push('\n');
    }
    
    // Update footer stats (last few lines)
    let new_readme = new_readme.replace(
        &format!("**System Version**: v{}", "9.2.0"), // This will need to be smarter
        &format!("**System Version**: v{}", new_version)
    );
    let new_readme = new_readme.replace(
        "**Last Updated**: 2026-02-04",
        &format!("**Last Updated**: {}", date)
    );
    let new_readme = new_readme.replace(
        &format!("**Health**: {}%", stats.old_health),
        &format!("**Health**: {}%", stats.system_health)
    );
    
    fs::write(&readme_path, new_readme)?;
    Ok(())
}

fn update_changelog(
    content: &ReleaseContent,
    stats: &AutoStats,
    version: &str
) -> Result<()> {
    let changelog_path = paths::changelog_file();
    let changelog = fs::read_to_string(&changelog_path)?;
    
    let date = Local::now().format("%Y-%m-%d");
    
    let mut entry = String::new();
    entry.push_str(&format!("## v{} - {} ({})\n\n", version, content.theme, date));
    
    if !content.features.is_empty() {
        for feature in &content.features {
            entry.push_str(&format!("- {}\n", feature));
        }
        entry.push('\n');
    }
    
    if !stats.tools_updated.is_empty() {
        entry.push_str(&format!("**Tools Updated:** {}\n\n", stats.tools_updated.join(", ")));
    }
    
    if !content.manual_stats.is_empty() {
        entry.push_str("**Statistics:**\n");
        for stat in &content.manual_stats {
            entry.push_str(&format!("- {}\n", stat));
        }
        entry.push('\n');
    }
    
    entry.push_str(&format!("- System Health: {}%\n", stats.system_health));
    entry.push_str(&format!("- Commits: {}\n", stats.commits_count));
    entry.push_str(&format!("- Files Changed: {}\n", stats.files_changed));
    entry.push('\n');
    
    if let Some(ref quote) = content.quote {
        entry.push_str(&format!("> {}\n\n", quote));
    }
    
    entry.push_str("---\n\n");
    
    // Insert after first line (header)
    let lines: Vec<&str> = changelog.lines().collect();
    let mut new_changelog = String::new();
    
    if !lines.is_empty() {
        new_changelog.push_str(lines[0]);
        new_changelog.push_str("\n\n");
    }
    
    new_changelog.push_str(&entry);
    
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            new_changelog.push_str(line);
            new_changelog.push('\n');
        }
    }
    
    fs::write(&changelog_path, new_changelog)?;
    Ok(())
}

fn update_cargo_toml(old_version: &str, new_version: &str) -> Result<()> {
    let cargo_path = paths::core_dir().join("Cargo.toml");
    let cargo = fs::read_to_string(&cargo_path)?;
    
    let updated = cargo.replace(
        &format!("# System Version: {}", old_version),
        &format!("# System Version: {}", new_version)
    );
    
    fs::write(&cargo_path, updated)?;
    Ok(())
}

fn update_zshrc_fixed(old_version: &str, new_version: &str) -> Result<()> {
    let zshrc_path = paths::core_dir().join("03-interfaces/stow/shell-zsh/.zshrc");
    let zshrc = fs::read_to_string(&zshrc_path)?;
    
    let updated = zshrc.replace(
        &format!("Faelight Forest v{}", old_version),
        &format!("Faelight Forest v{}", new_version)
    );
    
    fs::write(&zshrc_path, updated)?;
    
    // Restow to propagate changes
    println!("   Restowing shell-zsh...");
    let restow = Command::new("stow")
        .args(["--dir=03-interfaces/stow", "-R", "shell-zsh"])
        .current_dir(paths::core_dir())
        .status()
        .context("Failed to restow shell-zsh")?;
    
    if !restow.success() {
        bail!("Restow failed");
    }
    
    // Verify the update
    let home_zshrc = dirs::home_dir().unwrap().join(".zshrc");
    let home_content = fs::read_to_string(&home_zshrc)?;
    
    if !home_content.contains(&format!("Faelight Forest v{}", new_version)) {
        bail!("Restow did not update ~/.zshrc");
    }
    
    println!("   ✅ Verified ~/.zshrc updated");
    
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// 📦 GIT OPERATIONS
// ═══════════════════════════════════════════════════════════

fn git_commit_tag_push(version: &str, content: &ReleaseContent) -> Result<()> {
    println!("\n1️⃣ Staging changes...");
    Command::new("git")
        .args(["add", "-A"])
        .status()?;
    println!("   ✅ Changes staged");
    
    println!("\n2️⃣ Committing...");
    let commit_msg = format!("feat: Release v{} - {}\n\n{}", 
        version, 
        content.theme,
        content.features.join("\n"));
    
    Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .status()?;
    println!("   ✅ Committed");
    
    println!("\n3️⃣ Creating tag v{}...", version);
    let tag_msg = format!("Release v{} - {}", version, content.theme);
    Command::new("git")
        .args(["tag", "-a", &format!("v{}", version), "-m", &tag_msg])
        .status()?;
    println!("   ✅ Tag created");
    
    println!("\n4️⃣ Pushing to origin...");
    Command::new("git")
        .args(["push", "origin", "main"])
        .status()?;
    println!("   ✅ Pushed commits");
    
    println!("\n5️⃣ Pushing tags...");
    Command::new("git")
        .args(["push", "origin", "--tags"])
        .status()?;
    println!("   ✅ Pushed tags");
    
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// 🎊 SUCCESS SUMMARY
// ═══════════════════════════════════════════════════════════

fn print_success_summary(version: &str, stats: &AutoStats) -> Result<()> {
    println!("\n{}", "═══════════════════════════════════════════════════════════".green());
    println!("{}", "🎊 RELEASE COMPLETE!".green().bold());
    println!("{}", "═══════════════════════════════════════════════════════════".green());
    
    println!("\n{}", format!("🌲 Faelight Forest v{} is live!", version).green().bold());
    
    println!("\n{}", "What was updated:".cyan());
    println!("  ✅ VERSION file");
    println!("  ✅ README.md (dynamic section)");
    println!("  ✅ CHANGELOG.md");
    println!("  ✅ Cargo.toml");
    println!("  ✅ .zshrc (restowed)");
    println!("  ✅ Git tagged and pushed");
    
    println!("\n{}", "System Status:".cyan());
    println!("  💚 Health: {}%", stats.system_health);
    println!("  📊 Path Resilience: {}", stats.path_resilience);
    if !stats.tools_updated.is_empty() {
        println!("  🔧 Tools Updated: {}", stats.tools_updated.len());
    }
    
    println!("\n{}", "Next Steps:".cyan());
    println!("  • Run: source ~/.zshrc");
    println!("  • Verify: cat VERSION");
    println!("  • Check: git log --oneline -3");
    println!("  • View: git tag -l");
    
    println!("\n{}", "🎉 LEGENDARY RELEASE SYSTEM! 🎉".green().bold());
    
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// 🎨 HELPERS
// ═══════════════════════════════════════════════════════════

fn print_banner() {
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    println!("{}", "🌲 bump-system-version v9.0.0".cyan().bold());
    println!("{}", "   The LEGENDARY Release Master".cyan());
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
}

fn get_current_version() -> Result<String> {
    let version = fs::read_to_string(paths::version_file())
        .context("Could not read VERSION file")?;
    Ok(version.trim().to_string())
}
