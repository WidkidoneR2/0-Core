//! archaeology-0-core v2.0 - System-Wide History Explorer
//! 🌲 Faelight Forest - Dig through your system's evolution

use anyhow::{Context, Result};
use chrono::DateTime;
use clap::{Parser, Subcommand};
use colored::*;
use faelight_core::paths;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Parser)]
#[command(name = "archaeology-0-core")]
#[command(about = "🏛️ System-wide history explorer for 0-Core", long_about = None)]
#[command(version = "2.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Package to explore (if no subcommand provided)
    package: Option<String>,
    
    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Show chronological timeline of all packages
    Timeline {
        /// Limit number of commits
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
    
    /// Show activity from last N days
    Recent {
        /// Number of days to look back
        #[arg(short, long, default_value = "7")]
        days: i32,
    },
    
    /// Show commits for a specific intent
    Intent {
        /// Intent ID to filter by
        id: String,
    },
    
    /// Show changes since a version or tag
    Since {
        /// Version or tag (e.g., v7.0.0)
        version: String,
    },
    
    /// Show history for specific package
    Package {
        /// Package name
        name: String,
        
        /// Limit number of commits
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
    
    /// Show statistics about system evolution
    Stats {
        /// Show stats since version/tag
        #[arg(long)]
        since: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct Commit {
    hash: String,
    short_hash: String,
    date: String,
    timestamp: i64,
    subject: String,
    intent: Option<String>,
    packages: Vec<String>,
    stats: Option<CommitStats>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommitStats {
    files: usize,
    insertions: usize,
    deletions: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let core_dir = paths::core_dir();
    
    // Check if we're in a git repo
    if !core_dir.join(".git").exists() {
        eprintln!("{}", "❌ Not in a git repository!".red().bold());
        std::process::exit(1);
    }
    
    match cli.command {
        Some(Commands::Timeline { limit }) => {
            show_timeline(&core_dir, limit, cli.json)?;
        }
        Some(Commands::Recent { days }) => {
            show_recent(&core_dir, days, cli.json)?;
        }
        Some(Commands::Intent { id }) => {
            show_by_intent(&core_dir, &id, cli.json)?;
        }
        Some(Commands::Since { version }) => {
            show_since(&core_dir, &version, cli.json)?;
        }
        Some(Commands::Package { name, limit }) => {
            show_package(&core_dir, &name, limit, cli.json)?;
        }
        Some(Commands::Stats { since }) => {
            show_stats(&core_dir, since.as_deref(), cli.json)?;
        }
        None => {
            // If no subcommand but package name provided
            if let Some(pkg) = cli.package {
                show_package(&core_dir, &pkg, 50, cli.json)?;
            } else {
                // Default: show recent activity
                show_recent(&core_dir, 7, cli.json)?;
            }
        }
    }
    
    Ok(())
}

fn show_timeline(core_dir: &std::path::PathBuf, limit: usize, json: bool) -> Result<()> {
    if !json {
        println!();
        println!("{}", "🏛️ 0-Core System Timeline".cyan().bold());
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
        println!();
    }
    
    let commits = get_commits_with_files(
        core_dir,
        &["-n", &limit.to_string()],
    )?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&commits)?);
    } else {
        display_commits(&commits);
        println!("{}", format!("Total: {} commits", commits.len()).cyan());
        println!();
    }
    
    Ok(())
}

fn show_recent(core_dir: &std::path::PathBuf, days: i32, json: bool) -> Result<()> {
    if !json {
        println!();
        println!("{}", format!("🏛️ Last {} Days", days).cyan().bold());
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
        println!();
    }
    
    let since = format!("{} days ago", days);
    let commits = get_commits_with_files(
        core_dir,
        &["--since", &since],
    )?;
    
    if commits.is_empty() {
        if !json {
            println!("{}", format!("No commits in last {} days", days).yellow());
        }
        return Ok(());
    }
    
    if json {
        println!("{}", serde_json::to_string_pretty(&commits)?);
    } else {
        display_commits(&commits);
        println!("{}", format!("Total: {} commits", commits.len()).cyan());
        println!();
    }
    
    Ok(())
}

fn show_by_intent(core_dir: &std::path::PathBuf, intent_id: &str, json: bool) -> Result<()> {
    if !json {
        println!();
        println!("{}", format!("🏛️ Intent {}", intent_id).cyan().bold());
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
        println!();
    }
    
    let grep = format!("Intent.*{}", intent_id);
    let commits = get_commits_with_files(
        core_dir,
        &["--grep", &grep],
    )?;
    
    if commits.is_empty() {
        if !json {
            println!("{}", format!("No commits for Intent {}", intent_id).yellow());
        }
        return Ok(());
    }
    
    if json {
        println!("{}", serde_json::to_string_pretty(&commits)?);
    } else {
        display_commits(&commits);
        println!("{}", format!("Total: {} commits", commits.len()).cyan());
        println!();
    }
    
    Ok(())
}

fn show_since(core_dir: &std::path::PathBuf, version: &str, json: bool) -> Result<()> {
    if !json {
        println!();
        println!("{}", format!("🏛️ Since {}", version).cyan().bold());
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
        println!();
    }
    
    let range = format!("{}..HEAD", version);
    let commits = get_commits_with_files(
        core_dir,
        &[&range],
    )?;
    
    if commits.is_empty() {
        if !json {
            println!("{}", format!("No commits since {}", version).yellow());
        }
        return Ok(());
    }
    
    if json {
        println!("{}", serde_json::to_string_pretty(&commits)?);
    } else {
        display_commits(&commits);
        println!("{}", format!("Total: {} commits", commits.len()).cyan());
        println!();
    }
    
    Ok(())
}

fn show_package(core_dir: &std::path::PathBuf, package: &str, limit: usize, json: bool) -> Result<()> {
    if !json {
        println!();
        println!("{}", format!("🏛️ Package: {}", package).cyan().bold());
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
        println!();
    }
    
    let pkg_path = format!("{}/", package);
    let commits = get_commits_simple(
        core_dir,
        &["-n", &limit.to_string(), "--", &pkg_path],
    )?;
    
    if commits.is_empty() {
        if !json {
            println!("{}", format!("No commits for package '{}'", package).yellow());
        }
        return Ok(());
    }
    
    if json {
        println!("{}", serde_json::to_string_pretty(&commits)?);
    } else {
        display_commits(&commits);
        println!("{}", format!("Total: {} commits", commits.len()).cyan());
        println!();
    }
    
    Ok(())
}

fn show_stats(core_dir: &std::path::PathBuf, since: Option<&str>, json: bool) -> Result<()> {
    let mut args = vec!["--shortstat", "--format=%H"];
    let range;
    if let Some(ver) = since {
        range = format!("{}..HEAD", ver);
        args.push(&range);
    }
    
    let commits = get_commits_with_files(core_dir, &args)?;
    
    let total_commits = commits.len();
    let total_files: usize = commits.iter().filter_map(|c| c.stats.as_ref().map(|s| s.files)).sum();
    let total_insertions: usize = commits.iter().filter_map(|c| c.stats.as_ref().map(|s| s.insertions)).sum();
    let total_deletions: usize = commits.iter().filter_map(|c| c.stats.as_ref().map(|s| s.deletions)).sum();
    
    if json {
        let stats = serde_json::json!({
            "commits": total_commits,
            "files_changed": total_files,
            "insertions": total_insertions,
            "deletions": total_deletions,
            "since": since,
        });
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        println!();
        println!("{}", "📊 System Statistics".cyan().bold());
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
        println!();
        if let Some(ver) = since {
            println!("  Since: {}", ver.yellow());
        } else {
            println!("  Scope: {}", "All time".yellow());
        }
        println!();
        println!("  Commits:      {}", total_commits.to_string().green());
        println!("  Files changed: {}", total_files.to_string().blue());
        println!("  Insertions:   {}", format!("+{}", total_insertions).green());
        println!("  Deletions:    {}", format!("-{}", total_deletions).red());
        println!();
    }
    
    Ok(())
}

fn get_commits_simple(core_dir: &std::path::PathBuf, extra_args: &[&str]) -> Result<Vec<Commit>> {
    let mut args = vec!["-C", core_dir.to_str().unwrap(), "log", "--format=%H|%h|%ai|%s"];
    args.extend_from_slice(extra_args);
    
    let output = Command::new("git")
        .args(&args)
        .output()
        .context("Failed to run git log")?;
    
    if !output.status.success() {
        return Ok(Vec::new());
    }
    
    parse_simple_commits(&String::from_utf8_lossy(&output.stdout), core_dir)
}

fn get_commits_with_files(core_dir: &std::path::PathBuf, extra_args: &[&str]) -> Result<Vec<Commit>> {
    let mut args = vec!["-C", core_dir.to_str().unwrap(), "log", "--format=%H|%h|%ai|%s", "--name-only"];
    args.extend_from_slice(extra_args);
    
    let output = Command::new("git")
        .args(&args)
        .output()
        .context("Failed to run git log")?;
    
    parse_commits_with_files(&String::from_utf8_lossy(&output.stdout), core_dir)
}

fn parse_simple_commits(log: &str, core_dir: &std::path::PathBuf) -> Result<Vec<Commit>> {
    let mut commits = Vec::new();
    
    for line in log.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 4 {
            let date_str = parts[2];
            let timestamp = parse_timestamp(date_str);
            
            let subject = parts[3..].join("|");
            let intent = extract_intent(&subject);
            
            let hash = parts[0].to_string();
            let stats = get_commit_stats(core_dir, &hash, None)?;
            
            commits.push(Commit {
                hash,
                short_hash: parts[1].to_string(),
                date: date_str.to_string(),
                timestamp,
                subject,
                intent,
                packages: Vec::new(),
                stats,
            });
        }
    }
    
    Ok(commits)
}

