#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use chrono::{DateTime, Duration, Local};
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

// ── view: delegates to workspace-view (swaymsg TUI) ──────────────────────────
pub fn view(_ctx: &AppContext, active: bool, summary: bool, json: bool) -> CoreResult<()> {
    let mut args = vec![];
    if active {
        args.push("--active");
    }
    if summary {
        args.push("--summary");
    }
    if json {
        args.push("--json");
    }
    Command::new("workspace-view").args(&args).status()?;
    Ok(())
}

// ── fm: delegates to faelight-fm (full TUI) ──────────────────────────────────
pub fn fm(_ctx: &AppContext, args: &[String]) -> CoreResult<()> {
    Command::new("faelight-fm").args(args).status()?;
    Ok(())
}

// ── recent: native walkdir implementation ────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FileType {
    Code,
    Document,
    Image,
    Video,
    Archive,
    Config,
    Other,
}

impl FileType {
    fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "go" | "java" | "sh") => {
                FileType::Code
            }
            Some("md" | "txt" | "pdf" | "doc" | "docx" | "odt") => FileType::Document,
            Some("png" | "jpg" | "jpeg" | "gif" | "svg" | "webp") => FileType::Image,
            Some("mp4" | "mkv" | "avi" | "mov" | "webm") => FileType::Video,
            Some("zip" | "tar" | "gz" | "xz" | "7z" | "rar") => FileType::Archive,
            Some("toml" | "yaml" | "yml" | "json" | "ini" | "conf") => FileType::Config,
            _ => FileType::Other,
        }
    }
    fn icon(&self) -> &str {
        match self {
            FileType::Code => "🦀",
            FileType::Document => "📄",
            FileType::Image => "🖼️ ",
            FileType::Video => "🎬",
            FileType::Archive => "📦",
            FileType::Config => "⚙️ ",
            FileType::Other => "📁",
        }
    }
    fn label(&self) -> &str {
        match self {
            FileType::Code => "Code",
            FileType::Document => "Documents",
            FileType::Image => "Images",
            FileType::Video => "Videos",
            FileType::Archive => "Archives",
            FileType::Config => "Config",
            FileType::Other => "Other",
        }
    }
}

#[derive(Clone)]
struct RecentFile {
    path: PathBuf,
    modified: DateTime<Local>,
    size: u64,
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

fn format_time_ago(time: &DateTime<Local>) -> String {
    let diff = Local::now().signed_duration_since(*time);
    if diff.num_hours() < 1 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else {
        format!("{}d ago", diff.num_days())
    }
}

fn range_duration(range: &str) -> Duration {
    match range {
        "hour" => Duration::hours(1),
        "week" => Duration::weeks(1),
        "month" => Duration::days(30),
        _ => Duration::hours(24), // "today" default
    }
}

fn range_label(range: &str) -> &str {
    match range {
        "hour" => "Last Hour",
        "week" => "Last Week",
        "month" => "Last Month",
        _ => "Last 24 Hours",
    }
}

pub fn recent(_ctx: &AppContext, range: &str, limit: u32, full_paths: bool) -> CoreResult<()> {
    let home = std::env::var("HOME").unwrap_or_default();
    let search_dir = PathBuf::from(&home);
    let cutoff = Local::now() - range_duration(range);
    let limit = limit as usize;

    println!("{}", "🕒 Recent Files Dashboard".bright_cyan().bold());
    println!("{}", "═".repeat(60).bright_black());
    println!(
        "📂 Searching: {}",
        search_dir.display().to_string().bright_white()
    );
    println!("⏰ Range: {}", range_label(range).bright_yellow());
    println!();

    let mut files_by_type: HashMap<FileType, Vec<RecentFile>> = HashMap::new();

    for entry in WalkDir::new(&search_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.components().any(|c| {
            c.as_os_str()
                .to_str()
                .map(|s| s.starts_with('.') || s == "node_modules" || s == "target")
                .unwrap_or(false)
        }) {
            continue;
        }

        if let Ok(meta) = fs::metadata(path) {
            if let Ok(modified) = meta.modified() {
                let modified: DateTime<Local> = modified.into();
                if modified > cutoff {
                    files_by_type
                        .entry(FileType::from_path(path))
                        .or_default()
                        .push(RecentFile {
                            path: path.to_path_buf(),
                            modified,
                            size: meta.len(),
                        });
                }
            }
        }
    }

    let order = [
        FileType::Code,
        FileType::Document,
        FileType::Config,
        FileType::Image,
        FileType::Video,
        FileType::Archive,
        FileType::Other,
    ];

    for category in order {
        if let Some(files) = files_by_type.get_mut(&category) {
            if files.is_empty() {
                continue;
            }
            files.sort_by(|a, b| b.modified.cmp(&a.modified));
            let count = files.len();
            println!(
                "{} {} ({})",
                category.icon(),
                category.label().bright_green().bold(),
                format!("{} files", count).bright_black()
            );
            for file in files.iter().take(limit) {
                let display = if full_paths {
                    file.path.clone()
                } else {
                    file.path
                        .strip_prefix(&search_dir)
                        .unwrap_or(&file.path)
                        .to_path_buf()
                };
                println!(
                    "  {} {} {}",
                    display.display().to_string().bright_white(),
                    format_size(file.size).bright_black(),
                    format_time_ago(&file.modified).bright_yellow()
                );
            }
            if count > limit {
                println!(
                    "  {}",
                    format!("(+{} more...)", count - limit).bright_black()
                );
            }
            println!();
        }
    }

    let total: usize = files_by_type.values().map(|v| v.len()).sum();
    println!("{}", "─".repeat(60).bright_black());
    println!(
        "{} {}",
        "📊 Total:".bright_cyan(),
        format!("{} files modified in {}", total, range_label(range)).bright_white()
    );
    Ok(())
}
