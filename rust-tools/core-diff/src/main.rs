use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command};

// ANSI colors
const RED: &str = "\x1b[0;31m";
const ORANGE: &str = "\x1b[0;33m";
const BLUE: &str = "\x1b[0;34m";
const GREEN: &str = "\x1b[0;32m";
const NC: &str = "\x1b[0m";

fn health_check() {
    println!("🏥 core-diff health check");
    
    // Check git available
    match Command::new("git").arg("--version").output() {
        Ok(_) => println!("✅ git: available"),
        Err(e) => {
            eprintln!("❌ git: not found - {}", e);
            std::process::exit(1);
        }
    }
    
    // Check 0-core exists
    let home = env::var("HOME").expect("HOME not set");
    let core_dir = PathBuf::from(&home).join("0-core");
    
    if core_dir.exists() {
        println!("✅ 0-core: {} exists", core_dir.display());
    } else {
        eprintln!("❌ 0-core: not found at {}", core_dir.display());
        std::process::exit(1);
    }
    
    // Check it's a git repo
    match Command::new("git")
        .args(["-C", &core_dir.to_string_lossy(), "rev-parse", "--git-dir"])
        .output() {
        Ok(output) if output.status.success() => println!("✅ git repo: valid"),
        _ => {
            eprintln!("❌ git repo: not a repository");
            std::process::exit(1);
        }
    }
    
    println!("\n✅ Core checks passed!");
}

