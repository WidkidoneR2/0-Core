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

fn main() {
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
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => { show_help(); return; }
            "--version" => { println!("core-diff version 1.0.0 (Rust)"); return; }
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
    println!("   core-diff wm-hypr                   # Deep dive on package");
    println!("   core-diff --open delta              # Terminal diff all");
    println!("   core-diff --high-risk               # Critical/high only");
}
