//! Quick commit and push — native git2, no shell-out for git ops

use crate::git::GitRepo;
use crate::is_locked;
use anyhow::{bail, Result};
use colored::*;

pub fn run(message: &str) -> Result<()> {
    let repo = GitRepo::open()?;

    if is_locked() {
        bail!("Core is locked. Run 'unlock-core' first.");
    }

    let status = repo.status()?;
    if status.is_empty() {
        println!("{}", "  ✅ Nothing to commit".green());
        return Ok(());
    }

    println!("{}", "🌲 faelight-git quick".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    // Show what's being committed
    for f in &status.files {
        println!("  {} {}", f.state.symbol().yellow(), f.path.yellow());
    }
    println!();

    // Stage all
    repo.stage_all()?;
    println!("{}", "  ✅ Staged".green());

    // Commit
    let hash = repo.commit(message)?;
    println!(
        "  {} {} {}",
        "✅".green(),
        hash.yellow().bold(),
        message.white()
    );

    // Push
    println!("  {} Pushing...", "→".cyan());
    let push = std::process::Command::new("git").arg("push").status()?;

    if push.success() {
        println!("{}", "  🚀 Pushed".green().bold());
    } else {
        println!("{}", "  ❌ Push failed — run 'git push' manually".red());
    }

    println!("{}", "━".repeat(52).dimmed());

    Ok(())
}
