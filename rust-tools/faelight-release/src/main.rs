//! faelight-release v0.1.0
//! 🌲 Intelligent release and generation manager
//! Phase 2 — Smart changelog engine

mod changelog;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "faelight-release", about = "🌲 Intelligent release and generation manager", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Preview the auto-generated changelog for a new version
    Preview {
        /// New version number (e.g. 10.5.0)
        version: String,
        /// Release theme
        #[arg(short, long, default_value = "")]
        theme: String,
    },
    /// Show current generation
    Status,
    /// List all release generations
    History,
    /// Show changelog diff since a version
    Diff {
        /// Version to diff from
        version: String,
    },
}

fn core_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/home/christian"))
        .join("0-core")
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = core_root();

    match cli.command {
        Command::Preview { version, theme } => {
            let theme = if theme.is_empty() {
                "Unnamed Release".to_string()
            } else {
                theme
            };

            println!("🌲 faelight-release — changelog preview for v{}", version);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            let data = changelog::ChangelogData::build(&root, &version, &theme)?;
            let stats = changelog::ReleaseStats::gather(&root);

            println!("📊 Gathered from git log since {}", data.last_tag);
            println!("   {} total commits analyzed", data.total_commits);
            println!("   {} features", data.features.len());
            println!("   {} fixes", data.fixes.len());
            println!("   {} intents shipped", data.intents.len());
            println!();
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("📝 Generated Changelog Entry:");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("{}", data.render_markdown(&stats));
        }

        Command::Status => {
            let gen_path = root.join("runtime/generation");
            let current = std::fs::read_to_string(&gen_path)
                .unwrap_or_else(|_| "unknown".to_string());
            let current = current.trim();

            println!("🌲 faelight-release status");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  Current generation: {}", current);

            let manifest_path = root.join(format!("00-meta/releases/{}/manifest.toml", current));
            if manifest_path.exists() {
                let manifest = std::fs::read_to_string(&manifest_path)?;
                for line in manifest.lines().take(4) {
                    println!("  {}", line);
                }
            }
        }

        Command::History => {
            let releases_dir = root.join("00-meta/releases");
            let gen_path = root.join("runtime/generation");
            let current = std::fs::read_to_string(&gen_path)
                .unwrap_or_default();
            let current = current.trim();

            println!("🌲 Release History");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            let mut versions: Vec<String> = std::fs::read_dir(&releases_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.metadata().map(|m| m.is_dir()).unwrap_or(false))
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            versions.sort();
            versions.reverse();

            for v in &versions {
                let manifest_path = releases_dir.join(v).join("manifest.toml");
                let date = if manifest_path.exists() {
                    let content = std::fs::read_to_string(&manifest_path).unwrap_or_default();
                    content.lines()
                        .find(|l| l.starts_with("date"))
                        .and_then(|l| l.split('"').nth(1))
                        .unwrap_or("unknown")
                        .to_string()
                } else {
                    "unknown".to_string()
                };
                let theme = if manifest_path.exists() {
                    let content = std::fs::read_to_string(&manifest_path).unwrap_or_default();
                    content.lines()
                        .find(|l| l.starts_with("theme"))
                        .and_then(|l| l.split('"').nth(1))
                        .unwrap_or("—")
                        .to_string()
                } else {
                    "—".to_string()
                };
                let marker = if v == current { " ← current" } else { "" };
                println!("  {}  {}  {}{}", v, date, theme, marker);
            }
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }

        Command::Diff { version } => {
            let tag = format!("v{}", version);
            println!("🌲 Changes since {}", tag);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            let commits = changelog::get_commits_since(&root, &tag)?;
            let intents = changelog::find_shipped_intents(&root, &tag);

            if !intents.is_empty() {
                println!("🎯 Intents shipped:");
                for i in &intents {
                    println!("  INT-{}  {}", i.id, i.title);
                }
                println!();
            }

            println!("🔀 {} commits:", commits.len());
            for c in &commits {
                println!("  {}  {}", c.hash, c.raw);
            }
        }
    }

    Ok(())
}
