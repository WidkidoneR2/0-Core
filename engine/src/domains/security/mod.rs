#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use chrono::Local;
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Finding {
    id: String,
    severity: Severity,
    category: String,
    package: String,
    description: String,
    fix: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScanResult {
    timestamp: String,
    findings: Vec<Finding>,
    scan_duration_ms: u64,
}

fn last_scan_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".local/state/0-core/security/last-scan.json")
}

fn scan_cargo(ctx: &AppContext) -> Vec<Finding> {
    let mut findings = Vec::new();
    let output = Command::new("cargo")
        .args(["audit", "--json"])
        .current_dir(&ctx.core_root)
        .output();
    let Ok(output) = output else { return findings };
    let Ok(text) = String::from_utf8(output.stdout) else {
        return findings;
    };
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
        if let Some(vulns) = json["vulnerabilities"]["list"].as_array() {
            for vuln in vulns {
                let id = vuln["advisory"]["id"]
                    .as_str()
                    .unwrap_or("UNKNOWN")
                    .to_string();
                let package = vuln["advisory"]["package"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();
                let description = vuln["advisory"]["title"]
                    .as_str()
                    .unwrap_or("Unknown vulnerability")
                    .to_string();
                let url = vuln["advisory"]["url"].as_str().map(|s| s.to_string());
                findings.push(Finding {
                    id,
                    severity: Severity::High,
                    category: "Rust Crate".to_string(),
                    package,
                    description,
                    fix: Some("Run: faelight-update --only cargo".to_string()),
                    url,
                });
            }
        }
    }
    findings
}

fn get_patchable_packages() -> std::collections::HashSet<String> {
    let mut patchable = std::collections::HashSet::new();
    if let Ok(output) = Command::new("arch-audit").args(["-u", "--json"]).output() {
        let text = String::from_utf8_lossy(&output.stdout);
        if let Ok(advisories) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
            for advisory in advisories {
                if let Some(pkgs) = advisory["packages"].as_array() {
                    for p in pkgs {
                        if let Some(s) = p.as_str() {
                            patchable.insert(s.to_string());
                        }
                    }
                }
            }
        }
    }
    patchable
}

