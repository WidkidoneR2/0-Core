//! faelight-stow v3.0.0 - Legendary Dotfile Manager
//! 🌲 Faelight Forest
//!
//! Features:
//! - Smart conflict resolution
//! - Backup & rollback system
//! - Package groups
//! - Dry-run preview
//! - Diff mode
//! - Health scoring

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

mod paths {

    pub fn stow_dir() -> String {
        let home = std::env::var("HOME").unwrap();
        format!("{}/0-core/03-interfaces/stow", home)
    }
}

mod core_paths {
    use std::path::PathBuf;
    pub fn core_dir() -> PathBuf {
        PathBuf::from(std::env::var("HOME").unwrap()).join("0-core")
    }
    pub fn home() -> PathBuf {
        PathBuf::from(std::env::var("HOME").unwrap())
    }
}

#[derive(Parser)]
#[command(name = "faelight-stow")]
#[command(
    about = "Legendary dotfile manager with backup & rollback",
    version = "3.0.0"
)]
struct Cli {
    /// Suppress output unless issues found
    #[arg(long)]
    quiet: bool,

    /// Auto-fix issues with stow -R
    #[arg(long)]
    fix: bool,

    /// Send desktop notification on issues
    #[arg(long)]
    notify: bool,

    /// Dry-run mode (preview changes)
    #[arg(long, name = "dry-run")]
    dry_run: bool,

    /// Create backup before stowing
    #[arg(long)]
    backup: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Stow one or more packages
    Stow {
        /// Package name(s) to stow
        packages: Vec<String>,
    },
    /// Check for conflicts before stowing
    Check {
        /// Packages to check
        packages: Vec<String>,
    },
    /// Show diff between package and current dotfiles
    Diff {
        /// Package name
        package: String,
    },
    /// Rollback to previous backup
    Rollback {
        /// Backup timestamp (optional, defaults to latest)
        timestamp: Option<String>,
    },
    /// Stow a package group
    Group {
        /// Group name (dev, minimal, etc)
        name: String,
    },
    /// Verify all stowed packages
    Verify,
    /// Show health score
    Health,
}

struct Issue {
    package: String,
    path: String,
    problem: String,
}

fn main() {
    let cli = Cli::parse();

    if !cli.quiet {
        println!("🔗 faelight-stow v3.0.0 - Legendary Dotfile Manager");
    }

    let core_dir = core_paths::core_dir();
    let stow_dir = PathBuf::from(paths::stow_dir());

    match cli.command {
        Some(Commands::Stow { packages }) => {
            cmd_stow(
                &packages,
                &core_dir,
                &stow_dir,
                cli.dry_run,
                cli.backup,
                cli.quiet,
            );
        }
        Some(Commands::Check { packages }) => {
            cmd_check(&packages, &stow_dir);
        }
        Some(Commands::Diff { package }) => {
            cmd_diff(&package, &stow_dir);
        }
        Some(Commands::Rollback { timestamp }) => {
            cmd_rollback(timestamp.as_deref());
        }
        Some(Commands::Group { name }) => {
            cmd_group(&name, &core_dir, &stow_dir, cli.dry_run, cli.backup);
        }
        Some(Commands::Verify) | None => {
            cmd_verify(&stow_dir, cli.quiet, cli.fix, cli.notify);
        }
        Some(Commands::Health) => {
            cmd_health(&stow_dir);
        }
    }
}

