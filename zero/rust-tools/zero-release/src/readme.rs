//! Smart README writer — updates the dynamic section from release data.
//! Never touches the static section (line 38+).

use crate::changelog::{ChangelogData, ReleaseStats};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const DYNAMIC_START: &str = "<!-- DYNAMIC SECTION - Updated by bump-system-version -->";
const DYNAMIC_END: &str = "<!-- END DYNAMIC SECTION -->";

// REMOVED 2026-09-15: update_tool_counts (INT-247 Layer 1).
//
// Sixty-five lines that had never run and could not have worked:
//
//   #[allow(dead_code)]   no callers, anywhere in the workspace -- grepped
//   for old in 50..60     it only tried counts between 50 and 59
//
// The real count is 26. So even called, every substitution would have matched nothing. And it
// did not compute -- it string-replaced six HARDCODED PHRASINGS with the old number spliced
// in. Reword the sentence and the replace silently stops firing.
//
// That is why the README said 30 in two places and 38 in a third: the numbers were never
// generated. They were typed once, in different sentences, and drifted apart.
//
// The README now states its numbers in prose a person has to mean. That is more reliable than
// a machine that silently matches nothing -- the INT-119 lesson, one layer up.

pub fn update_readme(
    path: &PathBuf,
    version: &str,
    theme: &str,
    date: &str,
    data: &ChangelogData,
    stats: &ReleaseStats,
) -> Result<()> {
    let content =
        fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;

    let new_dynamic = build_dynamic_section(version, theme, date, data, stats);

    // Find the dynamic section bounds
    content
        .find(DYNAMIC_START)
        .context("DYNAMIC SECTION start marker not found in README")?;
    let end = content
        .find(DYNAMIC_END)
        .context("DYNAMIC SECTION end marker not found in README")?;
    let end_full = end + DYNAMIC_END.len();

    // Preserve everything after the dynamic section
    let static_section = &content[end_full..];

    let new_content = format!("{}\n{}{}", new_dynamic, DYNAMIC_END, static_section);

    fs::write(path, new_content)?;
    Ok(())
}

/// Clean intent title for public display -- removes internal language
fn clean_intent_title(title: &str) -> String {
    // Keep full title -- just strip leading "Study -- " or "INT-NNN: " prefixes
    let t = title.trim();
    let t = if t.to_lowercase().starts_with("study -- ") {
        &t[9..]
    } else {
        t
    };
    let t = t.trim();
    // Capitalize first letter
    let mut chars = t.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

fn is_internal_title(title: &str) -> bool {
    let lower = title.to_lowercase();
    lower.contains("gate audit")
        || lower.contains("study --")
        || lower.contains("deferred")
        || lower.contains("int-")
        || lower.contains("cancelled")
        || lower.contains("f-dwl")
        || lower.starts_with("chore:")
        || lower.starts_with("refactor:")
}

fn build_dynamic_section(
    version: &str,
    theme: &str,
    date: &str,
    data: &ChangelogData,
    stats: &ReleaseStats,
) -> String {
    let mut s = String::new();
    s.push_str(DYNAMIC_START);
    s.push('\n');
    // NO VERSION IN THE TITLE, DECIDED 2026-09-15.
    //
    // The headline said "Faelight Forest 1.0.0". Renaming it to "Project 0 1.0.0" would carry
    // a number earned by a DIFFERENT project, and 1.0 reads as FINISHED. The honest state is
    // one shell being made good and twenty-four tools waiting their turn.
    //
    // The version still appears -- in the BADGE below and in the release heading -- where it
    // reads as a release number rather than a claim about maturity. The subtitle carries the
    // history instead, which is what INT-247 Layer 1 permits: "a subtitle may honestly say
    // *formerly Faelight Forest*".
    s.push_str("# Project 0\n");
    s.push_str("\n*_formerly Faelight Forest_*\n\n");
    s.push_str(&format!(
        "![Version](https://img.shields.io/badge/version-{}-green?style=flat-square)\n",
        version.replace('-', "--")
    ));
    s.push_str("![Rust](https://img.shields.io/badge/Rust-97%25-dea584?style=flat-square)\n");
    s.push_str("![Lines](https://img.shields.io/badge/lines-141k-blue?style=flat-square)\n");
    s.push_str("![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)\n\n");
    s.push_str("> **Only what is needed. A shell being made good, and the tools that survived the question.**\n\n");
    s.push_str(&format!("## 🎊 {} -- {} ({})\n\n", version, theme, date));
    // What Shipped -- public titles only, no INT numbers
    let public_intents: Vec<String> = data
        .intents
        .iter()
        .map(|i| clean_intent_title(&i.title))
        .filter(|t| !t.is_empty() && !is_internal_title(t))
        .collect();
    if !public_intents.is_empty() {
        s.push_str("### ✅ What Shipped\n\n");
        let cap = 15usize;
        for title in public_intents.iter().take(cap) {
            s.push_str(&format!("- {}\n", title));
        }
        if public_intents.len() > cap {
            s.push_str(&format!(
                "\n_…and {} more -- see the [full changelog](zero/meta/CHANGELOG.md)._\n",
                public_intents.len() - cap
            ));
        }
        s.push('\n');
    }
    // Feature highlights (non-internal)
    let public_features: Vec<_> = data
        .features
        .iter()
        .filter(|f| !is_internal_title(&f.message))
        .take(5)
        .collect();
    if !public_features.is_empty() {
        s.push_str("### 🔧 Notable Changes\n\n");
        for feat in public_features {
            let scope = if feat.scope.is_empty() {
                String::new()
            } else {
                format!("{}: ", feat.scope)
            };
            s.push_str(&format!(
                "- {}{}\n",
                scope,
                clean_intent_title(&feat.message)
            ));
        }
        s.push('\n');
    }
    // Project 0 DNA
    s.push_str("## Project 0 DNA\n\n");
    s.push_str("| | |\n|---|---|\n");
    s.push_str(&format!(
        "| 🛠 **Tools** | {} custom Rust tools |\n",
        stats.tools_deployed
    ));
    s.push_str(&format!(
        "| 📋 **Shipped** | {} features complete |\n",
        stats.intents_complete
    ));
    s.push_str(&format!("| 🏥 **Health** | {}% |\n", stats.health));
    // THE STACK IS WHAT IS ACTUALLY USED. Smithay (a Wayland compositor library) and
    // wgpu (GPU rendering) belonged to a compositor and a GPU terminal this project no
    // longer builds -- Hyprland is the compositor. Listing a library nothing links is the
    // same class of claim as a tool count nobody measured.
    s.push_str("| ⚡ **Stack** | Rust · SQLite · ratatui · Hyprland · Wayland |\n");
    s.push_str("| 🌍 **Philosophy** | Understanding over convenience · No mystery packages |\n");
    s.push('\n');
    s.push_str("> Every tool written or fully understood. Nothing runs blindly.\n\n");
    s.push_str("[Full Changelog →](zero/meta/CHANGELOG.md)\n\n");
    s.push_str("---\n");
    s
}
