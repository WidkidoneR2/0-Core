//! faelight-git v2.0 - Git Governance Layer
//! 🌲 Git becomes a policy boundary

use clap::{Parser, Subcommand};
use colored::*;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, exit};

// Import our new library modules
use faelight_git::commands;

#[derive(Parser)]
#[command(name = "faelight-git")]
#[command(about = "🌲 Git Governance for Faelight Forest")]
#[command(version = "2.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show risk-aware repository status (NEW v2.0)
    Status,
    
    /// Show detailed risk assessment (NEW v2.0)
    Risk,
    
    /// Install git hooks
    InstallHooks,
    
    /// Remove git hooks
    RemoveHooks,
    
    /// Verify commit/push readiness
    Verify,
    
    /// Check if core is locked (DEPRECATED - use 'status')
    #[command(hide = true)]
    OldStatus,
    
    /// Pre-commit hook (called by git)
    #[command(hide = true)]
    HookPreCommit,
    
    /// Commit-msg hook (called by git)
    #[command(hide = true)]
    HookCommitMsg { file: String },
    
    /// Pre-push hook (called by git)
    #[command(hide = true)]
    HookPrePush,
}

fn main() {
    let cli = Cli::parse();
    
    let exit_code = match cli.command {
        // NEW v2.0 commands using git2-rs
        Commands::Status => {
            match commands::status::run() {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                    1
                }
            }
        }
        
        Commands::Risk => {
            match commands::risk::run() {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                    1
                }
            }
        }
        
        // Existing v0.1 commands (keep working)
        Commands::InstallHooks => install_hooks(),
        Commands::RemoveHooks => remove_hooks(),
        Commands::Verify => verify(),
        Commands::OldStatus => old_status(),
        Commands::HookPreCommit => hook_pre_commit(),
        Commands::HookCommitMsg { file } => hook_commit_msg(&file),
        Commands::HookPrePush => hook_pre_push(),
    };
    
    exit(exit_code);
}

fn get_core_dir() -> PathBuf {
    let home = env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join("0-core")
}

fn get_hooks_dir() -> PathBuf {
    get_core_dir().join(".git/hooks")
}

fn is_locked() -> bool {
    let home = env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".0-core-locked").exists()
}

// ═══════════════════════════════════════════════════════════
// 🔧 INSTALL/REMOVE HOOKS
// ═══════════════════════════════════════════════════════════

fn install_hooks() -> i32 {
    println!("{}", "🔧 Installing git hooks...".cyan());
    
    let hooks_dir = get_hooks_dir();
    if !hooks_dir.exists() {
        eprintln!("{} .git/hooks directory not found", "Error:".red());
        return 1;
    }

    let hooks = [
        ("pre-commit", PRE_COMMIT_HOOK),
        ("commit-msg", COMMIT_MSG_HOOK),
        ("pre-push", PRE_PUSH_HOOK),
    ];

    for (name, content) in hooks {
        let path = hooks_dir.join(name);
        
        // Backup existing
        if path.exists() {
            let backup = hooks_dir.join(format!("{}.backup", name));
            fs::rename(&path, &backup).ok();
            println!("   {} Backed up existing {}", "📦".yellow(), name);
        }

        // Write new hook
        if let Err(e) = fs::write(&path, content) {
            eprintln!("{} Failed to write {}: {}", "Error:".red(), name, e);
            return 1;
        }

        // Make executable
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).ok();

        println!("   {} Installed {}", "✅".green(), name);
    }

    println!();
    println!("{}", "Git hooks installed! 🌲".green());
    println!();
    println!("Hooks will:");
    println!("  • {} Block commits when core is locked", "pre-commit:".cyan());
    println!("  • {} Suggest intent references", "commit-msg:".cyan());
    println!("  • {} Final verification before push", "pre-push:".cyan());
    
    0
}

