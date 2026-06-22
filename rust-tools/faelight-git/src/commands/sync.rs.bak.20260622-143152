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

    let pull = Command::new("git").args(["pull"]).output()?;

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

    // Phase 2: Repository Status with ACTUAL FILES
    println!("{}", "📊 Phase 2: Repository Status".yellow().bold());

    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;

    if status.stdout.is_empty() {
        println!("{}", "  ✅ No changes to commit".green());
        println!();
        println!("{}", "━".repeat(50));
        println!("{}", "ℹ️  Working tree is clean!".cyan());
        return Ok(());
    }

    // Parse and show ACTUAL FILES with colors!
    let changes = String::from_utf8_lossy(&status.stdout);
    let mut modified_files = Vec::new();
    let mut added_files = Vec::new();
    let mut deleted_files = Vec::new();

    for line in changes.lines() {
        let status_code = &line[0..2];
        let filename = line[3..].trim();

        match status_code {
            " M" | "M " | "MM" => modified_files.push(filename),
            "A " | "??" => added_files.push(filename),
            " D" | "D " => deleted_files.push(filename),
            _ => {}
        }
    }

    // Display files with beautiful colors!
    if !modified_files.is_empty() {
        println!("  {} Modified files:", "📝".yellow());
        for file in &modified_files {
            println!("    {} {}", "M".yellow().bold(), file.yellow());
        }
    }

    if !added_files.is_empty() {
        println!("  {} New files:", "➕".green());
        for file in &added_files {
            println!("    {} {}", "+".green().bold(), file.green());
        }
    }

    if !deleted_files.is_empty() {
        println!("  {} Deleted files:", "➖".red());
        for file in &deleted_files {
            println!("    {} {}", "D".red().bold(), file.red());
        }
    }

    println!();
    println!(
        "  Summary: {} modified, {} added, {} deleted",
        modified_files.len().to_string().yellow(),
        added_files.len().to_string().green(),
        deleted_files.len().to_string().red()
    );
    println!();

    // Phase 2.5: Show diff preview?
    print!("  ❓ Show diff preview? (y/n): ");
    io::stdout().flush()?;

    let mut preview = String::new();
    io::stdin().read_line(&mut preview)?;

    if preview.trim().to_lowercase() == "y" {
        println!();
        println!("{}", "  📄 Diff Preview:".cyan().bold());
        println!("{}", "  ━".repeat(25));

        let diff = Command::new("git")
            .args(["diff", "--color=always", "--stat"])
            .output()?;

        if diff.status.success() {
            print!("{}", String::from_utf8_lossy(&diff.stdout));
        }
        println!("{}", "  ━".repeat(25));
        println!();
    }

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

    let stage = Command::new("git").args(["add", "-A"]).status()?;

    if !stage.success() {
        anyhow::bail!("Failed to stage changes\n💡 Check: git status shows files?\n💡 Try: git add . manually");
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

    // Preview with file counts
    println!();
    println!("{}", "  Preview:".cyan());
    println!("{}", "  ━".repeat(25));
    println!("  {}", message);
    println!();
    println!(
        "  Files: {} modified, {} added, {} deleted",
        modified_files.len(),
        added_files.len(),
        deleted_files.len()
    );
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
        .args(["commit", "-m", message])
        .status()?;

    if !commit.success() {
        anyhow::bail!(
            "Failed to commit\n💡 Check: Commit message not empty?\n💡 Try: git commit manually"
        );
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

    let push = Command::new("git").args(["push"]).status()?;

    if !push.success() {
        anyhow::bail!("Failed to push\n💡 Check: Remote exists?\n💡 Try: git remote -v");
    }
    println!("{}", "  🚀 Pushed to origin".green());

    println!();
    println!("{}", "━".repeat(50));
    println!("{}", "🎉 Sync Complete!".green().bold());
    println!("{}", "🌲 The forest stays in harmony.".cyan());
    println!("{}", "━".repeat(50));

    Ok(())
}