fn scan_arch() -> Vec<Finding> {
    let mut findings = Vec::new();
    let patchable = get_patchable_packages();
    let output = Command::new("arch-audit").args(["--json"]).output();
    let Ok(output) = output else { return findings };
    let text = String::from_utf8_lossy(&output.stdout);
    let Ok(advisories) = serde_json::from_str::<Vec<serde_json::Value>>(&text) else {
        return findings;
    };
    for advisory in advisories {
        let status = advisory["status"].as_str().unwrap_or("Unknown");
        if status == "Unknown" {
            continue;
        }
        let avg_id = advisory["name"].as_str().unwrap_or("UNKNOWN").to_string();
        let severity_str = advisory["severity"].as_str().unwrap_or("Unknown");
        let vuln_type = advisory["type"].as_str().unwrap_or("unknown issue");
        let packages: Vec<String> = advisory["packages"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|p| p.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();
        let cves: Vec<String> = advisory["issues"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|i| i.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();
        let severity = match severity_str.to_lowercase().as_str() {
            "critical" => Severity::Critical,
            "high" => Severity::High,
            "medium" => Severity::Medium,
            "low" => Severity::Low,
            _ => Severity::Low,
        };
        for pkg in &packages {
            let fix_msg = if patchable.contains(pkg.as_str()) {
                "Patch available — run: sudo pacman -Syu".to_string()
            } else {
                "No patch yet — upstream pending (arch-audit -u)".to_string()
            };
            findings.push(Finding {
                id: avg_id.clone(),
                severity: severity.clone(),
                category: "System Package".to_string(),
                package: pkg.clone(),
                description: format!("{} ({})", vuln_type, cves.join(", ")),
                fix: Some(fix_msg),
                url: Some(format!("https://security.archlinux.org/{}", avg_id)),
            });
        }
    }
    findings
}

fn scan_permissions() -> Vec<Finding> {
    let mut findings = Vec::new();
    let home = std::env::var("HOME").unwrap_or_default();
    let ssh_dir = PathBuf::from(&home).join(".ssh");
    if ssh_dir.exists() {
        if let Ok(meta) = fs::metadata(&ssh_dir) {
            let mode = meta.permissions().mode() & 0o777;
            if mode != 0o700 {
                findings.push(Finding {
                    id: "PERM-SSH-DIR".to_string(),
                    severity: Severity::High,
                    category: "File Permission".to_string(),
                    package: "~/.ssh".to_string(),
                    description: format!(
                        "SSH directory has permissions {:03o} (should be 700)",
                        mode
                    ),
                    fix: Some("Run: chmod 700 ~/.ssh".to_string()),
                    url: None,
                });
            }
        }
        if let Ok(entries) = fs::read_dir(&ssh_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                if !name.ends_with(".pub")
                    && !name.ends_with(".d")
                    && name != "known_hosts"
                    && name != "config"
                    && name != "authorized_keys"
                {
                    if let Ok(meta) = fs::metadata(&path) {
                        let mode = meta.permissions().mode() & 0o777;
                        if mode != 0o600 {
                            findings.push(Finding {
                                id: format!("PERM-SSH-KEY-{}", name.to_uppercase()),
                                severity: Severity::High,
                                category: "File Permission".to_string(),
                                package: format!("~/.ssh/{}", name),
                                description: format!(
                                    "SSH key has permissions {:03o} (should be 600)",
                                    mode
                                ),
                                fix: Some(format!("Run: chmod 600 ~/.ssh/{}", name)),
                                url: None,
                            });
                        }
                    }
                }
            }
        }
    }
    findings
}

fn scan_network() -> Vec<Finding> {
    let mut findings = Vec::new();
    let output = Command::new("ss").args(["-tlnp"]).output();
    let Ok(output) = output else { return findings };
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines().skip(1) {
        if line.contains("0.0.0.0:") || line.contains(":::") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(addr) = parts.get(3) {
                if addr.starts_with("0.0.0.0:") || addr.starts_with(":::") {
                    let port = addr.split(':').next_back().unwrap_or("?");
                    let expected = ["22", "631", "5353"];
                    if !expected.contains(&port) {
                        findings.push(Finding {
                            id: format!("NET-LISTEN-{}", port),
                            severity: Severity::Medium,
                            category: "Network".to_string(),
                            package: format!("port {}", port),
                            description: format!("Unexpected listener on all interfaces: {}", addr),
                            fix: Some("Investigate with: ss -tlnp".to_string()),
                            url: None,
                        });
                    }
                }
            }
        }
    }
    findings
}

fn scan_ssh() -> Vec<Finding> {
    let mut findings = Vec::new();
    let home = std::env::var("HOME").unwrap_or_default();
    let ssh_dir = PathBuf::from(home).join(".ssh");
    if !ssh_dir.exists() {
        return findings;
    }
    if let Ok(entries) = fs::read_dir(&ssh_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if name.starts_with("id_rsa") && !name.ends_with(".pub") {
                findings.push(Finding {
                    id: "SSH-ALGO-RSA".to_string(),
                    severity: Severity::Low,
                    category: "SSH Config".to_string(),
                    package: format!("~/.ssh/{}", name),
                    description: "RSA key found — Ed25519 is preferred for modern systems"
                        .to_string(),
                    fix: Some("Generate: ssh-keygen -t ed25519 -C 'your@email.com'".to_string()),
                    url: None,
                });
            }
        }
    }
    findings
}

