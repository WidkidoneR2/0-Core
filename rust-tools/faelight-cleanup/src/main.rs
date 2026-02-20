//! faelight-cleanup v1.0.0 - Faelight Forest Hygiene Tool

use clap::Parser;
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const VERSION: &str = "1.0.0";

#[derive(Parser)]
#[command(name = "faelight-cleanup")]
#[command(about = "🌲 Faelight Forest Cleanup - System hygiene scanner")]
#[command(version = VERSION)]
struct Cli {
    /// Actually remove files (default: scan only)
    #[arg(long, short)]
    clean: bool,
    /// Skip confirmation prompts
    #[arg(long, short)]
    yes: bool,
    /// Show verbose output including orphaned scripts
    #[arg(long, short)]
    verbose: bool,
}

struct Finding {
    category: &'static str,
    path: PathBuf,
    description: String,
    size: u64,
}

fn dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

fn file_size(path: &Path) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

fn fmt_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

fn scan_backup_dirs(home: &Path, findings: &mut Vec<Finding>) {
    let patterns = ["backup", "migration", "bak", "v3-backup"];
    // Legitimate dirs that should never be flagged
    let exclusions = ["04-runtime/backups", "0-core/BACKUPS", "0-core/backups"];
    for entry in WalkDir::new(home)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_dir() {
            continue;
        }
        let full_path = entry.path().to_string_lossy().to_string();
        if exclusions.iter().any(|e| full_path.contains(e)) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if patterns.iter().any(|p| name.contains(p)) {
            let path = entry.path().to_path_buf();
            let size = dir_size(&path);
            findings.push(Finding {
                category: "Backup Dirs",
                description: format!("Backup directory: {}", name),
                size,
                path,
            });
        }
    }
}

fn scan_session_notes(core: &Path, findings: &mut Vec<Finding>) {
    let patterns = [
        "TONIGHT",
        "TODO-TOMORROW",
        "TODO-TODAY",
        "START_HERE",
        "BALANCED_PROMPT",
        "CODE_QUALITY",
    ];
    for entry in WalkDir::new(core)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_uppercase();
        if name.ends_with(".MD") && patterns.iter().any(|p| name.contains(p)) {
            let path = entry.path().to_path_buf();
            let size = file_size(&path);
            findings.push(Finding {
                category: "Session Notes",
                description: format!(
                    "Stale session note: {}",
                    entry.file_name().to_string_lossy()
                ),
                size,
                path,
            });
        }
    }
}

fn scan_tmp_logs(findings: &mut Vec<Finding>) {
    if let Ok(entries) = fs::read_dir("/tmp") {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("faelight-") && name.ends_with(".log") {
                let path = entry.path();
                let size = file_size(&path);
                findings.push(Finding {
                    category: "Temp Logs",
                    description: format!("Faelight temp log: {}", name),
                    size,
                    path,
                });
            }
        }
    }
}

fn scan_yazi_cache(home: &Path, findings: &mut Vec<Finding>) {
    let cache = home.join(".cache/yazi");
    if cache.exists() {
        let size = dir_size(&cache);
        findings.push(Finding {
            category: "Yazi Cache",
            description: "Yazi file manager cache (ghost entries included)".to_string(),
            size,
            path: cache,
        });
    }
}

fn scan_cargo_cache(home: &Path, findings: &mut Vec<Finding>) {
    let registry = home.join(".cargo/registry/cache");
    if registry.exists() {
        let size = dir_size(&registry);
        if size > 50_000_000 {
            findings.push(Finding {
                category: "Cargo Cache",
                description: format!("Cargo registry cache ({})", fmt_size(size)),
                size,
                path: registry,
            });
        }
    }
}

fn scan_duplicate_intents(core: &Path, findings: &mut Vec<Finding>) {
    let intents_dir = core.join("intents/complete");
    if !intents_dir.exists() {
        return;
    }
    let mut numbers: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for entry in WalkDir::new(&intents_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(num) = name.split('-').next() {
            if num.parse::<u32>().is_ok() {
                numbers
                    .entry(num.to_string())
                    .or_default()
                    .push(entry.path().to_path_buf());
            }
        }
    }
    for (num, paths) in numbers {
        if paths.len() > 1 {
            for path in &paths[1..] {
                let size = file_size(path);
                findings.push(Finding {
                    category: "Duplicate Intents",
                    description: format!(
                        "Duplicate intent #{}: {}",
                        num,
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ),
                    size,
                    path: path.clone(),
                });
            }
        }
    }
}

