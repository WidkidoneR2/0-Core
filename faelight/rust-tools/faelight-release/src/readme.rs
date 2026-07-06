//! Smart README writer — updates the dynamic section from release data.
//! Never touches the static section (line 38+).

use crate::changelog::{ChangelogData, ReleaseStats};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const DYNAMIC_START: &str = "<!-- DYNAMIC SECTION - Updated by bump-system-version -->";
const DYNAMIC_END: &str = "<!-- END DYNAMIC SECTION -->";

/// Update static tool count references throughout README
#[allow(dead_code)]
pub fn update_tool_counts(readme_path: &std::path::Path, _core_root: &str) {
    let Ok(mut content) = std::fs::read_to_string(readme_path) else {
        return;
    };

    // Count tools from registry
    let registry_path = faelight_core::paths::tools_registry();
    let tool_count = {
        let raw = std::fs::read_to_string(&registry_path).unwrap_or_default();
        let mut count = 0usize;
        let mut in_retired = false;
        let mut saw_name = false;
        for line in raw.lines() {
            let t = line.trim();
            if t == "[[tool]]" {
                if saw_name && !in_retired {
                    count += 1;
                }
                in_retired = false;
                saw_name = false;
            } else if t == "retired = true" {
                in_retired = true;
            } else if t.starts_with("name =") {
                saw_name = true;
            }
        }
        if saw_name && !in_retired {
            count += 1;
        }
        count
    };

    if tool_count == 0 {
        return;
    }

    // Replace all tool count references
    for old in 50..60 {
        if old == tool_count {
            continue;
        }
        content = content.replace(
            &format!("{} Rust tools you fully understand", old),
            &format!("{} Rust tools you fully understand", tool_count),
        );
        content = content.replace(
            &format!("# {} custom Rust tools", old),
            &format!("# {} custom Rust tools", tool_count),
        );
        content = content.replace(
            &format!("score all {} tools", old),
            &format!("score all {} tools", tool_count),
        );
        content = content.replace(
            &format!("The Rust Ecosystem ({} Tools)", old),
            &format!("The Rust Ecosystem ({} Tools)", tool_count),
        );
        content = content.replace(
            &format!("Build all {} tools", old),
            &format!("Build all {} tools", tool_count),
        );
    }

    std::fs::write(readme_path, content).ok();
}

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
    s.push_str(&format!("# 🌲 Faelight Forest {}\n\n", version));
    s.push_str(&format!(
        "![Version](https://img.shields.io/badge/version-{}-green?style=flat-square)\n",
        version.replace('-', "--")
    ));
    s.push_str("![Rust](https://img.shields.io/badge/Rust-96.5%25-dea584?style=flat-square)\n");
    s.push_str("![Lines](https://img.shields.io/badge/lines-113k-blue?style=flat-square)\n");
    s.push_str("![NixOS](https://img.shields.io/badge/NixOS-26.05_Yarara-7ebae4?style=flat-square)\n");
    s.push_str("![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)\n\n");
    s.push_str("> **A self-aware personal computing environment built from first principles. Pure Rust. No Electron. No telemetry.**\n\n");
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
        for title in &public_intents {
            s.push_str(&format!("- {}\n", title));
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
            s.push_str(&format!("- {}{}\n", scope, feat.message));
        }
        s.push('\n');
    }
    // Forest DNA
    s.push_str("## 🌲 Forest DNA\n\n");
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
    s.push_str("| ⚡ **Stack** | Rust · Wayland · Smithay · ratatui · wgpu |\n");
    s.push_str("| 🌍 **Philosophy** | Understanding over convenience · No mystery packages |\n");
    s.push('\n');
    s.push_str("> Built by one developer. Every tool written or fully understood.\n\n");
    s.push_str("[Full Changelog →](faelight/meta/CHANGELOG.md)\n\n");
    s.push_str("---\n");
    s
}