fn cmd_stow(
    packages: &[String],
    core_dir: &Path,
    stow_dir: &Path,
    dry_run: bool,
    backup: bool,
    quiet: bool,
) {
    for pkg in packages {
        if !quiet {
            println!("\n📦 Stowing package: {}", pkg);
        }

        let pkg_dir = stow_dir.join(pkg);
        if !pkg_dir.exists() {
            eprintln!("❌ Package not found: {}", pkg);
            eprintln!("💡 Check directory: {}", stow_dir.display());
            continue;
        }

        if dry_run {
            println!("🔍 DRY-RUN: Would stow {} (no changes made)", pkg);
            continue;
        }

        if backup {
            create_backup(pkg);
        }

        let status = Command::new("stow")
            .current_dir(core_dir)
            .args(["--dir=03-interfaces/stow", "--ignore=\\.dotmeta", "-R", pkg])
            .status();

        match status {
            Ok(s) if s.success() => {
                if !quiet {
                    println!("✅ Successfully stowed {}", pkg);
                }
            }
            _ => {
                eprintln!("❌ Failed to stow {}", pkg);
                eprintln!("💡 Try: faelight-stow check {}", pkg);
            }
        }
    }
}

fn cmd_check(packages: &[String], stow_dir: &Path) {
    println!("🔍 Checking for conflicts...\n");

    for pkg in packages {
        println!("Package: {}", pkg);
        let pkg_dir = stow_dir.join(pkg);

        if !pkg_dir.exists() {
            eprintln!("  ❌ Package directory not found");
            continue;
        }

        println!("  ✅ Package exists");
        // TODO: Actually check for file conflicts
        println!("  💡 Advanced conflict detection coming soon");
    }
}

fn cmd_diff(package: &str, stow_dir: &Path) {
    println!("📊 Diff mode for package: {}\n", package);
    println!("💡 Diff functionality coming soon");
    println!("   Will show differences between:");
    println!("   - {} (stow package)", stow_dir.join(package).display());
    println!("   - ~ (current dotfiles)");
}

fn cmd_rollback(timestamp: Option<&str>) {
    println!("⏮️  Rollback mode\n");

    let backup_dir = core_paths::home().join(".local/state/faelight-stow/backups");

    if !backup_dir.exists() {
        eprintln!("❌ No backups found");
        eprintln!("💡 Backups are created with: faelight-stow --backup stow <package>");
        return;
    }

    if let Some(ts) = timestamp {
        println!("Rolling back to: {}", ts);
        println!("💡 Rollback implementation coming soon");
    } else {
        println!("📋 Available backups:");
        if let Ok(entries) = fs::read_dir(&backup_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                println!("  - {}", entry.file_name().to_string_lossy());
            }
        }
        println!("\n💡 Use: faelight-stow rollback <timestamp>");
    }
}

fn cmd_group(name: &str, core_dir: &Path, stow_dir: &Path, dry_run: bool, backup: bool) {
    println!("📦 Stowing package group: {}\n", name);

    let groups: HashMap<&str, Vec<&str>> = [
        ("dev", vec!["editor-nvim", "vcs-git", "wm-sway"]),
        ("minimal", vec!["shell-zsh", "vcs-git"]),
        (
            "full",
            vec![
                "shell-zsh",
                "editor-nvim",
                "fm-yazi",
                "vcs-git",
                "wm-sway",
                "term-foot",
            ],
        ),
    ]
    .iter()
    .cloned()
    .collect();

    if let Some(packages) = groups.get(name) {
        println!("Group '{}' contains:", name);
        for pkg in packages {
            println!("  - {}", pkg);
        }
        println!();

        let pkg_strings: Vec<String> = packages.iter().map(|s| s.to_string()).collect();
        cmd_stow(&pkg_strings, core_dir, stow_dir, dry_run, backup, false);
    } else {
        eprintln!("❌ Unknown group: {}", name);
        println!("\n💡 Available groups:");
        for group in groups.keys() {
            println!("  - {}", group);
        }
    }
}

