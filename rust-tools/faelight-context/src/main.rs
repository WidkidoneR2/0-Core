//! faelight-context v1.0.0 — Deep Codebase Understanding Engine
//! INT-159 — The forest understands what it is made of.

use clap::{Parser, Subcommand};
use colored::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(
    name = "faelight-context",
    about = "🌲 Deep codebase understanding engine",
    version = "1.0.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    /// Health check
    #[arg(long)]
    health: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Scan current directory and index codebase
    Scan {
        /// Directory to scan (default: current)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Show architectural map of domains and modules
    Map {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Detect recurring patterns and conventions
    Patterns {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Natural language summary of codebase
    Summary {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Link code changes to intent decisions
    Decisions {
        #[arg(default_value = ".")]
        path: String,
    },
}

#[derive(Debug)]
struct FileInfo {
    path: String,
    language: String,
    lines: usize,
    #[allow(dead_code)]
    size: u64,
}

#[derive(Debug, Default)]
struct CodebaseStats {
    files: Vec<FileInfo>,
    total_lines: usize,
    total_files: usize,
    by_language: HashMap<String, usize>,
    by_domain: HashMap<String, usize>,
    largest_files: Vec<(String, usize)>,
}

fn detect_language(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => "Rust",
        Some("toml") => "TOML",
        Some("md") => "Markdown",
        Some("sh") => "Shell",
        Some("zsh") => "Shell",
        Some("kdl") => "KDL",
        Some("json") => "JSON",
        Some("py") => "Python",
        _ => "Other",
    }
}

fn detect_domain(path: &str) -> String {
    if path.contains("/domains/") {
        let parts: Vec<&str> = path.split("/domains/").collect();
        if let Some(rest) = parts.get(1) {
            let domain = rest.split('/').next().unwrap_or("unknown");
            return domain.to_string();
        }
    }
    if path.contains("rust-tools/") {
        let parts: Vec<&str> = path.split("rust-tools/").collect();
        if let Some(rest) = parts.get(1) {
            let tool = rest.split('/').next().unwrap_or("unknown");
            return format!("tool:{}", tool);
        }
    }
    if path.contains("intents/") {
        return "intents".to_string();
    }
    if path.contains("docs/") {
        return "docs".to_string();
    }
    if path.contains("scripts/") {
        return "scripts".to_string();
    }
    "core".to_string()
}

fn scan_directory(root: &str, lang_filter: Option<&str>) -> CodebaseStats {
    let mut stats = CodebaseStats::default();
    let skip_dirs = ["target", ".git", "node_modules", ".cache"];

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();

        // Skip build artifacts and git
        if skip_dirs.iter().any(|d| {
            path_str.contains(&format!("/{}/", d)) || path_str.contains(&format!("\\{}", d))
        }) {
            continue;
        }

        let lang = detect_language(path);
        if let Some(filter) = lang_filter {
            if lang != filter {
                continue;
            }
        }
        if lang == "Other" {
            continue;
        }

        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let lines = std::fs::read_to_string(path)
            .map(|c| c.lines().count())
            .unwrap_or(0);

        *stats.by_language.entry(lang.to_string()).or_insert(0) += lines;
        let domain = detect_domain(&path_str);
        *stats.by_domain.entry(domain).or_insert(0) += lines;
        stats.total_lines += lines;
        stats.total_files += 1;
        stats.files.push(FileInfo {
            path: path_str,
            language: lang.to_string(),
            lines,
            size,
        });
    }

    // Find largest files
    let mut by_lines: Vec<(String, usize)> = stats
        .files
        .iter()
        .filter(|f| f.language == "Rust")
        .map(|f| (f.path.clone(), f.lines))
        .collect();
    by_lines.sort_by(|a, b| b.1.cmp(&a.1));
    stats.largest_files = by_lines.into_iter().take(10).collect();
    stats
}

fn cmd_scan(path: &str) {
    println!();
    println!("  {} Scanning: {}", "🔍".normal(), path.bright_white());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );

    let stats = scan_directory(path, None);

    println!();
    println!("  {} Overview", "▶".bright_cyan());
    println!(
        "    {} {} files indexed",
        "·".dimmed(),
        stats.total_files.to_string().bright_white()
    );
    println!(
        "    {} {} total lines",
        "·".dimmed(),
        stats.total_lines.to_string().bright_white()
    );
    println!();

    println!("  {} By language", "▶".bright_cyan());
    let mut langs: Vec<(&String, &usize)> = stats.by_language.iter().collect();
    langs.sort_by(|a, b| b.1.cmp(a.1));
    for (lang, lines) in &langs {
        let pct = (*lines * 100) / stats.total_lines.max(1);
        let bar = "█".repeat((pct / 5).max(if **lines > 0 { 1 } else { 0 }));
        println!(
            "    {:<12} {} {:>6} lines ({}%)",
            lang.bright_white(),
            bar.green(),
            lines,
            pct
        );
    }
    println!();

    if !stats.largest_files.is_empty() {
        println!("  {} Largest Rust files", "▶".bright_cyan());
        for (path, lines) in stats.largest_files.iter().take(5) {
            let short = path
                .split('/')
                .rev()
                .take(3)
                .collect::<Vec<_>>()
                .iter()
                .rev()
                .cloned()
                .collect::<Vec<_>>()
                .join("/");
            println!(
                "    {} {:>5} lines  {}",
                "·".dimmed(),
                lines.to_string().bright_white(),
                short.dimmed()
            );
        }
    }
    println!();
}

fn cmd_map(path: &str) {
    println!();
    println!("  {} Architectural Map", "🗺️ ".normal());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );

    let stats = scan_directory(path, Some("Rust"));

    let mut domains: Vec<(&String, &usize)> = stats.by_domain.iter().collect();
    domains.sort_by(|a, b| b.1.cmp(a.1));

    println!();
    println!("  {} Core Engine Domains", "▶".bright_cyan());
    for (domain, lines) in &domains {
        if domain.starts_with("tool:")
            || *domain == "intents"
            || *domain == "docs"
            || *domain == "scripts"
        {
            continue;
        }
        println!(
            "    {} {:<30} {} lines",
            "·".dimmed(),
            domain.bright_white(),
            lines.to_string().dimmed()
        );
    }
    println!();
    println!("  {} Rust Tools", "▶".bright_cyan());
    for (domain, lines) in &domains {
        if !domain.starts_with("tool:") {
            continue;
        }
        let tool = domain.trim_start_matches("tool:");
        println!(
            "    {} {:<30} {} lines",
            "·".dimmed(),
            tool.bright_white(),
            lines.to_string().dimmed()
        );
    }
    println!();
}