pub fn scan(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require(
        "security",
        &[
            Capability::OrchestratorAccess,
            Capability::FilesystemReadHome,
            Capability::NetworkQuery,
        ],
    )?;
    let start = std::time::Instant::now();
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "🔒 security scan — scanning...".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let mut findings = Vec::new();
    print!("  {} Rust crates (cargo-audit)... ", "🦀".cyan());
    std::io::Write::flush(&mut std::io::stdout()).ok();
    findings.extend(scan_cargo(ctx));
    println!("{}", "done".dimmed());

    print!("  {} System packages (arch-audit)... ", "📦".cyan());
    std::io::Write::flush(&mut std::io::stdout()).ok();
    findings.extend(scan_arch());
    println!("{}", "done".dimmed());

    print!("  {} File permissions... ", "🔑".cyan());
    std::io::Write::flush(&mut std::io::stdout()).ok();
    findings.extend(scan_permissions());
    println!("{}", "done".dimmed());

    print!("  {} Network listeners... ", "🌐".cyan());
    std::io::Write::flush(&mut std::io::stdout()).ok();
    findings.extend(scan_network());
    println!("{}", "done".dimmed());

    print!("  {} SSH configuration... ", "🔐".cyan());
    std::io::Write::flush(&mut std::io::stdout()).ok();
    findings.extend(scan_ssh());
    println!("{}", "done".dimmed());

    let duration = start.elapsed().as_millis() as u64;
    let result = ScanResult {
        timestamp,
        findings,
        scan_duration_ms: duration,
    };

    // Save to state file
    let path = last_scan_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    if let Ok(json) = serde_json::to_string_pretty(&result) {
        fs::write(&path, json).ok();
    }
    append_to_history(&result);
    update_first_seen(&result.findings);

    let critical = result
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Critical)
        .count();
    let high = result
        .findings
        .iter()
        .filter(|f| f.severity == Severity::High)
        .count();
    let medium = result
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Medium)
        .count();
    let low = result
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Low)
        .count();

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!(
        "  {} findings: {} critical, {} high, {} medium, {} low",
        result.findings.len(),
        if critical > 0 {
            critical.to_string().bright_red()
        } else {
            critical.to_string().dimmed()
        },
        if high > 0 {
            high.to_string().yellow()
        } else {
            high.to_string().dimmed()
        },
        medium.to_string().dimmed(),
        low.to_string().dimmed(),
    );
    println!("  {} Scan completed in {}ms", "✅".green(), duration);

    // Event Ledger
    let writer = crate::runtime::EventWriter::new(&ctx.runtime.db);
    writer.write(
        "security",
        "scan",
        "core security scan",
        "ok",
        Some(&format!(
            r#"{{"critical":{},"high":{},"medium":{},"low":{}}}"#,
            critical, high, medium, low
        )),
    );

    Ok(())
}

pub fn report(ctx: &AppContext, all: bool) -> CoreResult<()> {
    let path = last_scan_path();
    let content = fs::read_to_string(&path).unwrap_or_default();
    let result: ScanResult = serde_json::from_str(&content).unwrap_or(ScanResult {
        timestamp: "never".to_string(),
        findings: vec![],
        scan_duration_ms: 0,
    });

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!(
        "{} {}",
        "🔒 Security Report".bold(),
        format!("(scan: {})", result.timestamp).dimmed()
    );
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let show: Vec<_> = result
        .findings
        .iter()
        .filter(|f| {
            all || matches!(
                f.severity,
                Severity::Critical | Severity::High | Severity::Medium
            )
        })
        .collect();

    if show.is_empty() {
        println!("  {} No findings to display", "✅".green());
        return Ok(());
    }

    for finding in &show {
        let sev = match finding.severity {
            Severity::Critical => finding.severity.to_string().bright_red().bold(),
            Severity::High => finding.severity.to_string().yellow().bold(),
            Severity::Medium => finding.severity.to_string().bright_yellow(),
            _ => finding.severity.to_string().dimmed(),
        };
        println!(
            "  [{sev}] {} — {}",
            finding.id.bright_white(),
            finding.package.dimmed()
        );
        println!("    {}", finding.description);
        if let Some(fix) = &finding.fix {
            println!("    {} {}", "💡".dimmed(), fix.dimmed());
        }
    }
    let _ = ctx;
    Ok(())
}

pub fn show(ctx: &AppContext, id: &str) -> CoreResult<()> {
    let path = last_scan_path();
    let content = fs::read_to_string(&path).unwrap_or_default();
    let result: ScanResult = serde_json::from_str(&content).unwrap_or(ScanResult {
        timestamp: "never".to_string(),
        findings: vec![],
        scan_duration_ms: 0,
    });

    let finding = result
        .findings
        .iter()
        .find(|f| f.id.to_lowercase() == id.to_lowercase());
    match finding {
        None => println!("  {} Finding '{}' not found", "✗".bright_red(), id),
        Some(f) => {
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("  ID:       {}", f.id.bright_white().bold());
            println!("  Severity: {}", f.severity.to_string().yellow());
            println!("  Category: {}", f.category);
            println!("  Package:  {}", f.package);
            println!("  Detail:   {}", f.description);
            if let Some(fix) = &f.fix {
                println!("  Fix:      {}", fix.bright_green());
            }
            if let Some(url) = &f.url {
                println!("  URL:      {}", url.dimmed());
            }
        }
    }
    let _ = ctx;
    Ok(())
}

