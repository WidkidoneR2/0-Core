#![allow(dead_code)]
use crate::app::context::AppContext;
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
