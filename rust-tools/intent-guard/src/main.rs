//! intent-guard v0.1 - Command Safety Guard
//! 🌲 Faelight Forest

use std::env;
use std::io::{self, Write};
use std::process;

// ============================================================================
// TYPES & STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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
    check: fn(&str) -> bool, // Function to check pattern
    risk: RiskLevel,
}

// ============================================================================
// PATTERN CHECKERS
// ============================================================================

fn check_rm_rf_root(cmd: &str) -> bool {
    // Only trigger on actual rm commands
    if !cmd.trim_start().starts_with("rm ") && !cmd.trim_start().starts_with("sudo rm ") {
        return false;
    }

    // Check for -rf or -fr flags AND dangerous targets
    let has_recursive =
        cmd.contains("-rf") || cmd.contains("-fr") || (cmd.contains("-r") && cmd.contains("-f"));

    if !has_recursive {
        return false;
    }

    // Check for dangerous targets
    cmd.contains(" /") || cmd.contains("/*") || cmd.contains(" ~") || cmd.contains("~/")
}

fn check_rm_rf_core(cmd: &str) -> bool {
    if !cmd.trim_start().starts_with("rm ") && !cmd.trim_start().starts_with("sudo rm ") {
        return false;
    }

    cmd.contains("~/0-core") || cmd.contains("/home/") && cmd.contains("/0-core")
}

fn check_chmod_777(cmd: &str) -> bool {
    cmd.trim_start().starts_with("chmod ") && cmd.contains("777")
}

fn check_dd(cmd: &str) -> bool {
    cmd.trim_start().starts_with("dd ") && (cmd.contains("of=/dev/") || cmd.contains("if=/dev/"))
}

fn check_pacman_remove(cmd: &str) -> bool {
    (cmd.trim_start().starts_with("pacman ") || cmd.trim_start().starts_with("sudo pacman "))
        && (cmd.contains("-R") || cmd.contains("--remove"))
}

fn check_mv_core(cmd: &str) -> bool {
    (cmd.trim_start().starts_with("mv ") || cmd.trim_start().starts_with("sudo mv "))
        && (cmd.contains("~/0-core") || cmd.contains("/home/") && cmd.contains("/0-core"))
}

fn check_mkfs(cmd: &str) -> bool {
    cmd.trim_start().starts_with("mkfs.") || cmd.trim_start().starts_with("sudo mkfs.")
}

fn check_systemctl_disable(cmd: &str) -> bool {
    cmd.trim_start().starts_with("systemctl ")
        && (cmd.contains("disable ") || cmd.contains("stop "))
}

fn check_shell_overwrite(cmd: &str) -> bool {
    // Detect output redirection to critical shell configs
    (cmd.contains("> ~/.zshrc") || cmd.contains("> ~/.bashrc") || cmd.contains("> ~/0-core"))
        && !cmd.contains(">>") // Append is safer than overwrite
}

fn check_piped_execution(cmd: &str) -> bool {
    // wget/curl piped to shell
    (cmd.contains("wget") || cmd.contains("curl"))
        && (cmd.contains("| sh")
            || cmd.contains("| bash")
            || cmd.contains("|sh")
            || cmd.contains("|bash"))
}

fn check_find_delete(cmd: &str) -> bool {
    cmd.trim_start().starts_with("find ") && (cmd.contains("-delete") || cmd.contains("--delete"))
}

// ============================================================================
// PATTERN DATABASE
// ============================================================================

const PATTERNS: &[Pattern] = &[
    // CRITICAL - Data loss patterns
    Pattern {
        name: "rm_rf_dangerous",
        description: "Recursive delete on critical paths",
        check: check_rm_rf_root,
        risk: RiskLevel::Critical,
    },
    Pattern {
        name: "rm_rf_core",
        description: "Delete 0-Core system",
        check: check_rm_rf_core,
        risk: RiskLevel::Critical,
    },
    Pattern {
        name: "mkfs",
        description: "Format filesystem (DESTRUCTIVE)",
        check: check_mkfs,
        risk: RiskLevel::Critical,
    },
    Pattern {
        name: "mv_core",
        description: "Move 0-Core system",
        check: check_mv_core,
        risk: RiskLevel::Critical,
    },
    Pattern {
        name: "shell_overwrite",
        description: "Overwrite shell configuration",
        check: check_shell_overwrite,
        risk: RiskLevel::Critical,
    },
    // HIGH - Dangerous system operations
    Pattern {
        name: "chmod_777",
        description: "World-writable permissions",
        check: check_chmod_777,
        risk: RiskLevel::High,
    },
    Pattern {
        name: "dd_device",
        description: "Direct disk write",
        check: check_dd,
        risk: RiskLevel::High,
    },
    Pattern {
        name: "piped_execution",
        description: "Download and execute script",
        check: check_piped_execution,
        risk: RiskLevel::High,
    },
    Pattern {
        name: "find_delete",
        description: "Find with delete flag",
        check: check_find_delete,
        risk: RiskLevel::High,
    },
    // MEDIUM - Package operations
    Pattern {
        name: "pacman_remove",
        description: "Remove system packages",
        check: check_pacman_remove,
        risk: RiskLevel::Medium,
    },
    Pattern {
        name: "systemctl_disable",
        description: "Disable/stop system service",
        check: check_systemctl_disable,
        risk: RiskLevel::Medium,
    },
];

// ============================================================================
// CORE LOGIC
// ============================================================================

fn check_command(cmd: &str) {
    // Find matching patterns
    let mut matches: Vec<&Pattern> = PATTERNS.iter().filter(|p| (p.check)(cmd)).collect();

    if matches.is_empty() {
        // Safe command - pass through
        process::exit(0);
    }

    // Sort by risk (highest first)
    matches.sort_by(|a, b| b.risk.partial_cmp(&a.risk).unwrap());

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
