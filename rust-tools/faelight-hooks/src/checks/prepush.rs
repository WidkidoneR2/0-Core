use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

pub fn check_push_to_main() -> Result<bool> {
    println!("{}", "🎣 Checking push target...".cyan());

    // Get current branch
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to get current branch")?;

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if branch == "main" || branch == "master" {
        println!();
        println!("{}", "⚠️  PUSHING DIRECTLY TO MAIN".yellow().bold());
        println!();
        println!("Current branch: {}", branch.yellow());
        println!();
        println!("{}", "This is a protected branch.".yellow());
        println!("{}", "Are you sure you want to continue?".yellow());
        println!();
        
        // This just warns - the actual confirmation is in the hook script
        return Ok(true);
    }

    println!("{}", format!("✅ Pushing to branch: {}", branch).green());
    Ok(true)
}

pub fn check_unpushed_changes() -> Result<bool> {
    println!("{}", "🔍 Checking for uncommitted changes...".cyan());

    // Check for uncommitted changes
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;

    let status = String::from_utf8_lossy(&output.stdout);

    if !status.trim().is_empty() {
        println!();
        println!("{}", "⚠️  UNCOMMITTED CHANGES DETECTED".yellow().bold());
        println!();
        println!("You have uncommitted changes that won't be pushed:");
        println!("{}", status.dimmed());
        println!("{}", "Consider committing them first!".yellow());
        println!();
        return Ok(true); // Warning, not error
    }

    println!("{}", "✅ No uncommitted changes".green());
    Ok(true)
}