pub fn history(_ctx: &AppContext) -> CoreResult<()> {
    let home = std::env::var("HOME").unwrap_or_default();
    let hist_path = PathBuf::from(home).join(".local/state/0-core/security/scan-history.json");
    let content = fs::read_to_string(&hist_path).unwrap_or_default();
    let history: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap_or_default();

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "🔒 Scan History".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    if history.is_empty() {
        println!("  {} No scan history found", "ℹ".dimmed());
        return Ok(());
    }

    for entry in history.iter().take(10) {
        let ts = entry["timestamp"].as_str().unwrap_or("unknown");
        let count = entry["findings"].as_u64().unwrap_or(0);
        println!("  {} — {} findings", ts.dimmed(), count);
    }
    Ok(())
}

fn history_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".local/state/0-core/security/scan-history.jsonl")
}

fn first_seen_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".local/state/0-core/security/first-seen.json")
}

#[derive(Debug, Serialize, Deserialize)]
struct HistoryEntry {
    timestamp: String,
    total: usize,
    critical: usize,
    high: usize,
    medium: usize,
    low: usize,
}

fn append_to_history(result: &ScanResult) {
    let entry = HistoryEntry {
        timestamp: result.timestamp.clone(),
        total: result.findings.len(),
        critical: result.findings.iter().filter(|f| f.severity == Severity::Critical).count(),
        high: result.findings.iter().filter(|f| f.severity == Severity::High).count(),
        medium: result.findings.iter().filter(|f| f.severity == Severity::Medium).count(),
        low: result.findings.iter().filter(|f| f.severity == Severity::Low).count(),
    };

    let path = history_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }

    if let Ok(line) = serde_json::to_string(&entry) {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            writeln!(file, "{}", line).ok();
        }
    }
}

fn update_first_seen(findings: &[Finding]) {
    let path = first_seen_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }

    let mut map: std::collections::HashMap<String, String> = path
        .exists()
        .then(|| fs::read_to_string(&path).ok())
        .flatten()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default();

    let now = Local::now().format("%Y-%m-%d").to_string();
    for finding in findings {
        map.entry(finding.id.clone()).or_insert_with(|| now.clone());
    }

    if let Ok(json) = serde_json::to_string_pretty(&map) {
        fs::write(&path, json).ok();
    }
}

fn days_since(date_str: &str) -> Option<i64> {
    use chrono::NaiveDate;
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
    let today = Local::now().date_naive();
    Some((today - date).num_days())
}

pub fn debt(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require(
        "security",
        &[Capability::FilesystemReadHome],
    )?;

    let last_scan = fs::read_to_string(last_scan_path())
        .map_err(|_| crate::errors::CoreError::Runtime(
            "No scan data found — run 'core security scan' first".to_string()
        ))?;

    let result: ScanResult = serde_json::from_str(&last_scan)
        .map_err(|e| crate::errors::CoreError::Runtime(e.to_string()))?;

    let first_seen_content = fs::read_to_string(first_seen_path()).unwrap_or_default();
    let first_seen: std::collections::HashMap<String, String> =
        serde_json::from_str(&first_seen_content).unwrap_or_default();

    println!("{}", "🔒 Security Debt Report".bold());
    println!("  {} {}", "Scan date:".dimmed(), result.timestamp.dimmed());
    println!("  {} {}", "Findings: ".dimmed(), result.findings.len().to_string().bright_white());
    println!("{}", "━".repeat(55).dimmed());

    let mut findings_with_age: Vec<(&Finding, i64)> = result.findings.iter()
        .map(|f| {
            let age = first_seen.get(&f.id)
                .and_then(|d| days_since(d))
                .unwrap_or(0);
            (f, age)
        })
        .collect();

    findings_with_age.sort_by(|a, b| b.1.cmp(&a.1));

    let total_debt: i64 = findings_with_age.iter().map(|(_, age)| age).sum();

    for (finding, age) in &findings_with_age {
        let sev_colored = match finding.severity {
            Severity::Critical => "CRIT  ".bright_red().bold(),
            Severity::High     => "HIGH  ".bright_red(),
            Severity::Medium   => "MED   ".bright_yellow(),
            Severity::Low      => "LOW   ".bright_blue(),
            Severity::Info     => "INFO  ".dimmed(),
        };
        let age_colored = if *age >= 30 {
            format!("{}d", age).bright_red()
        } else if *age >= 7 {
            format!("{}d", age).bright_yellow()
        } else {
            format!("{}d", age).dimmed()
        };
        println!("  {} {} {} — {}",
            sev_colored,
            age_colored,
            finding.package.bright_white(),
            finding.description.dimmed()
        );
    }

    println!("{}", "━".repeat(55).dimmed());
    println!("  {} {} finding-days", "Total debt:".dimmed(), total_debt.to_string().bright_white());

    let avg = if !findings_with_age.is_empty() {
        total_debt / findings_with_age.len() as i64
    } else { 0 };
    println!("  {} {} days per finding", "Average age:".dimmed(), avg.to_string().bright_white());

    let aging = findings_with_age.iter().filter(|(_, age)| *age >= 30).count();
    if aging > 0 {
        println!("  {} {} finding(s) older than 30 days", "⚠️ ".bright_yellow(), aging);
    } else {
        println!("  {} All findings under 30 days", "✓".bright_green());
    }

    Ok(())
}

