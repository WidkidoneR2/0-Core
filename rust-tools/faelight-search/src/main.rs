//! faelight-search v1.0.0
//! 🌲 Unified search — the forest remembers everything. Now you can find it.

use anyhow::Result;
use clap::Parser;
use colored::*;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "faelight-search", about = "🌲 Search everything the forest knows", version = "1.0.0")]
struct Cli {
    /// Search query
    query: String,
    /// Search only files
    #[arg(long)]
    files: bool,
    /// Search only intents
    #[arg(long)]
    intents: bool,
    /// Search only commits
    #[arg(long)]
    commits: bool,
    /// Search only events
    #[arg(long)]
    events: bool,
    /// Search only aliases
    #[arg(long)]
    aliases: bool,
    /// Maximum results per category
    #[arg(short, long, default_value = "5")]
    limit: usize,
}

fn core_root() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/christian")).join("0-core")
}

// ─── FILE SEARCH ─────────────────────────────────────────────────────────────
fn search_files(query: &str, root: &PathBuf, limit: usize) -> Vec<String> {
    let output = Command::new("grep")
        .args([
            "-r", "--include=*.rs", "--include=*.toml", "--include=*.md",
            "--include=*.kdl", "--include=*.zsh", "-l", "-i", query,
            root.to_str().unwrap_or("."),
        ])
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![], stderr: vec![],
        });

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.contains("/target/"))
        .map(|l| {
            // Make path relative
            l.replace(root.to_str().unwrap_or(""), "").trim_start_matches('/').to_string()
        })
        .take(limit)
        .collect()
}

// ─── INTENT SEARCH ───────────────────────────────────────────────────────────
fn search_intents(query: &str, root: &PathBuf, limit: usize) -> Vec<(String, String, String)> {
    let mut results = vec![];
    let intents_dir = root.join("intents");
    let q = query.to_lowercase();

    for subdir in &["future", "complete", "decisions", "incidents", "philosophy"] {
        let dir = intents_dir.join(subdir);
        if !dir.exists() { continue; }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|x| x.to_str()) != Some("md") { continue; }
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                if filename.to_lowercase().contains(&q) || content.to_lowercase().contains(&q) {
                    let title = content.lines()
                        .find(|l| l.starts_with("title:"))
                        .and_then(|l| l.split('"').nth(1))
                        .unwrap_or(&filename)
                        .to_string();
                    let status = content.lines()
                        .find(|l| l.starts_with("status:"))
                        .map(|l| l.replace("status:", "").trim().to_string())
                        .unwrap_or_default();
                    results.push((filename, title, status));
                    if results.len() >= limit { return results; }
                }
            }
        }
    }
    results
}

// ─── COMMIT SEARCH ───────────────────────────────────────────────────────────
fn search_commits(query: &str, root: &PathBuf, limit: usize) -> Vec<(String, String)> {
    let output = Command::new("git")
        .args(["-C", root.to_str().unwrap_or("."),
               "log", "--oneline", &format!("--grep={}", query), "-i",
               &format!("-{}", limit)])
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![], stderr: vec![],
        });

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| {
            let mut parts = l.splitn(2, ' ');
            let hash = parts.next()?.to_string();
            let msg = parts.next()?.to_string();
            Some((hash, msg))
        })
        .collect()
}

// ─── EVENT SEARCH ────────────────────────────────────────────────────────────
fn search_events(query: &str, root: &PathBuf, limit: usize) -> Vec<(String, String, String)> {
    let db_path = root.join("runtime/state.db");
    if !db_path.exists() { return vec![]; }

    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let sql = format!(
        "SELECT domain, action, payload, timestamp FROM events \
         WHERE payload LIKE '%{}%' OR action LIKE '%{}%' OR domain LIKE '%{}%' \
         ORDER BY id DESC LIMIT {}",
        query, query, query, limit
    );

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let rows = match stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(3)?,
        ))
    }) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    rows.filter_map(|r| r.ok())
        .map(|(domain, action, ts)| {
            let time = chrono::DateTime::from_timestamp(ts, 0)
                .map(|d| d.with_timezone(&chrono::Local).format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "??:??:??".to_string());
            (time, domain, action)
        })
        .collect()
}

// ─── ALIAS SEARCH ────────────────────────────────────────────────────────────
fn search_aliases(query: &str, root: &PathBuf, limit: usize) -> Vec<(String, String)> {
    let alias_file = root.join("03-interfaces/stow/shell-zsh/.config/zsh/aliases.zsh");
    if !alias_file.exists() { return vec![]; }

    let content = std::fs::read_to_string(&alias_file).unwrap_or_default();
    let q = query.to_lowercase();

    content.lines()
        .filter(|l| l.starts_with("alias ") && l.to_lowercase().contains(&q))
        .filter_map(|l| {
            let rest = l.trim_start_matches("alias ");
            let mut parts = rest.splitn(2, '=');
            let name = parts.next()?.trim().to_string();
            let cmd = parts.next()?.trim().trim_matches('\'').trim_matches('"').to_string();
            Some((name, cmd))
        })
        .take(limit)
        .collect()
}

// ─── MAIN ─────────────────────────────────────────────────────────────────────
fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = core_root();
    let q = &cli.query;
    let all = !cli.files && !cli.intents && !cli.commits && !cli.events && !cli.aliases;

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("🌲 {}  {}", "faelight-search".bold(), format!("\"{}\"", q).bright_white());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let mut total = 0;

    // Intents
    if all || cli.intents {
        let results = search_intents(q, &root, cli.limit);
        if !results.is_empty() {
            println!("\n{} {}", "📋 Intents".bold().bright_green(), format!("({})", results.len()).dimmed());
            for (file, title, status) in &results {
                println!("  {}  {}", status.dimmed(), title.bright_white());
                println!("  {}", file.dimmed());
            }
            total += results.len();
        }
    }

    // Files
    if all || cli.files {
        let results = search_files(q, &root, cli.limit);
        if !results.is_empty() {
            println!("\n{} {}", "📁 Files".bold().bright_blue(), format!("({})", results.len()).dimmed());
            for f in &results {
                println!("  {}", f.bright_white());
            }
            total += results.len();
        }
    }

    // Commits
    if all || cli.commits {
        let results = search_commits(q, &root, cli.limit);
        if !results.is_empty() {
            println!("\n{} {}", "🔀 Commits".bold().bright_yellow(), format!("({})", results.len()).dimmed());
            for (hash, msg) in &results {
                println!("  {}  {}", hash.dimmed(), msg.bright_white());
            }
            total += results.len();
        }
    }

    // Events
    if all || cli.events {
        let results = search_events(q, &root, cli.limit);
        if !results.is_empty() {
            println!("\n{} {}", "⚡ Events".bold().bright_cyan(), format!("({})", results.len()).dimmed());
            for (time, domain, action) in &results {
                println!("  {}  {}  {}", time.dimmed(), domain.bright_white(), action.dimmed());
            }
            total += results.len();
        }
    }

    // Aliases
    if all || cli.aliases {
        let results = search_aliases(q, &root, cli.limit);
        if !results.is_empty() {
            println!("\n{} {}", "⌨️  Aliases".bold().bright_magenta(), format!("({})", results.len()).dimmed());
            for (name, cmd) in &results {
                println!("  {}  {}", name.bright_white(), cmd.dimmed());
            }
            total += results.len();
        }
    }

    if total == 0 {
        println!("\n  {} No results found for \"{}\"", "·".dimmed(), q);
    }

    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    Ok(())
}
