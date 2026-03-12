//! faelight-release v0.1.0
//! 🌲 Intelligent release and generation manager

mod changelog;
mod learning;
mod readme;
mod rollback;
mod tui;

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
    /// Publish a new release (TUI)
    #[command(name = "publish")]
    Publish {
        /// New version number (e.g. 10.5.0)
        version: String,
        /// Release theme
        #[arg(short, long, default_value = "")]
        theme: String,
    },
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
    /// Rollback to a previous generation
    Rollback {
        /// Specific version to rollback to (optional — defaults to previous)
        version: Option<String>,
    },
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
        Command::Publish { version, theme } => {
            let theme = if theme.is_empty() { "Unnamed Release".to_string() } else { theme };
            let data = changelog::ChangelogData::build(&root, &version, &theme)?;
            let stats = changelog::ReleaseStats::gather(&root);
            let version_str = version.clone();
            let published = tui::ReleaseTui::new(version, theme, data, stats).run(&root)?;
            if published {
                // Sync /etc/faelight/ so faelight-login shows correct version
                let version_file = std::path::Path::new("/etc/faelight/VERSION");
                if version_file.parent().map(|p| p.exists()).unwrap_or(false) {
                    let v = format!("v{}", version_str);
                    if let Err(e) = std::fs::write(version_file, &v) {
                        eprintln!("⚠️  Could not update /etc/faelight/VERSION: {}", e);
                        eprintln!("   Run manually: sudo sh -c echo {} > /etc/faelight/VERSION", v);
                    } else {
                        println!("✅ /etc/faelight/VERSION updated to {}", v);
                    }
                }
                println!("🌲 Release complete! Push with: fg sync");
            } else {
                println!("Release aborted.");
            }
        }
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

            // Phase 6 — Learning insights
            let insights = learning::analyze(&root, &data, &version);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("🧠 Learning Insights (confidence: {}%)", insights.confidence);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("💡 Theme suggestion: {}", insights.theme_suggestion);
            println!("📅 Cadence: {}", insights.release_cadence);
            if !insights.anomalies.is_empty() {
                println!("🔍 Anomalies:");
                for a in &insights.anomalies { println!("   {}", a); }
            }
            if !insights.pattern_notes.is_empty() {
                println!("📊 Patterns:");
                for n in &insights.pattern_notes { println!("   {}", n); }
            }
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

        Command::Rollback { version } => {
            rollback::rollback(&root, version.as_deref())?;
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
