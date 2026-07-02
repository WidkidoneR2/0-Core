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

    // INT-233 -- Only lint packages with staged files, not entire workspace
    let staged_files = String::from_utf8_lossy(&output.stdout).to_string();
    let mut packages: Vec<String> = Vec::new();
    for file in staged_files.lines() {
        // Extract package from path: rust-tools/faelight-X/src/... -> faelight-X
        let parts: Vec<&str> = file.split('/').collect();
        if parts.len() >= 2 {
            let pkg = parts[1].to_string();
            if !packages.contains(&pkg) {
                packages.push(pkg);
            }
        }
    }
    let mut args = vec!["clippy".to_string()];
    if packages.is_empty() {
        args.push("--workspace".to_string());
    } else {
        for pkg in &packages {
            args.push("-p".to_string());
            args.push(pkg.clone());
        }
    }
    args.extend(["--".to_string(), "-D".to_string(), "warnings".to_string()]);
    let check = Command::new("cargo").args(&args).output()?;

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
