// doctor/cockpit.rs
use super::{CheckResult, Status};
use colored::*;

pub fn print_result(r: &CheckResult) {
    match r.status {
        Status::Pass => println!("✅ {}: {}", r.name, r.message),
        Status::Warn => println!("⚠️  {}: {}", r.name, r.message),
        Status::Fail => println!("❌ {}: {}", r.name, r.message),
        Status::Blocked => println!("⏭ {}: {}", r.name, r.message),
        Status::Unknown => println!("❔ {}: {}", r.name, r.message),
    }
}

pub fn status_icon(s: &Status) -> &'static str {
    match s {
        Status::Pass => "✅",
        Status::Warn => "⚠️ ",
        Status::Fail => "❌",
        Status::Blocked => "⏭ ",
        Status::Unknown => "❔",
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
            Status::Blocked => r.message.dimmed().to_string(),
            Status::Unknown => r.message.dimmed().to_string(),
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
    unknown: u32,
    integrity_pct: u32,
) {
    // ── Summary header ────────────────────────────────────────────────
    // INT-222: the percentage is a TREND, not a verdict. It carried its own colour
    // thresholds, so a green number could sit beside the word ADVISORY, and today one
    // prunable-generations advisory is the only thing between this machine and HEALTHY.
    // A weight factor is subjective; comparing the number to its own past is not.
    // Dimmed and uncoloured so the light does the judging and the number shows movement.
    let health_color = format!("{}%", health).dimmed().to_string();

    // INT-222: ONE verdict, derived by super::verdict from tier and status. This block used
    // to count failed and warnings itself, the quick scan counted them again with a different
    // word for the worst state, and the percentage carried a third set of colour thresholds --
    // so a red number could sit beside the word ADVISORY in the same line.
    let status_str = match super::verdict(checks) {
        super::Verdict::Red => "DEGRADED".bright_red().bold().to_string(),
        super::Verdict::Amber => "ADVISORY".yellow().bold().to_string(),
        super::Verdict::Green => "HEALTHY".bright_green().bold().to_string(),
    };

    println!();
    println!(
        "{}",
        "  ╭──────────────────────────────────────────────────────╮".bright_cyan()
    );
    println!(
        "  │  🏥 {}  {}  {}  │  {}/{} checks  │",
        format!("Zero Core {}", version).bright_white().bold(),
        status_str,
        health_color,
        passed,
        checks.len(),
    );
    println!(
        "{}",
        "  ╰──────────────────────────────────────────────────────╯".bright_cyan()
    );

    // ── Group checks ─────────────────────────────────────────────────
    let system_names = [
        "System Services",
        "Broken Symlinks",
        "Binary Dependencies",
        "Disk Space",
    ];
    // "Scripts" was here and no check produces it -- a name carried by the display alone, which
    // the filter_map swallowed as quietly as it swallowed Git Hooks in the other direction.
    let git_names = ["Git Repository", "Git Hooks", "Rust Toolchain", "Rust Docs"];
    let tools_names = ["Tool Installation", "Path Resilience", "Alias Coverage"];
    let forest_names = [
        "Intent Ledger",
        "Zero Config",
        "Schema Validation",
        "Friday",
        "Deadwood",
    ];
    let security_names = ["Security Hardening", "Security Audit", "Sandbox"];
    let boot_names = ["Boot Errors", "Boot Time"];
    let system_state_names = [
        "Reboot Needed",
        "Update Readiness",
        "Package Cache",
        "Orphan Packages",
    ];
    // ⚠️ FOUR PHANTOM NAMES REMOVED 2026-09-04 (INT-222 gate: the census).
    //
    // Dotfile Symlinks, Compositor Keybinds, Theme Packages, Package Metadata and Compositor
    // named checks that no longer exist -- stow went with INT-107, mango and the NixOS theme
    // packages went with the migration, and Package Metadata was check_dotmeta, the hardcoded
    // Pass this whole intent was written about.
    //
    // A NAME WITH NO CHECK IS THE OTHER HALF OF THE DRIFT. The catch-all added 2026-09-02
    // makes an unclaimed CHECK visible; filter_map still drops a claimed NAME silently, so
    // these sat here reading as coverage. The census called this class PHANTOM: not real,
    // label or lie, because there is nothing behind it to classify.
    let runtime_names = ["VM State", "Network"];

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
    render_section("🥾 Boot", &group(&boot_names));
    render_section("📦 System State", &group(&system_state_names));
    render_section("🖥  Runtime", &group(&runtime_names));
    render_section("🔒 Security", &group(&security_names));

    // ⭐ ANYTHING NOT CLAIMED ABOVE STILL RENDERS. The eight lists are a SECOND registry of check
    // names -- all_checks() decides what RUNS, these decide what is SEEN, and they drift apart
    // silently in both directions.
    //
    // MEASURED 2026-09-02: check_hooks was written, registered in all_checks, compiled, and
    // invisible. filter_map drops a name nobody listed, with no error. In the other direction
    // git_names carries "Scripts", which no check produces, and the forest and system lists still
    // name Dotfile Symlinks, Compositor Keybinds, Theme Packages and Package Metadata -- checks
    // deleted when Omarchy replaced their subjects.
    //
    // That is the check-count discrepancy this file's sibling already recorded: "it said 23 while
    // this file held 30, and the docs said 22 and 14". Two owners of one set.
    //
    // A catch-all does not fix the drift, it makes the drift VISIBLE -- the same move INT-148 made
    // for Unknown. Forgetting to categorise a check is now cosmetic instead of silent, and a check
    // can never again run without being seen.
    let claimed: Vec<&str> = system_names
        .iter()
        .chain(git_names.iter())
        .chain(tools_names.iter())
        .chain(forest_names.iter())
        .chain(boot_names.iter())
        .chain(system_state_names.iter())
        .chain(runtime_names.iter())
        .chain(security_names.iter())
        .copied()
        .collect();
    let unclaimed: Vec<&CheckResult> = checks
        .iter()
        .filter(|c| !claimed.contains(&c.name.as_str()))
        .collect();
    if !unclaimed.is_empty() {
        render_section("❔ Uncategorised", &unclaimed);
    }

    // ── Stats strip ───────────────────────────────────────────────────
    println!();
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    // INT-148: show Unknown count only when > 0 (checks that could not run; excluded
    // from the health denominator but surfaced so they are not silently invisible).
    // INT-151: name the unknown checks, not just count them -- "which one couldn't run?"
    // is the actionable question. Names pulled from the checks slice already in scope.
    let unknown_seg = if unknown > 0 {
        let names: Vec<&str> = checks
            .iter()
            .filter(|c| c.status == Status::Unknown)
            .map(|c| c.name.as_str())
            .collect();
        format!(
            "   ❔ {}",
            format!("Unknown: {} ({})", unknown, names.join(", ")).dimmed()
        )
    } else {
        String::new()
    };
    println!(
        "  {} {}   ⚠️  {}   ❌ {}{}   📊 {}",
        "✅".green(),
        format!("Passed:  {}", passed).bright_white(),
        format!("Warnings: {}", warnings).yellow(),
        format!("Failed: {}", failed).bright_red(),
        unknown_seg,
        format!("Health: {}%", health).bright_white().bold(),
    );
    // Integrity score
    let int_str = if integrity_pct >= 95 {
        format!("Integrity: {}%", integrity_pct)
            .bright_green()
            .to_string()
    } else if integrity_pct >= 75 {
        format!("Integrity: {}%", integrity_pct)
            .yellow()
            .to_string()
    } else {
        format!("Integrity: {}%", integrity_pct)
            .bright_red()
            .to_string()
    };
    println!("  {} {}", "🔍".normal(), int_str);
}
