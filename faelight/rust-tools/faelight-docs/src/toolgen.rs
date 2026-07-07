//! INT-037: per-tool README generator + index.
//! Reads each rust-tools/<tool>/Cargo.toml (ground truth) and enriches with
//! registry/tools.toml status, then emits rich NixOS-era READMEs + a top-level index.
//! Self-maintaining: re-run any time tools change; docs never drift stale.

use std::path::PathBuf;

pub fn core_root() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join("0-core")
}

/// Combined metadata for one tool: Cargo.toml fields + registry status.
#[derive(Debug, Clone, Default)]
pub struct ToolMeta {
    pub name: String,
    pub version: String,
    pub license: String,
    pub edition: String,
    pub description: String,
    pub intent: Option<String>, // INT-NNN parsed from description
    // registry enrichment:
    pub category: String,
    pub status: String, // active / retired / deferred
    pub expected_usage: String,
    pub depends_on: Vec<String>,
    pub retired: bool,
}

/// Parse a Cargo.toml [package] block by key (fields may be in ANY order).
fn parse_cargo(path: &PathBuf) -> Option<ToolMeta> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut m = ToolMeta::default();
    let mut in_package = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_package = t == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some((key, val)) = t.split_once('=') {
            let key = key.trim();
            let val = val.trim().trim_matches('"').to_string();
            match key {
                "name" => m.name = val,
                "version" => m.version = val,
                "license" => m.license = val,
                "edition" => m.edition = val,
                "description" => m.description = val,
                _ => {}
            }
        }
    }
    if m.name.is_empty() {
        return None;
    }
    // Extract INT-NNN from the description if present.
    if let Some(pos) = m.description.find("INT-") {
        let tail = &m.description[pos..];
        let num: String = tail
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        m.intent = Some(num);
    }
    Some(m)
}

/// Enrich a ToolMeta from registry/tools.toml (line-parsed, matching existing style).
fn enrich_from_registry(
    tools: &[(String, String, String, String, bool, Vec<String>)],
    m: &mut ToolMeta,
) {
    // tools: (name, category, expected_usage, description, retired)
    if let Some((_, cat, usage, _desc, retired, deps)) =
        tools.iter().find(|(n, _, _, _, _, _)| *n == m.name)
    {
        m.category = cat.clone();
        m.expected_usage = usage.clone();
        m.retired = *retired;
        m.depends_on = deps.clone();
        m.status = if *retired {
            "retired".into()
        } else {
            "active".into()
        };
    } else {
        m.status = "unregistered".into();
        m.category = "uncategorized".into();
    }
}

