//! Branch management — native git2

use crate::git::GitRepo;
use anyhow::Result;
use colored::*;
use std::io::{self, Write};
use std::process::Command;

pub fn run() -> Result<()> {
    let repo = GitRepo::open()?;
    show_branches(&repo)?;

    println!();
    println!("  {} switch      {} create      {} delete      {} cancel",
        "[s]".cyan().bold(),
        "[c]".cyan().bold(),
        "[d]".red().bold(),
        "[q]".dimmed(),
    );
    println!();
    print!("  Action: ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;

    match choice.trim() {
        "s" | "switch" => switch_branch()?,
        "c" | "create" => create_branch()?,
        "d" | "delete" => delete_branch(&repo)?,
        "q" | "" => {}
        _ => println!("{}", "  ❌ Unknown action".red()),
    }

    Ok(())
}

fn show_branches(repo: &GitRepo) -> Result<()> {
    println!("{}", "🌲 faelight-git branch".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    let current = repo.current_branch()?;
    let (ahead, behind) = repo.ahead_behind()?;

    // Get all branches via git2 by shelling — git2 branch listing is verbose,
    // this gives us remote tracking info cleanly
    let output = Command::new("git")
        .args(["branch", "-a", "--format=%(refname:short)|%(upstream:short)|%(upstream:track)"])
        .output()?;

    let raw = String::from_utf8_lossy(&output.stdout);
    let mut locals: Vec<String> = Vec::new();
    let mut remotes: Vec<String> = Vec::new();

    for line in raw.lines() {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        let name = parts[0].trim();

        if name.starts_with("origin/") || name.starts_with("remotes/") {
            // Skip remote entries that mirror a local branch
            let short = name.trim_start_matches("origin/").trim_start_matches("remotes/origin/");
            if !locals.contains(&short.to_string()) {
                remotes.push(name.to_string());
            }
            continue;
        }

        let upstream = parts.get(1).map(|s| s.trim()).unwrap_or("");
        let track    = parts.get(2).map(|s| s.trim()).unwrap_or("");

        let is_current = name == current;

        let track_info = if track.is_empty() {
            String::new()
        } else {
            format!(" {}", track.yellow())
        };

        let upstream_info = if upstream.is_empty() {
            format!(" {}", "no upstream".dimmed())
        } else {
            format!(" → {}", upstream.dimmed())
        };

        if is_current {
            // Current branch — show sync status
            let sync = match (ahead, behind) {
                (0, 0) => "✓ synced".green().to_string(),
                (a, 0) => format!("↑{} ahead", a).yellow().to_string(),
                (0, b) => format!("↓{} behind", b).red().to_string(),
                (a, b) => format!("↑{} ↓{}", a, b).red().to_string(),
            };
            println!(
                "  {} {} {}{}",
                "►".green().bold(),
                name.green().bold(),
                sync,
                upstream_info,
            );
        } else {
            println!(
                "  {} {}{}{}",
                " ".dimmed(),
                name.white(),
                upstream_info,
                track_info,
            );
        }

        locals.push(name.to_string());
    }

    // Show remotes that have no local counterpart
    if !remotes.is_empty() {
        println!();
        println!("{}", "  Remote only".dimmed());
        for r in &remotes {
            println!("  {} {}", " ".dimmed(), r.dimmed());
        }
    }

    Ok(())
}

fn switch_branch() -> Result<()> {
    println!();
    print!("  Switch to branch: ");
    io::stdout().flush()?;

    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let name = name.trim();

    if name.is_empty() {
        println!("{}", "  ⚠️  Cancelled".yellow());
        return Ok(());
    }

    // Check for uncommitted changes first
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;

    if !status.stdout.is_empty() {
        println!("{}", "  ⚠️  You have uncommitted changes".yellow());
        print!("  Stash them before switching? (y/n): ");
        io::stdout().flush()?;
        let mut ans = String::new();
        io::stdin().read_line(&mut ans)?;
        if ans.trim().to_lowercase() == "y" {
            Command::new("git").args(["stash"]).status()?;
            println!("{}", "  ✅ Changes stashed".green());
        }
    }

    let result = Command::new("git")
        .args(["checkout", name])
        .status()?;

    if result.success() {
        println!("{}", format!("  ✅ Switched to '{}'", name).green().bold());
    } else {
        // Try creating from remote
        let remote = format!("origin/{}", name);
        let result2 = Command::new("git")
            .args(["checkout", "-b", name, "--track", &remote])
            .status()?;
        if result2.success() {
            println!("{}", format!("  ✅ Checked out '{}' from remote", name).green().bold());
        } else {
            println!("{}", format!("  ❌ Branch '{}' not found", name).red());
        }
    }

    Ok(())
}

fn create_branch() -> Result<()> {
    println!();
    print!("  New branch name: ");
    io::stdout().flush()?;

    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let name = name.trim();

    if name.is_empty() {
        println!("{}", "  ⚠️  Cancelled".yellow());
        return Ok(());
    }

    print!("  Switch to it immediately? (y/n): ");
    io::stdout().flush()?;
    let mut ans = String::new();
    io::stdin().read_line(&mut ans)?;

    let args = if ans.trim().to_lowercase() == "y" {
        vec!["checkout", "-b", name]
    } else {
        vec!["branch", name]
    };

    let result = Command::new("git").args(&args).status()?;

    if result.success() {
        if ans.trim().to_lowercase() == "y" {
            println!("{}", format!("  ✅ Created and switched to '{}'", name).green().bold());
        } else {
            println!("{}", format!("  ✅ Created branch '{}'", name).green().bold());
        }
    } else {
        println!("{}", format!("  ❌ Failed to create '{}'", name).red());
    }

    Ok(())
}

fn delete_branch(repo: &GitRepo) -> Result<()> {
    let current = repo.current_branch()?;

    println!();
    print!("  Branch to delete: ");
    io::stdout().flush()?;

    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let name = name.trim();

    if name.is_empty() {
        println!("{}", "  ⚠️  Cancelled".yellow());
        return Ok(());
    }

    if name == current {
        println!("{}", "  ❌ Cannot delete current branch".red());
        return Ok(());
    }

    if name == "main" || name == "master" {
        println!("{}", "  ❌ Refusing to delete main/master".red().bold());
        return Ok(());
    }

    print!("  {} Delete '{}'? (y/n): ", "⚠️ ".yellow(), name);
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    if confirm.trim().to_lowercase() != "y" {
        println!("{}", "  ⚠️  Delete cancelled".yellow());
        return Ok(());
    }

    let result = Command::new("git")
        .args(["branch", "-d", name])
        .status()?;

    if result.success() {
        println!("{}", format!("  ✅ Deleted '{}'", name).green());
    } else {
        println!("{}", "  ⚠️  Branch has unmerged changes".yellow());
        print!("  Force delete? (y/n): ");
        io::stdout().flush()?;
        let mut force = String::new();
        io::stdin().read_line(&mut force)?;
        if force.trim().to_lowercase() == "y" {
            Command::new("git").args(["branch", "-D", name]).status()?;
            println!("{}", format!("  ✅ Force deleted '{}'", name).green());
        }
    }

    Ok(())
}
