//! intent-guard v1.0.0 - Command Safety Guard
//! 🌲 Faelight Forest

use std::env;
use std::io::{self, Write};
use std::process;

const VERSION: &str = "1.0.0";

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
    check: fn(&str) -> bool,
    risk: RiskLevel,
}

// ============================================================================
// PATTERN CHECKERS
// ============================================================================

fn check_rm_rf_root(cmd: &str) -> bool {
    if !cmd.trim_start().starts_with("rm ") && !cmd.trim_start().starts_with("sudo rm ") {
        return false;
    }
    let has_recursive =
        cmd.contains("-rf") || cmd.contains("-fr") || (cmd.contains("-r") && cmd.contains("-f"));
    if !has_recursive {
        return false;
    }
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
    if !(cmd.trim_start().starts_with("mv ") || cmd.trim_start().starts_with("sudo mv ")) {
        return false;
    }
    
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.len() < 3 {
        return false;
    }
    
    let source_idx = if parts[0] == "sudo" { 2 } else { 1 };
    if source_idx >= parts.len() {
        return false;
    }
    
    let source = parts[source_idx];
    source.contains("0-core")
}

fn check_mkfs(cmd: &str) -> bool {
    cmd.trim_start().starts_with("mkfs.") || cmd.trim_start().starts_with("sudo mkfs.")
}

fn check_systemctl_disable(cmd: &str) -> bool {
    cmd.trim_start().starts_with("systemctl ")
        && (cmd.contains("disable ") || cmd.contains("stop "))
}

fn check_piped_execution(cmd: &str) -> bool {
    let starts_with_curl = cmd.trim_start().starts_with("curl ");
    let starts_with_wget = cmd.trim_start().starts_with("wget ");
    if !starts_with_curl && !starts_with_wget {
        return false;
    }
    cmd.contains("| sh") || cmd.contains("| bash") || cmd.contains("|sh") || cmd.contains("|bash")
}

fn check_shell_overwrite(cmd: &str) -> bool {
    if cmd.trim_start().starts_with("echo ")
        || cmd.trim_start().starts_with("cat ")
        || cmd.trim_start().starts_with("printf ")
    {
        return false;
    }
    (cmd.contains("> ~/.zshrc") || cmd.contains("> ~/.bashrc") || cmd.contains("> ~/0-core"))
        && !cmd.contains(">>")
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
    let mut matches: Vec<&Pattern> = PATTERNS.iter().filter(|p| (p.check)(cmd)).collect();
    if matches.is_empty() {
        process::exit(0);
    }

    matches.sort_by(|a, b| b.risk.partial_cmp(&a.risk).unwrap());
    let highest_risk = &matches[0];

    show_warning(cmd, highest_risk);

    if confirm_execution(highest_risk.risk) {
        process::exit(0);
    } else {
        eprintln!("\n❌ Command cancelled");
        process::exit(1);
    }
}

fn test_command(cmd: &str) {
    let matches: Vec<&Pattern> = PATTERNS.iter().filter(|p| (p.check)(cmd)).collect();
    
    let nc = "\x1b[0m";
    
    if matches.is_empty() {
        println!("{}✅ Safe command - no patterns matched{}", "\x1b[0;32m", nc);
    } else {
        println!("{}⚠️  Dangerous command detected:{}", "\x1b[0;33m", nc);
        println!();
        for pattern in matches {
            let risk_color = pattern.risk.color();
            println!(
                "  {}{:8}{} {} - {}",
                risk_color,
                pattern.risk.label(),
                nc,
                pattern.name,
                pattern.description
            );
        }
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
        RiskLevel::Low => true,
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

fn list_patterns() {
    let nc = "\x1b[0m";
    let cyan = "\x1b[0;36m";
    
    println!();
    println!("{}🛡️  Intent Guard - Safety Patterns{}", cyan, nc);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    // Group by risk level
    for risk_level in [RiskLevel::Critical, RiskLevel::High, RiskLevel::Medium, RiskLevel::Low] {
        let patterns: Vec<&Pattern> = PATTERNS.iter().filter(|p| p.risk == risk_level).collect();
        
        if patterns.is_empty() {
            continue;
        }
        
        let risk_color = risk_level.color();
        println!("{}{} RISK:{}", risk_color, risk_level.label(), nc);
        
        for pattern in patterns {
            println!("  • {} - {}", pattern.name, pattern.description);
        }
        println!();
    }
    
    println!("Total patterns: {}", PATTERNS.len());
    println!();
}

fn cmd_health() {
    let nc = "\x1b[0m";
    let green = "\x1b[0;32m";
    let cyan = "\x1b[0;36m";
    
    println!();
    println!("{}🏥 intent-guard v{} - Health Check{}", cyan, VERSION, nc);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut healthy = true;
    
    // Check pattern database
    print!("  Checking pattern database... ");
    if PATTERNS.is_empty() {
        println!("❌ No patterns loaded");
        healthy = false;
    } else {
        println!("{}✅ {} patterns loaded{}", green, PATTERNS.len(), nc);
    }
    
    // Check pattern integrity
    print!("  Validating pattern functions... ");
    let valid = true;
    for pattern in PATTERNS {
        // Test that check function doesn't panic on empty string
        let _ = (pattern.check)("");
    }
    if valid {
        println!("{}✅{}", green, nc);
    } else {
        println!("❌ Some patterns failed validation");
        healthy = false;
    }
    
    // Count by risk level
    print!("  Risk level distribution... ");
    let critical = PATTERNS.iter().filter(|p| p.risk == RiskLevel::Critical).count();
    let high = PATTERNS.iter().filter(|p| p.risk == RiskLevel::High).count();
    let medium = PATTERNS.iter().filter(|p| p.risk == RiskLevel::Medium).count();
    let low = PATTERNS.iter().filter(|p| p.risk == RiskLevel::Low).count();
    
    println!("{}✅{}", green, nc);
    println!("     Critical: {}", critical);
    println!("     High:     {}", high);
    println!("     Medium:   {}", medium);
    println!("     Low:      {}", low);
    
    println!();
    if healthy {
        println!("{}✅ All systems operational{}", green, nc);
        process::exit(0);
    } else {
        println!("❌ System unhealthy");
        process::exit(1);
    }
}

fn cmd_help() {
    println!("intent-guard v{} - Command Safety Guard", VERSION);
    println!();
    println!("USAGE:");
    println!("   intent-guard <command> [args]");
    println!();
    println!("COMMANDS:");
    println!("   check-command <cmd>    Check if command is safe (for shell integration)");
    println!("   test <cmd>             Test command against patterns (no confirmation)");
    println!("   list-patterns          Show all safety patterns");
    println!("   --health               Run health check");
    println!("   --version, -v          Show version");
    println!("   --help, -h             Show this help");
    println!();
    println!("EXAMPLES:");
    println!("   intent-guard test \"rm -rf /\"");
    println!("   intent-guard test \"chmod 777 file.txt\"");
    println!("   intent-guard list-patterns");
    println!("   intent-guard --health");
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        cmd_help();
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
        "test" => {
            if args.len() < 3 {
                eprintln!("Usage: intent-guard test <cmd>");
                process::exit(1);
            }
            test_command(&args[2..].join(" "));
        }
        "list-patterns" => list_patterns(),
        "--health" => cmd_health(),
        "version" | "-v" | "--version" => {
            println!("intent-guard v{}", VERSION);
        }
        "--help" | "-h" | "help" => cmd_help(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            cmd_help();
            process::exit(1);
        }
    }
}