/// Line-parse registry/tools.toml into (name, category, expected_usage, description, retired).
fn parse_registry() -> Vec<(String, String, String, String, bool, Vec<String>)> {
    let path = core_root().join("registry/tools.toml");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let mut out = vec![];
    let (mut name, mut cat, mut usage, mut desc, mut retired, mut deps) = (
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        false,
        Vec::<String>::new(),
    );
    let flush = |v: &mut Vec<_>,
                 name: &mut String,
                 cat: &mut String,
                 usage: &mut String,
                 desc: &mut String,
                 retired: &mut bool,
                 deps: &mut Vec<String>| {
        if !name.is_empty() {
            v.push((
                name.clone(),
                cat.clone(),
                usage.clone(),
                desc.clone(),
                *retired,
                deps.clone(),
            ));
        }
        *name = String::new();
        *cat = String::new();
        *usage = String::new();
        *desc = String::new();
        *retired = false;
        deps.clear();
    };
    for line in text.lines() {
        let t = line.trim();
        if t == "[[tool]]" {
            flush(
                &mut out,
                &mut name,
                &mut cat,
                &mut usage,
                &mut desc,
                &mut retired,
                &mut deps,
            );
            continue;
        }
        if let Some((k, val)) = t.split_once('=') {
            let val = val.trim().trim_matches('"').to_string();
            match k.trim() {
                "name" => name = val,
                "category" => cat = val,
                "expected_usage" => usage = val,
                "description" => desc = val,
                "retired" => retired = val == "true",
                "depends_on" => {
                    deps = val
                        .trim_matches(|c| c == '[' || c == ']')
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                _ => {}
            }
        }
    }
    flush(
        &mut out,
        &mut name,
        &mut cat,
        &mut usage,
        &mut desc,
        &mut retired,
        &mut deps,
    );
    out
}

/// Gather metadata for ALL tools on disk (rust-tools/*/Cargo.toml).
pub fn gather_all() -> Vec<ToolMeta> {
    let rt = faelight_core::paths::rust_tools_dir();
    let registry = parse_registry();
    let mut metas = vec![];
    if let Ok(entries) = std::fs::read_dir(&rt) {
        for e in entries.flatten() {
            let p = e.path();
            if !p.is_dir() {
                continue;
            }
            let cargo = p.join("Cargo.toml");
            if !cargo.exists() {
                continue;
            }
            if let Some(mut m) = parse_cargo(&cargo) {
                enrich_from_registry(&registry, &mut m);
                metas.push(m);
            }
        }
    }
    // INT-111/037: the engine (core) lives at faelight/engine, not rust-tools/.
    // Include it explicitly so it joins the self-maintaining README pipeline.
    let engine_cargo = core_root().join("faelight/engine/Cargo.toml");
    if engine_cargo.exists() {
        if let Some(mut m) = parse_cargo(&engine_cargo) {
            enrich_from_registry(&registry, &mut m);
            metas.push(m);
        }
    }
    metas.sort_by(|a, b| a.name.cmp(&b.name));
    metas
}

/// PIECE 2a: print parsed metadata for verification (no file writing).
pub fn cmd_readme_tools_dryprint() {
    let metas = gather_all();
    println!(
        "  Parsed {} tools from rust-tools/*/Cargo.toml:\n",
        metas.len()
    );
    for m in &metas {
        let intent = m.intent.clone().unwrap_or_else(|| "-".into());
        let deps = if m.depends_on.is_empty() {
            String::new()
        } else {
            format!("deps={}", m.depends_on.join(","))
        };
        println!(
            "  {:22} v{:8} [{}] cat={:14} intent={} {} {}",
            m.name,
            m.version,
            m.status,
            m.category,
            intent,
            if m.retired { "(RETIRED)" } else { "" },
            deps
        );
    }
    let active = metas.iter().filter(|m| !m.retired).count();
    println!(
        "\n  Total: {} on disk, {} active, {} retired",
        metas.len(),
        active,
        metas.len() - active
    );
}

// INT-111/037: strip a leading "INT-NNN:" or "INT-NNN " prefix from a commit subject,
// so auto-seeded changelogs never expose internal intent numbers on public READMEs.
fn strip_int_prefix(subject: &str) -> String {
    let t = subject.trim();
    if let Some(rest) = t.strip_prefix("INT-") {
        // skip the digits, then an optional ':' or ' ' separator
        let after_num: String = rest
            .chars()
            .skip_while(|c| c.is_ascii_digit() || *c == '-')
            .collect();
        return after_num.trim_start_matches([':', ' ']).trim().to_string();
    }
    t.to_string()
}

// INT-111/037: the Changelog SECTION embedded in a tool's README.
// Curated source: {tool}/CHANGELOG.md (hand-written, version-grouped, emoji'd) -- embedded verbatim.
// Fallback: auto-seed from git history with INT-numbers stripped (presentable until curated).
fn render_changelog_section(m: &ToolMeta) -> String {
    let mut out = String::new();
    out.push_str("## \u{1f4dd} Changelog\n\n");
    // Curated source of truth, if present.
    let rel = if m.name == "core" {
        "faelight/engine/CHANGELOG.md".to_string()
    } else {
        format!("faelight/rust-tools/{}/CHANGELOG.md", m.name)
    };
    let curated_path = core_root().join(&rel);
    if let Ok(curated) = std::fs::read_to_string(&curated_path) {
        // Embed the curated body (skip a leading top-level "# ..." title if present;
        // the README already titles the tool).
        let body: String = curated
            .lines()
            .skip_while(|l| l.trim_start().starts_with("# "))
            .collect::<Vec<_>>()
            .join("\n");
        out.push_str(body.trim());
        out.push_str("\n\n");
        return out;
    }
    // Fallback: auto-seed from git, INT-numbers stripped.
    let hist = tool_history(&m.name, 12);
    if hist.is_empty() {
        out.push_str("_No changelog yet._\n\n");
    } else {
        out.push_str(&format!(
            "### \u{1f331} {}\n\n",
            if m.version.is_empty() {
                "current"
            } else {
                &m.version
            }
        ));
        for (_hash, subj) in &hist {
            let clean = strip_int_prefix(subj);
            if !clean.is_empty() {
                out.push_str(&format!("- {}\n", clean));
            }
        }
        out.push_str("\n> _Auto-seeded from history; curated entries coming._\n\n");
    }
    out
}

/// PIECE 2b: render a rich NixOS-era README for one tool.
pub fn render_readme(m: &ToolMeta) -> String {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let status_badge = match m.status.as_str() {
        "active" => "active",
        "retired" => "retired",
        "unregistered" => "active (unregistered)",
        other => other,
    };
    let mut out = String::new();

    // Title
    out.push_str(&format!("# \u{1f332} {}\n\n", m.name));

    // Badge line
    out.push_str(&format!(
        "**Version:** {} &nbsp;|&nbsp; **License:** {} &nbsp;|&nbsp; **Status:** {} &nbsp;|&nbsp; **Category:** {}\n\n",
        if m.version.is_empty() { "-" } else { &m.version },
        if m.license.is_empty() { "-" } else { &m.license },
        status_badge,
        if m.category.is_empty() { "-" } else { &m.category },
    ));

    // Description (curated prose from Cargo.toml)
    out.push_str("## \u{1f4d6} Description\n\n");
    if !m.description.is_empty() {
        let desc = match m.description.find("-- INT-") {
            Some(pos) => m.description[..pos].trim_end().to_string(),
            None => m.description.clone(),
        };
        out.push_str(&format!("{}\n\n", desc));
    }

    out.push_str("---\n\n");
    // Changelog: curated CHANGELOG.md if present, else auto-seed from git (INT stripped)
    out.push_str(&render_changelog_section(m));

    out.push_str("---\n\n");

    // Build & install (NixOS-native -- NOT Arch/stow)
    out.push_str("## Build\n\n");
    out.push_str("```sh\n");
    out.push_str(&format!(
        "nix develop ~/0-core#faelight-forest -c cargo build -p {}\n",
        m.name
    ));
    out.push_str("```\n\n");
    out.push_str("## Deploy\n\n");
    out.push_str("```sh\n");
    out.push_str("deploy   # sudo nixos-rebuild switch --flake .#framework16\n");
    out.push_str("```\n\n");

    // Dependencies
    if !m.depends_on.is_empty() {
        out.push_str("## Forest dependencies\n\n");
        for d in &m.depends_on {
            out.push_str(&format!("- `{}`\n", d));
        }
        out.push_str("\n");
    }

    // Footer
    out.push_str("---\n\n");
    out.push_str(&format!(
        "*Part of [Faelight Forest](../../). Last verified {}.*\n",
        date,
    ));
    out.push_str("*This README is generated by `faelight-docs sync` -- do not hand-edit; re-run to refresh.*\n");

    out
}

/// PIECE 2b verify: print the rendered README for ONE named tool (no writing).
pub fn cmd_preview_one(name: &str) {
    let metas = gather_all();
    match metas.iter().find(|m| m.name == name) {
        Some(m) => {
            println!("----- README preview: {} -----\n", name);
            print!("{}", render_readme(m));
            println!("----- end preview -----");
        }
        None => println!("  tool not found on disk: {}", name),
    }
}

/// PIECE 2c: render the top-level rust-tools/README.md index (catalog by category).
pub fn render_index(metas: &[ToolMeta]) -> String {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let active: Vec<&ToolMeta> = metas.iter().filter(|m| !m.retired).collect();
    let retired: Vec<&ToolMeta> = metas.iter().filter(|m| m.retired).collect();

    let mut out = String::new();
    out.push_str("# Faelight Forest -- Rust Tools\n\n");
    out.push_str(&format!(
        "The forest's tool ecosystem: {} active tools (plus {} retired), each a purpose-built Rust program.\n\n",
        active.len(), retired.len()
    ));
    out.push_str(&format!(
        "**Generated:** {} by `faelight-docs sync`\n\n",
        date
    ));
    out.push_str("---\n\n");

    // Group active tools by category.
    let mut cats: Vec<String> = active.iter().map(|m| m.category.clone()).collect();
    cats.sort();
    cats.dedup();

    for cat in &cats {
        let in_cat: Vec<&&ToolMeta> = active.iter().filter(|m| &m.category == cat).collect();
        if in_cat.is_empty() {
            continue;
        }
        out.push_str(&format!("## {}\n\n", cap_first(cat)));
        out.push_str("| Tool | Version | Description |\n");
        out.push_str("|------|---------|-------------|\n");
        for m in &in_cat {
            let desc = match m.description.find("-- INT-") {
                Some(pos) => m.description[..pos].trim_end().to_string(),
                None => m.description.clone(),
            };
            let desc = if desc.is_empty() {
                "-".to_string()
            } else {
                desc
            };
            out.push_str(&format!(
                "| [`{}`](./{}/) | {} | {} |\n",
                m.name,
                m.name,
                if m.version.is_empty() {
                    "-"
                } else {
                    &m.version
                },
                desc,
            ));
        }
        out.push_str("\n");
    }

    if !retired.is_empty() {
        out.push_str("## Retired\n\n");
        for m in &retired {
            out.push_str(&format!("- `{}` (v{})\n", m.name, m.version));
        }
        out.push_str("\n");
    }

    out.push_str("---\n\n");
    out.push_str("*This index is generated -- do not hand-edit; re-run `faelight-docs readme-generate` to refresh.*\n");
    out
}

fn cap_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// PIECE 2c: generate per-tool READMEs + the index. dry_run prints what WOULD be written.
pub fn cmd_generate(dry_run: bool) {
    let metas = gather_all();
    let rt = faelight_core::paths::rust_tools_dir();
    let mut written = 0usize;
    let mut skipped = 0usize;

    for m in &metas {
        // Generate READMEs for all tools (active + retired get a stub via the same template).
        // INT-111/037: core (engine) writes to faelight/engine/, not rust-tools/core/.
        let readme_path = if m.name == "core" {
            core_root().join("faelight/engine/README.md")
        } else {
            rt.join(&m.name).join("README.md")
        };
        let content = render_readme(m);
        if dry_run {
            println!(
                "  would write: rust-tools/{}/README.md ({} bytes)",
                m.name,
                content.len()
            );
            written += 1;
            continue;
        }
        match std::fs::write(&readme_path, &content) {
            Ok(_) => {
                written += 1;
            }
            Err(e) => {
                eprintln!("  failed: {}/README.md -- {}", m.name, e);
                skipped += 1;
            }
        }
    }

    // Top-level index.
    let index = render_index(&metas);
    let index_path = rt.join("README.md");
    if dry_run {
        println!(
            "  would write: rust-tools/README.md (index, {} bytes)",
            index.len()
        );
    } else {
        match std::fs::write(&index_path, &index) {
            Ok(_) => println!("  wrote: rust-tools/README.md (index)"),
            Err(e) => eprintln!("  failed: index -- {}", e),
        }
    }

    if dry_run {
        println!(
            "\n  DRY RUN: {} per-tool READMEs + 1 index would be written",
            written
        );
    } else {
        println!(
            "\n  Done: {} per-tool READMEs written, {} skipped, + index",
            written, skipped
        );
    }
}

/// Truncate a string at a word boundary near `max` chars, adding an ellipsis
/// if the original was longer. Never cuts mid-word.
fn truncate_words(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    // Take up to `max` chars, then back off to the last space.
    let head: String = s.chars().take(max).collect();
    let cut = match head.rfind(' ') {
        Some(pos) if pos > 0 => &head[..pos],
        _ => &head,
    };
    format!("{}\u{2026}", cut.trim_end())
}

/// PIECE 3: pull recent SUBSTANTIVE commits touching a tool's dir.
/// Subject-line only, noise-filtered, capped. Returns Vec<(short_hash, subject)>.
fn tool_history(name: &str, cap: usize) -> Vec<(String, String)> {
    let path = format!("rust-tools/{}/", name);
    let new_path = format!("faelight/rust-tools/{}/", name);
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(core_root())
        .args(["log", "--pretty=format:%h\x1f%s", "--", &path, &new_path])
        .output();
    let text = match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => return vec![],
    };
    let noise = [
        "wip",
        "typo",
        "fmt",
        "format",
        "merge",
        "bump version",
        "checkpoint",
    ];
    let mut entries = vec![];
    for line in text.lines() {
        if let Some((hash, subj)) = line.split_once('\x1f') {
            let s = subj.trim();
            // subject only; truncate at a word boundary near 80 chars, add ellipsis.
            let subject = truncate_words(s, 80);
            let low = subject.to_lowercase();
            if noise.iter().any(|n| low == *n || low.starts_with(n)) {
                continue;
            }
            if subject.is_empty() {
                continue;
            }
            entries.push((hash.to_string(), subject));
            if entries.len() >= cap {
                break;
            }
        }
    }
    entries
}

