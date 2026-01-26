use anyhow::{Context, Result};
use colored::Colorize;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
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
        // Get repo name
        let repo_output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()?;
        let repo_path = String::from_utf8_lossy(&repo_output.stdout).trim().to_string();
        let repo_name = std::path::Path::new(&repo_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repository");

        println!();
        println!("{}", format!("⚠️  Pushing directly to MAIN in {}", repo_name).yellow().bold());
        print!("Proceed? (type 'push-main'): ");
        io::stdout().flush()?;

        // Read from /dev/tty instead of stdin (git hooks need this)
        let tty = File::open("/dev/tty").context("Failed to open /dev/tty")?;
        let mut reader = BufReader::new(tty);
        let mut input = String::new();
        reader.read_line(&mut input)?;

        if input.trim() != "push-main" {
            println!("{}", "❌ Push cancelled".red());
            return Ok(false);
        }
        
        println!();
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
