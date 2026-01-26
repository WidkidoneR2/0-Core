use chrono::{DateTime, Duration, Local};
use clap::{Parser, ValueEnum};
use colored::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "recent-files")]
#[command(about = "🕒 Recent Files Dashboard - Find files by modification time")]
struct Args {
    /// Time range to search
    #[arg(value_enum, default_value_t = TimeRange::Today)]
    range: TimeRange,

    /// Directory to search (default: $HOME)
    #[arg(short, long)]
    dir: Option<PathBuf>,

    /// Show full paths instead of relative
    #[arg(short, long)]
    full_paths: bool,

    /// Maximum results per category
    #[arg(short, long, default_value_t = 10)]
    limit: usize,

    /// Open files interactively with fzf
    #[arg(short = 'o', long)]
    open: bool,

    /// Open the most recent file immediately
    #[arg(long)]
    open_first: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum TimeRange {
    Hour,
    Today,
    Week,
    Month,
}

impl TimeRange {
    fn duration(&self) -> Duration {
        match self {
            TimeRange::Hour => Duration::hours(1),
            TimeRange::Today => Duration::hours(24),
            TimeRange::Week => Duration::weeks(1),
            TimeRange::Month => Duration::days(30),
        }
    }

    fn label(&self) -> &str {
        match self {
            TimeRange::Hour => "Last Hour",
            TimeRange::Today => "Last 24 Hours",
            TimeRange::Week => "Last Week",
            TimeRange::Month => "Last Month",
        }
    }
}

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
            Some("rs") | Some("py") | Some("js") | Some("ts") | Some("c") | Some("cpp") 
            | Some("h") | Some("go") | Some("java") | Some("sh") => FileType::Code,
            
            Some("md") | Some("txt") | Some("pdf") | Some("doc") | Some("docx") 
            | Some("odt") => FileType::Document,
            
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("svg") 
            | Some("webp") => FileType::Image,
            
            Some("mp4") | Some("mkv") | Some("avi") | Some("mov") | Some("webm") => FileType::Video,
            
            Some("zip") | Some("tar") | Some("gz") | Some("xz") | Some("7z") 
            | Some("rar") => FileType::Archive,
            