fn cmd_patterns(path: &str) {
    println!();
    println!("  {} Codebase Patterns", "🔎".normal());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();

    let stats = scan_directory(path, Some("Rust"));
    let rust_lines: usize = *stats.by_language.get("Rust").unwrap_or(&0);

    // Pattern detection via grep-style counting
    let mut ensure_tables = 0usize;
    let mut pub_fn_count = 0usize;
    let mut colored_usage = 0usize;
    let mut result_returns = 0usize;

    for file in &stats.files {
        if file.language != "Rust" {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&file.path) {
            ensure_tables += content.matches("ensure_tables").count();
            pub_fn_count += content.matches("pub fn ").count();
            colored_usage += content.matches("bright_").count();
            result_returns += content.matches("CoreResult").count();
        }
    }

    println!("  {} Architectural patterns detected:", "▶".bright_cyan());
    println!(
        "    {} {} public functions (pub fn)",
        "·".dimmed(),
        pub_fn_count.to_string().bright_white()
    );
    println!(
        "    {} {} CoreResult return types — consistent error handling",
        "·".dimmed(),
        result_returns.to_string().bright_white()
    );
    println!(
        "    {} {} ensure_tables calls — DB init pattern",
        "·".dimmed(),
        ensure_tables.to_string().bright_white()
    );
    println!(
        "    {} {} colored output calls — Faelight Visual Language",
        "·".dimmed(),
        colored_usage.to_string().bright_white()
    );
    println!();
    println!("  {} Codebase conventions:", "▶".bright_cyan());
    println!(
        "    {} Domain-per-directory structure (engine/src/domains/)",
        "·".dimmed()
    );
    println!(
        "    {} Each domain owns its DB tables via ensure_tables()",
        "·".dimmed()
    );
    println!(
        "    {} CoreResult<()> for all public functions",
        "·".dimmed()
    );
    println!(
        "    {} Faelight Visual Language — forest green + colored output",
        "·".dimmed()
    );
    println!(
        "    {} {} total Rust lines across {} files",
        "·".dimmed(),
        rust_lines.to_string().bright_white(),
        stats.total_files.to_string().bright_white()
    );
    println!();
}