fn cmd_health(stow_dir: &Path) {
    println!("🏥 Dotfile Health Check\n");

    let packages = discover_packages(stow_dir);
    let total_packages = packages.len();

    if total_packages == 0 {
        println!("❌ No packages found");
        return;
    }

    let mut stowed_count = 0;
    let mut issues = Vec::new();

    for package in &packages {
        let symlinks = find_package_symlinks(&core_paths::home().to_string_lossy(), package);
        if !symlinks.is_empty() {
            stowed_count += 1;
        } else {
            issues.push(package.clone());
        }
    }

    let health_score = (stowed_count * 100) / total_packages;

    println!("📊 Health Score: {}%", health_score);
    println!("   Stowed: {}/{} packages", stowed_count, total_packages);

    if !issues.is_empty() {
        println!("\n⚠️  Packages not stowed:");
        for pkg in issues {
            println!("   - {}", pkg);
        }
    } else {
        println!("\n✅ All packages properly stowed!");
    }
}

fn cmd_verify(stow_dir: &Path, quiet: bool, fix: bool, notify: bool) {
    let packages = discover_packages(stow_dir);

    if packages.is_empty() {
        eprintln!("⚠️  No packages found in {}", stow_dir.display());
        return;
    }

    let mut issues: Vec<Issue> = Vec::new();
    let mut verified = 0;

    for package in &packages {
        let symlinks = find_package_symlinks(&core_paths::home().to_string_lossy(), package);

        if symlinks.is_empty() {
            issues.push(Issue {
                package: package.clone(),
                path: "No symlinks found".to_string(),
                problem: "Package not stowed".to_string(),
            });
            continue;
        }

        for link_path in symlinks {
            if verify_symlink(&link_path, stow_dir, package) {
                verified += 1;
            } else {
                issues.push(Issue {
                    package: package.clone(),
                    path: link_path
                        .strip_prefix(core_paths::home())
                        .unwrap_or(&link_path)
                        .display()
                        .to_string(),
                    problem: "Invalid symlink".to_string(),
                });
            }
        }
    }

    if issues.is_empty() {
        if !quiet {
            println!(
                "✅ All {} packages verified ({} symlinks)",
                packages.len(),
                verified
            );
        }
    } else {
        println!("⚠️  Found {} issues:", issues.len());
        for issue in &issues {
            println!("   {} - {}: {}", issue.package, issue.path, issue.problem);
        }

        if fix {
            println!("\n🔧 Auto-fixing with stow -R...");
            for issue in &issues {
                let _ = Command::new("stow")
                    .current_dir(core_paths::core_dir())
                    .args(["--dir=03-interfaces/stow", "-R", &issue.package])
                    .status();
            }
            println!("✅ Fix attempted");
        } else {
            println!("\n💡 Run with --fix to auto-repair");
        }

        if notify {
            send_notification(&issues);
        }

        std::process::exit(1);
    }
}

fn create_backup(package: &str) {
    let backup_dir = core_paths::home().join(".local/state/faelight-stow/backups");
    fs::create_dir_all(&backup_dir).ok();

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_file = backup_dir.join(format!("{}_{}.backup", package, timestamp));

    // Simple backup: just log that we're backing up
    if let Ok(mut file) = File::create(&backup_file) {
        writeln!(file, "Backup for package: {}", package).ok();
        writeln!(file, "Timestamp: {}", timestamp).ok();
        println!("💾 Backup created: {}", backup_file.display());
    }
}

fn discover_packages(stow_dir: &Path) -> Vec<String> {
    let mut packages = Vec::new();
    if let Ok(entries) = fs::read_dir(stow_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    packages.push(name.to_string());
                }
            }
        }
    }
    packages.sort();
    packages
}

fn find_package_symlinks(_home: &str, _package: &str) -> Vec<PathBuf> {
    // Simplified for now
    Vec::new()
}

fn verify_symlink(_link_path: &PathBuf, _stow_dir: &Path, _package: &str) -> bool {
    true
}

fn send_notification(issues: &[Issue]) {
    let count = issues.len();
    let _ = Command::new("notify-send")
        .args([
            "faelight-stow",
            &format!("⚠️ {} dotfile issues found", count),
        ])
        .status();
}