            Some("toml") | Some("yaml") | Some("yml") | Some("json") | Some("ini") 
            | Some("conf") => FileType::Config,
            
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

fn main() {
    let args = Args::parse();

    let search_dir = args.dir.clone().unwrap_or_else(|| {
        std::env::var("HOME")
            .map(PathBuf::from)
            .expect("HOME not set")
    });

    println!("{}", "🕒 Recent Files Dashboard".bright_cyan().bold());
    println!("{}", "═".repeat(60).bright_black());
    println!("📂 Searching: {}", search_dir.display().to_string().bright_white());
    println!("⏰ Range: {}", args.range.label().bright_yellow());
    println!();

    let cutoff = Local::now() - args.range.duration();
    let mut files_by_type: HashMap<FileType, Vec<RecentFile>> = HashMap::new();
    let mut all_files: Vec<RecentFile> = Vec::new();

    // Walk directory and find recent files
    for entry in WalkDir::new(&search_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        
        // Skip hidden files and common ignore patterns
        if path.components().any(|c| {
            c.as_os_str()
                .to_str()
                .map(|s| s.starts_with('.') || s == "node_modules" || s == "target")
                .unwrap_or(false)
        }) {
            continue;
        }

        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                let modified: DateTime<Local> = modified.into();
                
                if modified > cutoff {
                    let file_type = FileType::from_path(path);
                    let recent_file = RecentFile {
                        path: path.to_path_buf(),
                        modified,
                        size: metadata.len(),
                    };
                    
                    files_by_type
                        .entry(file_type)
                        .or_insert_with(Vec::new)
                        .push(recent_file.clone());
                    
                    all_files.push(recent_file);
                }
            }
        }
    }

    // Sort all files by modification time
    all_files.sort_by(|a, b| b.modified.cmp(&a.modified));

    // Feature: Open first file immediately
    if args.open_first {
        if let Some(first) = all_files.first() {
            let editor = env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
            println!("{}", format!("📝 Opening: {}", first.path.display()).bright_green());
            let _ = Command::new(editor)
                .arg(&first.path)
                .status();
            return;
        } else {
            println!("{}", "❌ No recent files found".bright_red());
            return;
        }
    }

    // Feature: Interactive opener
    if args.open {
        open_interactive(&all_files, &search_dir, args.full_paths);
        return;
    }

    // Sort and display by category
    let categories = [
        FileType::Code,
        FileType::Document,
        FileType::Config,
        FileType::Image,
        FileType::Video,
        FileType::Archive,
        FileType::Other,
    ];

    let base_path = if args.full_paths {
        PathBuf::from("")
    } else {
        search_dir.clone()
    };

    for category in categories {
        if let Some(files) = files_by_type.get_mut(&category) {
            if files.is_empty() {
                continue;
            }

            files.sort_by(|a, b| b.modified.cmp(&a.modified));
            let count = files.len();
            
            println!("{} {} ({})", 
                category.icon(), 
                category.label().bright_green().bold(),
                format!("{} files", count).bright_black()
            );
            
            for file in files.iter().take(args.limit) {
                let rel_path = if args.full_paths {
                    file.path.clone()
                } else {
                    file.path
                        .strip_prefix(&base_path)
                        .unwrap_or(&file.path)
                        .to_path_buf()
                };
                
                let size = format_size(file.size);
                let time_ago = format_time_ago(&file.modified);
                
                println!("  {} {} {}",
                    rel_path.display().to_string().bright_white(),
                    size.bright_black(),
                    time_ago.bright_yellow()
                );
            }
            
            if count > args.limit {
                println!("  {} more files...", 
                    format!("(+{}", count - args.limit).bright_black()
                );
            }
            
            println!();
        }
    }

    // Summary
    let total: usize = files_by_type.values().map(|v| v.len()).sum();
    println!("{}", "─".repeat(60).bright_black());
    println!("{} {}", 
        "📊 Total:".bright_cyan(),
        format!("{} files modified in {}", total, args.range.label()).bright_white()
    );
}

fn open_interactive(files: &[RecentFile], base_dir: &Path, full_paths: bool) {
    if files.is_empty() {
        println!("{}", "❌ No recent files found".bright_red());
        return;
    }

    // Create list for fzf
    let mut fzf_input = String::new();
    for file in files {
        let display_path = if full_paths {
            file.path.clone()
        } else {
            file.path
                .strip_prefix(base_dir)
                .unwrap_or(&file.path)
                .to_path_buf()
        };
        
        let size = format_size(file.size);
        let time_ago = format_time_ago(&file.modified);
        fzf_input.push_str(&format!("{} │ {} │ {}\n", 
            display_path.display(), 
            size, 
            time_ago
        ));
    }

    // Run fzf
    let mut fzf = Command::new("fzf")
        .arg("--prompt=Select file to open: ")
        .arg("--height=50%")
        .arg("--reverse")
        .arg("--preview=bat --color=always --style=numbers {1}")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start fzf");

    use std::io::Write;
    fzf.stdin
        .as_mut()
        .unwrap()
        .write_all(fzf_input.as_bytes())
        .unwrap();

    let output = fzf.wait_with_output().expect("Failed to read fzf output");

    if output.status.success() {
        let selection = String::from_utf8_lossy(&output.stdout);
        if let Some(filename) = selection.split(" │ ").next() {
            let path = if full_paths {
                PathBuf::from(filename.trim())
            } else {
                base_dir.join(filename.trim())
            };
            
            let editor = env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
            println!("{}", format!("📝 Opening: {}", path.display()).bright_green());
            let _ = Command::new(editor)
                .arg(path)
                .status();
        }
    }
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
    let now = Local::now();
    let diff = now.signed_duration_since(*time);

    if diff.num_hours() < 1 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else {
        format!("{}d ago", diff.num_days())
    }
}
