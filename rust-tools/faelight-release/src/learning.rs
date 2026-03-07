//! Phase 6 — Learning layer.
//! Reads past manifests, detects patterns, suggests themes, flags anomalies.

use crate::changelog::ChangelogData;
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone)]
pub struct ReleasePattern {
    #[allow(dead_code)]
    pub version: String,
    pub theme: String,
    pub days_since_last: i64,
    pub commits: u32,
    pub intents: u32,
    pub tools_added: u32,
    pub health: u32,
}

#[derive(Debug)]
pub struct LearningInsights {
    pub theme_suggestion: String,
    pub anomalies: Vec<String>,
    pub release_cadence: String,
    pub pattern_notes: Vec<String>,
    pub confidence: u8,
}

fn extract_str(content: &str, key: &str) -> Option<String> {
    content.lines()
        .find(|l| l.trim().starts_with(key))
        .and_then(|l| l.split('"').nth(1))
        .map(|s| s.to_string())
}

fn extract_u32(content: &str, key: &str) -> Option<u32> {
    content.lines()
        .find(|l| l.trim().starts_with(key))
        .and_then(|l| l.split('=').nth(1))
        .and_then(|s| s.trim().parse().ok())
}

fn load_patterns(core_root: &PathBuf) -> Vec<ReleasePattern> {
    let releases_dir = core_root.join("00-meta/releases");
    let mut patterns = vec![];

    let mut versions: Vec<String> = match fs::read_dir(&releases_dir) {
        Ok(d) => d.filter_map(|e| e.ok())
            .filter(|e| e.metadata().map(|m| m.is_dir()).unwrap_or(false))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect(),
        Err(_) => return vec![],
    };
    versions.sort_by(|a, b| {
        let a_parts: Vec<u32> = a.split('.').filter_map(|x| x.parse().ok()).collect();
        let b_parts: Vec<u32> = b.split('.').filter_map(|x| x.parse().ok()).collect();
        a_parts.cmp(&b_parts)
    });

    let mut prev_date: Option<chrono::NaiveDate> = None;

    for v in &versions {
        let manifest_path = releases_dir.join(v).join("manifest.toml");
        if !manifest_path.exists() { continue; }
        let content = fs::read_to_string(&manifest_path).unwrap_or_default();

        let theme = extract_str(&content, "theme").unwrap_or_default();
        let date_str = extract_str(&content, "date").unwrap_or_default();
        let health = extract_u32(&content, "health_at_release").unwrap_or(95);
        let commits = extract_u32(&content, "commits_since_last").unwrap_or(0);
        let tools_added = content.lines()
            .skip_while(|l| !l.contains("[tools_added]"))
            .skip(1)
            .take_while(|l| !l.starts_with('['))
            .filter(|l| l.contains('='))
            .count() as u32;
        let intents = content.lines()
            .find(|l| l.trim().starts_with("ids = ["))
            .map(|l| l.matches(',').count() as u32 + 1)
            .unwrap_or(0);

        let current_date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok();
        let days_since_last = match (prev_date, current_date) {
            (Some(prev), Some(curr)) => (curr - prev).num_days(),
            _ => 0,
        };
        prev_date = current_date;

        patterns.push(ReleasePattern {
            version: v.clone(), theme, days_since_last,
            commits, intents, tools_added, health,
        });
    }
    patterns
}

