use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use std::process::Command;

pub fn validate_branch_name() -> Result<bool> {
    println!("{}", "🔍 Validating branch name...".cyan());

    // Get current branch
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to get current branch")?;

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Skip validation for main/master
    if branch == "main" || branch == "master" || branch == "HEAD" {
        println!("{}", format!("  ℹ️  On {} branch", branch).dimmed());
        return Ok(true);
    }

    // Recommended branch naming: type/description
    // Examples: feature/add-hooks, fix/bug-123, chore/cleanup
    let branch_regex =
        Regex::new(r"^(feature|feat|fix|hotfix|bugfix|chore|docs|refactor|test|perf)/[a-z0-9-]+$")
            .unwrap();

    if !branch_regex.is_match(&branch) {
        println!();
        println!("{}", "⚠️  Branch name doesn't follow convention".yellow());
        println!("   Current: {}", branch.cyan());
        println!();
        println!("Recommended format:");
        println!("  {}/{}", "type".green(), "description".dimmed());
        println!();
        println!("Examples:");
        println!("  {}", "feature/add-new-check".green());
        println!("  {}", "fix/resolve-bug-123".green());
        println!("  {}", "chore/update-deps".green());
        println!();
        println!("{}", "💡 This is a recommendation, not enforced.".yellow());
        println!();
    } else {
        println!("{}", format!("✅ Branch: {}", branch).green());
    }

    Ok(true)
}