fn parse_commits_with_files(log: &str, core_dir: &std::path::PathBuf) -> Result<Vec<Commit>> {
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
                let timestamp = parse_timestamp(&date);
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
                
                let stats = get_commit_stats(core_dir, &hash, None)?;
                
                commits.push(Commit {
                    hash,
                    short_hash,
                    date,
                    timestamp,
                    subject,
                    intent,
                    packages,
                    stats,
                });
                
                continue;
            }
        }
        
        i += 1;
    }
    
    Ok(commits)
}

fn display_commits(commits: &[Commit]) {
    for commit in commits {
        println!("{} {}", "📅".cyan(), &commit.date[..10].blue());
        println!("   {}  {}", commit.short_hash.dimmed(), commit.subject);
        
        if let Some(intent) = &commit.intent {
            println!("   {} {}", "Intent:".green(), intent.green());
        }
        
        if !commit.packages.is_empty() {
            let pkg_list = commit.packages.join(", ");
            println!("   {} {}", "Packages:".magenta(), pkg_list.magenta());
        }
        
        if let Some(stats) = &commit.stats {
            if stats.files > 0 {
                print!("   Files: {}", stats.files);
                if stats.insertions > 0 {
                    print!(" {}", format!("+{}", stats.insertions).green());
                }
                if stats.deletions > 0 {
                    print!(" {}", format!("-{}", stats.deletions).red());
                }
                println!();
            }
        }
        
        println!();
    }
}

fn parse_stats(stat_output: &str) -> Option<CommitStats> {
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
    
    Some(CommitStats {
        files,
        insertions,
        deletions,
    })
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

fn parse_timestamp(date_str: &str) -> i64 {
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        dt.timestamp()
    } else {
        0
    }
}

fn get_commit_stats(core_dir: &std::path::PathBuf, hash: &str, package: Option<&str>) -> Result<Option<CommitStats>> {
    let pkg_path;
    let args = if let Some(pkg) = package {
        pkg_path = format!("{}/", pkg);
        vec!["-C", core_dir.to_str().unwrap(), "show", "--stat", "--format=", hash, "--", &pkg_path]
    } else {
        vec!["-C", core_dir.to_str().unwrap(), "show", "--stat", "--format=", hash]
    };
    
    let output = Command::new("git")
        .args(&args)
        .output()
        .context("Failed to get commit stats")?;
    
    Ok(parse_stats(&String::from_utf8_lossy(&output.stdout)))
}