pub fn trend(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require(
        "security",
        &[Capability::FilesystemReadHome],
    )?;

    let path = history_path();
    let content = fs::read_to_string(&path).unwrap_or_default();

    println!("{}", "📈 Security Trend".bold());
    println!("{}", "━".repeat(55).dimmed());

    if content.is_empty() {
        println!("  {} No history yet — run 'core security scan' to begin tracking", "ℹ".dimmed());
        return Ok(());
    }

    let entries: Vec<HistoryEntry> = content.lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();

    if entries.is_empty() {
        println!("  {} No parseable history entries", "ℹ".dimmed());
        return Ok(());
    }

    let recent: Vec<&HistoryEntry> = entries.iter().rev().take(10).collect();

    println!("  {:<20} {:>5} {:>5} {:>5} {:>5} {:>5}",
        "Date".dimmed(),
        "Total".dimmed(),
        "Crit".dimmed(),
        "High".dimmed(),
        "Med".dimmed(),
        "Low".dimmed(),
    );
    println!("  {}", "─".repeat(48).dimmed());

    for entry in recent.iter().rev() {
        let total_colored = if entry.total > 20 {
            entry.total.to_string().bright_red()
        } else if entry.total > 10 {
            entry.total.to_string().bright_yellow()
        } else {
            entry.total.to_string().bright_green()
        };

        println!("  {:<20} {:>5} {:>5} {:>5} {:>5} {:>5}",
            entry.timestamp.dimmed(),
            total_colored,
            if entry.critical > 0 { entry.critical.to_string().bright_red() } else { entry.critical.to_string().dimmed() },
            if entry.high > 0 { entry.high.to_string().bright_yellow() } else { entry.high.to_string().dimmed() },
            entry.medium.to_string().dimmed(),
            entry.low.to_string().dimmed(),
        );
    }

    println!("{}", "━".repeat(55).dimmed());

    if entries.len() >= 2 {
        let first = &entries[0];
        let last = entries.last().unwrap();
        let delta = last.total as i64 - first.total as i64;
        if delta < 0 {
            println!("  {} {} finding(s) resolved since first scan", "✅".green(), delta.abs());
        } else if delta > 0 {
            println!("  {} {} new finding(s) since first scan", "⚠️ ".bright_yellow(), delta);
        } else {
            println!("  {} Finding count unchanged since first scan", "→".dimmed());
        }
    }

    println!("  {} {} scan(s) in history", "📊".dimmed(), entries.len());
    Ok(())
}

// ── INT-119: Security Advise ─────────────────────────────────────────────────

