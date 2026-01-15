//! archaeology-0-core v0.2 - System-Wide History Explorer
//! 🌲 Faelight Forest

use std::env;
use std::process::{self, Command};

// ANSI colors
const CYAN: &str = "\x1b[0;36m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[0;33m";
const BLUE: &str = "\x1b[0;34m";
const GRAY: &str = "\x1b[0;90m";
const MAGENTA: &str = "\x1b[0;35m";
const NC: &str = "\x1b[0m";

struct Commit {
    hash: String,
    short_hash: String,
    date: String,
    subject: String,
    intent: Option<String>,
    packages: Vec<String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        show_help();
        return;
    }
    
    let home = env::var("HOME").expect("HOME not set");
    let core_dir = format!("{}/0-core", home);
    
    // Parse mode
    match args[1].as_str() {
        "--timeline" => show_timeline(&core_dir),
        "--this-week" => show_since_days(&core_dir, 7),
        "--by-intent" if args.len() > 2 => show_by_intent(&core_dir, &args[2]),
        "--since" if args.len() > 2 => show_since_version(&core_dir, &args[2]),
        package => show_package(&core_dir, package),
    }
}

fn show_package(core_dir: &str, package: &str) {
    println!();
    println!("{}🌲 0-Core Archaeology - {}{}", CYAN, package, NC);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    let output = Command::new("git")
        .args(["-C", core_dir, "log", "--format=%H|%h|%ai|%s", "--", &format!("{}/", package)])
        .output()
        .expect("Failed to run git log");
    
    if !output.status.success() {
        eprintln!("{}❌ Error: Package not found{}", YELLOW, NC);
        process::exit(1);
    }
    
    let commits = parse_simple_commits(&String::from_utf8_lossy(&output.stdout));
    
    if commits.is_empty() {
        println!("{}No commits found{}", YELLOW, NC);
        return;
    }
    
    display_commits(core_dir, &commits, package);
    println!("{}Total commits: {}{}", CYAN, commits.len(), NC);
    println!();
}

fn show_timeline(core_dir: &str) {
    println!();
    println!("{}🌲 0-Core System Timeline{}", CYAN, NC);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    let output = Command::new("git")
        .args(["-C", core_dir, "log", "--format=%H|%h|%ai|%s", "--name-only"])
        .output()
        .expect("Failed to run git log");
    
    let commits = parse_commits_with_files(&String::from_utf8_lossy(&output.stdout));
    
    display_commits_with_packages(core_dir, &commits);
    println!("{}Total commits: {}{}", CYAN, commits.len(), NC);
    println!();
}

fn show_since_days(core_dir: &str, days: i32) {
    println!();
    println!("{}🌲 0-Core - Last {} Days{}", CYAN, days, NC);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    let since = format!("{} days ago", days);
    let output = Command::new("git")
        .args(["-C", core_dir, "log", &format!("--since={}", since), "--format=%H|%h|%ai|%s", "--name-only"])
        .output()
        .expect("Failed to run git log");
    
    let commits = parse_commits_with_files(&String::from_utf8_lossy(&output.stdout));
    
    if commits.is_empty() {
        println!("{}No commits in last {} days{}", YELLOW, days, NC);
        return;
    }
    
    display_commits_with_packages(core_dir, &commits);
    println!("{}Total commits: {}{}", CYAN, commits.len(), NC);
    println!();
}

fn show_by_intent(core_dir: &str, intent_id: &str) {
    println!();
    println!("{}🌲 0-Core - Intent {}{}", CYAN, intent_id, NC);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    let output = Command::new("git")
        .args(["-C", core_dir, "log", "--format=%H|%h|%ai|%s", "--name-only", &format!("--grep=Intent.*{}", intent_id)])
        .output()
        .expect("Failed to run git log");
    
    let commits = parse_commits_with_files(&String::from_utf8_lossy(&output.stdout));
    
    if commits.is_empty() {
        println!("{}No commits found for Intent {}{}", YELLOW, intent_id, NC);
        return;
    }
    
    display_commits_with_packages(core_dir, &commits);
    println!("{}Total commits: {}{}", CYAN, commits.len(), NC);
    println!();
}

fn show_since_version(core_dir: &str, version: &str) {
    println!();
    println!("{}🌲 0-Core - Since {}{}", CYAN, version, NC);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    let output = Command::new("git")
        .args(["-C", core_dir, "log", &format!("{}..HEAD", version), "--format=%H|%h|%ai|%s", "--name-only"])
        .output()
        .expect("Failed to run git log");
    
    let commits = parse_commits_with_files(&String::from_utf8_lossy(&output.stdout));
    
    if commits.is_empty() {
        println!("{}No commits since {}{}", YELLOW, version, NC);
        return;
    }
    
    display_commits_with_packages(core_dir, &commits);
    println!("{}Total commits: {}{}", CYAN, commits.len(), NC);
    println!();
}

fn parse_simple_commits(log: &str) -> Vec<Commit> {
    let mut commits = Vec::new();
    
    for line in log.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 4 {
            commits.push(Commit {
                hash: parts[0].to_string(),
                short_hash: parts[1].to_string(),
                date: parts[2].to_string(),
                subject: parts[3..].join("|"),
                intent: extract_intent(&parts[3..].join("|")),
                packages: Vec::new(),
            });
        }
    }
    
    commits
}

