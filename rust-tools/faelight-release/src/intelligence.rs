//! INT-225 -- faelight-release v2 intelligence layer
//! synthesize_narrative, suggest_themes_v2, load_release_stats_from_db
use crate::changelog::{ChangelogData, ReleaseStats};
use std::path::PathBuf;
/// Rich stats loaded from state.db pattern tables
pub struct RichStats {
    pub sessions: u32,
    pub total_commits: u32,
    pub peak_velocity: f64,
    pub avg_health: f64,
    pub deploys: u32,
    pub lines_added: u32,
    pub _lines_removed: u32,
    pub health_at_release: u32,
    pub intents_completed: u32,
    pub _gates_closed: u32,
}
impl RichStats {
    pub fn load(core_root: &PathBuf) -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let db_path = PathBuf::from(&home).join("0-core/runtime/state.db");
        let mut stats = RichStats {
            sessions: 0,
            total_commits: 0,
            peak_velocity: 0.0,
            avg_health: 100.0,
            deploys: 0,
            lines_added: 0,
            _lines_removed: 0,
            health_at_release: 100,
            intents_completed: 0,
            _gates_closed: 0,
        };
        if !db_path.exists() {
            return stats;
        }
        let Ok(conn) = rusqlite::Connection::open(&db_path) else {
            return stats;
        };
        // Sessions from session_patterns
        stats.sessions = conn
            .query_row("SELECT COUNT(*) FROM session_patterns", [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap_or(0) as u32;
        // Total commits from commit_patterns
        stats.total_commits = conn
            .query_row("SELECT COUNT(*) FROM commit_patterns", [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap_or(0) as u32;
        // Peak velocity
        stats.peak_velocity = conn
            .query_row(
                "SELECT MAX(velocity_per_hour) FROM commit_patterns",
                [],
                |r| r.get::<_, Option<f64>>(0),
            )
            .unwrap_or(None)
            .unwrap_or(0.0);
        // Average health from signals
        let (health_sum, health_count): (f64, i64) = conn.query_row(
            "SELECT COALESCE(AVG(CAST(json_extract(payload, '$.health') AS REAL)), 100.0), COUNT(*)
             FROM engine_signals WHERE source='doctor' AND signal_type='health'",
            [], |r| Ok((r.get::<_, f64>(0).unwrap_or(100.0), r.get::<_, i64>(1).unwrap_or(0)))
        ).unwrap_or((100.0, 0));
        stats.avg_health = if health_count > 0 { health_sum } else { 100.0 };
        // Deploys from deploy_patterns
        stats.deploys = conn
            .query_row(
                "SELECT COUNT(*) FROM deploy_patterns WHERE outcome='success'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0) as u32;
        // Current health
        stats.health_at_release =
            std::fs::read_to_string(PathBuf::from(&home).join(".cache/faelight/health-status"))
                .unwrap_or_else(|_| "100".to_string())
                .trim()
                .parse::<u32>()
                .unwrap_or(100);
        // Lines from git
        if let Ok(output) = std::process::Command::new("git")
            .args([
                "-C",
                core_root.to_str().unwrap_or("."),
                "log",
                "--oneline",
                "--shortstat",
                "-50",
            ])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if line.contains("insertion") {
                    let nums: Vec<u32> = line
                        .split_whitespace()
                        .filter_map(|w| w.parse().ok())
                        .collect();
                    if !nums.is_empty() {
                        stats.lines_added += nums.iter().sum::<u32>() / nums.len() as u32;
                    }
                }
            }
        }
        // Intents completed from intents/complete/
        stats.intents_completed = std::fs::read_dir(core_root.join("intents/complete"))
            .map(|d| d.count())
            .unwrap_or(0) as u32;
        stats
    }
}
/// Synthesize a one-paragraph narrative from data + stats
pub fn synthesize_narrative(
    data: &ChangelogData,
    stats: &RichStats,
    base_stats: &ReleaseStats,
) -> String {
    // INT-264: Semantic synthesis -- describe what changed, not what was counted.
    let mut sentences: Vec<String> = Vec::new();
    let terminal_work = data.intents.iter().filter(|i|
        i.title.to_lowercase().contains("term") ||
        i.title.to_lowercase().contains("terminal")).count();
    let shell_work = data.intents.iter().filter(|i|
        i.title.to_lowercase().contains("fsh") ||
        i.title.to_lowercase().contains("shell") ||
        i.title.to_lowercase().contains("vocabulary") ||
        i.title.to_lowercase().contains("shell") || i.title.to_lowercase().contains("fsh")).count();
    let friday_work = data.intents.iter().filter(|i|
        i.title.to_lowercase().contains("friday") ||
        i.title.to_lowercase().contains("friday")).count();
    let tui_work = data.intents.iter().filter(|i|
        i.title.to_lowercase().contains("tui") || i.title.to_lowercase().contains("ratatui")).count();
    let infra_work = data.intents.iter().filter(|i|
        i.title.to_lowercase().contains("filter-repo") || i.title.to_lowercase().contains("binaries") || i.title.to_lowercase().contains("cleanup")).count();
    let vocab_count = data.intents.iter().filter(|i|
        i.title.to_lowercase().contains("vocabulary") ||
        i.title.to_lowercase().contains("copy") ||
        i.title.to_lowercase().contains("delete")).count();
    if terminal_work > 0 {
        sentences.push("The terminal was rebuilt from scratch -- GPU-ready architecture, full scrollback, Friday panel, split panes.".to_string());
    }
    if vocab_count > 0 {
        sentences.push("The shell learned its first human words -- commands that read like English, safe by default, UNIX as fallback.".to_string());
    } else if shell_work >= 2 {
        sentences.push("The shell gained new structure: data pipelines, smarter history, and tighter Friday integration.".to_string());
    }
    if friday_work > 0 {
        sentences.push("Friday gained a planning layer -- anticipating next steps, correlating sessions, surfacing the right knowledge at the right moment.".to_string());
    }
    if tui_work >= 2 {
        sentences.push(format!("{} TUIs shipped: health at a keypress, git workflow simplified, intent ledger always visible.", tui_work));
    } else if tui_work == 1 {
        sentences.push("A new TUI shipped, bringing the forest's intelligence into an interactive interface.".to_string());
    }
    if infra_work > 0 {
        sentences.push("The repository shed 320MB of binary history, running lean for the first time.".to_string());
    }
    let health_note = if stats.avg_health >= 99.0 {
        "Health held at 100% throughout."
    } else if stats.avg_health >= 95.0 {
        "Health stayed above 95% through the entire release cycle."
    } else {
        "Health recovered to 100% by release."
    };
    sentences.push(health_note.to_string());
    if sentences.is_empty() {
        format!("The forest grows. {} commits. {} intents. Another chapter.",
            base_stats.total_commits, data.intents.len())
    } else {
        sentences.join("\n")
    }
}
pub fn suggest_themes_v2(data: &ChangelogData, history: &[String]) -> [String; 3] {
    // INT-264: Theme emerges from highest-impact signals, not a template pool.
    // Read what actually shipped and derive a theme from the dominant story.
    let has_vocab = data.intents.iter().any(|i|
        i.title.to_lowercase().contains("vocabulary") ||
        i.title.to_lowercase().contains("human") ||
        i.title.to_lowercase().contains("vocabulary"));
    let has_terminal = data.intents.iter().any(|i|
        i.title.to_lowercase().contains("term") ||
        i.title.to_lowercase().contains("terminal"));
    let has_tui = data.intents.iter().filter(|i|
        i.title.to_lowercase().contains("tui") || i.title.to_lowercase().contains("ratatui")).count() >= 2;
    let has_friday = data.intents.iter().any(|i|
        i.title.to_lowercase().contains("friday") ||
        i.title.to_lowercase().contains("friday"));
    let has_cleanup = data.intents.iter().any(|i|
        i.title.to_lowercase().contains("filter-repo") || i.title.to_lowercase().contains("binaries"));
    let has_shell = data.intents.iter().filter(|i|
        i.title.to_lowercase().contains("shell") || i.title.to_lowercase().contains("fsh")).count() >= 2;
    let intent_count = data.intents.len();
    // Derive the primary theme from the dominant signal
    let primary = if has_vocab && has_terminal {
        "The Forest Speaks Human".to_string()
    } else if has_vocab {
        "Human First, UNIX as Fallback".to_string()
    } else if has_terminal && has_friday {
        "The Terminal That Thinks".to_string()
    } else if has_tui && has_shell {
        "The Interface Arrives".to_string()
    } else if has_friday {
        "Friday Awakens".to_string()
    } else if has_cleanup && intent_count >= 5 {
        "The Forest Runs Lean".to_string()
    } else {
        "The Forest Grows".to_string()
    };
    // Second theme: what changed for the human using it daily
    let secondary = if has_tui {
        "Three Keypresses, Three Windows Into the Forest".to_string()
    } else if has_vocab {
        "Seven Words the Forest Now Speaks".to_string()
    } else if has_shell {
        "The Shell That Grew Up".to_string()
    } else if has_friday {
        "The Partner, Not the Tool".to_string()
    } else {
        "Discipline Over Speed".to_string()
    };
    // Third theme: the philosophical framing
    let tertiary = if has_vocab && has_friday {
        "Meaning Before Metadata".to_string()
    } else if has_terminal && has_cleanup {
        "Own Everything, Understand Everything".to_string()
    } else if has_tui && has_friday {
        "Intelligence You Can See".to_string()
    } else {
        "The Forest Knows Itself".to_string()
    };
    // Filter used themes but keep the derived ones (they are already specific)
    let _ = history; // history used to filter generic templates -- not needed here
    [primary, secondary, tertiary]
}
pub fn load_theme_history(core_root: &PathBuf) -> Vec<String> {
    let changelog_path = core_root.join("CHANGELOG.md");
    let content = std::fs::read_to_string(&changelog_path).unwrap_or_default();
    let mut themes = Vec::new();
    for line in content.lines() {
        // Lines like:
        if line.starts_with("## [") {
            if let Some(dash_pos) = line.find(" -- ") {
                let after = &line[dash_pos + 3..];
                if let Some(paren) = after.find(" (") {
                    themes.push(after[..paren].to_string());
                } else {
                    themes.push(after.trim().to_string());
                }
            }
        }
    }
    themes
}
