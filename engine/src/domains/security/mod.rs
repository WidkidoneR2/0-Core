#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Finding {
    pub id: String,
    pub severity: String,
    pub category: String,
    pub package: String,
    pub description: String,
    pub fix: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScanResult {
    pub timestamp: String,
    pub findings: Vec<Finding>,
}

fn state_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/home/christian"))
        .join(".local/state/0-core/security")
}

fn last_scan() -> Option<ScanResult> {
    let path = state_dir().join("last-scan.json");
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn report(_ctx: &AppContext, all: bool) -> CoreResult<()> {
    let Some(scan) = last_scan() else {
        println!(
            "  {} No scan found. Run: core security scan",
            "✗".bright_red()
        );
        return Ok(());
    };

    let mut critical = vec![];
    let mut high = vec![];
    let mut medium = vec![];
    let mut low = vec![];

    for f in &scan.findings {
        match f.severity.to_lowercase().as_str() {
            "critical" => critical.push(f),
            "high" => high.push(f),
            "medium" => medium.push(f),
            _ => low.push(f),
        }
    }

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "🔒 Security Audit Report".bold());
    println!("   Last scan: {}", scan.timestamp.dimmed());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    if !critical.is_empty() {
        println!("  {} CRITICAL ({})", "🔴".normal(), critical.len());
        for f in &critical {
            println!(
                "    {} [{}] {} — {}",
                "▶".bright_red(),
                f.package,
                f.id,
                f.description
            );
            println!("      → {}", f.fix.dimmed());
        }
    }

    if !high.is_empty() {
        println!("  {} HIGH ({})", "🟠".normal(), high.len());
        for f in &high {
            println!(
                "    {} [{}] {} — {}",
                "▶".bright_yellow(),
                f.package,
                f.id,
                f.description
            );
            println!("      → {}", f.fix.dimmed());
        }
    }

    if !medium.is_empty() {
        println!("  {} MEDIUM ({})", "🟡".normal(), medium.len());
        for f in &medium {
            println!(
                "    {} [{}] {} — {}",
                "▶".yellow(),
                f.package,
                f.id,
                f.description
            );
            println!("      → {}", f.fix.dimmed());
        }
    }

    if !low.is_empty() {
        if all {
            println!("  {} LOW ({})", "🔵".normal(), low.len());
            for f in &low {
                println!(
                    "    {} [{}] {} — {}",
                    "▶".bright_blue(),
                    f.package,
                    f.id,
                    f.description
                );
                println!("      → {}", f.fix.dimmed());
            }
        } else {
            println!(
                "  {} {} low severity findings (use --all to show)",
                "🔵".normal(),
                low.len()
            );
        }
    }

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!(
        "  Total: {}  |  Critical: {}  High: {}  Medium: {}  Low: {}",
        scan.findings.len(),
        critical.len().to_string().bright_red(),
        high.len().to_string().bright_yellow(),
        medium.len().to_string().yellow(),
        low.len().to_string().bright_blue(),
    );
    Ok(())
}

pub fn show(_ctx: &AppContext, id: &str) -> CoreResult<()> {
    let Some(scan) = last_scan() else {
        println!("  {} No scan found", "✗".bright_red());
        return Ok(());
    };

    let found = scan.findings.iter().find(|f| f.id == id);
    let Some(f) = found else {
        println!("  {} Finding '{}' not found", "✗".bright_red(), id);
        return Ok(());
    };

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{} {}", "🔒".bold(), f.id.bright_white().bold());
    println!("  Severity: {}", f.severity.bright_yellow());
    println!("  Package:  {}", f.package.bright_white());
    println!("  Category: {}", f.category.dimmed());
    println!("  Issue:    {}", f.description);
    println!("  Fix:      {}", f.fix.bright_cyan());
    if let Some(url) = &f.url {
        println!("  URL:      {}", url.dimmed());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    Ok(())
}

pub fn history(_ctx: &AppContext) -> CoreResult<()> {
    let path = state_dir().join("scan-history.json");
    let Ok(content) = fs::read_to_string(&path) else {
        println!("  {} No scan history found", "○".dimmed());
        return Ok(());
    };

    #[derive(Deserialize)]
    struct HistoryEntry {
        timestamp: String,
        total: Option<u32>,
        critical: Option<u32>,
        high: Option<u32>,
        medium: Option<u32>,
        low: Option<u32>,
    }

    let entries: Vec<HistoryEntry> = serde_json::from_str(&content).unwrap_or_default();

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "📜 Scan History".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    if entries.is_empty() {
        println!("  {} No history yet", "○".dimmed());
    } else {
        for e in entries.iter().rev().take(10) {
            println!(
                "  {} {}  total: {}  high: {}  medium: {}",
                "▶".dimmed(),
                e.timestamp.bright_white(),
                e.total.unwrap_or(0),
                e.high.unwrap_or(0).to_string().bright_yellow(),
                e.medium.unwrap_or(0).to_string().yellow(),
            );
        }
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    Ok(())
}

pub fn scan(_ctx: &AppContext) -> CoreResult<()> {
    // Phase 2: delegate to v1 security-audit for actual scanning
    // Phase 3: implement full scan natively in this domain
    use std::process::Command;
    let status = Command::new("security-audit").arg("scan").status()?;
    if !status.success() {
        println!("  {} Scan failed", "✗".bright_red());
    }
    Ok(())
}