fn main() {
    // Check for health flag
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--health" || args[1] == "health") {
        health_check();
        return;
    }
    
    let home = env::var("HOME").expect("HOME not set");
    let core_dir = PathBuf::from(&home).join("0-core");
    
    // Change to core dir
    if env::set_current_dir(&core_dir).is_err() {
        eprintln!("❌ Error: Cannot access {}", core_dir.display());
        process::exit(1);
    }
    
    // Verify git repo
    if !Command::new("git").args(["rev-parse", "--git-dir"]).output()
        .map(|o| o.status.success()).unwrap_or(false) {
        eprintln!("❌ Error: Not a git repository");
        process::exit(1);
    }
    
    let args: Vec<String> = env::args().collect();
    
    let mut mode = "working-tree";
    let mut git_ref = String::new();
    let mut target_package = String::new();
    let mut open_tool = String::new();
    let mut verbose = false;
    let mut high_risk = false;
    let mut summary = false;
    let mut policy_mode = String::new();
    let mut scan_all = false;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => { show_help(); return; }
            "--version" => { println!("core-diff version 1.2.0 (Rust)"); return; }
            "since" => {
                mode = "since";
                i += 1;
                if i >= args.len() {
                    eprintln!("❌ Error: 'since' requires a git reference");
                    process::exit(2);
                }
                git_ref = args[i].clone();
            }
            "working-tree" => mode = "working-tree",
            "summary" => summary = true,
            "--open" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("❌ Error: --open requires 'delta' or 'meld'");
                    process::exit(2);
                }
                open_tool = args[i].clone();
            }
            "-v" | "--verbose" => verbose = true,
            "--high-risk" => high_risk = true,
            "--all" => scan_all = true,
            "--policy" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("❌ Error: --policy requires 'shell'");
                    process::exit(2);
                }
                policy_mode = args[i].clone();
            }
            arg if arg.starts_with('-') => {
                eprintln!("❌ Error: Unknown option: {}", arg);
                process::exit(2);
            }
            arg => target_package = arg.to_string(),
        }
        i += 1;
    }
    
    // Get changes
    let changes = get_changes(mode, &git_ref);
    
    // Policy analysis mode
    if policy_mode == "shell" {
        if scan_all {
            analyze_shell_policy_all();
        } else {
            analyze_shell_policy(&changes);
        }
        return;
    }

    if changes.is_empty() {
        println!("✅ No changes detected");
        println!();
        println!("💡 Tip: Use 'core-diff since <ref>' to review historical changes");
        return;
    }
    
    // Filter by package if specified
    let filtered_changes: Vec<&str> = if !target_package.is_empty() {
        let pkg_dir = core_dir.join(&target_package);
        if !pkg_dir.exists() {
            eprintln!("❌ Error: Package does not exist: {}", target_package);
            process::exit(2);
        }
        changes.iter()
            .map(|s| s.as_str())
            .filter(|line| {
                let path = line.get(2..).unwrap_or("");
                path.starts_with(&format!("{}/", target_package))
            })
            .collect()
    } else {
        changes.iter().map(|s| s.as_str()).collect()
    };
    
    if filtered_changes.is_empty() {
        println!("✅ No changes in package: {}", target_package);
        return;
    }
    
    // Parse changes into packages
    let mut package_files: HashMap<String, Vec<String>> = HashMap::new();
    
    for line in &filtered_changes {
        if line.len() < 2 { continue; }
        let filepath = &line[2..];
        let package = filepath.split('/').next().unwrap_or(filepath).to_string();
        package_files.entry(package).or_default().push(filepath.to_string());
    }
    
    // Get risk levels
    let mut package_risk: HashMap<String, String> = HashMap::new();
    for package in package_files.keys() {
        package_risk.insert(package.clone(), get_risk_level(&core_dir, package));
    }
    
    // Group by risk
    let mut critical: Vec<&String> = vec![];
    let mut high: Vec<&String> = vec![];
    let mut medium: Vec<&String> = vec![];
    let mut low: Vec<&String> = vec![];
    
    for (pkg, risk) in &package_risk {
        match risk.as_str() {
            "critical" => critical.push(pkg),
            "high" => high.push(pkg),
            "medium" => medium.push(pkg),
            _ => low.push(pkg),
        }
    }
    
    // Sort each group
    critical.sort(); high.sort(); medium.sort(); low.sort();
    
    // Summary mode
    if summary {
        let total_pkgs = package_files.len();
        let total_files: usize = package_files.values().map(|v| v.len()).sum();
        println!("Packages: {}", total_pkgs);
        println!("Files: {}", total_files);
        let risk = if !critical.is_empty() { "CRITICAL" }
            else if !high.is_empty() { "HIGH" }
            else if !medium.is_empty() { "MEDIUM" }
            else { "LOW" };
        println!("Risk: {}", risk);
        return;
    }
    
    // High-risk filter check
    if high_risk && critical.is_empty() && high.is_empty() {
        println!();
        println!("📊 Changes detected in 0-core:");
        println!();
        println!("✅ No critical or high-risk changes");
        let other = medium.len() + low.len();
        if other > 0 {
            println!();
            println!("🔵 {} medium/low-risk package(s) changed", other);
        }
        return;
    }
    
    // Open tool if requested
    if !open_tool.is_empty() {
        match open_tool.as_str() {
            "delta" => open_delta(mode, &git_ref, &target_package),
            "meld" => println!("⚠️  Meld integration not yet in Rust version"),
            _ => eprintln!("❌ Unknown tool: {}", open_tool),
        }
    }
    
    println!();
    println!("📊 Changes detected in 0-core:");
    println!();
    
    // Display critical
    if !critical.is_empty() {
        println!("{}🔴 CRITICAL ({} package(s)):{}", RED, critical.len(), NC);
        for pkg in &critical {
            print_package(pkg, &package_files, verbose);
        }
        println!();
    }
    
    // Display high
    if !high.is_empty() {
        println!("{}🟠 HIGH ({} package(s)):{}", ORANGE, high.len(), NC);
        for pkg in &high {
            print_package(pkg, &package_files, verbose);
        }
        println!();
    }
    
    // Display medium (skip if high-risk filter)
    if !high_risk && !medium.is_empty() {
        println!("{}🔵 MEDIUM ({} package(s)):{}", BLUE, medium.len(), NC);
        for pkg in &medium {
            print_package(pkg, &package_files, verbose);
        }
        println!();
    }
    
    // Display low (skip if high-risk filter)
    if !high_risk && !low.is_empty() {
        println!("{}🟢 LOW ({} package(s)):{}", GREEN, low.len(), NC);
        for pkg in &low {
            print_package(pkg, &package_files, verbose);
        }
        println!();
    }
    
    // Summary
    let total_pkgs = package_files.len();
    let total_files: usize = package_files.values().map(|v| v.len()).sum();
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Summary:");
    println!("   Packages: {}", total_pkgs);
    println!("   Files: {}", total_files);
    
    let (risk_color, risk_label) = if !critical.is_empty() { (RED, "CRITICAL") }
        else if !high.is_empty() { (ORANGE, "HIGH") }
        else if !medium.is_empty() { (BLUE, "MEDIUM") }
        else { (GREEN, "LOW") };
    println!("   Risk: {}{}{}", risk_color, risk_label, NC);
}

