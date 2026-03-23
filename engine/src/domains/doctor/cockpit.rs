// doctor/cockpit.rs
use super::{CheckResult, Status};
use colored::*;

pub fn print_result(r: &CheckResult) {
    match r.status {
        Status::Pass => println!("✅ {}: {}", r.name, r.message),
        Status::Warn => println!("⚠️  {}: {}", r.name, r.message),
        Status::Fail => println!("❌ {}: {}", r.name, r.message),
        Status::Blocked => println!("⏭  {}: blocked", r.name),
    }
}

pub fn status_icon(s: &Status) -> &'static str {
    match s {
        Status::Pass => "✅",
        Status::Warn => "⚠️ ",
        Status::Fail => "❌",
        Status::Blocked => "⏭ ",
    }
}

pub fn render_section(title: &str, checks: &[&CheckResult]) {
    if checks.is_empty() {
        return;
    }
    println!();
    println!("{}", format!("  ╭─ {} ", title).bright_cyan());
    for r in checks {
        let icon = status_icon(&r.status);
        let name_col = format!("{:<22}", r.name);
        let msg = match r.status {
            Status::Pass => r.message.dimmed().to_string(),
            Status::Warn => r.message.yellow().to_string(),
            Status::Fail => r.message.bright_red().to_string(),
            Status::Blocked => "blocked".dimmed().to_string(),
        };
        println!("  │  {} {}  {}", icon, name_col.normal(), msg);
    }
    println!(
        "{}",
        "  ╰─────────────────────────────────────────────────".dimmed()
    );
}

pub fn render_cockpit(
    checks: &[CheckResult],
    version: &str,
    health: u32,
    passed: u32,
    warnings: u32,
    failed: u32,
) {
    // ── Summary header ────────────────────────────────────────────────
    let health_color = if health >= 95 {
        format!("{}%", health).bright_green().bold().to_string()
    } else if health >= 80 {
        format!("{}%", health).yellow().bold().to_string()
    } else {
        format!("{}%", health).bright_red().bold().to_string()
    };

    let status_str = if failed > 0 {
        "DEGRADED".bright_red().bold().to_string()
    } else if warnings > 0 {
        "ADVISORY".yellow().bold().to_string()
    } else {
        "HEALTHY".bright_green().bold().to_string()
    };

    println!();
    println!(
        "{}",
        "  ╭──────────────────────────────────────────────────────╮".bright_cyan()
    );
    println!(
        "  │  🏥 {}  {}  {}  │  {}/{} checks  │",
        format!("Faelight Forest {}", version).bright_white().bold(),
        health_color,
        status_str,
        passed,
        checks.len(),
    );
    println!(
        "{}",
        "  ╰──────────────────────────────────────────────────────╯".bright_cyan()
    );

    // ── Group checks ─────────────────────────────────────────────────
    let system_names = [
        "Stow Symlinks",
        "System Services",
        "Broken Symlinks",
        "Binary Dependencies",
        "Disk Space",
    ];
    let git_names = ["Git Repository", "Scripts", "Rust Toolchain"];
    let tools_names = [
        "Yazi Plugins",
        "Tool Installation",
        "Path Resilience",
        "Alias Coverage",
    ];
    let forest_names = [
        "Intent Ledger",
        "Profile System",
        "Faelight Config",
        "Niri Keybinds",
        "Theme Packages",
        "Package Metadata",
        "Archaeology",
        "Schema Validation",
    ];
    let security_names = ["Security Hardening", "Security Audit", "Core Protection"];

    let group = |names: &[&str]| -> Vec<&CheckResult> {
        names
            .iter()
            .filter_map(|n| checks.iter().find(|c| c.name == *n))
            .collect()
    };

    render_section("🖥  System", &group(&system_names));
    render_section("🌿 Git & Code", &group(&git_names));
    render_section("🛠  Tools", &group(&tools_names));
    render_section("📋 Forest", &group(&forest_names));
    render_section("🔒 Security", &group(&security_names));

    // ── Stats strip ───────────────────────────────────────────────────
    println!();
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!(
        "  {} {}   {} {}   {} {}   {} {}",
        "✅".green(),
        format!("Passed:  {}", passed).bright_white(),
        "⚠️ ",
        format!("Warnings: {}", warnings).yellow(),
        "❌",
        format!("Failed: {}", failed).bright_red(),
        "📊",
        format!("Health: {}%", health).bright_white().bold(),
    );
}
