//! faelight-clipboard — Rust clipboard manager for Faelight Forest
//! Implements wlr-data-control-unstable-v1 for Wayland clipboard access.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

// ─── CLI ────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "faelight-clipboard",
    about = "🌲 Faelight Forest clipboard manager",
    version = "0.1.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Copy text to clipboard (reads from stdin if no argument)
    Copy {
        /// Text to copy (optional — pipe via stdin instead)
        text: Option<String>,
    },
    /// Paste clipboard contents to stdout
    Paste,
    /// Show clipboard history
    History {
        /// Number of entries to show (default: 20)
        #[arg(short, long, default_value = "20")]
        limit: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Clear clipboard history
    Clear,
    /// Watch clipboard and record history (daemon mode)
    Watch,
    /// Pick from history using fzf
    Pick,
    /// Show current clipboard without history
    Status,
}

// ─── HISTORY ────────────────────────────────────────────────────────────────

const MAX_HISTORY: usize = 50;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ClipEntry {
    content: String,
    timestamp: String,
    mime: String,
}

fn history_path() -> PathBuf {
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"));
    base.join("faelight").join("clipboard").join("history.json")
}

fn load_history() -> Vec<ClipEntry> {
    let path = history_path();
    if !path.exists() {
        return Vec::new();
    }
    let raw = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_history(entries: &[ClipEntry]) -> Result<()> {
    let path = history_path();
    fs::create_dir_all(path.parent().unwrap())?;
    let json = serde_json::to_string_pretty(entries)?;
    fs::write(&path, json)?;
    Ok(())
}

fn push_to_history(content: &str) -> Result<()> {
    let mut entries = load_history();

    // Deduplicate — move existing to front if same content
    entries.retain(|e| e.content != content);

    let entry = ClipEntry {
        content: content.to_string(),
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        mime: "text/plain".to_string(),
    };

    entries.insert(0, entry);
    entries.truncate(MAX_HISTORY);
    save_history(&entries)
}

// ─── WAYLAND COPY/PASTE ─────────────────────────────────────────────────────
// Phase 1: delegate to wl-copy/wl-paste while we build the native impl.
// Phase 2: replace with native wlr-data-control implementation.

fn wl_copy(text: &str) -> Result<()> {
    use std::process::{Command, Stdio};

    // Try wl-copy first, fall back to xclip/xsel
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .context("wl-copy not found — install wl-clipboard or build native impl")?;

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(text.as_bytes())?;

    child.wait()?;
    Ok(())
}

fn wl_paste() -> Result<String> {
    use std::process::Command;

    let output = Command::new("wl-paste")
        .arg("--no-newline")
        .output()
        .context("wl-paste not found — install wl-clipboard or build native impl")?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ─── COMMANDS ───────────────────────────────────────────────────────────────

fn cmd_copy(text: Option<String>) -> Result<()> {
    let content = match text {
        Some(t) => t,
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf.trim_end_matches('\n').to_string()
        }
    };

    if content.is_empty() {
        anyhow::bail!("nothing to copy — provide text or pipe via stdin");
    }

    wl_copy(&content)?;
    push_to_history(&content)?;
    eprintln!("📋 Copied {} chars", content.len());
    Ok(())
}

fn cmd_paste() -> Result<()> {
    let content = wl_paste()?;
    print!("{}", content);
    Ok(())
}

fn cmd_history(limit: usize, json: bool) -> Result<()> {
    let entries = load_history();

    if entries.is_empty() {
        eprintln!("📋 No clipboard history yet — copy something first");
        return Ok(());
    }

    let shown: Vec<&ClipEntry> = entries.iter().take(limit).collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&shown)?);
        return Ok(());
    }

    println!("📋 Clipboard History (last {})\n", shown.len());
    for (i, entry) in shown.iter().enumerate() {
        let preview = entry.content
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(72)
            .collect::<String>();
        let truncated = if entry.content.len() > 72 || entry.content.contains('\n') {
            "…"
        } else {
            ""
        };
        println!(
            "  {:>2}  {}  {}{}",
            i + 1,
            entry.timestamp,
            preview,
            truncated
        );
    }
    Ok(())
}

fn cmd_clear() -> Result<()> {
    let path = history_path();
    if path.exists() {
        fs::remove_file(&path)?;
    }
    eprintln!("📋 Clipboard history cleared");
    Ok(())
}

fn cmd_status() -> Result<()> {
    let content = wl_paste()?;
    if content.is_empty() {
        eprintln!("📋 Clipboard is empty");
    } else {
        let preview: String = content.chars().take(80).collect();
        let truncated = if content.len() > 80 { "…" } else { "" };
        println!("📋 {}{}", preview, truncated);
        eprintln!("   ({} chars)", content.len());
    }
    Ok(())
}

fn cmd_pick() -> Result<()> {
    use std::process::{Command, Stdio};

    let entries = load_history();
    if entries.is_empty() {
        anyhow::bail!("no clipboard history to pick from");
    }

    // Build fzf input: one line per entry
    let input = entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let preview = e.content.lines().next().unwrap_or("").chars().take(72).collect::<String>();
            format!("{:>2}  {}  {}", i + 1, e.timestamp, preview)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = Command::new("fzf")
        .arg("--prompt=📋 clipboard> ")
        .arg("--height=40%")
        .arg("--reverse")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("fzf not found")?;

    child.stdin.as_mut().unwrap().write_all(input.as_bytes())?;
    let output = child.wait_with_output()?;

    if output.status.success() {
        let line = String::from_utf8_lossy(&output.stdout);
        let line = line.trim();
        // Parse index from line
        let idx: usize = line
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<usize>().ok())
            .context("could not parse selection")?;

        if let Some(entry) = entries.get(idx - 1) {
            wl_copy(&entry.content)?;
            eprintln!("📋 Copied entry {}", idx);
        }
    }

    Ok(())
}

fn cmd_watch() -> Result<()> {
    eprintln!("👁️  faelight-clipboard watch — monitoring clipboard");
    eprintln!("   History: {}", history_path().display());
    eprintln!("   Press Ctrl+C to stop\n");

    let mut last = String::new();

    loop {
        if let Ok(content) = wl_paste() {
            if !content.is_empty() && content != last {
                push_to_history(&content)?;
                let preview: String = content.chars().take(60).collect();
                eprintln!("📋 Recorded: {}…", preview);
                last = content;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

// ─── MAIN ───────────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Copy { text } => cmd_copy(text),
        Command::Paste => cmd_paste(),
        Command::History { limit, json } => cmd_history(limit, json),
        Command::Clear => cmd_clear(),
        Command::Status => cmd_status(),
        Command::Pick => cmd_pick(),
        Command::Watch => cmd_watch(),
    };

    if let Err(e) = result {
        eprintln!("❌ {e}");
        std::process::exit(1);
    }
}