/// PIECE 3: render a CHANGELOG for one tool -- header + NixOS migration entry + history.
pub fn render_changelog(m: &ToolMeta) -> String {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut out = String::new();

    out.push_str(&format!("# Changelog -- {}\n\n", m.name));
    out.push_str(
        "All notable changes to this tool. Generated by `faelight-docs changelog-tools`.\n\n",
    );
    out.push_str("This project follows the Faelight Forest versioning model; format loosely tracks Keep a Changelog.\n\n");
    out.push_str("---\n\n");

    // NixOS migration entry (the gate requirement).
    out.push_str(&format!(
        "## [{}] -- NixOS 26.05 era\n\n",
        if m.version.is_empty() {
            "current"
        } else {
            &m.version
        }
    ));
    out.push_str("### Changed\n");
    out.push_str("- Migrated from Arch Linux to NixOS 26.05 (Yarara).\n");
    out.push_str("- Build: `nix develop ~/0-core#faelight-forest -c cargo build`.\n");
    out.push_str("- Deploy: home-manager / `nixos-rebuild switch` (replaces stow).\n\n");

    // Recent substantive history from git.
    let hist = tool_history(&m.name, 15);
    if !hist.is_empty() {
        out.push_str("## Recent history\n\n");
        for (hash, subj) in &hist {
            out.push_str(&format!("- `{}` {}\n", hash, subj));
        }
        out.push_str("\n");
    }

    out.push_str("---\n\n");
    out.push_str(&format!("*Generated {} by `faelight-docs changelog-tools` -- do not hand-edit; re-run to refresh.*\n", date));
    out
}

