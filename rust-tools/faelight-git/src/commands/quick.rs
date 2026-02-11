//! Quick commit and push workflow
use anyhow::Result;
use colored::*;
use std::process::Command;

pub fn run(message: &str) -> Result<()> {
    println!("{}", "🌲 Faelight Git Quick Commit".cyan().bold());
    println!("{}", "━".repeat(50));
    println!();

    // Check if there are changes
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;

    if status.stdout.is_empty() {
        println!("{}", "  ℹ️  No changes to commit".yellow());
        return Ok(());
    }

    // Stage all changes
    let stage = Command::new("git").args(["add", "-A"]).status()?;

    if !stage.success() {
        anyhow::bail!("Failed to stage changes\n💡 Check: git status shows files?\n💡 Try: git add . manually");
    }
    println!("{}", "  ✅ Changes staged".green());

    // Commit
    let commit = Command::new("git")
        .args(["commit", "-m", message])
        .status()?;

    if !commit.success() {
        anyhow::bail!("Failed to commit\n💡 Check: Commit message format correct?\n💡 Try: git commit manually");
    }
    println!("{}", format!("  ✅ Commit: {}", message).green());

    // Push
    let push = Command::new("git").args(["push"]).status()?;

    if !push.success() {
        anyhow::bail!("Failed to push\n💡 Check: Upstream branch exists?\n💡 Try: git push -u origin <branch>");
    }
    println!("{}", "  🚀 Pushed to origin".green());

    println!();
    println!("{}", "━".repeat(50));
    println!("{}", "🎉 Quick commit complete!".green().bold());

    Ok(())
}
