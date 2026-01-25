//! Commit history viewer
use anyhow::Result;
use colored::*;
use std::io::{self, Write};
use std::process::Command;

pub fn run(count: Option<usize>) -> Result<()> {
    let n = count.unwrap_or(10);
    
    println!("{}", "🌲 Faelight Git Commit History".cyan().bold());
    println!("{}", "━".repeat(50));
    println!();
    
    // Get commit history with pretty format
    let output = Command::new("git")
        .args(&[
            "log",
            &format!("-{}", n),
            "--pretty=format:%h|%an|%ar|%s",
            "--color=never"
        ])
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!("Failed to get git log");
    }
    
    let log = String::from_utf8_lossy(&output.stdout);
    
    if log.is_empty() {
        println!("{}", "  ℹ️  No commits found".yellow());
        return Ok(());
    }
    
    // Parse and display commits
    for (i, line) in log.lines().enumerate() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 4 {
            continue;
        }
        
        let hash = parts[0];
        let author = parts[1];
        let time = parts[2];
        let message = parts[3];
        
        // Display with beautiful colors
        println!("{} {} {} {}", 
            hash.yellow().bold(),
            time.dimmed(),
            author.cyan(),
            message.white()
        );
        
        if i < log.lines().count() - 1 {
            println!("{}", "│".dimmed());
        }
    }
    
    println!();
    println!("{}", "━".repeat(50));
    
    // Show stats
    let stats = Command::new("git")
        .args(&["log", &format!("-{}", n), "--oneline"])
        .output()?;
    
    let commit_count = String::from_utf8_lossy(&stats.stdout).lines().count();
    println!("Showing {} commits", commit_count.to_string().cyan());
    
    println!();
    print!("Show detailed diff for a commit? (enter hash or 'n'): ");
    io::stdout().flush()?;
    
    let mut response = String::new();
    io::stdin().read_line(&mut response)?;
    let response = response.trim();
    
    if response != "n" && !response.is_empty() {
        show_commit_diff(response)?;
    }
    
    Ok(())
}

fn show_commit_diff(hash: &str) -> Result<()> {
    println!();
    println!("{}", format!("📄 Commit {} Details:", hash).cyan().bold());
    println!("{}", "━".repeat(50));
    println!();
    
    // Show commit details
    let show = Command::new("git")
        .args(&["show", "--stat", "--color=always", hash])
        .output()?;
    
    if show.status.success() {
        print!("{}", String::from_utf8_lossy(&show.stdout));
    } else {
        println!("{}", format!("❌ Commit '{}' not found", hash).red());
    }
    
    Ok(())
}
