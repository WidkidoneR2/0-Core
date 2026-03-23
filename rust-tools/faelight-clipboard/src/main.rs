//! faelight-clipboard v0.2.0 — Rust clipboard manager for Faelight Forest
//! Native wlr-data-control implementation — zero C clipboard dependencies.

mod wayland;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

// ─── CLI ────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "faelight-clipboard",
    about = "🌲 Faelight Forest clipboard manager — native Rust, zero C",
    version = "0.2.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Copy text to clipboard (reads from stdin if no argument)
    Copy {
        /// Text to copy
        text: Option<String>,
    },
    /// Paste clipboard contents to stdout
    Paste,
    /// Show clipboard history
    History {
        #[arg(short, long, default_value = "20")]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    /// Clear clipboard history
    Clear,
    /// Pick from history using fzf
    Pick,
    /// Show current clipboard content
    Status,
    /// Watch clipboard and record history (daemon mode)
    Watch,
    /// Hidden: hold Wayland selection (spawned by copy)
    #[command(hide = true)]
    Hold,
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
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("faelight")
        .join("clipboard")
        .join("history.json")
}

fn load_history() -> Vec<ClipEntry> {
    let path = history_path();
    if !path.exists() {
        return Vec::new();
    }
    serde_json::from_str(&fs::read_to_string(&path).unwrap_or_default()).unwrap_or_default()
}

fn save_history(entries: &[ClipEntry]) -> Result<()> {
    let path = history_path();
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, serde_json::to_string_pretty(entries)?)?;
    Ok(())
}

fn push_to_history(content: &str) -> Result<()> {
    let mut entries = load_history();
    entries.retain(|e| e.content != content);
    entries.insert(
        0,
        ClipEntry {
            content: content.to_string(),
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            mime: "text/plain;charset=utf-8".to_string(),
        },
    );
    entries.truncate(MAX_HISTORY);
    save_history(&entries)
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

    // Spawn daemon to hold the Wayland selection
    // Daemon exits when another app takes the clipboard
    let exe = std::env::current_exe().context("cannot find own binary")?;
    std::process::Command::new(&exe)
        .arg("hold")
        .env("FAELIGHT_CLIP_CONTENT", &content)
        .spawn()
        .context("failed to spawn hold daemon")?;

    push_to_history(&content)?;
    eprintln!("📋 Copied {} chars (native Wayland)", content.len());
    Ok(())
}

fn cmd_paste() -> Result<()> {
    let content = wayland::native_paste()?;
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
        let preview: String = entry
            .content
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(72)
            .collect();
        let cont = if entry.content.len() > 72 || entry.content.contains('\n') {
            "…"
        } else {
            ""
        };
        println!("  {:>2}  {}  {}{}", i + 1, entry.timestamp, preview, cont);
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
    let content = wayland::native_paste()?;
    if content.is_empty() {
        eprintln!("📋 Clipboard is empty");
    } else {
        let preview: String = content.chars().take(80).collect();
        let t = if content.len() > 80 { "…" } else { "" };
        println!("📋 {}{}", preview, t);
        eprintln!("   ({} chars, native Wayland)", content.len());
    }
    Ok(())
}

fn cmd_pick() -> Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let entries = load_history();
    if entries.is_empty() {
        anyhow::bail!("no clipboard history to pick from");
    }

    let input = entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let p: String = e
                .content
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(72)
                .collect();
            format!("{:>2}  {}  {}", i + 1, e.timestamp, p)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = Command::new("fzf")
        .args(["--prompt=📋 clipboard> ", "--height=40%", "--reverse"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("fzf not found")?;

    child.stdin.as_mut().unwrap().write_all(input.as_bytes())?;
    let output = child.wait_with_output()?;

    if output.status.success() {
        let line = String::from_utf8_lossy(&output.stdout);
        let idx: usize = line
            .trim()
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .context("could not parse selection")?;
        if let Some(entry) = entries.get(idx - 1) {
            // Spawn daemon to hold the new selection
            let exe = std::env::current_exe()?;
            std::process::Command::new(&exe)
                .arg("hold")
                .env("FAELIGHT_CLIP_CONTENT", &entry.content)
                .spawn()?;
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
        if let Ok(content) = wayland::native_paste() {
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

fn cmd_hold() -> Result<()> {
    let content = std::env::var("FAELIGHT_CLIP_CONTENT").unwrap_or_default();
    wayland::native_copy_daemon(content)
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
        Command::Hold => cmd_hold(),
    };
    if let Err(e) = result {
        eprintln!("❌ {e}");
        std::process::exit(1);
    }
}
