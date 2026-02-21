//! faelight security-audit v1.0.0
//! Security vulnerability scanner for Faelight Forest
//! Philosophy: Scan automatically. Report clearly. Fix only with permission.

use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "security-audit")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Security vulnerability scanner for Faelight Forest")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a full security scan (read-only)
    Scan,
    /// Show findings from last scan
    Report {
        /// Show all findings including low severity
        #[arg(long)]
        all: bool,
    },
    /// Show details for a specific finding
    Show {
        /// Finding ID (CVE or RUSTSEC ID)
        id: String,
    },
    /// List scan history
    History,
    /// Run quick scan and output for dot-doctor
    #[command(hide = true)]
    Doctor,
}

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

impl Severity {
    fn color(&self, s: &str) -> ColoredString {
        match self {
            Severity::Critical => s.bright_red().bold(),
            Severity::High => s.red(),
            Severity::Medium => s.yellow(),
            Severity::Low => s.bright_black(),
            Severity::Info => s.cyan(),
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

fn state_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/christian".to_string());
    PathBuf::from(home).join(".local/state/0-core/security")
}

fn last_scan_path() -> PathBuf {
    state_dir().join("last-scan.json")
}

fn history_path() -> PathBuf {
    state_dir().join("scan-history.json")
}

fn ensure_state_dir() -> Result<()> {
    fs::create_dir_all(state_dir())?;
    Ok(())
}

fn run_scan() -> Result<ScanResult> {
    let start = std::time::Instant::now();
    let mut findings = Vec::new();
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "🔒 faelight security-audit — scanning...".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    // --- Cargo audit ---
    print!("  {} Rust crates (cargo-audit)... ", "🦀".cyan());
    findings.extend(scan_cargo());
    println!("{}", "done".dimmed());

    // --- Arch audit ---
    print!("  {} System packages (arch-audit)... ", "📦".cyan());
    findings.extend(scan_arch());
    println!("{}", "done".dimmed());

    // --- File permissions ---
    print!("  {} File permissions... ", "🔑".cyan());
    findings.extend(scan_permissions());
    println!("{}", "done".dimmed());

    // --- Open ports ---
    print!("  {} Network listeners... ", "🌐".cyan());
    findings.extend(scan_network());
    println!("{}", "done".dimmed());

    // --- SSH config ---
    print!("  {} SSH configuration... ", "🔐".cyan());
    findings.extend(scan_ssh());
    println!("{}", "done".dimmed());

    let duration = start.elapsed().as_millis() as u64;

    Ok(ScanResult {
        timestamp,
        findings,
        scan_duration_ms: duration,
    })
}

fn scan_cargo() -> Vec<Finding> {
    let mut findings = Vec::new();
    let home = std::env::var("HOME").unwrap_or_default();
    let core_path = format!("{}/0-core", home);

    let output = Command::new("cargo")
        .args(["audit", "--json"])
        .current_dir(&core_path)
        .output();

    let Ok(output) = output else { return findings };
    let Ok(text) = String::from_utf8(output.stdout) else {
        return findings;
    };

    // Parse JSON output from cargo audit
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

fn scan_arch() -> Vec<Finding> {
    let mut findings = Vec::new();

    let output = Command::new("arch-audit").args(["--json"]).output();
    let Ok(output) = output else { return findings };

    // arch-audit writes JSON to stdout
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
            findings.push(Finding {
                id: avg_id.clone(),
                severity: severity.clone(),
                category: "System Package".to_string(),
                package: pkg.clone(),
                description: format!("{} ({})", vuln_type, cves.join(", ")),
                fix: Some(
                    "Run: sudo pacman -Syu (if patch available — check: arch-audit -u)".to_string(),
                ),
                url: Some(format!("https://security.archlinux.org/{}", avg_id)),
            });
        }
    }

    findings
}

fn scan_permissions() -> Vec<Finding> {
    let mut findings = Vec::new();
    let home = std::env::var("HOME").unwrap_or_default();

    // Check .ssh directory permissions
    let ssh_dir = PathBuf::from(&home).join(".ssh");
    if ssh_dir.exists() {
        if let Ok(meta) = fs::metadata(&ssh_dir) {
            use std::os::unix::fs::PermissionsExt;
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
        // Check key files
        if let Ok(entries) = fs::read_dir(&ssh_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                // Private keys should be 600
                if !name.ends_with(".pub")
                    && !name.ends_with(".d")
                    && name != "known_hosts"
                    && name != "config"
                    && name != "authorized_keys"
                {
                    if let Ok(meta) = fs::metadata(&path) {
                        use std::os::unix::fs::PermissionsExt;
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
        // Look for listeners on all interfaces (0.0.0.0 or ::)
        if line.contains("0.0.0.0:") || line.contains(":::") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(addr) = parts.get(3) {
                // Skip loopback
                if addr.starts_with("0.0.0.0:") || addr.starts_with(":::") {
                    let port = addr.split(':').next_back().unwrap_or("?");
                    // Common expected ports
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
    let ssh_dir = PathBuf::from(&home).join(".ssh");

    if !ssh_dir.exists() {
        return findings;
    }

    if let Ok(entries) = fs::read_dir(&ssh_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Check for RSA keys (weaker than Ed25519)
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

fn print_report(result: &ScanResult, show_all: bool) {
    let critical: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Critical)
        .collect();
    let high: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.severity == Severity::High)
        .collect();
    let medium: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Medium)
        .collect();
    let low: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Low)
        .collect();

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "🔒 Security Audit Report".bold());
    println!("   Last scan: {}", result.timestamp.dimmed());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    if result.findings.is_empty() {
        println!("\n  {} No vulnerabilities found!\n", "✅".green());
        return;
    }

    fn print_findings(findings: &[&Finding], severity: &Severity) {
        if findings.is_empty() {
            return;
        }
        println!(
            "\n  {} {} ({})",
            match severity {
                Severity::Critical => "🔴",
                Severity::High => "🟠",
                Severity::Medium => "🟡",
                Severity::Low => "🔵",
                Severity::Info => "⚪",
            },
            severity.color(&format!("{}", severity)),
            findings.len()
        );
        for f in findings {
            println!(
                "    {} [{}] {} — {}",
                "▶".dimmed(),
                f.package.bright_white(),
                f.id.cyan(),
                f.description
            );
            if let Some(fix) = &f.fix {
                println!("      {} {}", "→".dimmed(), fix.dimmed());
            }
        }
    }

    print_findings(&critical, &Severity::Critical);
    print_findings(&high, &Severity::High);
    print_findings(&medium, &Severity::Medium);
    if show_all {
        print_findings(&low, &Severity::Low);
    } else if !low.is_empty() {
        println!(
            "\n  {} {} low severity findings (use --all to show)",
            "🔵".dimmed(),
            low.len()
        );
    }

    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!(
        "  Total: {} findings  |  Critical: {}  High: {}  Medium: {}  Low: {}",
        result.findings.len(),
        if critical.is_empty() {
            "0".dimmed()
        } else {
            critical.len().to_string().bright_red()
        },
        if high.is_empty() {
            "0".dimmed()
        } else {
            high.len().to_string().red()
        },
        if medium.is_empty() {
            "0".dimmed()
        } else {
            medium.len().to_string().yellow()
        },
        low.len().to_string().dimmed(),
    );
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan => {
            ensure_state_dir()?;
            let result = run_scan()?;

            // Save last scan
            let json = serde_json::to_string_pretty(&result)?;
            fs::write(last_scan_path(), &json)?;

            // Append to history
            let mut history: Vec<ScanResult> = if history_path().exists() {
                let data = fs::read_to_string(history_path())?;
                serde_json::from_str(&data).unwrap_or_default()
            } else {
                Vec::new()
            };
            history.push(serde_json::from_str(&json)?);
            // Keep last 30 scans
            if history.len() > 30 {
                history.drain(0..history.len() - 30);
            }
            fs::write(history_path(), serde_json::to_string_pretty(&history)?)?;

            println!();
            print_report(&result, false);
            println!(
                "\n  {} Scan complete in {}ms",
                "✅".green(),
                result.scan_duration_ms
            );
            println!(
                "  {} Results saved to {}",
                "💾".dimmed(),
                last_scan_path().display().to_string().dimmed()
            );
        }

        Commands::Report { all } => {
            if !last_scan_path().exists() {
                println!(
                    "  {} No scan results found. Run: security-audit scan",
                    "⚠️".yellow()
                );
                return Ok(());
            }
            let data = fs::read_to_string(last_scan_path())?;
            let result: ScanResult = serde_json::from_str(&data)?;
            print_report(&result, all);
        }

        Commands::Show { id } => {
            if !last_scan_path().exists() {
                println!(
                    "  {} No scan results found. Run: security-audit scan",
                    "⚠️".yellow()
                );
                return Ok(());
            }
            let data = fs::read_to_string(last_scan_path())?;
            let result: ScanResult = serde_json::from_str(&data)?;

            let finding = result.findings.iter().find(|f| {
                f.id.to_lowercase().contains(&id.to_lowercase())
                    || f.package.to_lowercase().contains(&id.to_lowercase())
            });

            match finding {
                Some(f) => {
                    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
                    println!("  {} {}", "🔍".cyan(), f.id.bright_white().bold());
                    println!(
                        "  Severity:    {}",
                        f.severity.color(&f.severity.to_string())
                    );
                    println!("  Category:    {}", f.category);
                    println!("  Package:     {}", f.package.bright_white());
                    println!("  Description: {}", f.description);
                    if let Some(fix) = &f.fix {
                        println!("  Fix:         {}", fix.bright_green());
                    }
                    if let Some(url) = &f.url {
                        println!("  Reference:   {}", url.cyan());
                    }
                    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
                }
                None => {
                    println!(
                        "  {} Finding '{}' not found in last scan",
                        "⚠️".yellow(),
                        id
                    );
                }
            }
        }

        Commands::History => {
            if !history_path().exists() {
                println!(
                    "  {} No scan history found. Run: security-audit scan",
                    "⚠️".yellow()
                );
                return Ok(());
            }
            let data = fs::read_to_string(history_path())?;
            let history: Vec<ScanResult> = serde_json::from_str(&data)?;

            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("{}", "📋 Scan History".bold());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            for scan in history.iter().rev().take(10) {
                let critical = scan
                    .findings
                    .iter()
                    .filter(|f| f.severity == Severity::Critical)
                    .count();
                let high = scan
                    .findings
                    .iter()
                    .filter(|f| f.severity == Severity::High)
                    .count();
                let total = scan.findings.len();
                println!(
                    "  {} {}  findings: {}  critical: {}  high: {}",
                    "▶".dimmed(),
                    scan.timestamp.bright_white(),
                    total,
                    if critical > 0 {
                        critical.to_string().bright_red()
                    } else {
                        "0".dimmed()
                    },
                    if high > 0 {
                        high.to_string().red()
                    } else {
                        "0".dimmed()
                    },
                );
            }
        }

        Commands::Doctor => {
            if !last_scan_path().exists() {
                println!("NO_SCAN");
                return Ok(());
            }
            let data = fs::read_to_string(last_scan_path())?;
            let result: ScanResult = serde_json::from_str(&data)?;
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
            println!(
                "SCAN_OK:{}:{}:{}:{}",
                result.timestamp, critical, high, medium
            );
        }
    }

    Ok(())
}