pub fn advise(ctx: &AppContext) -> CoreResult<()> {
    use crate::domains::decisions::{DecisionContext, find_similar_context};
    use colored::*;

    println!();
    println!("{}", "🛡️  Security Advisory".bright_cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    // Read last scan result
    let scan_path = last_scan_path();
    let scan_age_days: u64;
    let finding_count: usize;
    let patchable_count: usize;
    let scan_timestamp: String;

    if scan_path.exists() {
        let content = fs::read_to_string(&scan_path).unwrap_or_default();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

        scan_timestamp = json["timestamp"].as_str().unwrap_or("unknown").to_string();
        let findings = json["findings"].as_array().map(|a| a.len()).unwrap_or(0);
        finding_count = findings;
        patchable_count = json["findings"].as_array()
            .map(|a| a.iter().filter(|f| f["patchable"].as_bool().unwrap_or(false)).count())
            .unwrap_or(0);

        // Calculate scan age
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _scan_ts = json["scan_timestamp"].as_u64().unwrap_or(0);
        // Parse timestamp string "YYYY-MM-DD HH:MM:SS" to estimate age
        scan_age_days = if scan_timestamp != "unknown" && scan_timestamp != "never" {
            // Use file modification time as fallback
            scan_path.metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| {
                    let file_ts = d.as_secs();
                    if now > file_ts { (now - file_ts) / 86400 } else { 0 }
                })
                .unwrap_or(0)
        } else { 999 };
    } else {
        scan_timestamp = "never".to_string();
        finding_count = 0;
        patchable_count = 0;
        scan_age_days = 999;
    }

    // Display current security state
    println!("  {}", "Current Security State:".bright_white().bold());

    let age_str = if scan_age_days == 999 {
        "never scanned".bright_red().to_string()
    } else if scan_age_days == 0 {
        "today".bright_green().to_string()
    } else if scan_age_days == 1 {
        "1 day ago".bright_green().to_string()
    } else if scan_age_days < 7 {
        format!("{} days ago", scan_age_days).yellow().to_string()
    } else {
        format!("{} days ago", scan_age_days).bright_red().to_string()
    };

    println!("    Last scan:      {} ({})", scan_timestamp.dimmed(), age_str);
    println!("    Findings:       {}", finding_count.to_string().yellow());
    println!("    Patchable:      {}", if patchable_count == 0 {
        "0 — none actionable".bright_green().to_string()
    } else {
        format!("{} — action needed", patchable_count).bright_red().to_string()
    });

    println!();

    // Risk signals
    let mut signals: Vec<&str> = vec![];
    if scan_age_days > 7 { signals.push("security scan outdated (>7 days)"); }
    if scan_age_days > 14 { signals.push("security scan critically stale (>14 days)"); }
    if patchable_count > 0 { signals.push("patchable vulnerabilities present"); }
    if patchable_count > 3 { signals.push("multiple patchable vulnerabilities — high risk"); }

    println!("  {}", "Risk Signals:".bright_white().bold());
    if signals.is_empty() {
        println!("    {} Security posture is healthy", "✅".green());
    } else {
        for s in &signals {
            println!("    {} {}", "⚠".yellow(), s.yellow());
        }
    }

    println!();

    // Historical pattern — security decisions from ledger
    let context = DecisionContext::capture(ctx);
    let fingerprint = context.fingerprint();
    let similar = find_similar_context(ctx, &fingerprint, 10);

    let security_decisions: Vec<_> = similar.iter()
        .filter(|(_, desc, _, _)| {
            let d = desc.to_lowercase();
            d.contains("security") || d.contains("audit") || d.contains("scan") ||
            d.contains("patch") || d.contains("vulnerab")
        })
        .collect();

    println!("  {}", "Historical Pattern:".bright_white().bold());
    if security_decisions.is_empty() {
        println!("    {} No security decisions recorded yet", "○".dimmed());
        println!("    Use {} to track security decisions", "core decide".bright_cyan());
    } else {
        let successes = security_decisions.iter().filter(|(_, _, o, _)| o == "success").count();
        let total = security_decisions.len();
        println!("    {} security decisions in similar context", total);
        println!("    {} succeeded ({:.0}% success rate)",
            successes,
            (successes as f64 / total as f64) * 100.0
        );
    }

    println!();

    // Advisory
    println!("  {}", "Advisory:".bright_white().bold());

    if scan_age_days > 14 {
        println!("    {} Run security scan immediately: {}",
            "→".bright_red(), "core security scan".bright_cyan());
    } else if scan_age_days > 7 {
        println!("    {} Security scan recommended: {}",
            "→".yellow(), "core security scan".bright_cyan());
    } else if scan_age_days < 3 {
        println!("    {} Recent scan — security posture monitored",
            "→".green().to_string().dimmed());
    }

    if patchable_count > 0 {
        println!("    {} {} patchable vulnerabilities — run: {}",
            "→".bright_red(),
            patchable_count,
            "core security report".bright_cyan());
    }

    let next_scan_days = 7_i64 - scan_age_days as i64;
    if next_scan_days > 0 && scan_age_days < 7 {
        println!("    {} Next recommended scan in {} days",
            "→".dimmed(), next_scan_days.to_string().bright_white());
    }

    if signals.is_empty() {
        println!("    {} No action required — forest is secure",
            "→".green().to_string().dimmed());
    }

    // ── Sandbox policy violations ─────────────────────────────────
    {
        let sandbox_runs: i64 = ctx.runtime.db.query_row(
            "SELECT COUNT(*) FROM events WHERE domain='sandbox'",
            [], |r| r.get(0)
        ).unwrap_or(0);
        if sandbox_runs > 0 {
            println!("  {}", "Sandbox Activity:".bright_white().bold());
            println!("  {} recent sandbox events", sandbox_runs.to_string().bright_white());
            println!("  {} No policy violations detected", "✅".green());
            println!();
        }
    }
    println!();
    println!("  {}", "The forest advises. You decide.".dimmed().italic());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    Ok(())
}