fn parse_commits_with_files(log: &str) -> Vec<Commit> {
    let mut commits = Vec::new();
    let lines: Vec<&str> = log.lines().collect();
    
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        
        if line.contains('|') {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                let hash = parts[0].to_string();
                let short_hash = parts[1].to_string();
                let date = parts[2].to_string();
                let subject = parts[3..].join("|");
                let intent = extract_intent(&subject);
                
                // Collect affected packages
                let mut packages = Vec::new();
                i += 1;
                
                while i < lines.len() && !lines[i].is_empty() && !lines[i].contains('|') {
                    let file = lines[i].trim();
                    if let Some(pkg) = file.split('/').next() {
                        if !packages.contains(&pkg.to_string()) {
                            packages.push(pkg.to_string());
                        }
                    }
                    i += 1;
                }
                
                commits.push(Commit {
                    hash,
                    short_hash,
                    date,
                    subject,
                    intent,
                    packages,
                });
                
                continue;
            }
        }
        
        i += 1;
    }
    
    commits
}

fn display_commits(core_dir: &str, commits: &[Commit], package: &str) {
    for commit in commits {
        let stats = get_commit_stats(core_dir, &commit.hash, package);
        
        println!("{}📅 {}{}", BLUE, &commit.date[..10], NC);
        println!("   {}{}  {}{}", GRAY, commit.short_hash, commit.subject, NC);
        
        if let Some(intent) = &commit.intent {
            println!("   {}Intent: {}{}", GREEN, intent, NC);
        }
        
        if let Some((files, insertions, deletions)) = stats {
            if files > 0 {
                print!("   Files: {}", files);
                if insertions > 0 {
                    print!(" {}(+{}){}", GREEN, insertions, NC);
                }
                if deletions > 0 {
                    print!(" {}(-{}){}", YELLOW, deletions, NC);
                }
                println!();
            }
        }
        
        println!();
    }
}

fn display_commits_with_packages(core_dir: &str, commits: &[Commit]) {
    for commit in commits {
        let stats = get_commit_stats_all(core_dir, &commit.hash);
        
        println!("{}📅 {}{}", BLUE, &commit.date[..10], NC);
        println!("   {}{}  {}{}", GRAY, commit.short_hash, commit.subject, NC);
        
        if let Some(intent) = &commit.intent {
            println!("   {}Intent: {}{}", GREEN, intent, NC);
        }
        
        // Show affected packages
        if !commit.packages.is_empty() {
            let pkg_list = commit.packages.join(", ");
            println!("   {}Packages: {}{}", MAGENTA, pkg_list, NC);
        }
        
        if let Some((files, insertions, deletions)) = stats {
            if files > 0 {
                print!("   Files: {}", files);
                if insertions > 0 {
                    print!(" {}(+{}){}", GREEN, insertions, NC);
                }
                if deletions > 0 {
                    print!(" {}(-{}){}", YELLOW, deletions, NC);
                }
                println!();
            }
        }
        
        println!();
    }
}

fn get_commit_stats(core_dir: &str, hash: &str, package: &str) -> Option<(usize, usize, usize)> {
    let output = Command::new("git")
        .args(["-C", core_dir, "show", "--stat", "--format=", hash, "--", &format!("{}/", package)])
        .output()
        .ok()?;
    
    parse_stats(&String::from_utf8_lossy(&output.stdout))
}

fn get_commit_stats_all(core_dir: &str, hash: &str) -> Option<(usize, usize, usize)> {
    let output = Command::new("git")
        .args(["-C", core_dir, "show", "--stat", "--format=", hash])
        .output()
        .ok()?;
    
    parse_stats(&String::from_utf8_lossy(&output.stdout))
}

fn parse_stats(stat_output: &str) -> Option<(usize, usize, usize)> {
    let mut files = 0;
    let mut insertions = 0;
    let mut deletions = 0;
    
    for line in stat_output.lines() {
        if line.contains("changed") {
            for part in line.split(',') {
                let part = part.trim();
                if part.contains("file") {
                    if let Some(num) = part.split_whitespace().next() {
                        files = num.parse().unwrap_or(0);
                    }
                }
                if part.contains("insertion") {
                    if let Some(num) = part.split_whitespace().next() {
                        insertions = num.parse().unwrap_or(0);
                    }
                }
                if part.contains("deletion") {
                    if let Some(num) = part.split_whitespace().next() {
                        deletions = num.parse().unwrap_or(0);
                    }
                }
            }
        }
    }
    
    Some((files, insertions, deletions))
}

fn extract_intent(subject: &str) -> Option<String> {
    if let Some(pos) = subject.to_lowercase().find("intent") {
        let after = &subject[pos..];
        for word in after.split_whitespace().skip(1) {
            let cleaned = word.trim_matches(|c: char| !c.is_numeric());
            if !cleaned.is_empty() {
                return Some(cleaned.to_string());
            }
        }
    }
    None
}

fn show_help() {
    println!("archaeology-0-core - System-Wide History Explorer");
    println!();
    println!("USAGE:");
    println!("   archaeology-0-core <package>              # Package-specific history");
    println!("   archaeology-0-core --timeline             # All packages chronologically");
    println!("   archaeology-0-core --this-week            # Last 7 days");
    println!("   archaeology-0-core --since <version>      # Changes since version/tag");
    println!("   archaeology-0-core --by-intent <id>       # Commits for specific intent");
    println!();
    println!("EXAMPLES:");
    println!("   archaeology-0-core wm-sway                # Sway history");
    println!("   archaeology-0-core --timeline             # System timeline");
    println!("   archaeology-0-core --this-week            # This week's work");
    println!("   archaeology-0-core --since v7.0.0         # Since v7.0.0");
    println!("   archaeology-0-core --by-intent 036        # Intent 036 commits");
    println!();
    println!("Shows git history with dates, changes, intents, and affected packages.");
}
