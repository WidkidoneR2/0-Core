#![allow(clippy::ptr_arg)]
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
    /// Plan a release -- dry-run showing exactly what will happen (INT-255)
    Plan {
        /// New version number (e.g. 12.1.0)
        version: String,
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
    /// Query the triad for a version: which generation is release X? (INT-034)
    Query {
        /// Version to look up (e.g. 14.1.0)
        version: String,
    },
    /// Warn if any release generation is at risk of garbage collection (INT-034)
    GcCheck,
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
    // Handle broken pipe gracefully (e.g. when piped to head)
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

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
                    &readme_path,
                    &version_str,
                    &final_theme,
                    &today,
                    &data,
                    &stats,
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
                let changelog_path = std::path::PathBuf::from(&root).join("meta/CHANGELOG.md");
                if changelog_path.exists() {
                    if let Ok(cl) = std::fs::read_to_string(&changelog_path) {
                        let fixed = cl.replace(
                            &format!("[{}] — Unnamed Release", version_str),
                            &format!("[{}] — {}", version_str, final_theme),
                        );
                        std::fs::write(&changelog_path, fixed).ok();
                    }
                }
                // Record the release triad in state.db (INT-031/034): version + generation +
                // commit_count + intent_range. Replaces the old immutable /etc/faelight/{VERSION,
                // COMMITS} writes (which failed on NixOS). /etc/faelight/VERSION is now populated
                // declaratively by the nix config from meta/VERSION; the rich triad lives here.
                {
                    let commit_count = std::process::Command::new("git")
                        .args(["-C", root.to_str().unwrap_or("."), "rev-list", "--count", "HEAD"])
                        .output()
                        .ok()
                        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                        .unwrap_or_default();
                    // Current generation: resolve /nix/var/nix/profiles/system -> system-NNN-link.
                    let generation = std::fs::read_link("/nix/var/nix/profiles/system")
                        .ok()
                        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned()))
                        .and_then(|s| {
                            s.strip_prefix("system-")
                                .and_then(|r| r.strip_suffix("-link"))
                                .map(|r| r.to_string())
                        })
                        .unwrap_or_else(|| "unknown".to_string());
                    // Intent range: count completed intents for a simple range string.
                    let complete_dir = root.join("intents/complete");
                    let intent_count = std::fs::read_dir(&complete_dir)
                        .map(|rd| rd.filter_map(|e| e.ok())
                            .filter(|e| e.file_name().to_string_lossy().ends_with(".md"))
                            .count())
                        .unwrap_or(0);
                    let intent_range = format!("{} complete", intent_count);

                    let db_path = root.join("runtime/state.db");
                    match rusqlite::Connection::open(&db_path) {
                        Ok(conn) => {
                            let _ = conn.execute(
                                "CREATE TABLE IF NOT EXISTS release_triad (
                                    id           INTEGER PRIMARY KEY AUTOINCREMENT,
                                    version      TEXT NOT NULL,
                                    generation   TEXT NOT NULL,
                                    commit_count TEXT NOT NULL,
                                    intent_range TEXT NOT NULL,
                                    theme        TEXT,
                                    timestamp    INTEGER NOT NULL
                                )",
                                [],
                            );
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs() as i64)
                                .unwrap_or(0);
                            match conn.execute(
                                "INSERT INTO release_triad
                                    (version, generation, commit_count, intent_range, theme, timestamp)
                                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                                rusqlite::params![
                                    version_str, generation, commit_count, intent_range,
                                    final_theme, now
                                ],
                            ) {
                                Ok(_) => println!(
                                    "\u{2705} Release triad recorded: v{} \u{b7} gen {} \u{b7} {} commits \u{b7} {}",
                                    version_str, generation, commit_count, intent_range
                                ),
                                Err(e) => eprintln!("\u{26a0}\u{fe0f}  Could not record release triad: {}", e),
                            }
                        }
                        Err(e) => eprintln!("\u{26a0}\u{fe0f}  Could not open state.db for triad: {}", e),
                    }
                }
                // Auto-commit the release
                let commit_msg =
                    format!("release: Faelight Forest {} — {}", version_str, final_theme);
                let _ = std::process::Command::new("git")
                    .args(["-C", root.to_str().unwrap_or("."), "add", "-A"])
                    .status();
                let commit_status = std::process::Command::new("git")
                    .args([
                        "-C",
                        root.to_str().unwrap_or("."),
                        "commit",
                        "-m",
                        &commit_msg,
                    ])
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
        Command::Plan { version } => {
            // INT-255: dry-run -- show what publish would do
            println!("🌲 faelight-release -- release plan for {}", version);
            println!("{}", "━".repeat(42));
            println!();
            // cargo audit check
            print!("  🦀 cargo audit... ");
            let audit_ok = std::process::Command::new("cargo")
                .args(["audit", "-q"])
                .current_dir(&root)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            println!(
                "{}",
                if audit_ok {
                    "✅ clean"
                } else {
                    "⚠️  vulnerabilities found"
                }
            );
            // git status
            let dirty = std::process::Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(&root)
                .output()
                .map(|o| !o.stdout.is_empty())
                .unwrap_or(false);
            println!(
                "  🌿 working tree... {}",
                if dirty {
                    "⚠️  uncommitted changes"
                } else {
                    "✅ clean"
                }
            );
            // changelog summary
            println!();
            let theme = String::new();
            let data = changelog::ChangelogData::build(&root, &version, &theme)?;
            let stats = changelog::ReleaseStats::gather(&root);
            println!("  📋 Intents shipping:          {}", data.intents.len());
            println!("  📊 Commits since last release: {}", data.total_commits);
            println!("  🏥 Health:                     {}%", stats.health);
            println!();
            for intent in data.intents.iter().take(5) {
                let clean = intent
                    .title
                    .trim_matches('"')
                    .split(" -- ")
                    .next()
                    .unwrap_or(&intent.title)
                    .trim();
                println!("  ✓ {}", clean);
            }
            if data.intents.len() > 5 {
                println!("  → ... and {} more", data.intents.len() - 5);
            }
            println!();
            let has_major = data.intents.iter().any(|i| {
                i.title.to_lowercase().contains("parallel")
                    || i.title.to_lowercase().contains("architecture")
                    || i.title.to_lowercase().contains("innovation")
            });
            let bump = if has_major {
                "MAJOR"
            } else if !data.features.is_empty() {
                "MINOR"
            } else {
                "PATCH"
            };
            println!("  💡 Suggested version bump: {}", bump);
            println!();
            println!(
                "  -> To publish: faelight-release publish {} --theme \"<theme>\"",
                version
            );
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
            println!(
                "  Peak velocity:   {:.1} commits/hour",
                rich_stats.peak_velocity
            );
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

            let manifest_path = root.join(format!("meta/releases/{}/manifest.toml", current));
            if manifest_path.exists() {
                let manifest = std::fs::read_to_string(&manifest_path)?;
                for line in manifest.lines().take(4) {
                    println!("  {}", line);
                }
            }
        }

        Command::History => {
            let releases_dir = root.join("meta/releases");
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

            // Triad history from state.db (INT-031/034): version, generation, commits, intents.
            let db_path = root.join("runtime/state.db");
            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                if let Ok(mut q) = conn.prepare(
                    "SELECT version, generation, commit_count, intent_range, theme
                     FROM release_triad ORDER BY timestamp DESC",
                ) {
                    let rows: Vec<(String, String, String, String, String)> = q
                        .query_map([], |r| {
                            Ok((
                                r.get::<_, String>(0)?,
                                r.get::<_, String>(1)?,
                                r.get::<_, String>(2)?,
                                r.get::<_, String>(3)?,
                                r.get::<_, String>(4).unwrap_or_default(),
                            ))
                        })
                        .map(|m| m.filter_map(|x| x.ok()).collect())
                        .unwrap_or_default();
                    if !rows.is_empty() {
                        println!();
                        println!("Release Triad (version / generation / commits / intents)");
                        for (ver, gen, commits, intents, th) in rows {
                            println!("  v{}  gen {}  {} commits  {}  {}", ver, gen, commits, intents, th);
                        }
                    }
                }
            }
        }

        Command::Query { version } => {
            let db_path = root.join("runtime/state.db");
            match rusqlite::Connection::open(&db_path) {
                Ok(conn) => {
                    let row = conn.query_row(
                        "SELECT generation, commit_count, intent_range, theme
                         FROM release_triad WHERE version = ?1 ORDER BY timestamp DESC LIMIT 1",
                        rusqlite::params![version],
                        |r| Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, String>(2)?,
                            r.get::<_, String>(3).unwrap_or_default(),
                        )),
                    );
                    match row {
                        Ok((gen, commits, intents, theme)) => {
                            println!("Release v{}", version);
                            println!("  generation: {}", gen);
                            println!("  commits:    {}", commits);
                            println!("  intents:    {}", intents);
                            if !theme.is_empty() {
                                println!("  theme:      {}", theme);
                            }
                        }
                        Err(_) => println!("No triad record found for version {}", version),
                    }
                }
                Err(e) => eprintln!("Could not open state.db: {}", e),
            }
        }
        Command::GcCheck => {
            // Warn if any release generation no longer has a live system-NNN-link (i.e. was GC'd
            // or is at risk). We list recorded release generations and check the profile links.
            let db_path = root.join("runtime/state.db");
            let conn = match rusqlite::Connection::open(&db_path) {
                Ok(c) => c,
                Err(e) => { eprintln!("Could not open state.db: {}", e); return Ok(()); }
            };
            let mut stmt = conn.prepare(
                "SELECT version, generation FROM release_triad ORDER BY timestamp DESC"
            )?;
            let rows: Vec<(String, String)> = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .map(|m| m.filter_map(|x| x.ok()).collect())
                .unwrap_or_default();
            if rows.is_empty() {
                println!("No release triad records to check.");
            } else {
                let mut warned = false;
                for (ver, gen) in rows {
                    let link = format!("/nix/var/nix/profiles/system-{}-link", gen);
                    if !std::path::Path::new(&link).exists() {
                        println!("\u{26a0}\u{fe0f}  Release v{} (generation {}) is GONE -- generation collected.", ver, gen);
                        warned = true;
                    }
                }
                if !warned {
                    println!("\u{2705} All release generations still present (none collected).");
                }
            }
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