fn remove_hooks() -> i32 {
    println!("{}", "🔧 Removing git hooks...".cyan());
    
    let hooks_dir = get_hooks_dir();
    let hooks = ["pre-commit", "commit-msg", "pre-push"];

    for name in hooks {
        let path = hooks_dir.join(name);
        if path.exists() {
            // Check if it's ours
            if let Ok(content) = fs::read_to_string(&path) {
                if content.contains("faelight-git") {
                    fs::remove_file(&path).ok();
                    println!("   {} Removed {}", "✅".green(), name);
                    
                    // Restore backup if exists
                    let backup = hooks_dir.join(format!("{}.backup", name));
                    if backup.exists() {
                        fs::rename(&backup, &path).ok();
                        println!("   {} Restored {}.backup", "📦".yellow(), name);
                    }
                } else {
                    println!("   {} {} is not a faelight hook, skipping", "⚠️".yellow(), name);
                }
            }
        }
    }

    println!();
    println!("{}", "Git hooks removed.".green());
    
    0
}

// ═══════════════════════════════════════════════════════════
// 🔍 VERIFY & STATUS
// ═══════════════════════════════════════════════════════════

fn verify() -> i32 {
    println!("{}", "🔍 Verifying git state...".cyan());
    println!();

    let mut issues = 0;

    // Check lock status
    if is_locked() {
        println!("  {} Core is locked - commits blocked", "❌".red());
        issues += 1;
    } else {
        println!("  {} Core is unlocked", "✅".green());
    }

    // Check for uncommitted changes
    let status = Command::new("git")
        .args(["-C", get_core_dir().to_str().unwrap(), "status", "--porcelain"])
        .output();

    if let Ok(output) = status {
        if output.stdout.is_empty() {
            println!("  {} Working tree clean", "✅".green());
        } else {
            let lines = String::from_utf8_lossy(&output.stdout)
                .lines()
                .count();
            println!("  {} {} uncommitted changes", "⚠️".yellow(), lines);
        }
    }

    // Check for unpushed commits
    let unpushed = Command::new("git")
        .args(["-C", get_core_dir().to_str().unwrap(), "log", "@{u}..", "--oneline"])
        .output();

    if let Ok(output) = unpushed {
        if output.stdout.is_empty() {
            println!("  {} All commits pushed", "✅".green());
        } else {
            let count = String::from_utf8_lossy(&output.stdout)
                .lines()
                .count();
            println!("  {} {} unpushed commits", "⚠️".yellow(), count);
        }
    }

    // Check hooks installed
    let hooks_dir = get_hooks_dir();
    let hooks_installed = ["pre-commit", "commit-msg", "pre-push"]
        .iter()
        .filter(|h| {
            let path = hooks_dir.join(h);
            if let Ok(content) = fs::read_to_string(&path) {
                content.contains("faelight-git")
            } else {
                false
            }
        })
        .count();

    if hooks_installed == 3 {
        println!("  {} All hooks installed", "✅".green());
    } else {
        println!("  {} {}/3 hooks installed", "⚠️".yellow(), hooks_installed);
        println!("     Run: {} to install", "faelight-git install-hooks".cyan());
    }

    println!();
    if issues > 0 {
        println!("{}", "Some issues found. Fix before committing.".red());
        1
    } else {
        println!("{}", "Ready to commit! 🌲".green());
        0
    }
}

fn old_status() -> i32 {
    if is_locked() {
        println!("🔒 Core is {}", "LOCKED".red());
        println!("   Commits are blocked. Run {} first.", "unlock-core".cyan());
        1
    } else {
        println!("🔓 Core is {}", "UNLOCKED".green());
        println!("   Commits are allowed.");
        0
    }
}

// ═══════════════════════════════════════════════════════════
// 🪝 HOOK IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════