/// PIECE 3: preview ONE tool's changelog (no writing).
pub fn cmd_changelog_preview(name: &str) {
    let metas = gather_all();
    match metas.iter().find(|m| m.name == name) {
        Some(m) => {
            println!("----- CHANGELOG preview: {} -----\n", name);
            print!("{}", render_changelog(m));
            println!("----- end preview -----");
        }
        None => println!("  tool not found on disk: {}", name),
    }
}

/// PIECE 3: generate CHANGELOG.md for all tools. dry_run prints what would be written.
pub fn cmd_changelog_generate(dry_run: bool) {
    let metas = gather_all();
    let rt = faelight_core::paths::rust_tools_dir();
    let mut written = 0usize;
    for m in &metas {
        let path = rt.join(&m.name).join("CHANGELOG.md");
        let content = render_changelog(m);
        if dry_run {
            println!(
                "  would write: rust-tools/{}/CHANGELOG.md ({} bytes)",
                m.name,
                content.len()
            );
            written += 1;
            continue;
        }
        match std::fs::write(&path, &content) {
            Ok(_) => written += 1,
            Err(e) => eprintln!("  failed: {}/CHANGELOG.md -- {}", m.name, e),
        }
    }
    if dry_run {
        println!("\n  DRY RUN: {} CHANGELOGs would be written", written);
    } else {
        println!("\n  Done: {} CHANGELOGs written", written);
    }
}
