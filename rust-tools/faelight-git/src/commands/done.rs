//! fg done -- stage all, commit with active intent prefix, push. No prompts.
//! INT-256: the fastest path from work to pushed commit.
use crate::git::GitRepo;
use crate::is_locked;
use anyhow::{bail, Result};
use colored::*;
pub fn run(extra: Option<&str>) -> Result<()> {
    if is_locked() {
        bail!("Core is locked. Run 'unlock-core' first.");
    }
    let repo = GitRepo::open()?;
    let status = repo.status()?;
    if status.is_empty() {
        println!("{}", "  ✅ Nothing to commit -- tree is clean".green());
        return Ok(());
    }
    println!("{}", "🌲 fg done".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    // Show staged files
    for f in &status.files {
        println!("  {} {}", f.state.symbol().yellow(), f.path.dimmed());
    }
    println!();
    // Build commit message from active intent
    let message = if let Some((id, title)) = super::get_active_intent() {
        if let Some(extra) = extra {
            format!("INT-{}: {} -- {}", id, title, extra)
        } else {
            format!("INT-{}: {}", id, title)
        }
    } else if let Some(extra) = extra {
        extra.to_string()
    } else {
        bail!("No active intent and no message provided. Use: fg done \"message\"");
    };
    // Stage all
    repo.stage_all()?;
    println!("  {} All changes staged", "✅".green());
    // Commit
    let hash = repo.commit(&message)?;
    println!(
        "  {} {} {}",
        "✅".green(),
        hash[..8.min(hash.len())].yellow().bold(),
        message.white()
    );
    // INT-312/INT-071: record into intent_commits via shared recorder
    super::record_commit(&hash, &message);
    // Push
    println!("  {} Pushing...", "→".cyan());
    let push = std::process::Command::new("git").arg("push").status()?;
    if push.success() {
        println!("{}", "  🚀 Pushed to origin".green().bold());
    } else {
        println!("{}", "  ❌ Push failed -- run 'git push' manually".red());
    }
    println!("{}", "━".repeat(52).dimmed());
    println!("{}", "  🌲 The forest remembers.".dimmed());
    Ok(())
}
