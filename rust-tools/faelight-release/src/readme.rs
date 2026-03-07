//! Smart README writer — updates the dynamic section from release data.
//! Never touches the static section (line 38+).

use crate::changelog::{ChangelogData, ReleaseStats};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs;

const DYNAMIC_START: &str = "<!-- DYNAMIC SECTION - Updated by bump-system-version -->";
const DYNAMIC_END: &str = "<!-- END DYNAMIC SECTION -->";

pub fn update_readme(
    path: &PathBuf,
    version: &str,
    theme: &str,
    date: &str,
    data: &ChangelogData,
    stats: &ReleaseStats,
) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("cannot read {}", path.display()))?;

    let new_dynamic = build_dynamic_section(version, theme, date, data, stats);

    // Find the dynamic section bounds
    content.find(DYNAMIC_START)
        .context("DYNAMIC SECTION start marker not found in README")?;
    let end = content.find(DYNAMIC_END)
        .context("DYNAMIC SECTION end marker not found in README")?;
    let end_full = end + DYNAMIC_END.len();

    // Preserve everything after the dynamic section
    let static_section = &content[end_full..];

    let new_content = format!(
        "{}\n{}{}",
        new_dynamic,
        DYNAMIC_END,
        static_section
    );

    fs::write(path, new_content)?;
    Ok(())
}

fn build_dynamic_section(
    version: &str,
    theme: &str,
    date: &str,
    data: &ChangelogData,
    stats: &ReleaseStats,
) -> String {
    let health_color = if stats.health >= 95 { "brightgreen" } else if stats.health >= 80 { "yellow" } else { "red" };

    let mut s = String::new();

    s.push_str(DYNAMIC_START);
    s.push('\n');

    // Title
    s.push_str(&format!("# 🌲 Faelight Forest v{}\n\n", version));

    // Badges — read from real data
    s.push_str(&format!(
        "![Version](https://img.shields.io/badge/version-{}-green?style=flat-square)\n",
        version.replace('-', "--")
    ));
    s.push_str(&format!(
        "![Health](https://img.shields.io/badge/health-{}%25-{}?style=flat-square)\n",
        stats.health, health_color
    ));
    s.push_str("![Path Resilience](https://img.shields.io/badge/path_resilience-100%25-brightgreen?style=flat-square)\n");
    s.push_str("![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)\n");
    s.push_str("![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)\n\n");

    // Tagline
    s.push_str("> **A self-aware, path-resilient personal computing environment built from first principles.**\n\n");

    // Latest release section
    s.push_str("## 🎊 Latest Release\n\n");
    s.push_str(&format!(
        "### v{} - 🌲 {} ({})\n\n",
        version, theme, date
    ));

    // Completed intents as bullet points
    if !data.intents.is_empty() {
        for intent in &data.intents {
            s.push_str(&format!("- {} — {}\n", intent.id, intent.title));
        }
    }

    // Feature highlights
    for feat in data.features.iter().take(5) {
        let scope = if feat.scope.is_empty() {
            String::new()
        } else {
            format!("{}: ", feat.scope)
        };
        s.push_str(&format!("- {}{}\n", scope, feat.message));
    }

    s.push('\n');

    // Stats line
    s.push_str(&format!("- Commits: {}\n", stats.total_commits));
    s.push_str(&format!("- Tools: {} deployed\n", stats.tools_deployed));
    s.push_str(&format!("- Health: {}%\n", stats.health));
    s.push_str(&format!("- Intents: {} complete\n", stats.intents_complete));
    s.push('\n');

    s.push_str("[Full Changelog →](CHANGELOG.md)\n\n");
    s.push_str("---\n");

    s
}
