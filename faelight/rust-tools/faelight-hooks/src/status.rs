use anyhow::Result;
use colored::Colorize;
use faelight_core::paths;
use std::fs;
use std::process::Command;

pub fn show_status() -> Result<()> {
    println!("{}", "🎣 faelight-hooks status".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();

    // Hook installation status
    let hooks_dir = paths::git_hooks_dir();
    println!("{}", "📦 Installed Hooks:".bold());

    let hooks = [
        (
            "pre-commit",
            "Branch, filesize, secrets, conflicts, rustfmt, clippy",
        ),
        ("pre-push", "Push target warning, uncommitted changes"),
        ("commit-msg", "Conventional commit format validation"),
    ];

    for (hook, description) in &hooks {
        let hook_path = hooks_dir.join(hook);
        if hook_path.exists() {
            // Check if it's actually managed by faelight-hooks
            let content = fs::read_to_string(&hook_path).unwrap_or_default();
            if content.contains("faelight-hooks") {
                println!(
                    "  {} {} — {}",
                    "✅".green(),
                    hook.green().bold(),
                    description.dimmed()
                );
            } else {
                println!(
                    "  {} {} — {}",
                    "⚠️ ".yellow(),
                    hook.yellow().bold(),
                    "exists but NOT managed by faelight-hooks".yellow()
                );
            }
        } else {
            println!(
                "  {} {} — {}",
                "❌".red(),
                hook.red().bold(),
                "not installed".dimmed()
            );
        }
    }

    println!();

    // External tool availability
    println!("{}", "🔧 External Tools:".bold());

    let tools = [
        ("gitleaks", "Secret scanning"),
        ("rustfmt", "Rust formatting checks"),
        ("cargo", "Clippy linting"),
    ];

    for (tool, purpose) in &tools {
        let available = Command::new("which")
            .arg(tool)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if available {
            println!(
                "  {} {} — {}",
                "✅".green(),
                tool.green().bold(),
                purpose.dimmed()
            );
        } else {
            println!(
                "  {} {} — {} {}",
                "⚠️ ".yellow(),
                tool.yellow().bold(),
                "not found —".yellow(),
                purpose.dimmed()
            );
        }
    }

    println!();

    // Hooks directory location
    println!("{}", "📍 Location:".bold());
    println!("  {}", hooks_dir.display().to_string().dimmed());
    println!();

    // Quick reinstall hint if anything missing
    let any_missing = hooks.iter().any(|(hook, _)| !hooks_dir.join(hook).exists());
    if any_missing {
        println!("{}", "💡 To install missing hooks:".yellow());
        println!("   {}", "faelight-hooks install".cyan());
        println!();
    }

    Ok(())
}