fn hook_pre_commit() -> i32 {
    // Check if core is locked
    if is_locked() {
        eprintln!();
        eprintln!("{}", "═══════════════════════════════════════════".red());
        eprintln!("{}", "🔒 COMMIT BLOCKED - Core is locked!".red().bold());
        eprintln!("{}", "═══════════════════════════════════════════".red());
        eprintln!();
        eprintln!("Run {} to unlock before committing.", "unlock-core".cyan());
        eprintln!();
        return 1;
    }

    // Run gitleaks to scan for secrets
    println!("{}", "🔍 Scanning for secrets with gitleaks...".cyan());
    let gitleaks = Command::new("gitleaks")
        .args(["protect", "--verbose", "--staged"])
        .current_dir(get_core_dir())
        .status();

    match gitleaks {
        Ok(status) => {
            if status.success() {
                println!("{}", "✅ No secrets detected".green());
                0
            } else {
                eprintln!();
                eprintln!("{}", "═══════════════════════════════════════════".red());
                eprintln!("{}", "🔒 COMMIT BLOCKED - Secrets detected!".red().bold());
                eprintln!("{}", "═══════════════════════════════════════════".red());
                eprintln!();
                eprintln!("Gitleaks found potential secrets in your staged changes.");
                eprintln!("Remove them before committing.");
                eprintln!();
                1
            }
        }
        Err(e) => {
            eprintln!("{} Failed to run gitleaks: {}", "Error:".red(), e);
            eprintln!("Blocking commit for safety.");
            1
        }
    }
}

fn hook_commit_msg(file: &str) -> i32 {
    let msg = match fs::read_to_string(file) {
        Ok(m) => m,
        Err(_) => return 0,
    };

    let first_line = msg.lines().next().unwrap_or("");
    
    // Check for very short messages
    if first_line.len() < 10 {
        eprintln!();
        eprintln!("{}", "⚠️  Commit message is very short".yellow());
        eprintln!("   Consider being more descriptive.");
        eprintln!();
    }

    // Suggest intent reference for significant changes
    let has_intent = msg.contains("Intent:") || msg.contains("intent:");
    
    let staged = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output();

    if let Ok(output) = staged {
        let files = String::from_utf8_lossy(&output.stdout);
        let significant = files.lines().any(|f| {
            f.starts_with("rust-tools/") ||
            f.starts_with("INTENT/") ||
            f.ends_with("main.rs") ||
            f == "VERSION" ||
            f == "CHANGELOG.md"
        });

        if significant && !has_intent {
            eprintln!();
            eprintln!("{}", "💡 This looks like a significant change.".cyan());
            eprintln!("   Consider adding an intent reference:");
            eprintln!("   {}", "Intent: 0XX".yellow());
            eprintln!();
        }
    }

    0
}

fn hook_pre_push() -> i32 {
    println!("{}", "🔍 Pre-push verification...".cyan());

    let health = Command::new("dot-doctor")
        .arg("--check")
        .arg("git")
        .output();

    if let Ok(output) = health {
        if !output.status.success() {
            eprintln!();
            eprintln!("{}", "⚠️  Health check has warnings".yellow());
            eprintln!("   Run {} for details.", "dot-doctor".cyan());
            eprintln!();
        }
    }

    0
}

// ═══════════════════════════════════════════════════════════
// 📜 HOOK SCRIPTS
// ═══════════════════════════════════════════════════════════

const PRE_COMMIT_HOOK: &str = r#"#!/bin/bash
# Faelight Forest Git Hook - Pre-Commit
# Managed by faelight-git
exec faelight-git hook-pre-commit
"#;

const COMMIT_MSG_HOOK: &str = r#"#!/bin/bash
# Faelight Forest Git Hook - Commit Message
# Managed by faelight-git
exec faelight-git hook-commit-msg "$1"
"#;

const PRE_PUSH_HOOK: &str = r#"#!/bin/bash
# Faelight Forest Git Hook - Pre-Push
# Managed by faelight-git
exec faelight-git hook-pre-push
"#;