fn get_changes(mode: &str, git_ref: &str) -> Vec<String> {
    let output = match mode {
        "since" => Command::new("git")
            .args(["diff", "--name-status", git_ref])
            .output(),
        _ => Command::new("git")
            .args(["diff", "--name-status"])
            .output(),
    };
    
    output.map(|o| {
        String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect()
    }).unwrap_or_default()
}

fn get_risk_level(core_dir: &PathBuf, package: &str) -> String {
    let dotmeta = core_dir.join(package).join(".dotmeta");
    
    if let Ok(content) = fs::read_to_string(&dotmeta) {
        for line in content.lines() {
            if line.starts_with("blast_radius") {
                if let Some(start) = line.find('"') {
                    if let Some(end) = line.rfind('"') {
                        if start < end {
                            return line[start + 1..end].to_string();
                        }
                    }
                }
            }
        }
    }
    
    // Defaults
    if package == "docs" || package.starts_with("theme-") {
        "low".to_string()
    } else if package == "scripts" {
        "medium".to_string()
    } else if package == "hooks" || package == "system" {
        "high".to_string()
    } else {
        "medium".to_string()
    }
}

fn print_package(pkg: &str, files: &HashMap<String, Vec<String>>, verbose: bool) {
    let file_list = files.get(pkg).map(|v| v.as_slice()).unwrap_or(&[]);
    if verbose {
        println!("   {} ({} files):", pkg, file_list.len());
        for f in file_list {
            println!("      {}", f);
        }
    } else {
        println!("   {} ({} files)", pkg, file_list.len());
    }
}

fn open_delta(mode: &str, git_ref: &str, package: &str) {
    println!("🔍 Opening delta for review...");
    println!();
    
    let mut cmd = Command::new("git");
    cmd.arg("diff");
    
    if mode == "since" {
        cmd.arg(git_ref);
    }
    
    if !package.is_empty() {
        cmd.args(["--", &format!("{}/", package)]);
    }
    
    let output = cmd.output();
    
    if let Ok(o) = output {
        // Pipe to delta
        let delta = Command::new("delta")
            .stdin(std::process::Stdio::piped())
            .spawn();
        
        if let Ok(mut child) = delta {
            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                stdin.write_all(&o.stdout).ok();
            }
            child.wait().ok();
        } else {
            // Fallback to colored output
            print!("{}", String::from_utf8_lossy(&o.stdout));
        }
    }
}

fn show_help() {
    println!("core-diff - Package-aware diff tool for 0-core");
    println!();
    println!("USAGE:");
    println!("   core-diff [MODE] [OPTIONS] [PACKAGE]");
    println!();
    println!("MODES:");
    println!("   (default)              Show uncommitted changes");
    println!("   since <ref>            Compare to commit/tag");
    println!("   summary                Stats only");
    println!();
    println!("OPTIONS:");
    println!("   --open <tool>          Open diff tool (delta)");
    println!("   --verbose, -v          Show individual files");
    println!("   --high-risk            Show only critical/high");
    println!("   --help, -h             Show this help");
    println!("   --version              Show version");
    println!();
    println!("EXAMPLES:");
    println!("   core-diff                           # Quick morning check");
    println!("   core-diff since v3.3.5              # Review since release");
    println!("   core-diff wm-sway                   # Deep dive on package");
    println!("   core-diff --open delta              # Terminal diff all");
    println!("   core-diff --high-risk               # Critical/high only");
}

// ═══════════════════════════════════════════════════════════
// 🛡️ SHELL POLICY ANALYSIS
// ═══════════════════════════════════════════════════════════


// ═══════════════════════════════════════════════════════════
// 🛡️ SHELL POLICY ANALYSIS
// ═══════════════════════════════════════════════════════════