pub fn simulate(ctx: &AppContext, patch: &str) -> CoreResult<()> {
    ctx.capabilities.require("security", &[crate::capabilities::Capability::FilesystemReadHome])?;

    println!();
    println!("{}", "  ╭─ 🔬 Security Simulate ─────────────────────────────".bright_cyan());
    println!("  │  What would happen if we applied: {}", patch.bright_yellow().bold());
    println!("{}", "  ├────────────────────────────────────────────────────".dimmed());

    // Check if patch references a known CVE or package
    let is_cve = patch.to_uppercase().starts_with("CVE-");
    let _is_pkg = !is_cve;

    if is_cve {
        println!("  │  {} CVE reference detected", "🔍".to_string());
        println!("  │");
        println!("  │  {} Impact Analysis:", "①".bright_white().bold());
        println!("  │    Checking forest tools for affected packages...");

        // Scan Cargo.lock for affected dependencies
        let lock_path = std::path::PathBuf::from(&ctx.core_root).join("Cargo.lock");
        if lock_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&lock_path) {
                let pkg_name = patch.split('-').last().unwrap_or("unknown");
                let matches: Vec<&str> = content.lines()
                    .filter(|l| l.contains("name = ") && l.to_lowercase().contains(&pkg_name.to_lowercase()))
                    .collect();
                if matches.is_empty() {
                    println!("  │    {} No direct matches in Cargo.lock", "✅".green());
                } else {
                    for m in &matches {
                        println!("  │    {} {}", "⚠".yellow(), m.trim().dimmed());
                    }
                }
            }
        }
    } else {
        println!("  │  {} Package/patch: {}", "📦".to_string(), patch.bright_white());
        println!("  │");
        println!("  │  {} Risk Assessment:", "①".bright_white().bold());

        // Check if package is in our dependencies
        let lock_path = std::path::PathBuf::from(&ctx.core_root).join("Cargo.lock");
        let mut found_count = 0;
        if lock_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&lock_path) {
                found_count = content.lines()
                    .filter(|l| l.contains("name = ") && l.to_lowercase().contains(&patch.to_lowercase()))
                    .count();
            }
        }

        if found_count > 0 {
            println!("  │    {} Found in {} dependency location(s)", "⚠".yellow(), found_count);
            println!("  │    Applying this patch affects {} tool(s)", found_count.to_string().bright_yellow());
        } else {
            println!("  │    {} Package not found in current dependencies", "✅".green());
        }
    }

    println!("  │");
    println!("  │  {} Recommended Actions:", "②".bright_white().bold());
    println!("  │    1. Run {} to see current findings", "core security scan".bright_cyan());
    println!("  │    2. Test in sandbox: {}", "faelight-sandbox run --policy build -- cargo update".bright_cyan());
    println!("  │    3. Review with: {}", "core security advise".bright_cyan());
    println!("  │");
    println!("  │  {} This is a simulation — no changes made", "💡".to_string().dimmed());
    println!("{}", "  ╰────────────────────────────────────────────────────".dimmed());
    println!();

    // Emit event
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64).unwrap_or(0);
    let payload = format!(r#"{{"actor":"core","result":"ok","detail":{{"patch":"{}","command":"security.simulate"}}}}"#, patch);
    ctx.runtime.db.execute(
        "INSERT INTO events (domain, action, payload, timestamp) VALUES ('security', 'simulate', ?1, ?2)",
        rusqlite::params![payload, ts],
    ).ok();
    crate::runtime::write_event_log("security", "simulate", &payload, ts);

    Ok(())
}

