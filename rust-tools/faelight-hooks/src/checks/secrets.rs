use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

pub fn check_secrets() -> Result<bool> {
    println!("{}", "🔍 Scanning for secrets with gitleaks...".cyan());

    // Find git root
    let git_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to find git root")?;

    let git_root = String::from_utf8_lossy(&git_root.stdout).trim().to_string();

    // Run gitleaks
    let output = Command::new("gitleaks")
        .args([
            "protect",
            "--staged",
            "-c",
            ".gitleaks.toml",
            "--redact",
            "-v",
        ])
        .current_dir(&git_root)
        .output()
        .context("Failed to run gitleaks - is it installed?")?;

    // Check exit code
    if !output.status.success() {
        println!();
        println!("{}", "❌ GITLEAKS DETECTED SECRETS!".red().bold());
        println!("{}", "⚠️  Commit blocked to protect you!".yellow());
        println!();
        println!("Review the findings above and remove the secrets.");
        println!();
        println!("{}", "To bypass (NOT RECOMMENDED):".yellow());
        println!("  git commit --no-verify");
        println!();
        return Ok(false);
    }

    println!("{}", "✅ No secrets detected".green());
    Ok(true)
}
