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
            sessions: 0, total_commits: 0, peak_velocity: 0.0,
            avg_health: 100.0, deploys: 0, lines_added: 0,
            _lines_removed: 0, health_at_release: 100,
            intents_completed: 0, _gates_closed: 0,
        };
        if !db_path.exists() { return stats; }
        let Ok(conn) = rusqlite::Connection::open(&db_path) else { return stats; };
        // Sessions from session_patterns
        stats.sessions = conn.query_row(
            "SELECT COUNT(*) FROM session_patterns", [],
            |r| r.get::<_, i64>(0)
        ).unwrap_or(0) as u32;
        // Total commits from commit_patterns
        stats.total_commits = conn.query_row(
            "SELECT COUNT(*) FROM commit_patterns", [],
            |r| r.get::<_, i64>(0)
        ).unwrap_or(0) as u32;
        // Peak velocity
        stats.peak_velocity = conn.query_row(
            "SELECT MAX(velocity_per_hour) FROM commit_patterns",
            [], |r| r.get::<_, Option<f64>>(0)
        ).unwrap_or(None).unwrap_or(0.0);
        // Average health from signals
        let (health_sum, health_count): (f64, i64) = conn.query_row(
            "SELECT COALESCE(AVG(CAST(json_extract(payload, '$.health') AS REAL)), 100.0), COUNT(*)
             FROM engine_signals WHERE source='doctor' AND signal_type='health'",
            [], |r| Ok((r.get::<_, f64>(0).unwrap_or(100.0), r.get::<_, i64>(1).unwrap_or(0)))
        ).unwrap_or((100.0, 0));
        stats.avg_health = if health_count > 0 { health_sum } else { 100.0 };
        // Deploys from deploy_patterns
        stats.deploys = conn.query_row(
            "SELECT COUNT(*) FROM deploy_patterns WHERE outcome='success'",
            [], |r| r.get::<_, i64>(0)
        ).unwrap_or(0) as u32;
        // Current health
        stats.health_at_release = std::fs::read_to_string(
            PathBuf::from(&home).join(".cache/faelight/health-status")
        ).unwrap_or_else(|_| "100".to_string())
        .trim().parse::<u32>().unwrap_or(100);
        // Lines from git
        if let Ok(output) = std::process::Command::new("git")
            .args(["-C", core_root.to_str().unwrap_or("."),
                   "log", "--oneline", "--shortstat", "-50"])
            .output() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if line.contains("insertion") {
                    let nums: Vec<u32> = line.split_whitespace()
                        .filter_map(|w| w.parse().ok()).collect();
                    if nums.len() >= 1 { stats.lines_added += nums.iter().sum::<u32>() / nums.len() as u32; }
                }
            }
        }
        // Intents completed from intents/complete/
        stats.intents_completed = std::fs::read_dir(core_root.join("intents/complete"))
            .map(|d| d.count()).unwrap_or(0) as u32;
        stats
    }
}
/// Synthesize a one-paragraph narrative from data + stats
pub fn synthesize_narrative(data: &ChangelogData, stats: &RichStats, base_stats: &ReleaseStats) -> String {
    let mut parts: Vec<String> = Vec::new();
    // What was built -- from intents, cleaned up
    if !data.intents.is_empty() {
        let capabilities: Vec<String> = data.intents.iter()
            .take(5)
            .filter_map(|i| {
                let title = i.title.split(" -- ").next()
                    .unwrap_or(&i.title).trim();
                // Extract clean name: words until em-dash, colon, or parens
                let clean = {
                    let mut words = Vec::new();
                    for word in title.split_whitespace() {
                        if word.contains('—') || word == "--" || word.starts_with(':') || word.starts_with('(') {
                            break;
                        }
                        // stop at word ending with colon like "Partnership:"
                        let w = word.trim_end_matches(':');
                        words.push(w.to_string());
                        if words.len() >= 3 { break; }
                    }
                    words.join(" ")
                };
                if clean.is_empty() { None } else { Some(clean.to_string()) }
            })
            .collect();
        if !capabilities.is_empty() {
            let more = if data.intents.len() > 5 {
                format!(" ({} total intents shipped)", data.intents.len())
            } else { String::new() };
            let cap_str = match capabilities.len() {
                1 => capabilities[0].clone(),
                2 => format!("{} and {}", capabilities[0], capabilities[1]),
                _ => {
                    let last = capabilities.last().cloned().unwrap_or_default();
                    let rest = capabilities[..capabilities.len()-1].join(", ");
                    format!("{}, and {}", rest, last)
                }
            };
            parts.push(format!("This release delivers {}.{}", cap_str, more));
        }
    }

    // Shell capabilities if present
    let shell_work = data.features.iter()
        .filter(|c| c.scope == "fsh" || c.message.contains("fsh") || c.message.contains("shell"))
        .count();
    if shell_work >= 3 {
        parts.push("The shell grows deeper -- new builtins, smarter history, and tighter coordination with the forest.".to_string());
    }
    // Intelligence work
    let intel_work = data.features.iter()
        .filter(|c| c.message.contains("signal") || c.message.contains("pattern") || c.message.contains("intelligence"))
        .count();
    if intel_work >= 2 {
        parts.push("Pattern learning flows through the coordination layer, giving every tool memory of what came before.".to_string());
    }
    // Session stats
    let session_str = if stats.sessions > 0 {
        format!("{} sessions", stats.sessions)
    } else { String::new() };
    let commit_str = format!("{} commits", base_stats.total_commits);
    let health_str = if stats.avg_health >= 99.5 {
        "Health held at 100% throughout.".to_string()
    } else {
        format!("Average health: {:.1}%.", stats.avg_health)
    };
    let stat_parts: Vec<String> = [session_str, commit_str]
        .iter().filter(|s| !s.is_empty()).cloned().collect();
    if !stat_parts.is_empty() {
        parts.push(format!("{}. {}", stat_parts.join(". "), health_str));
    }
    if parts.is_empty() {
        "The forest grows. Another release. Another chapter.".to_string()
    } else {
        parts.join(" ")
    }
}
/// Suggest 3 distinct theme options based on the release content
pub fn suggest_themes_v2(data: &ChangelogData, history: &[String]) -> [String; 3] {
    let has_shell = data.features.iter().any(|c|
        c.scope == "fsh" || c.message.contains("shell") || c.message.contains("fsh"));
    let has_intelligence = data.features.iter().any(|c|
        c.message.contains("signal") || c.message.contains("pattern") ||
        c.message.contains("intelligence") || c.message.contains("friday"));
    let has_tools = data.features.iter().any(|c|
        c.message.contains("v3.0") || c.message.contains("v4.0") || c.message.contains("v2.0"));
    let has_git = data.features.iter().any(|c|
        c.message.contains("faelight-git") || c.message.contains("commit"));
    let has_core = data.features.iter().any(|c|
        c.scope == "core" || c.message.contains("core v"));
    let intent_count = data.intents.len();
    // Build 3 distinct framings of the same release
    let candidates: Vec<(&str, &str, bool)> = vec![
        // (theme, category, relevant)
        ("The Shell Remembers", "shell", has_shell),
        ("Roots and Signals", "intelligence", has_intelligence),
        ("The Living Toolkit", "tools", has_tools),
        ("Commit Intelligence", "git", has_git),
        ("The Thinking Core", "core", has_core),
        ("The Bloom", "completion", intent_count >= 4),
        ("Every Tool Knows", "coordination", has_intelligence && has_tools),
        ("The Forest Speaks", "voice", has_core && has_intelligence),
        ("Smarter by Design", "general", true),
    ];
    let mut selected: Vec<String> = Vec::new();
    for (name, _, relevant) in &candidates {
        if *relevant && !history.contains(&name.to_string()) && selected.len() < 3 {
            selected.push(name.to_string());
        }
    }
    // Fill remaining slots
    while selected.len() < 3 {
        selected.push(format!("The Forest Grows -- Chapter {}", selected.len() + 1));
    }
    [selected[0].clone(), selected[1].clone(), selected[2].clone()]
}
/// Load past theme history from CHANGELOG.md
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