fn suggest_theme(data: &ChangelogData) -> String {
    let has_core = data.features.iter().any(|c| c.scope == "core" || c.scope == "release");
    let has_compositor = data.features.iter().any(|c| c.scope == "niri" || c.scope == "compositor");
    let has_intelligence = data.features.iter().any(|c|
        c.message.contains("forecast") || c.message.contains("pulse") || c.message.contains("intelligence"));
    let has_new_tools = data.features.iter().any(|c|
        c.message.contains("v1.0.0") || c.message.contains("v0.1.0"));

    if has_core && has_intelligence {
        "The Awakening — Core Learns to See".to_string()
    } else if has_core && has_new_tools {
        "The Architect — New Siblings Join the Forest".to_string()
    } else if has_compositor {
        "The Foundation — Roots Reach Deeper".to_string()
    } else if has_intelligence {
        "The Oracle — The Forest Sees Ahead".to_string()
    } else if data.intents.len() >= 4 {
        "The Bloom — Many Intents Find Completion".to_string()
    } else if let Some(intent) = data.intents.first() {
        let short = intent.title.split('—').next().unwrap_or(&intent.title).trim();
        format!("{} — The Forest Grows", short.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
    } else {
        "The Living Forest — Continuous Growth".to_string()
    }
}

fn detect_anomalies(data: &ChangelogData, patterns: &[ReleasePattern]) -> Vec<String> {
    let mut anomalies = vec![];
    if patterns.len() < 2 { return anomalies; }

    let avg_commits = patterns.iter().filter(|p| p.commits > 0)
        .map(|p| p.commits as f64).sum::<f64>() / patterns.len().max(1) as f64;
    let avg_intents = patterns.iter()
        .map(|p| p.intents as f64).sum::<f64>() / patterns.len().max(1) as f64;

    if avg_commits > 0.0 && data.total_commits as f64 > avg_commits * 2.5 {
        anomalies.push(format!("⚠️  {} commits — {:.0}x above average ({:.0}). Consider splitting.",
            data.total_commits, data.total_commits as f64 / avg_commits, avg_commits));
    }
    if avg_intents > 0.0 && data.intents.len() as f64 > avg_intents * 2.0 {
        anomalies.push(format!("⚠️  {} intents — unusually high (avg: {:.1}). Big release!",
            data.intents.len(), avg_intents));
    }
    if data.intents.is_empty() {
        anomalies.push("💡 No completed intents — maintenance release.".to_string());
    }
    anomalies
}

fn analyze_cadence(patterns: &[ReleasePattern]) -> String {
    if patterns.len() < 2 { return "Not enough history yet.".to_string(); }
    let cadences: Vec<i64> = patterns.iter().filter(|p| p.days_since_last > 0)
        .map(|p| p.days_since_last).collect();
    if cadences.is_empty() { return "Cadence data not available.".to_string(); }
    let avg_days = cadences.iter().sum::<i64>() / cadences.len() as i64;
    format!("Avg {} days between releases ({} releases tracked)", avg_days, patterns.len())
}

fn build_pattern_notes(patterns: &[ReleasePattern], data: &ChangelogData) -> Vec<String> {
    let mut notes = vec![];
    if patterns.len() >= 2 {
        if patterns.iter().all(|p| p.theme.contains(" — ")) {
            notes.push("📐 Theme pattern: \"<Subject> — <Subtitle>\" — consistent".to_string());
        }
        if patterns.iter().all(|p| p.health >= 95) {
            notes.push("🏥 All releases at 95%+ health — excellent discipline".to_string());
        }
        let avg_tools = patterns.iter().map(|p| p.tools_added as f64).sum::<f64>() / patterns.len() as f64;
        if avg_tools > 0.0 {
            notes.push(format!("🛠️  Avg {:.1} new tools per release", avg_tools));
        }
    }
    if !data.intents.is_empty() {
        notes.push(format!("🎯 {} intents closing in this release", data.intents.len()));
    }
    notes
}

pub fn analyze(core_root: &PathBuf, data: &ChangelogData, _version: &str) -> LearningInsights {
    let patterns = load_patterns(core_root);
    let confidence = (patterns.len() * 15).min(100) as u8;
    LearningInsights {
        theme_suggestion: suggest_theme(data),
        anomalies: detect_anomalies(data, &patterns),
        release_cadence: analyze_cadence(&patterns),
        pattern_notes: build_pattern_notes(&patterns, data),
        confidence,
    }
}