fn cmd_summary(path: &str) {
    let stats = scan_directory(path, None);
    let rust_lines = stats.by_language.get("Rust").copied().unwrap_or(0);
    let domain_count = stats
        .by_domain
        .iter()
        .filter(|(k, _)| !k.starts_with("tool:"))
        .count();
    let tool_count = stats
        .by_domain
        .iter()
        .filter(|(k, _)| k.starts_with("tool:"))
        .count();

    println!();
    println!("  {} Codebase Summary", "📝".normal());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();
    println!("  Faelight Forest is a custom Arch Linux personal computing environment");
    println!(
        "  written in {:.0}% Rust across {} files ({} lines).",
        (rust_lines as f64 / stats.total_lines.max(1) as f64 * 100.0),
        stats.total_files,
        stats.total_lines
    );
    println!();
    println!(
        "  The core engine contains {} domains implementing the intelligence",
        domain_count
    );
    println!("  stack: Intent → Reaction → Prediction → Strategy → Autonomy.");
    println!(
        "  {} standalone Rust tools provide the user-facing interface.",
        tool_count
    );
    println!();
    println!("  Architecture: domain-per-directory, SQLite state persistence,");
    println!("  Faelight Visual Language (forest green palette), and a fully");
    println!("  self-documenting intent ledger tracking every architectural decision.");
    println!();
}

fn cmd_decisions(path: &str) {
    let core_root = std::env::var("HOME").unwrap_or_default() + "/0-core";
    let intents_dir = PathBuf::from(&core_root).join("intents/complete");

    println!();
    println!("  {} Code ↔ Intent Links", "🔗".normal());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();

    // Find recent intents and match to domains
    let mut intent_count = 0;
    if let Ok(entries) = std::fs::read_dir(&intents_dir) {
        let mut intents: Vec<_> = entries.flatten().collect();
        intents.sort_by_key(|e| e.file_name());
        for entry in intents.iter().rev().take(10) {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let title = content
                    .lines()
                    .find(|l| l.starts_with("title:"))
                    .map(|l| {
                        l.trim_start_matches("title:")
                            .trim()
                            .trim_matches('"')
                            .to_string()
                    })
                    .unwrap_or_else(|| name.clone());
                let tags: Vec<&str> = content
                    .lines()
                    .find(|l| l.starts_with("tags:"))
                    .map(|l| l.split('[').nth(1).unwrap_or("").trim_end_matches(']'))
                    .unwrap_or("")
                    .split(',')
                    .map(|t| t.trim())
                    .filter(|t| !t.is_empty())
                    .collect();
                println!("  {} {}", "→".bright_green(), title.bright_white());
                if !tags.is_empty() {
                    println!("    {} {}", "tags:".dimmed(), tags.join(", ").dimmed());
                }
                intent_count += 1;
            }
        }
    }
    println!();
    println!("  {} {} recent decisions shown", "·".dimmed(), intent_count);
    println!("  {} Full ledger: core intent list", "→".dimmed());
    println!();
    let _ = path; // suppress warning
}

fn main() {
    let cli = Cli::parse();

    if cli.health {
        println!("faelight-context v1.0.0 — healthy");
        return;
    }

    match cli.command {
        Some(Command::Scan { path }) => cmd_scan(&path),
        Some(Command::Map { path }) => cmd_map(&path),
        Some(Command::Patterns { path }) => cmd_patterns(&path),
        Some(Command::Summary { path }) => cmd_summary(&path),
        Some(Command::Decisions { path }) => cmd_decisions(&path),
        None => {
            println!();
            println!("  {} faelight-context v1.0.0", "🌲".normal());
            println!("  {} Deep codebase understanding engine", "·".dimmed());
            println!();
            println!("  Commands:");
            println!("    scan       — index codebase files and lines");
            println!("    map        — architectural domain map");
            println!("    patterns   — detect conventions and patterns");
            println!("    summary    — natural language overview");
            println!("    decisions  — link code to intent ledger");
            println!();
        }
    }
}
