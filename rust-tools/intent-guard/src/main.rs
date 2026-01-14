//! intent-guard v0.1 - Command Safety Guard
//! 🌲 Faelight Forest

use std::env;
use std::io::{self, Write};
use std::process;

// ============================================================================
// TYPES & STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    fn color(&self) -> &str {
        match self {
            RiskLevel::Low => "\x1b[0;34m",      // Blue
            RiskLevel::Medium => "\x1b[0;33m",   // Yellow
            RiskLevel::High => "\x1b[0;31m",     // Red
            RiskLevel::Critical => "\x1b[1;31m", // Bold Red
        }
    }

    fn label(&self) -> &str {
        match self {
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM",
            RiskLevel::High => "HIGH",
            RiskLevel::Critical => "CRITICAL",
        }
    }
}

struct Pattern {
    name: &'static str,
    description: &'static str,
    pattern: &'static str, // Simple string matching for now
    risk: RiskLevel,
}

// ============================================================================
// PATTERN DATABASE
// ============================================================================

const PATTERNS: &[Pattern] = &[
    // CRITICAL - Data loss patterns
    Pattern {
        name: "rm_rf_root",
        description: "Recursive delete from root",
        pattern: "rm -rf /",
        risk: RiskLevel::Critical,
    },
    Pattern {
        name: "rm_rf_home",
        description: "Recursive delete from home",
        pattern: "rm -rf ~",
        risk: RiskLevel::Critical,
    },
    Pattern {
        name: "rm_rf_core",
        description: "Delete 0-Core system",
        pattern: "rm -rf ~/0-core",
        risk: RiskLevel::Critical,
    },
    // HIGH - Dangerous system operations
    Pattern {
        name: "chmod_777",
        description: "World-writable permissions",
        pattern: "chmod 777",
        risk: RiskLevel::High,
    },
    Pattern {
        name: "dd_device",
        description: "Direct disk write",
        pattern: "dd", // We'll make this smarter later
        risk: RiskLevel::High,
    },
    // MEDIUM - Package operations
    Pattern {
        name: "pacman_remove",
        description: "Remove system packages",
        pattern: "pacman -R",
        risk: RiskLevel::Medium,
    },
];

// ============================================================================
// CORE LOGIC
// ============================================================================

fn check_command(cmd: &str) {
    // Find matching patterns
    let mut matches: Vec<&Pattern> = PATTERNS
        .iter()
        .filter(|p| cmd.contains(p.pattern))
        .collect();

    if matches.is_empty() {
        // Safe command - pass through
        process::exit(0);
    }

    // Sort by risk (highest first)
    matches.sort_by(|a, b| (b.risk as u8).cmp(&(a.risk as u8)));

    let highest_risk = &matches[0];

    // Show warning
    show_warning(cmd, highest_risk);

    // Get confirmation
    if confirm_execution(highest_risk.risk) {
        process::exit(0); // Allow execution
    } else {
        eprintln!("\n❌ Command cancelled");
        process::exit(1); // Block execution
    }
}

fn show_warning(cmd: &str, pattern: &Pattern) {
    let nc = "\x1b[0m";
    let risk_color = pattern.risk.color();

    eprintln!(
        "\n{}⚠️  {} RISK DETECTED{}",
        risk_color,
        pattern.risk.label(),
        nc
    );
    eprintln!("   Command: {}", cmd);
    eprintln!("   Pattern: {}", pattern.description);
    eprintln!();
}

fn confirm_execution(risk: RiskLevel) -> bool {
    match risk {
        RiskLevel::Low => true, // No confirmation needed
        RiskLevel::Medium => confirm_yes_no(),
        RiskLevel::High => confirm_yes_no(),
        RiskLevel::Critical => confirm_exact("DELETE"),
    }
}

fn confirm_yes_no() -> bool {
    print!("Continue? [y/N]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn confirm_exact(word: &str) -> bool {
    print!("Type '{}' to confirm: ", word);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    input.trim() == word
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: intent-guard check-command <cmd>");
        process::exit(1);
    }

    match args[1].as_str() {
        "check-command" => {
            if args.len() < 3 {
                eprintln!("Usage: intent-guard check-command <cmd>");
                process::exit(1);
            }
            check_command(&args[2..].join(" "));
        }
        "version" | "-v" | "--version" => {
            println!("intent-guard v0.1.0");
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            process::exit(1);
        }
    }
}
