use anyhow::Result;
use colored::Colorize;
use std::process::Command;

fn is_clippy_installed() -> bool {
    Command::new("cargo")
        .args(["clippy", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn check_clippy() -> Result<bool> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("git diff --cached --name-only --diff-filter=ACM | grep '\\.rs$'")
        .output()?;

    if output.stdout.is_empty() {
        println!("{}", "✅ Clippy: No Rust files staged".green());
        return Ok(true);
    }

    if !is_clippy_installed() {
        println!("{}", "⚠️  clippy not found — lint check skipped".yellow());
        println!(
            "   Install with: {}",
            "rustup component add clippy".dimmed()
        );
        println!();
        return Ok(true);
    }

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
        println!("   Fix with: {}", "cargo clippy --fix".cyan());
        return Ok(false);
    }

    println!("{}", "✅ Clippy: No warnings".green());
    Ok(true)
}
