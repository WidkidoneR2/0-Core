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
        format!("{:<15}", "Project 0").bright_white().bold(),
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
    // SECTIONS ARE DECLARED, NOT LISTED HERE.
    //
    // This block used to be EIGHT HARDCODED NAME LISTS -- a SECOND REGISTRY deciding what was
    // SEEN while the check set decided what RAN. They drifted in both directions and the file
    // said so: check_hooks ran for weeks while invisible because no list named it, four names
    // outlived the checks they pointed at, and Zero Alias fell to Uncategorised from the day
    // it was written.
    //
    // Now each check DECLARES its section in checks.toml and carries it through. A check
    // cannot be unclaimed, a section cannot outlive its checks, and the order is the order of
    // the file -- so rearranging the panel means rearranging the declarations, which is the
    // one place the answer lives.
    let mut order: Vec<&str> = Vec::new();
    for c in checks {
        if !order.contains(&c.section.as_str()) {
            order.push(&c.section);
        }
    }
    for sec in order {
        let group: Vec<&CheckResult> = checks.iter().filter(|c| c.section == sec).collect();
        render_section(sec, &group);
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
    // INT-222: THE PERCENTAGE STATES ITS BASIS. A bare 84% cannot be read -- the charter
    // complaint is that a reader seeing 97% cannot tell whether the missing 3% is a stale
    // doc or a failing boot check. Half that fix is weighting, which is a separate gate.
    // The other half is showing the denominator, because it MOVES: Unknown and Blocked are
    // excluded from the ratio (INT-148), so a run where nine probes cannot answer scores
    // 7 of 18 rather than 7 of 27, and the shrinking denominator is invisible today.
    let blocked = checks
        .iter()
        .filter(|c| c.status == Status::Blocked)
        .count() as u32;
    let determinable = (checks.len() as u32)
        .saturating_sub(unknown)
        .saturating_sub(blocked);
    println!(
        "  {} {}   ⚠️  {}   ❌ {}{}   📊 {}",
        "✅".green(),
        format!("Passed:  {}", passed).bright_white(),
        format!("Warnings: {}", warnings).yellow(),
        format!("Failed: {}", failed).bright_red(),
        unknown_seg,
        format!(
            "Health: {}% ({} of {} determinable)",
            health, passed, determinable
        )
        .bright_white()
        .bold(),
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
