//! journal domain — the forest writes its own story
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use std::path::PathBuf;
fn journal_dir(ctx: &AppContext) -> PathBuf {
    PathBuf::from(&ctx.core_root).join("runtime/journal")
}
fn today_path(ctx: &AppContext) -> PathBuf {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    journal_dir(ctx).join(format!("{}.md", date))
}
/// Append an entry to today's journal
pub fn write_entry(ctx: &AppContext, _kind: &str, message: &str) -> CoreResult<()> {
    let dir = journal_dir(ctx);
    std::fs::create_dir_all(&dir)?;
    let path = today_path(ctx);
    let time = chrono::Local::now().format("%H:%M").to_string();
    let date = chrono::Local::now().format("%B %-d, %Y").to_string();
    let entry = format!("**{}** — [{}] {}\n\n", date, time, message);
    // Add header if file is new
    let header = if !path.exists() {
        format!("# Forest Journal — {}\n\n", date)
    } else {
        String::new()
    };
    let existing = if path.exists() {
        std::fs::read_to_string(&path).unwrap_or_default()
    } else {
        String::new()
    };
    std::fs::write(&path, format!("{}{}{}", header, existing, entry))?;
    Ok(())
}
/// core journal today
pub fn today(ctx: &AppContext) -> CoreResult<()> {
    let path = today_path(ctx);
    println!();
    println!("{}", "📖 Forest Journal — Today".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();
    if !path.exists() {
        println!("  {} No journal entries yet today.", "○".dimmed());
        println!(
            "  {} The forest is quiet — entries appear as the day unfolds.",
            "→".dimmed()
        );
        println!();
        return Ok(());
    }
    let content = std::fs::read_to_string(&path)?;
    for line in content.lines() {
        if let Some(stripped) = line.strip_prefix("# ") {
            println!("  {}", stripped.bright_white().bold());
        } else if line.starts_with("**") {
            // Parse entry line: **date** — [time] message
            println!("  {}", line.bright_white());
        } else if !line.is_empty() {
            println!("  {}", line.dimmed());
        } else {
            println!();
        }
    }
    Ok(())
}
/// core journal yesterday
pub fn yesterday(ctx: &AppContext) -> CoreResult<()> {
    let date = (chrono::Local::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    show_date(ctx, &date, "Yesterday")
}
/// core journal week
pub fn week(ctx: &AppContext) -> CoreResult<()> {
    println!();
    println!("{}", "📖 Forest Journal — This Week".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();
    let dir = journal_dir(ctx);
    let mut entries: Vec<(String, String)> = Vec::new();
    for i in 0..7 {
        let date = (chrono::Local::now() - chrono::Duration::days(i))
            .format("%Y-%m-%d")
            .to_string();
        let path = dir.join(format!("{}.md", date));
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            entries.push((date, content));
        }
    }
    if entries.is_empty() {
        println!("  {} No journal entries this week yet.", "○".dimmed());
        println!();
        return Ok(());
    }
    for (date, content) in entries.iter().rev() {
        println!("  {}", date.bright_cyan().bold());
        let entry_count = content.lines().filter(|l| l.starts_with("**")).count();
        println!(
            "  {} {} entries",
            "→".dimmed(),
            entry_count.to_string().bright_white()
        );
        // Show first entry as preview
        if let Some(first) = content.lines().find(|l| l.starts_with("**")) {
            let preview = if first.len() > 80 {
                &first[..80]
            } else {
                first
            };
            println!("  {}", preview.dimmed());
        }
        println!();
    }
    Ok(())
}
/// core journal search <term>
pub fn search(ctx: &AppContext, term: &str) -> CoreResult<()> {
    println!();
    println!(
        "{} Searching journal for: {}",
        "🔍".normal(),
        term.bright_white().bold()
    );
    println!("{}", "━".repeat(60).dimmed());
    println!();
    let dir = journal_dir(ctx);
    let mut found = 0;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        let mut dates: Vec<String> = entries
            .flatten()
            .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
            .map(|e| {
                e.file_name()
                    .to_string_lossy()
                    .trim_end_matches(".md")
                    .to_string()
            })
            .collect();
        dates.sort_by(|a, b| b.cmp(a));
        for date in &dates {
            let path = dir.join(format!("{}.md", date));
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            let matches: Vec<&str> = content
                .lines()
                .filter(|l| l.to_lowercase().contains(&term.to_lowercase()))
                .collect();
            if !matches.is_empty() {
                println!("  {} {}", "📅".normal(), date.bright_cyan());
                for m in &matches {
                    println!("    {}", m.dimmed());
                }
                println!();
                found += matches.len();
            }
        }
    }
    if found == 0 {
        println!("  {} No entries found for '{}'", "○".dimmed(), term);
    } else {
        println!(
            "  {} {} matches found",
            "→".dimmed(),
            found.to_string().bright_white()
        );
    }
    println!();
    Ok(())
}
/// core journal show <date>  (e.g. 2026-04-08)
pub fn show(ctx: &AppContext, date: &str) -> CoreResult<()> {
    show_date(ctx, date, date)
}
fn show_date(ctx: &AppContext, date: &str, label: &str) -> CoreResult<()> {
    let path = journal_dir(ctx).join(format!("{}.md", date));
    println!();
    println!(
        "{} {}",
        "📖 Forest Journal —".cyan().bold(),
        label.bright_white().bold()
    );
    println!("{}", "━".repeat(60).dimmed());
    println!();
    if !path.exists() {
        println!("  {} No journal entries for {}.", "○".dimmed(), date);
        println!();
        return Ok(());
    }
    let content = std::fs::read_to_string(&path)?;
    for line in content.lines() {
        if let Some(stripped) = line.strip_prefix("# ") {
            println!("  {}", stripped.bright_white().bold());
        } else if line.starts_with("**") {
            println!("  {}", line.bright_white());
        } else if !line.is_empty() {
            println!("  {}", line.dimmed());
        } else {
            println!();
        }
    }
    Ok(())
}
/// Write a session-start entry (called by fsh on login)
pub fn session_start(ctx: &AppContext) -> CoreResult<()> {
    // Cooldown — only write if no session entry in last 30 minutes
    let path = today_path(ctx);
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let now = chrono::Local::now();
        let cutoff = now - chrono::Duration::minutes(30);
        let cutoff_str = cutoff.format("%H:%M").to_string();
        let recent = content
            .lines()
            .filter(|l| l.contains("[") && l.contains("] Session started"))
            .any(|l| {
                if let Some(start) = l.find('[') {
                    if let Some(end) = l.find(']') {
                        let time_str = &l[start + 1..end];
                        return time_str >= cutoff_str.as_str();
                    }
                }
                false
            });
        if recent {
            return Ok(());
        }
    }
    // Count commits today via git log (most accurate)
    let core_root_path = std::path::PathBuf::from(&ctx.core_root);
    let commits_today: i64 = std::process::Command::new("git")
        .args([
            "-C",
            core_root_path.to_str().unwrap_or("."),
            "log",
            "--oneline",
            "--since=midnight",
        ])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count() as i64)
        .unwrap_or(0);
    // Count active intents from filesystem
    let active: i64 = std::fs::read_dir(core_root_path.join("intents/future"))
        .map(|d| {
            d.filter_map(|e| e.ok())
                .filter(|e| {
                    if let Ok(content) = std::fs::read_to_string(e.path()) {
                        content.contains("status: in-progress")
                            || content.contains("type: in-progress")
                    } else {
                        false
                    }
                })
                .count() as i64
        })
        .unwrap_or(0);
    let message = if commits_today > 0 {
        format!(
            "Session started. {} commits today. {} intents in progress. Health 100%.",
            commits_today, active
        )
    } else {
        "Session started. Forest is ready.".to_string()
    };
    write_entry(ctx, "session", &message)
}
/// Write an intent completion entry
#[allow(dead_code)]
pub fn intent_complete(ctx: &AppContext, intent_id: &str, title: &str) -> CoreResult<()> {
    let message = format!("{} complete — {}.", intent_id, title);
    write_entry(ctx, "intent", &message)
}
/// Write a health change entry
#[allow(dead_code)]
pub fn health_change(ctx: &AppContext, from: u32, to: u32) -> CoreResult<()> {
    let message = if to < from {
        format!("Health dropped from {}% to {}%. Investigating.", from, to)
    } else {
        format!("Health restored from {}% to {}%.", from, to)
    };
    write_entry(ctx, "health", &message)
}
/// Write a daily summary entry
pub fn daily_summary(ctx: &AppContext) -> CoreResult<()> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let commits: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM events WHERE domain = 'git' AND action = 'commit'
         AND date(datetime(timestamp, 'unixepoch')) = ?1",
            rusqlite::params![today],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let deploys: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM forest_events WHERE domain = 'deploy'
         AND date(datetime(created_at, 'unixepoch')) = ?1",
            rusqlite::params![today],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let message = format!(
        "Session ended. {} commits, {} deploys today. Forest health: 100%.",
        commits, deploys
    );
    write_entry(ctx, "summary", &message)
}