fn scan_pacman_cache(findings: &mut Vec<Finding>) {
    let cache = Path::new("/var/cache/pacman/pkg");
    if cache.exists() {
        let size = dir_size(cache);
        if size > 500_000_000 {
            findings.push(Finding {
                category: "Pacman Cache",
                description: format!("Pacman package cache ({})", fmt_size(size)),
                size,
                path: cache.to_path_buf(),
            });
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let home = dirs::home_dir().expect("Cannot find home directory");
    let core = home.join("0-core");

    println!(
        "{}",
        "═══════════════════════════════════════════════════════════".green()
    );
    println!(
        "{}",
        format!("🌲 faelight-cleanup v{}", VERSION).green().bold()
    );
    if cli.clean {
        println!(
            "{}",
            "   🧹 CLEAN MODE - Files will be removed".yellow().bold()
        );
    } else {
        println!("{}", "   🔍 SCAN MODE  - No files will be removed".cyan());
        println!("{}", "   Tip: run with --clean to actually remove".dimmed());
    }
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════".green()
    );
    println!();

    let mut findings: Vec<Finding> = Vec::new();

    print!("  Scanning backup directories... ");
    scan_backup_dirs(&home, &mut findings);
    println!("{}", "done".dimmed());

    print!("  Scanning session notes... ");
    scan_session_notes(&core, &mut findings);
    println!("{}", "done".dimmed());

    print!("  Scanning temp logs... ");
    scan_tmp_logs(&mut findings);
    println!("{}", "done".dimmed());

    print!("  Scanning yazi cache... ");
    scan_yazi_cache(&home, &mut findings);
    println!("{}", "done".dimmed());

    print!("  Scanning cargo cache... ");
    scan_cargo_cache(&home, &mut findings);
    println!("{}", "done".dimmed());

    print!("  Scanning duplicate intents... ");
    scan_duplicate_intents(&core, &mut findings);
    println!("{}", "done".dimmed());

    print!("  Scanning pacman cache... ");
    scan_pacman_cache(&mut findings);
    println!("{}", "done".dimmed());

    println!();

    if findings.is_empty() {
        println!(
            "{}",
            "✅ Forest is clean! Nothing to remove.".green().bold()
        );
        return;
    }

    // Group and display
    let mut categories: HashMap<&str, Vec<&Finding>> = HashMap::new();
    for f in &findings {
        categories.entry(f.category).or_default().push(f);
    }

    let total_size: u64 = findings.iter().map(|f| f.size).sum();
    let mut sorted_cats: Vec<&str> = categories.keys().copied().collect();
    sorted_cats.sort();

    println!(
        "{}",
        "─── Findings ───────────────────────────────────────────────".dimmed()
    );
    for cat in &sorted_cats {
        let items = &categories[cat];
        let cat_size: u64 = items.iter().map(|f| f.size).sum();
        println!();
        println!(
            "  {} {} {}",
            "▶".yellow(),
            cat.bold(),
            format!("({})", fmt_size(cat_size)).dimmed()
        );
        for item in items.iter() {
            println!("    {} {}", "•".dimmed(), item.description);
            if cli.verbose {
                println!("      {}", item.path.display().to_string().dimmed());
            }
        }
    }

    println!();
    println!(
        "{}",
        "─── Summary ────────────────────────────────────────────────".dimmed()
    );
    println!(
        "  {} findings  |  {} recoverable",
        findings.len().to_string().yellow(),
        fmt_size(total_size).yellow().bold()
    );
    println!();

    if !cli.clean {
        println!("{}", "  Run with --clean to remove.".cyan());
        return;
    }

    if !cli.yes {
        print!("  Confirm removal? [y/N]: ");
        use std::io::Write;
        std::io::stdout().flush().ok();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        if input.trim().to_lowercase() != "y" {
            println!("{}", "  Aborted.".red());
            return;
        }
    }

    println!();
    let mut removed = 0u32;
    let mut errors = 0u32;
    let mut freed: u64 = 0;

    for finding in &findings {
        let result = if finding.path.is_dir() {
            fs::remove_dir_all(&finding.path).map_err(|e| e.to_string())
        } else {
            fs::remove_file(&finding.path).map_err(|e| e.to_string())
        };
        match result {
            Ok(_) => {
                freed += finding.size;
                removed += 1;
                println!(
                    "  {} {}",
                    "✅".green(),
                    finding.path.display().to_string().dimmed()
                );
            }
            Err(e) => {
                errors += 1;
                println!("  {} {} — {}", "❌".red(), finding.path.display(), e);
            }
        }
    }

    println!();
    println!(
        "{}",
        "─── Results ────────────────────────────────────────────────".dimmed()
    );
    println!(
        "  {} removed  |  {} errors  |  {} freed",
        removed.to_string().green(),
        errors.to_string().red(),
        fmt_size(freed).green().bold()
    );
    println!();
    println!("{}", "🌲 Forest cleaned.".green().bold());
}
