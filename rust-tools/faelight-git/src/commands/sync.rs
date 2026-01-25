//! Interactive git sync workflow
use anyhow::Result;
use colored::*;
use std::io::{self, Write};
use std::process::Command;

pub fn run() -> Result<()> {
    println!("{}", "🌲 Faelight Git Sync v3.0".cyan().bold());
    println!("{}", "━".repeat(50));
    println!();
    
    // Phase 1: Pull Latest
    println!("{}", "📥 Phase 1: Pull Latest Changes".yellow().bold());
    print!("  🔄 Pulling from origin... ");
    io::stdout().flush()?;
    
    let pull = Command::new("git")
        .args(&["pull"])
        .output()?;
    
    if pull.status.success() {
        let output = String::from_utf8_lossy(&pull.stdout);
        if output.contains("Already up to date") {
            println!("{}", "✅ Already up to date".green());
        } else {
            println!("{}", "✅ Pulled successfully".green());
        }
    } else {
        println!("{}", "⚠️  Pull had conflicts or issues".yellow());
    }
    println!();
    
    // Phase 2: Repository Status
    println!("{}", "📊 Phase 2: Repository Status".yellow().bold());
    
    let status = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()?;
    
    if status.stdout.is_empty() {
        println!("{}", "  ✅ No changes to commit".green());
        println!();
        println!("{}", "━".repeat(50));
        println!("{}", "ℹ️  Working tree is clean!".cyan());
        return Ok(());
    }
    
    // Parse and show changes
    let changes = String::from_utf8_lossy(&status.stdout);
    let mut modified = 0;
    let mut added = 0;
    let mut deleted = 0;
    
    for line in changes.lines() {
        if line.starts_with(" M") || line.starts_with("M ") {
            modified += 1;
        } else if line.starts_with("A ") || line.starts_with("??") {
            added += 1;
        } else if line.starts_with(" D") {
            deleted += 1;
        }
    }
    
    println!("  📝 Modified: {}", modified.to_string().yellow());
    println!("  ➕ Added: {}", added.to_string().green());
    if deleted > 0 {
        println!("  ➖ Deleted: {}", deleted.to_string().red());
    }
    println!();
    
    // Phase 3: Stage Changes
    println!("{}", "📝 Phase 3: Stage Changes".yellow().bold());
    print!("  ❓ Stage all changes? (y/n): ");
    io::stdout().flush()?;
    
    let mut response = String::new();
    io::stdin().read_line(&mut response)?;
    
    if response.trim().to_lowercase() != "y" {
        println!("{}", "  ⚠️  Sync cancelled".yellow());
        return Ok(());
    }
    
    let stage = Command::new("git")
        .args(&["add", "-A"])
        .status()?;
    
    if !stage.success() {
        anyhow::bail!("Failed to stage changes");
    }
    println!("{}", "  ✅ All changes staged".green());
    println!();
    
    // Phase 4: Commit Message
    println!("{}", "💬 Phase 4: Commit Message".yellow().bold());
    print!("  ❓ Enter commit message: ");
    io::stdout().flush()?;
    
    let mut message = String::new();
    io::stdin().read_line(&mut message)?;
    let message = message.trim();
    
    if message.is_empty() {
        println!("{}", "  ⚠️  Empty message, sync cancelled".yellow());
        return Ok(());
    }
    
    // Preview
    println!();
    println!("{}", "  Preview:".cyan());
    println!("{}", "  ━".repeat(25));
    println!("  {}", message);
    println!();
    println!("  Files: {} modified, {} added", modified, added);
    println!("{}", "  ━".repeat(25));
    println!();
    
    print!("  ❓ Create commit? (y/n): ");
    io::stdout().flush()?;
    
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    
    if confirm.trim().to_lowercase() != "y" {
        println!("{}", "  ⚠️  Commit cancelled".yellow());
        return Ok(());
    }
    
    let commit = Command::new("git")
        .args(&["commit", "-m", message])
        .status()?;
    
    if !commit.success() {
        anyhow::bail!("Failed to commit");
    }
    println!("{}", "  ✅ Commit created".green());
    println!();
    
    // Phase 5: Push
    println!("{}", "📤 Phase 5: Push to Remote".yellow().bold());
    print!("  ❓ Push to origin? (y/n): ");
    io::stdout().flush()?;
    
    let mut push_confirm = String::new();
    io::stdin().read_line(&mut push_confirm)?;
    
    if push_confirm.trim().to_lowercase() != "y" {
        println!("{}", "  ⚠️  Push skipped".yellow());
        println!();
        println!("{}", "━".repeat(50));
        println!("{}", "✅ Changes committed locally".green());
        return Ok(());
    }
    
    let push = Command::new("git")
        .args(&["push"])
        .status()?;
    
    if !push.success() {
        anyhow::bail!("Failed to push");
    }
    println!("{}", "  🚀 Pushed to origin".green());
    
    println!();
    println!("{}", "━".repeat(50));
    println!("{}", "🎉 Sync Complete!".green().bold());
    println!("{}", "🌲 The forest stays in harmony.".cyan());
    println!("{}", "━".repeat(50));
    
    Ok(())
}
