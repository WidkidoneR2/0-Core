use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

pub fn install_hooks(hook_name: Option<String>) -> Result<()> {
    // Find git root
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Not in a git repository")?;

    let git_root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let hooks_dir = PathBuf::from(&git_root).join(".git/hooks");

    // Create hooks directory if it doesn't exist
    fs::create_dir_all(&hooks_dir)?;

    match hook_name {
        Some(name) => install_single_hook(&hooks_dir, &name)?,
        None => install_all_hooks(&hooks_dir)?,
    }

    Ok(())
}

fn install_all_hooks(hooks_dir: &PathBuf) -> Result<()> {
    println!("{}", "📦 Installing all hooks...".cyan());
    
    install_pre_commit(hooks_dir)?;
    // TODO: Add pre-push, commit-msg later
    
    println!();
    println!("{}", "✅ All hooks installed successfully!".green().bold());
    println!();
    println!("Hooks are now active in: {}", hooks_dir.display().to_string().dimmed());
    
    Ok(())
}

fn install_single_hook(hooks_dir: &PathBuf, name: &str) -> Result<()> {
    println!("Installing {} hook...", name.green());
    
    match name {
        "pre-commit" => install_pre_commit(hooks_dir)?,
        "pre-push" => {
            println!("{}", "  ⚠️  pre-push not yet implemented".yellow());
        }
        "commit-msg" => {
            println!("{}", "  ⚠️  commit-msg not yet implemented".yellow());
        }
        _ => {
            println!("{}", format!("  ❌ Unknown hook: {}", name).red());
            return Ok(());
        }
    }
    
    Ok(())
}

fn install_pre_commit(hooks_dir: &PathBuf) -> Result<()> {
    let hook_path = hooks_dir.join("pre-commit");
    
    // Create the hook script
    let hook_content = r#"#!/usr/bin/env bash
# 🌲 Faelight Forest Pre-Commit Hook
# Managed by faelight-hooks

# Run faelight-hooks check
if command -v faelight-hooks &> /dev/null; then
    faelight-hooks check
    exit $?
else
    echo "❌ faelight-hooks not found in PATH"
    echo "Install with: cargo build --release -p faelight-hooks"
    exit 1
fi
"#;

    fs::write(&hook_path, hook_content)
        .context("Failed to write pre-commit hook")?;

    // Make executable
    let mut perms = fs::metadata(&hook_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&hook_path, perms)?;

    println!("{}", "  ✅ pre-commit hook installed".green());
    
    Ok(())
}
