use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn check_conflicts() -> Result<bool> {
    println!("{}", "🔍 Checking for merge conflicts...".cyan());

    // Get staged files
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .context("Failed to get staged files")?;

    let files: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();

    if files.is_empty() {
        println!("{}", "  ℹ️  No staged files".dimmed());
        return Ok(true);
    }

    let conflict_regex = Regex::new(r"^(<{7}|={7}|>{7})").unwrap();
    let mut conflicts_found = false;

    for file in files {
        let path = PathBuf::from(&file);
        
        // Skip binary files and files that don't exist
        if !path.exists() {
            continue;
        }

        // Try to read as text
        let contents = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue, // Skip binary files
        };

        // Check for conflict markers
        for (line_num, line) in contents.lines().enumerate() {
            if conflict_regex.is_match(line) {
                if !conflicts_found {
                    println!();
                    println!("{}", "❌ MERGE CONFLICT MARKERS DETECTED!".red().bold());
                    println!();
                }
                conflicts_found = true;
                println!("  {}:{} {}", 
                    file.yellow(), 
                    (line_num + 1).to_string().yellow(),
                    line.red()
                );
            }
        }
    }

    if conflicts_found {
        println!();
        println!("{}", "⚠️  Please resolve merge conflicts before committing!".yellow());
        println!();
        return Ok(false);
    }

    println!("{}", "✅ No merge conflicts detected".green());
    Ok(true)
}
