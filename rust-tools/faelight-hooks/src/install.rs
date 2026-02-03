use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use faelight_core::paths;

pub fn install_hooks(hook_name: Option<String>) -> Result<()> {
    let hooks_dir = paths::git_hooks_dir();
    
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
    install_pre_push(hooks_dir)?;
    install_commit_msg(hooks_dir)?;
    
    println!();
    println!("{}", "✅ All hooks installed successfully!".green().bold());
    println!();
    println!("Hooks installed:");
    println!("  • {} - Branch validation, file size, secrets, conflicts", "pre-commit".green());
    println!("  • {} - Branch warnings, uncommitted changes", "pre-push".green());
    println!("  • {} - Conventional commit validation", "commit-msg".green());
    println!();
    println!("Location: {}", hooks_dir.display().to_string().dimmed());
    
    Ok(())
}

fn install_single_hook(hooks_dir: &PathBuf, name: &str) -> Result<()> {
    println!("Installing {} hook...", name.green());
    
    match name {
        "pre-commit" => install_pre_commit(hooks_dir)?,
        "pre-push" => install_pre_push(hooks_dir)?,
        "commit-msg" => install_commit_msg(hooks_dir)?,
        _ => {
            println!("{}", format!("  ❌ Unknown hook: {}", name).red());
            println!();
            println!("Available hooks: pre-commit, pre-push, commit-msg");
            return Ok(());
        }
    }
    
    println!("{}", "✅ Hook installed!".green());
    Ok(())
}

fn install_pre_commit(hooks_dir: &PathBuf) -> Result<()> {
    let hook_path = hooks_dir.join("pre-commit");
    
    let hook_content = r#"#!/usr/bin/env bash
# 🌲 Faelight Forest Pre-Commit Hook
# Managed by faelight-hooks

if command -v faelight-hooks &> /dev/null; then
    faelight-hooks check
    exit $?
else
    echo "❌ faelight-hooks not found in PATH"
    echo "Install with: cargo install --path rust-tools/faelight-hooks"
    exit 1
fi
"#;

    fs::write(&hook_path, hook_content)
        .context("Failed to write pre-commit hook")?;
        
    let mut perms = fs::metadata(&hook_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&hook_path, perms)?;
    
    println!("{}", "  ✅ pre-commit hook installed".green());
    Ok(())
}

fn install_pre_push(hooks_dir: &PathBuf) -> Result<()> {
    let hook_path = hooks_dir.join("pre-push");
    
    let hook_content = r#"#!/usr/bin/env bash
# 🌲 Faelight Forest Pre-Push Hook
# Managed by faelight-hooks

if command -v faelight-hooks &> /dev/null; then
    faelight-hooks check --pre-push
    exit $?
else
    echo "❌ faelight-hooks not found in PATH"
    exit 1
fi
"#;

    fs::write(&hook_path, hook_content)
        .context("Failed to write pre-push hook")?;
        
    let mut perms = fs::metadata(&hook_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&hook_path, perms)?;
    
    println!("{}", "  ✅ pre-push hook installed".green());
    Ok(())
}

fn install_commit_msg(hooks_dir: &PathBuf) -> Result<()> {
    let hook_path = hooks_dir.join("commit-msg");
    
    let hook_content = r#"#!/usr/bin/env bash
# 🌲 Faelight Forest Commit-Msg Hook
# Managed by faelight-hooks

if command -v faelight-hooks &> /dev/null; then
    faelight-hooks check --commit-msg "$1"
    exit $?
else
    echo "❌ faelight-hooks not found in PATH"
    exit 1
fi
"#;

    fs::write(&hook_path, hook_content)
        .context("Failed to write commit-msg hook")?;
        
    let mut perms = fs::metadata(&hook_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&hook_path, perms)?;
    
    println!("{}", "  ✅ commit-msg hook installed".green());
    Ok(())
}
