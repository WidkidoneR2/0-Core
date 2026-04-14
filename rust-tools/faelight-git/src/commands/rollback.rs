//! INT-221 -- fg rollback: interactive commit rollback with risk scores
use anyhow::Result;
use colored::*;
use std::process::Command;
pub fn run(hash: Option<&str>, dry_run: bool, list: bool) -> Result<()> {
    if list || hash.is_none() {
        show_rollback_list(dry_run)?;
    } else if let Some(h) = hash {
        do_rollback(h, dry_run)?;
    }
    Ok(())
}
fn get_recent_commits(n: usize) -> Vec<(String, String, String)> {
    let output = Command::new("git")
        .args(["log", "--oneline", &format!("-{}", n), "--format=%H|%s|%ai"])
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![], stderr: vec![],
        });
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|l| {
            let parts: Vec<&str> = l.splitn(3, '|').collect();
            if parts.len() == 3 {
                Some((parts[0].to_string(), parts[1].to_string(), parts[2][..10].to_string()))
            } else { None }
        })
        .collect()
}
fn compute_risk_score(hash: &str) -> u32 {
    // Risk = files changed * 2 + any deploy after this commit
    let diff_stat = Command::new("git")
        .args(["diff", "--stat", &format!("{}^..{}", hash, hash)])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let files: u32 = diff_stat.lines()
        .filter(|l| l.contains("|"))
        .count() as u32;
    (files * 2).min(100)
}
fn show_rollback_list(dry_run: bool) -> Result<()> {
    println!();
    println!("{}", "🔄 fg rollback -- Last 10 commits".bright_cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();
    let commits = get_recent_commits(10);
    if commits.is_empty() {
        println!("  {} No commits found", "○".dimmed());
        return Ok(());
    }
    for (i, (hash, msg, date)) in commits.iter().enumerate() {
        let short_hash = &hash[..7];
        let risk = compute_risk_score(hash);
        let risk_color = if risk < 20 { short_hash.bright_green().to_string() }
                         else if risk < 50 { short_hash.bright_yellow().to_string() }
                         else { short_hash.bright_red().to_string() };
        let risk_label = if risk < 20 { "low".bright_green() }
                         else if risk < 50 { "med".bright_yellow() }
                         else { "high".bright_red() };
        println!("  {} {} {} {} {}",
            format!("{:2}.", i+1).dimmed(),
            risk_color,
            date.dimmed(),
            msg.chars().take(45).collect::<String>().bright_white(),
            format!("[risk:{}]", risk_label));
    }
    println!();
    if dry_run {
        println!("  {} dry-run mode -- select a commit to see rollback plan", "○".dimmed());
        return Ok(());
    }
    print!("  Rollback to commit # (or hash, or 'cancel'): ");
    use std::io::Write;
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if input == "cancel" || input.is_empty() {
        println!("  {} cancelled", "○".dimmed());
        return Ok(());
    }
    // Parse number or hash
    let target_hash = if let Ok(n) = input.parse::<usize>() {
        if n >= 1 && n <= commits.len() {
            commits[n-1].0.clone()
        } else {
            println!("  {} invalid selection", "✗".bright_red());
            return Ok(());
        }
    } else {
        input.to_string()
    };
    do_rollback(&target_hash, false)
}
fn do_rollback(hash: &str, dry_run: bool) -> Result<()> {
    println!();
    println!("  {} Rolling back to: {}", "🔄".normal(), hash.bright_cyan());
    // Show what will change
    let diff = Command::new("git")
        .args(["diff", "--stat", &format!("{}..HEAD", hash)])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    if diff.is_empty() {
        println!("  {} Already at this commit", "○".dimmed());
        return Ok(());
    }
    println!("  {} Changes that will be undone:", "→".bright_cyan());
    for line in diff.lines().take(10) {
        println!("    {}", line.dimmed());
    }
    println!();
    if dry_run {
        println!("  {} dry-run -- no changes made", "○".dimmed());
        return Ok(());
    }
    print!("  Confirm rollback? (y/n): ");
    use std::io::Write;
    std::io::stdout().flush()?;
    let mut confirm = String::new();
    std::io::stdin().read_line(&mut confirm)?;
    if confirm.trim().to_lowercase() != "y" {
        println!("  {} cancelled", "○".dimmed());
        return Ok(());
    }
    let status = Command::new("git")
        .args(["revert", "--no-commit", &format!("{}..HEAD", hash)])
        .status()?;
    if status.success() {
        println!("  {} reverted -- changes staged, review and commit", "✅".green());
        println!("  {} run: fg commit to complete rollback", "→".bright_cyan());
    } else {
        println!("  {} revert failed -- resolve conflicts manually", "✗".bright_red());
    }
    println!();
    Ok(())
}
