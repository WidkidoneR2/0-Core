use anyhow::Result;
use colored::Colorize;
use std::process::Command;

pub fn check_clippy() -> Result<bool> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("git diff --cached --name-only --diff-filter=ACM | grep '\\.rs$'")
        .output()?;

    if output.stdout.is_empty() {
        return Ok(true);
    }

    // Run clippy on workspace
    let check = Command::new("cargo")
        .args([
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ])
        .output()?;

    if !check.status.success() {
        println!("{}", "❌ Clippy check failed".red().bold());
        println!("{}", String::from_utf8_lossy(&check.stdout));
        println!("{}", String::from_utf8_lossy(&check.stderr));
        println!();
        println!("   Fix warnings with: {}", "cargo clippy --fix".cyan());
        return Ok(false);
    }

    println!("{}", "✅ Clippy: No warnings".green());
    Ok(true)
}
