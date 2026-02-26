#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::process::Command;

fn git(args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn git_err(args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stderr).trim().to_string())
        .unwrap_or_default()
}

fn in_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn status(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require(
        "git",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
            Capability::SpawnProcess,
        ],
    )?;
    if !in_git_repo() {
        println!("  {} Not in a git repository", "✗".bright_red());
        return Ok(());
    }

    let branch = git(&["branch", "--show-current"]);
    let modified = git(&["diff", "--name-only"])
        .lines()
        .filter(|l| !l.is_empty())
        .count();
    let untracked = git(&["ls-files", "--others", "--exclude-standard"])
        .lines()
        .filter(|l| !l.is_empty())
        .count();
    let staged = git(&["diff", "--cached", "--name-only"])
        .lines()
        .filter(|l| !l.is_empty())
        .count();
    let ahead_behind = git(&["rev-list", "--left-right", "--count", "HEAD...@{upstream}"]);
    let (ahead, behind) = parse_ahead_behind(&ahead_behind);

    // Risk score
    let risk = calc_risk(modified, untracked, staged, ahead, behind);
    let risk_label = risk_label(risk);

    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!("{}", "🌲 Git Status — Faelight Forest".bold());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!("Branch: {}", branch.bright_cyan());
    println!("Upstream: {}", "origin/main".dimmed());
    println!("Working Tree:");
    println!("  {} Modified files: {}", "•".dimmed(), modified);
    println!("  {} Staged files: {}", "•".dimmed(), staged);
    println!("  {} Untracked files: {}", "•".dimmed(), untracked);
    println!("Commits:");
    println!("  {} Ahead: {}", "•".dimmed(), ahead);
    println!("  {} Behind: {}", "•".dimmed(), behind);
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!("Risk Score: {} {} / 100", risk_label, risk);
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );

    // Event Ledger
    let writer = crate::runtime::EventWriter::new(&ctx.runtime.db);
    writer.write(
        "git",
        "status",
        "core git status",
        "ok",
        Some(&format!(r#"{{"branch":"{}","modified":{},"staged":{},"risk":{}}}"#,
            branch, modified, staged, risk)),
    );

    Ok(())
}

pub fn risk(_ctx: &AppContext) -> CoreResult<()> {
    if !in_git_repo() {
        println!("  {} Not in a git repository", "✗".bright_red());
        return Ok(());
    }

    let modified = git(&["diff", "--name-only"])
        .lines()
        .filter(|l| !l.is_empty())
        .count();
    let untracked = git(&["ls-files", "--others", "--exclude-standard"])
        .lines()
        .filter(|l| !l.is_empty())
        .count();
    let staged = git(&["diff", "--cached", "--name-only"])
        .lines()
        .filter(|l| !l.is_empty())
        .count();
    let ahead_behind = git(&["rev-list", "--left-right", "--count", "HEAD...@{upstream}"]);
    let (ahead, behind) = parse_ahead_behind(&ahead_behind);
    let risk = calc_risk(modified, untracked, staged, ahead, behind);

    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!("{}", "🎯 Git Risk Assessment".bold());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!("  Risk Score: {} {} / 100", risk_label(risk), risk);
    println!();
    println!("  Factors:");
    if modified > 0 {
        println!(
            "    {} {} modified files (+{})",
            "▶".bright_yellow(),
            modified,
            modified * 5
        );
    }
    if staged > 0 {
        println!(
            "    {} {} staged files  (+{})",
            "▶".bright_cyan(),
            staged,
            staged * 3
        );
    }
    if untracked > 0 {
        println!(
            "    {} {} untracked     (+{})",
            "▶".dimmed(),
            untracked,
            untracked * 2
        );
    }
    if behind > 0 {
        println!(
            "    {} {} commits behind (+{})",
            "▶".bright_red(),
            behind,
            behind * 10
        );
    }
    if ahead > 0 {
        println!(
            "    {} {} commits ahead  (+{})",
            "▶".bright_green(),
            ahead,
            ahead * 2
        );
    }
    if risk == 0 {
        println!("    {} Clean working tree", "✅".green());
    }
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    Ok(())
}

pub fn log_cmd(_ctx: &AppContext, n: u32) -> CoreResult<()> {
    let output = git(&[
        "log",
        &format!("--max-count={}", n),
        "--oneline",
        "--color=always",
    ]);
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!("{}", "📜 Commit History".bold());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!("{}", output);
    Ok(())
}

pub fn verify(_ctx: &AppContext) -> CoreResult<()> {
    let modified = git(&["diff", "--name-only"])
        .lines()
        .filter(|l| !l.is_empty())
        .count();
    let staged = git(&["diff", "--cached", "--name-only"])
        .lines()
        .filter(|l| !l.is_empty())
        .count();
    let ahead_behind = git(&["rev-list", "--left-right", "--count", "HEAD...@{upstream}"]);
    let (_, behind) = parse_ahead_behind(&ahead_behind);

    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!("{}", "✅ Git Verify".bold());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );

    let clean = modified == 0 && staged == 0 && behind == 0;
    if modified == 0 {
        println!("  {} No uncommitted changes", "✅".green());
    } else {
        println!("  {} {} modified files", "⚠️".normal(), modified);
    }
    if behind == 0 {
        println!("  {} Up to date with upstream", "✅".green());
    } else {
        println!("  {} {} commits behind upstream", "✗".bright_red(), behind);
    }

    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    if clean {
        println!("  {} Ready to push", "✅".green());
    } else {
        println!("  {} Resolve issues before pushing", "⚠️".normal());
    }
    Ok(())
}

// Delegate complex operations to v1
pub fn delegate(subcmd: &str, args: &[String]) -> CoreResult<()> {
    let status = Command::new(format!(
        "{}/scripts/faelight-git",
        std::env::var("HOME").unwrap_or_default() + "/0-core"
    ))
    .arg(subcmd)
    .args(args)
    .status()?;
    if !status.success() {
        println!("  {} faelight-git {} failed", "✗".bright_red(), subcmd);
    }
    Ok(())
}

fn parse_ahead_behind(s: &str) -> (usize, usize) {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() >= 2 {
        (parts[0].parse().unwrap_or(0), parts[1].parse().unwrap_or(0))
    } else {
        (0, 0)
    }
}

fn calc_risk(
    modified: usize,
    untracked: usize,
    staged: usize,
    ahead: usize,
    behind: usize,
) -> usize {
    let score = modified * 5 + staged * 3 + untracked * 2 + ahead * 2 + behind * 10;
    score.min(100)
}

fn risk_label(risk: usize) -> &'static str {
    if risk == 0 {
        "🟢"
    } else if risk < 30 {
        "🟡"
    } else if risk < 60 {
        "🟠"
    } else {
        "🔴"
    }
}
