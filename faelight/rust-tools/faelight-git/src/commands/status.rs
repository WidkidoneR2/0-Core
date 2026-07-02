//! Risk-aware git status — shows actual files, not just counts

use crate::git::{FileState, GitRepo};
use crate::is_locked;
use crate::risk::RiskScore;
use anyhow::Result;
use colored::*;

pub fn run() -> Result<()> {
    let repo = GitRepo::open()?;
    let risk = RiskScore::calculate(&repo)?;
    let status = repo.status()?;
    let branch = repo.current_branch()?;
    let upstream = repo.upstream()?;
    let (ahead, behind) = repo.ahead_behind()?;

    // ── Header ────────────────────────────────────────────────
    println!("{}", "🌲 faelight-git status".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    // Branch line
    let branch_line = match (ahead, behind) {
        (0, 0) => format!(" {} {}", "branch".dimmed(), branch.green().bold()),
        (a, 0) => format!(
            " {} {}  {} ahead",
            "branch".dimmed(),
            branch.green().bold(),
            format!("↑{}", a).yellow()
        ),
        (0, b) => format!(
            " {} {}  {} behind",
            "branch".dimmed(),
            branch.green().bold(),
            format!("↓{}", b).red()
        ),
        (a, b) => format!(
            " {} {}  {} ahead  {} behind",
            "branch".dimmed(),
            branch.green().bold(),
            format!("↑{}", a).yellow(),
            format!("↓{}", b).red()
        ),
    };
    println!("{}", branch_line);

    // Upstream line
    if let Some(up) = upstream {
        println!(" {} {}", "upstream".dimmed(), up.dimmed());
    }

    // Lock line
    if is_locked() {
        println!(
            " {} {}",
            "core".dimmed(),
            "🔒 LOCKED — commits blocked".red()
        );
    }

    if status.is_empty() {
        println!();
        println!("{}", "  ✅ Working tree clean".green());
        println!("{}", "━".repeat(52).dimmed());
        print_risk(&risk);
        return Ok(());
    }

    // ── Staged ────────────────────────────────────────────────
    let staged = status.staged_files();
    if !staged.is_empty() {
        println!();
        println!("{}", "  Staged".green().bold());
        for f in &staged {
            let label = match &f.state {
                FileState::StagedRenamed(new) => {
                    format!("  {} {} → {}", "●".green(), f.path.dimmed(), new.green())
                }
                _ => format!(
                    "  {} {} {}",
                    "●".green(),
                    f.state.label().dimmed(),
                    f.path.green()
                ),
            };
            println!("{}", label);
        }
    }

    // ── Unstaged ──────────────────────────────────────────────
    let unstaged = status.unstaged_files();
    if !unstaged.is_empty() {
        println!();
        println!("{}", "  Unstaged".yellow().bold());
        for f in &unstaged {
            println!(
                "  {} {} {}",
                "○".yellow(),
                f.state.label().dimmed(),
                f.path.yellow()
            );
        }
    }

    // ── Untracked ─────────────────────────────────────────────
    let untracked = status.untracked_files();
    if !untracked.is_empty() {
        println!();
        println!("{}", "  Untracked".dimmed().bold());
        for f in &untracked {
            println!("  {} {}", "?".dimmed(), f.path.dimmed());
        }
    }

    // ── Summary ───────────────────────────────────────────────
    println!();
    println!("{}", "━".repeat(52).dimmed());

    let mut parts = Vec::new();
    if status.staged > 0 {
        parts.push(format!("{} staged", status.staged).green().to_string());
    }
    if status.modified > 0 {
        parts.push(format!("{} unstaged", status.modified).yellow().to_string());
    }
    if status.untracked > 0 {
        parts.push(
            format!("{} untracked", status.untracked)
                .dimmed()
                .to_string(),
        );
    }
    println!("  {}", parts.join("  "));

    // ── Hints ─────────────────────────────────────────────────
    if status.staged == 0 && status.modified > 0 {
        println!(
            "  {} {}",
            "hint:".dimmed(),
            "fg sync to stage and commit".dimmed()
        );
    } else if status.staged > 0 {
        println!(
            "  {} {}",
            "hint:".dimmed(),
            "faelight-git commit to commit staged files".dimmed()
        );
    }

    print_risk(&risk);

    Ok(())
}

fn print_risk(risk: &crate::risk::RiskScore) {
    println!(
        "  {} {} {}",
        "risk".dimmed(),
        risk.emoji(),
        format!("{}/100", risk.total).color(risk.color()).bold()
    );

    for factor in &risk.breakdown {
        let sign = if factor.delta > 0 { "↑" } else { "↓" };
        println!(
            "    {} {} {}",
            sign.dimmed(),
            factor.name.dimmed(),
            format!("{:+}", factor.delta).color(risk.color()).dimmed()
        );
    }
}
