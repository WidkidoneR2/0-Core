//! docs domain — access forest documentation
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
/// core docs commands — show the core commands guide
pub fn commands(ctx: &AppContext) -> CoreResult<()> {
    let docs_path = std::path::PathBuf::from(&ctx.core_root)
        .join("docs/core-commands.md");
    if !docs_path.exists() {
        println!("  {} Core commands guide not found at {}",
            "⚠️ ".yellow(), docs_path.display());
        println!("  {} Run: core docs generate", "💡".bright_cyan());
        return Ok(());
    }
    // Try to open with bat, fall back to less, fall back to cat
    let pager = if std::process::Command::new("bat")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "bat"
    } else if std::process::Command::new("less")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "less"
    } else {
        "cat"
    };
    let _ = std::process::Command::new(pager)
        .arg(&docs_path)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();
    Ok(())
}
/// core docs list — show available documentation files
pub fn list(ctx: &AppContext) -> CoreResult<()> {
    let docs_dir = std::path::PathBuf::from(&ctx.core_root).join("docs");
    println!();
    println!("{}", "📚 Forest Documentation".cyan().bold());
    println!("{}", "━".repeat(50).dimmed());
    println!();
    if !docs_dir.exists() {
        println!("  {} No docs directory found", "○".dimmed());
        return Ok(());
    }
    if let Ok(entries) = std::fs::read_dir(&docs_dir) {
        let mut docs: Vec<String> = entries
            .flatten()
            .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        docs.sort();
        for doc in &docs {
            let name = doc.trim_end_matches(".md");
            println!("  {} {}  {}",
                "📄".normal(),
                name.bright_white(),
                format!("core docs {}", name.replace("-", " ").split_whitespace().next().unwrap_or(name)).dimmed()
            );
        }
    }
    println!();
    println!("  {} core docs commands — full command reference", "→".dimmed());
    println!();
    Ok(())
}