fn analyze_shell_policy(changes: &Vec<String>) {
    println!("{}🛡️  Shell Authority Policy Analysis{}", "\x1b[0;36m", NC);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let forbidden_patterns: &[(&str, &str, &str)] = &[
        ("sudo", "Privilege Escalation", "Critical"),
        ("systemctl", "Service Management", "High"),
        ("pacman -S", "Package Management", "High"),
        ("yay -S", "Package Management", "High"),
        ("rm -rf /", "Destructive Operation", "Critical"),
        ("chmod 777", "Insecure Permissions", "High"),
        ("curl | sh", "Remote Execution", "Critical"),
        ("curl | bash", "Remote Execution", "Critical"),
        ("wget | sh", "Remote Execution", "Critical"),
        ("eval \"$(", "Dynamic Execution", "Medium"),
    ];

    let mut total_violations = 0;
    let mut file_violations: Vec<(String, Vec<(&str, &str, &str)>)> = Vec::new();

    let home = env::var("HOME").unwrap_or_default();

    for file in changes {
        // Only check shell scripts
        if !file.ends_with(".sh") && !file.contains("scripts/") {
            continue;
        }

        let file_path = PathBuf::from(&home).join("0-core").join(file);
        
        if let Ok(content) = fs::read_to_string(&file_path) {
            let mut violations: Vec<(&str, &str, &str)> = Vec::new();
            
            for (pattern, domain, severity) in forbidden_patterns {
                if content.contains(pattern) {
                    violations.push((pattern, domain, severity));
                    total_violations += 1;
                }
            }
            
            if !violations.is_empty() {
                file_violations.push((file.clone(), violations));
            }
        }
    }

    if file_violations.is_empty() {
        println!("{}✅ No shell authority violations detected{}", GREEN, NC);
        println!();
        println!("All changed shell scripts follow the Tooling Authority Policy.");
        return;
    }

    for (file, violations) in &file_violations {
        let severity_icon = if violations.iter().any(|(_, _, s)| *s == "Critical") {
            format!("{}🔴 CRITICAL{}", RED, NC)
        } else if violations.iter().any(|(_, _, s)| *s == "High") {
            format!("{}🟠 HIGH{}", ORANGE, NC)
        } else {
            format!("{}🟡 MEDIUM{}", BLUE, NC)
        };

        println!("{}File: {}{} ({})", "\x1b[1m", file, NC, severity_icon);
        
        for (pattern, domain, severity) in violations {
            let sev_color = match *severity {
                "Critical" => RED,
                "High" => ORANGE,
                _ => BLUE,
            };
            println!("  {}❌{} Pattern: {}", sev_color, NC, pattern);
            println!("     Domain: {} | Severity: {}{}{}", domain, sev_color, severity, NC);
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{}Summary:{} {} violations in {} files", "\x1b[1m", NC, total_violations, file_violations.len());
    println!();
    println!("{}Recommendations:{}", "\x1b[1m", NC);
    println!("  • Graduate shell scripts with authority violations to Rust");
    println!("  • Use 'faelight' unified CLI instead of direct commands");
    println!("  • Add shell-policy headers for temporary exceptions");
}

fn analyze_shell_policy_all() {
    println!("{}🛡️  Shell Authority Policy Analysis (Full Scan){}", "\x1b[0;36m", NC);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let forbidden_patterns: &[(&str, &str, &str)] = &[
        ("sudo", "Privilege Escalation", "Critical"),
        ("systemctl", "Service Management", "High"),
        ("pacman -S", "Package Management", "High"),
        ("yay -S", "Package Management", "High"),
        ("rm -rf /", "Destructive Operation", "Critical"),
        ("chmod 777", "Insecure Permissions", "High"),
        ("curl | sh", "Remote Execution", "Critical"),
        ("curl | bash", "Remote Execution", "Critical"),
        ("wget | sh", "Remote Execution", "Critical"),
        ("eval \"$(", "Dynamic Execution", "Medium"),
    ];

    let home = env::var("HOME").unwrap_or_default();
    let scripts_dir = PathBuf::from(&home).join("0-core/scripts");
    
    let mut total_violations = 0;
    let mut file_violations: Vec<(String, Vec<(&str, &str, &str)>)> = Vec::new();

    // Scan scripts directory
    if let Ok(entries) = fs::read_dir(&scripts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            
            let filename = match path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };
            
            // Skip compiled binaries (check if it's a shell script)
            if let Ok(content) = fs::read_to_string(&path) {
                // Skip if not a shell script
                if !content.starts_with("#!/bin/bash") && !content.starts_with("#!/bin/sh") && !content.starts_with("#!/usr/bin/env bash") {
                    continue;
                }
                
                let mut violations: Vec<(&str, &str, &str)> = Vec::new();
                
                for (pattern, domain, severity) in forbidden_patterns {
                    if content.contains(pattern) {
                        violations.push((pattern, domain, severity));
                        total_violations += 1;
                    }
                }
                
                if !violations.is_empty() {
                    file_violations.push((filename, violations));
                }
            }
        }
    }

    // Also scan shell config files
    let shell_files = [
        "shell-zsh/.config/zsh/.zshrc",
        "shell-zsh/.config/zsh/aliases.zsh",
    ];
    
    for file in shell_files {
        let path = PathBuf::from(&home).join("0-core").join(file);
        if let Ok(content) = fs::read_to_string(&path) {
            let mut violations: Vec<(&str, &str, &str)> = Vec::new();
            
            for (pattern, domain, severity) in forbidden_patterns {
                if content.contains(pattern) {
                    violations.push((pattern, domain, severity));
                    total_violations += 1;
                }
            }
            
            if !violations.is_empty() {
                file_violations.push((file.to_string(), violations));
            }
        }
    }

    if file_violations.is_empty() {
        println!("{}✅ No shell authority violations detected{}", GREEN, NC);
        println!();
        println!("All shell scripts follow the Tooling Authority Policy.");
        return;
    }

    // Sort by number of violations
    file_violations.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    for (file, violations) in &file_violations {
        let severity_icon = if violations.iter().any(|(_, _, s)| *s == "Critical") {
            format!("{}🔴 CRITICAL{}", RED, NC)
        } else if violations.iter().any(|(_, _, s)| *s == "High") {
            format!("{}🟠 HIGH{}", ORANGE, NC)
        } else {
            format!("{}🟡 MEDIUM{}", BLUE, NC)
        };

        println!("{}File: {}{} ({})", "\x1b[1m", file, NC, severity_icon);
        
        for (pattern, domain, severity) in violations {
            let sev_color = match *severity {
                "Critical" => RED,
                "High" => ORANGE,
                _ => BLUE,
            };
            println!("  {}❌{} Pattern: {}", sev_color, NC, pattern);
            println!("     Domain: {} | Severity: {}{}{}", domain, sev_color, severity, NC);
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{}Summary:{} {} violations in {} files", "\x1b[1m", NC, total_violations, file_violations.len());
    println!();
    println!("{}Recommendations:{}", "\x1b[1m", NC);
    println!("  • Graduate shell scripts with authority violations to Rust");
    println!("  • Use 'faelight' unified CLI instead of direct commands");
    println!("  • Document exceptions with shell-policy headers");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_risk_level_docs() {
        let core_dir = PathBuf::from("/tmp");
        assert_eq!(get_risk_level(&core_dir, "docs"), "low");
    }

    #[test]
    fn test_risk_level_theme_packages() {
        let core_dir = PathBuf::from("/tmp");
        assert_eq!(get_risk_level(&core_dir, "theme-faelight"), "low");
        assert_eq!(get_risk_level(&core_dir, "theme-dark"), "low");
    }

    #[test]
    fn test_risk_level_scripts() {
        let core_dir = PathBuf::from("/tmp");
        assert_eq!(get_risk_level(&core_dir, "scripts"), "medium");
    }

    #[test]
    fn test_risk_level_hooks() {
        let core_dir = PathBuf::from("/tmp");
        assert_eq!(get_risk_level(&core_dir, "hooks"), "high");
    }

    #[test]
    fn test_risk_level_system() {
        let core_dir = PathBuf::from("/tmp");
        assert_eq!(get_risk_level(&core_dir, "system"), "high");
    }

    #[test]
    fn test_risk_level_default() {
        let core_dir = PathBuf::from("/tmp");
        assert_eq!(get_risk_level(&core_dir, "wm-sway"), "medium");
        assert_eq!(get_risk_level(&core_dir, "shell-zsh"), "medium");
    }
}
