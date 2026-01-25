//! Branch management
use anyhow::Result;
use colored::*;
use std::io::{self, Write};
use std::process::Command;

pub fn run() -> Result<()> {
    println!("{}", "🌲 Faelight Git Branch Manager".cyan().bold());
    println!("{}", "━".repeat(50));
    println!();
    
    // Get current branch
    let current = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()?;
    let current_branch = String::from_utf8_lossy(&current.stdout).trim().to_string();
    
    // List all branches
    let branches = Command::new("git")
        .args(&["branch", "-a"])
        .output()?;
    
    println!("{}", "📋 Branches:".yellow().bold());
    println!();
    
    for line in String::from_utf8_lossy(&branches.stdout).lines() {
        let branch = line.trim();
        if branch.starts_with("*") {
            // Current branch
            let name = branch.trim_start_matches("* ");
            println!("  {} {}", "→".green().bold(), name.green().bold());
        } else if branch.starts_with("remotes/") {
            // Remote branch
            let name = branch.trim_start_matches("remotes/");
            println!("  {} {}", " ".dimmed(), name.dimmed());
        } else {
            // Local branch
            println!("  {} {}", " ", branch);
        }
    }
    
    println!();
    println!("{}", "━".repeat(50));
    println!();
    
    // Operations menu
    println!("What would you like to do?");
    println!("  {} Switch to a branch", "1)".cyan());
    println!("  {} Create new branch", "2)".cyan());
    println!("  {} Delete a branch", "3)".cyan());
    println!("  {} Exit", "4)".cyan());
    println!();
    
    print!("Choice (1-4): ");
    io::stdout().flush()?;
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    
    match choice.trim() {
        "1" => switch_branch(&current_branch)?,
        "2" => create_branch()?,
        "3" => delete_branch(&current_branch)?,
        "4" => println!("{}", "👋 Exiting...".cyan()),
        _ => println!("{}", "❌ Invalid choice".red()),
    }
    
    Ok(())
}

fn switch_branch(current: &str) -> Result<()> {
    println!();
    print!("Enter branch name to switch to: ");
    io::stdout().flush()?;
    
    let mut branch = String::new();
    io::stdin().read_line(&mut branch)?;
    let branch = branch.trim();
    
    if branch.is_empty() {
        println!("{}", "⚠️  No branch specified".yellow());
        return Ok(());
    }
    
    if branch == current {
        println!("{}", format!("ℹ️  Already on branch '{}'", branch).cyan());
        return Ok(());
    }
    
    let result = Command::new("git")
        .args(&["checkout", branch])
        .status()?;
    
    if result.success() {
        println!("{}", format!("✅ Switched to branch '{}'", branch).green());
    } else {
        println!("{}", format!("❌ Failed to switch to '{}'", branch).red());
    }
    
    Ok(())
}

fn create_branch() -> Result<()> {
    println!();
    print!("Enter new branch name: ");
    io::stdout().flush()?;
    
    let mut branch = String::new();
    io::stdin().read_line(&mut branch)?;
    let branch = branch.trim();
    
    if branch.is_empty() {
        println!("{}", "⚠️  No branch name specified".yellow());
        return Ok(());
    }
    
    print!("Switch to new branch after creating? (y/n): ");
    io::stdout().flush()?;
    
    let mut switch = String::new();
    io::stdin().read_line(&mut switch)?;
    
    let args = if switch.trim().to_lowercase() == "y" {
        vec!["checkout", "-b", branch]
    } else {
        vec!["branch", branch]
    };
    
    let result = Command::new("git")
        .args(&args)
        .status()?;
    
    if result.success() {
        println!("{}", format!("✅ Created branch '{}'", branch).green());
    } else {
        println!("{}", format!("❌ Failed to create '{}'", branch).red());
    }
    
    Ok(())
}

fn delete_branch(current: &str) -> Result<()> {
    println!();
    print!("Enter branch name to delete: ");
    io::stdout().flush()?;
    
    let mut branch = String::new();
    io::stdin().read_line(&mut branch)?;
    let branch = branch.trim();
    
    if branch.is_empty() {
        println!("{}", "⚠️  No branch specified".yellow());
        return Ok(());
    }
    
    if branch == current {
        println!("{}", "❌ Cannot delete current branch!".red());
        return Ok(());
    }
    
    print!("Are you sure you want to delete '{}'? (y/n): ", branch);
    io::stdout().flush()?;
    
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    
    if confirm.trim().to_lowercase() != "y" {
        println!("{}", "⚠️  Delete cancelled".yellow());
        return Ok(());
    }
    
    let result = Command::new("git")
        .args(&["branch", "-d", branch])
        .status()?;
    
    if result.success() {
        println!("{}", format!("✅ Deleted branch '{}'", branch).green());
    } else {
        println!("{}", "⚠️  Branch has unmerged changes. Use -D to force delete.".yellow());
    }
    
    Ok(())
}
