//! faelight-release v0.1.0
//! 🌲 Intelligent release and generation manager

mod changelog;
mod intelligence;
mod learning;
mod readme;
mod rollback;
mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "faelight-release",
    about = "🌲 Intelligent release and generation manager",
    version = "0.1.0"
)]
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
            let theme = if theme.is_empty() {
                "Unnamed Release".to_string()
            } else {
                theme
            };
            let data = changelog::ChangelogData::build(&root, &version, &theme)?;
            let stats = changelog::ReleaseStats::gather(&root);
            let version_str = version.clone();
            let (published, final_theme) =
                tui::ReleaseTui::new(version, theme, data.clone(), stats.clone()).run(&root)?;
            if published {
                // Auto-update full dynamic README section
                let readme_path = std::path::PathBuf::from(&root).join("README.md");
                let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                if let Err(e) = crate::readme::update_readme(
                    &readme_path, &version_str, &final_theme, &today, &data, &stats
                ) {
                    eprintln!("⚠️  README update failed: {}", e);
                } else {
                    println!("✅ README dynamic section updated");
                }
                // Auto-sync docs via faelight-docs
                let _ = std::process::Command::new("faelight-docs")
                    .arg("sync")
                    .status();

                // Re-write changelog with the actual theme from TUI
                let changelog_path = std::path::PathBuf::from(&root).join("00-meta/CHANGELOG.md");
                if changelog_path.exists() {
                    if let Ok(cl) = std::fs::read_to_string(&changelog_path) {
                        let fixed = cl.replace(
                            &format!("[{}] — Unnamed Release", version_str),
                            &format!("[{}] — {}", version_str, final_theme),
                        );
                        std::fs::write(&changelog_path, fixed).ok();
                    }
                }
                // Sync /etc/faelight/ so faelight-login shows correct version
                // Update commit count
                let commits_file = std::path::Path::new("/etc/faelight/COMMITS");
                if let Ok(output) = std::process::Command::new("git")
                    .args([
                        "-C",
                        root.to_str().unwrap_or("."),
                        "rev-list",
                        "--count",
                        "HEAD",
                    ])
                    .output()
                {
                    let count = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if let Err(e) = std::fs::write(commits_file, &count) {
                        eprintln!("⚠️  Could not update /etc/faelight/COMMITS: {}", e);
                        eprintln!("   Run manually: git rev-list --count HEAD | sudo tee /etc/faelight/COMMITS");
                    } else {
                        println!("✅ /etc/faelight/COMMITS updated to {}", count);
                    }
                }
                let version_file = std::path::Path::new("/etc/faelight/VERSION");
                if version_file.parent().map(|p| p.exists()).unwrap_or(false) {
                    let v = if version_str.starts_with("v") {
                        version_str.clone()
                    } else {
                        format!("v{}", version_str)
                    };
                    if let Err(e) = std::fs::write(version_file, &v) {
                        eprintln!("⚠️  Could not update /etc/faelight/VERSION: {}", e);
                        eprintln!(
                            "   Run manually: sudo sh -c echo {} > /etc/faelight/VERSION",
                            v
                        );
                    } else {
                        println!("✅ /etc/faelight/VERSION updated to {}", v);
                    }
                }
                // Auto-commit the release
                let commit_msg = format!("release: Faelight Forest {} — {}", version_str, final_theme);
                let _ = std::process::Command::new("git")
                    .args(["-C", root.to_str().unwrap_or("."), "add", "-A"])
                    .status();
                let commit_status = std::process::Command::new("git")
                    .args(["-C", root.to_str().unwrap_or("."), "commit", "-m", &commit_msg])
                    .status();
                match commit_status {
                    Ok(s) if s.success() => println!("✅ Release committed: {}", commit_msg),
                    _ => println!("⚠️  Auto-commit failed — run: fg commit"),
                }
                println!("🌲 Release complete! Push with: gp");
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

            println!("🌲 faelight-release — changelog preview for {}", version);
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
            println!(
                "🧠 Learning Insights (confidence: {}%)",
                insights.confidence
            );
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            // v2: synthesize narrative and suggest 3 themes
        let rich_stats = intelligence::RichStats::load(&root);
        let theme_history = intelligence::load_theme_history(&root);
        let narrative = intelligence::synthesize_narrative(&data, &rich_stats, &stats);
        let themes = intelligence::suggest_themes_v2(&data, &theme_history);
        println!();
        println!("\n📖 Release Narrative");
        println!("{}", "─".repeat(60));
        println!("{}", narrative);
        println!();
        println!("🌿 Suggested Themes");
        println!("  [1] {}", themes[0]);
        println!("  [2] {}", themes[1]);
        println!("  [3] {}", themes[2]);
        println!();
        println!("📊 Rich Stats");
        println!("  Sessions:        {}", rich_stats.sessions);
        println!("  Commits:         {}", stats.total_commits);
        println!("  Peak velocity:   {:.1} commits/hour", rich_stats.peak_velocity);
        println!("  Avg health:      {:.1}%", rich_stats.avg_health);
        println!("  Deploys:         {}", rich_stats.deploys);
        println!("  Intents done:    {}", rich_stats.intents_completed);
        println!("  Health now:      {}%", rich_stats.health_at_release);
            println!("📅 Cadence: {}", insights.release_cadence);
            if !insights.anomalies.is_empty() {
                println!("🔍 Anomalies:");
                for a in &insights.anomalies {
                    println!("   {}", a);
                }
            }
            if !insights.pattern_notes.is_empty() {
                println!("📊 Patterns:");
                for n in &insights.pattern_notes {
                    println!("   {}", n);
                }
            }
        }

        Command::Status => {
            let gen_path = root.join("runtime/generation");
            let current =
                std::fs::read_to_string(&gen_path).unwrap_or_else(|_| "unknown".to_string());
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
            let current = std::fs::read_to_string(&gen_path).unwrap_or_default();
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
                    content
                        .lines()
                        .find(|l| l.starts_with("date"))
                        .and_then(|l| l.split('"').nth(1))
                        .unwrap_or("unknown")
                        .to_string()
                } else {
                    "unknown".to_string()
                };
                let theme = if manifest_path.exists() {
                    let content = std::fs::read_to_string(&manifest_path).unwrap_or_default();
                    content
                        .lines()
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
