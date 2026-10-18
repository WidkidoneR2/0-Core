//! INT-037: per-tool README generator + index.
//! Reads each `tools/<tool>/Cargo.toml` (ground truth) and enriches with
//! registry/tools.toml status, then emits rich READMEs + a top-level index.
//! Self-maintaining: re-run any time tools change; docs never drift stale.

use std::path::PathBuf;

/// A path as the messages show it: relative to the repo, so the message names the file the
/// write actually touches (INT-267 L2).
fn shown(p: &std::path::Path) -> String {
    p.strip_prefix(zero_core::paths::core_dir())
        .unwrap_or(p)
        .display()
        .to_string()
}

/// The catalog's link to a tool, relative to the catalog itself (the tools directory): ./name/
/// for a crate there, up and across for one under another parent. Registry name core is the
/// engine (INT-267 step 2a).
fn index_link(name: &str) -> String {
    let index_dir = zero_core::paths::tools_dir();
    let dir_name = if name == "core" { "engine" } else { name };
    match zero_core::paths::crate_dir(dir_name) {
        Some(dir) if dir.parent() == Some(index_dir.as_path()) => format!("./{}/", dir_name),
        Some(dir) => match index_dir.parent().and_then(|z| dir.strip_prefix(z).ok()) {
            Some(rel) => format!("../{}/", rel.display()),
            None => format!("./{}/", name),
        },
        None => format!("./{}/", name),
    }
}

/// Combined metadata for one tool: Cargo.toml fields + registry status.
#[derive(Debug, Clone, Default)]
pub struct ToolMeta {
    pub name: String,
    pub version: String,
    pub license: String,
    pub edition: String,
    pub description: String,
    // registry enrichment:
    pub category: String,
    pub status: String, // active / retired / deferred
    pub expected_usage: String,
    pub depends_on: Vec<String>,
    pub retired: bool,
    /// Has a binary target: a `[[bin]]` section or `src/main.rs`. Only binaries are tools.
    pub binary: bool,
}

/// Parse a Cargo.toml `[package]` block by key (fields may be in ANY order).
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
    m.binary = text.contains("[[bin]]")
        || path
            .parent()
            .map(|d| d.join("src/main.rs").exists())
            .unwrap_or(false);
    Some(m)
}

/// The registry rows, read once through the core. Decision 149: the core owns the path and the
/// parse. This read once failed silently for a month (the path moved and nothing followed it),
/// so every README said "unregistered". A registry that cannot be read is said, never swallowed:
/// the generator continues, and every tool then renders as unregistered, as the message says.
fn read_registry() -> Vec<zero_core::registry::Row> {
    match zero_core::registry::read() {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!(
                "  registry not read: {} -- every tool will render as unregistered",
                e
            );
            Vec::new()
        }
    }
}

/// Enrich a ToolMeta from the registry row that names it.
fn enrich_from_registry(rows: &[zero_core::registry::Row], m: &mut ToolMeta) {
    if let Some(row) = rows.iter().find(|r| r.name == m.name) {
        m.category = row.category.clone().unwrap_or_default();
        m.expected_usage = row.expected_usage.clone().unwrap_or_default();
        m.retired = row.retired;
        m.depends_on = row.depends_on.clone();
        m.status = if row.retired {
            "retired".into()
        } else {
            "active".into()
        };
    } else {
        m.status = "unregistered".into();
        m.category = "uncategorized".into();
    }
}

/// Gather metadata for ALL tools on disk (tools/*/Cargo.toml).
pub fn gather_all() -> Vec<ToolMeta> {
    let registry = read_registry();
    let mut metas = vec![];
    if let Ok(entries) = zero_core::paths::tool_parent_entries() {
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
    // INT-111/037: the engine (core) lives at zero/engine, not tools/.
    // Include it explicitly so it joins the self-maintaining README pipeline.
    let engine_cargo = zero_core::paths::core_dir().join("zero/engine/Cargo.toml");
    if engine_cargo.exists() {
        if let Some(mut m) = parse_cargo(&engine_cargo) {
            enrich_from_registry(&registry, &mut m);
            metas.push(m);
        }
    }
    metas.sort_by(|a, b| a.name.cmp(&b.name));
    metas
}

// INT-111/037: the Changelog SECTION embedded in a tool's README.
// Curated source: {tool}/CHANGELOG.md (hand-written, version-grouped, emoji'd) -- linked from the README, never copied into it.
// No CHANGELOG.md: the README says so -- git history is never copied into it.
fn render_changelog_section(m: &ToolMeta) -> String {
    let mut out = String::new();
    out.push_str("## \u{1f4dd} Changelog\n\n");
    // Curated source of truth, if present.
    let dir: &str = if m.name == "core" { "engine" } else { &m.name };
    let rel = zero_core::paths::crate_rel_dir(dir)
        .map(|d| format!("{}/CHANGELOG.md", d))
        .unwrap_or_default();
    let curated_path = zero_core::paths::core_dir().join(&rel);
    if curated_path.is_file() {
        // INT-247: the README links the curated CHANGELOG.md instead of embedding it,
        // so the history it carries lives in one place and is never copied.
        out.push_str("See [CHANGELOG.md](./CHANGELOG.md).\n\n");
        return out;
    }
    // INT-247: no CHANGELOG.md means no changelog -- git subjects are never copied into a README.
    out.push_str("_No changelog yet._\n\n");
    out
}

/// PIECE 2b: the SCAFFOLD for one tool's README (INT-291 G1a, G1b).
///
/// A title, the Cargo.toml description, the changelog link, Build and Deploy -- and nothing a
/// tool does not own. No version, date, status or category: Cargo.toml owns the version and the
/// registry owns the rest, and a copy written into prose is a fact that goes stale (14 of 16 did).
/// No footer either: the scaffold is written once, and the README is the person's from then on.
pub fn render_readme(m: &ToolMeta) -> String {
    let mut out = String::new();

    // Title
    out.push_str(&format!("# \u{1f332} {}\n\n", m.name));

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
    // Changelog: the curated CHANGELOG.md is linked when it exists (INT-291 G1e).
    out.push_str(&render_changelog_section(m));

    out.push_str("---\n\n");

    // Build & install
    out.push_str("## Build\n\n");
    out.push_str("```sh\n");
    out.push_str(&format!("cargo build -p {}\n", m.name));
    out.push_str("```\n\n");
    out.push_str("## Deploy\n\n");
    out.push_str("```sh\n");
    out.push_str("ship   # builds the release and deploys to ~/.local/bin\n");
    out.push_str("```\n");

    out
}

/// The last line every README this tool generated before INT-291 carried. It is the ONLY proof
/// that a file was generated, because a person's file never ends with it.
const LEGACY_FOOTER: &str =
    "*This README is generated by `zero-docs sync` -- do not hand-edit; re-run to refresh.*";

/// Whether `text` is a README this tool generated, and so may overwrite (INT-291 G1b).
///
/// Positive proof, so it fails safe: the last non-empty line is exactly the generated footer and
/// no line says HAND-MAINTAINED. Anything else -- prose, a hand-written file whose own footer
/// only MENTIONS the generator, an empty file -- belongs to a person.
fn is_generated(text: &str) -> bool {
    if text.lines().any(|l| l.contains("HAND-MAINTAINED")) {
        return false;
    }
    text.lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .is_some_and(|l| l.trim() == LEGACY_FOOTER)
}

/// What `readme-generate` may do with one tool's README (INT-291 G1b).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Disposition {
    /// No file: write the scaffold.
    Scaffold,
    /// A file this tool generated: replace it with the scaffold, once. It is a person's after.
    Rewrite,
    /// Anything else: leave it byte-identical.
    Keep,
}

/// Decide from the file on disk. A file that cannot be read (permissions, not UTF-8) cannot be
/// proven generated, so it is kept, never overwritten.
fn classify(path: &std::path::Path) -> Disposition {
    match std::fs::read_to_string(path) {
        Ok(t) if is_generated(&t) => Disposition::Rewrite,
        Ok(_) => Disposition::Keep,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Disposition::Scaffold,
        Err(_) => Disposition::Keep,
    }
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

/// The tools the index lists: binaries that are not retired. The ONE place that is decided --
/// the header and readme-index's own report both count this, so they cannot disagree.
pub fn active_tools(metas: &[ToolMeta]) -> Vec<&ToolMeta> {
    metas.iter().filter(|m| !m.retired && m.binary).collect()
}

/// PIECE 2c: render the top-level tools/README.md index (catalog by category).
pub fn render_index(metas: &[ToolMeta]) -> String {
    let active = active_tools(metas);
    let retired: Vec<&ToolMeta> = metas.iter().filter(|m| m.retired).collect();

    let mut out = String::new();
    out.push_str("# Project 0 -- Rust Tools\n\n");
    out.push_str(&format!(
        "Project 0's tool ecosystem: {} active tools (plus {} retired), each a purpose-built Rust program.\n\n",
        active.len(), retired.len()
    ));
    out.push_str(
        "**Generated** by `zero-docs readme-index` from each tool's Cargo.toml and the registry; nothing here is typed by hand.\n\n",
    );
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
                "| [`{}`]({}) | {} | {} |\n",
                m.name,
                index_link(&m.name),
                if m.version.is_empty() {
                    "-"
                } else {
                    &m.version
                },
                desc,
            ));
        }
        out.push('\n');
    }

    if !retired.is_empty() {
        out.push_str("## Retired\n\n");
        for m in &retired {
            out.push_str(&format!("- `{}` (v{})\n", m.name, m.version));
        }
        out.push('\n');
    }

    out.push_str("---\n\n");
    out.push_str("*This index is generated -- do not hand-edit; re-run `zero-docs readme-generate` to refresh.*\n");
    out
}

/// INT-291 G1a: the names of every tool whose line in tools/README.md differs from what the index
/// renders now from Cargo.toml and the registry. Empty means the index is true.
pub fn check_index() -> Result<Vec<String>, String> {
    let metas = gather_all();
    let path = zero_core::paths::tools_dir().join("README.md");
    let disk = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {}", shown(&path), e))?;
    let names: Vec<&str> = metas.iter().map(|m| m.name.as_str()).collect();
    Ok(stale_names(&render_index(&metas), &disk, &names))
}

/// Compare two renderings line by line, in no particular order, and name each tool that has a line
/// in the difference. A difference that names no tool (header, counts, a removed tool) is reported
/// as "other lines of the index", so a stale index is never reported clean.
fn stale_names(expected: &str, disk: &str, names: &[&str]) -> Vec<String> {
    let exp: Vec<&str> = expected.lines().collect();
    let got: Vec<&str> = disk.lines().collect();
    let differing: Vec<&str> = exp
        .iter()
        .filter(|l| !got.contains(l))
        .chain(got.iter().filter(|l| !exp.contains(l)))
        .copied()
        .collect();
    let mut stale: Vec<String> = names
        .iter()
        .filter(|n| {
            let tick = format!("`{}`", n);
            differing.iter().any(|l| l.contains(&tick))
        })
        .map(|n| n.to_string())
        .collect();
    if stale.is_empty() && !differing.is_empty() {
        stale.push("other lines of the index".to_string());
    }
    stale
}

fn cap_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// PIECE 2c: write ONLY the top-level index. dry_run prints what WOULD be written.
///
/// ⚠️ `cmd_generate` used to DESTROY HAND-WRITTEN DOCUMENTATION (measured 2026-09-15 on
/// novashell/README.md). Since INT-291 it writes only a README it can prove it generated, and this
/// function never touches a per-tool README at all.
///
/// The index is entirely derived: tools come from `gather_all()` (which reads DISK, so a deleted
/// crate cannot survive in it), versions from each tool's Cargo.toml, the rest from the registry.
/// Nobody hand-edits it.
pub fn cmd_index(dry_run: bool) {
    let metas = gather_all();
    let rt = zero_core::paths::tools_dir();
    let index = render_index(&metas);
    let path = rt.join("README.md");
    if dry_run {
        println!(
            "  would write: {} (index, {} bytes, {} active tools, {} crates read)",
            shown(&path),
            index.len(),
            active_tools(&metas).len(),
            metas.len()
        );
        return;
    }
    match std::fs::write(&path, &index) {
        Ok(_) => println!(
            "  wrote: {} (index, {} active tools, {} crates read)",
            shown(&path),
            active_tools(&metas).len(),
            metas.len()
        ),
        Err(e) => {
            eprintln!("  failed: index -- {}", e);
            std::process::exit(1);
        }
    }
}

/// PIECE 2d: scaffold missing per-tool READMEs, retire the ones this tool generated, and write the
/// index. A README a person owns is never touched (INT-291 G1b). dry_run prints what WOULD happen.
pub fn cmd_generate(dry_run: bool) {
    let metas = gather_all();
    let rt = zero_core::paths::tools_dir();
    let (mut scaffolded, mut rewritten, mut kept, mut failed) = (0usize, 0usize, 0usize, 0usize);

    for m in &metas {
        // Generate READMEs for all tools (active + retired get a stub via the same template).
        // INT-111/037: core (engine) writes to zero/engine/, not tools/core/.
        let readme_path = if m.name == "core" {
            zero_core::paths::core_dir().join("zero/engine/README.md")
        } else {
            zero_core::paths::crate_dir(&m.name)
                .unwrap_or_else(|| rt.join(&m.name))
                .join("README.md")
        };
        // Only a file this tool can prove it generated is ever replaced.
        let what = classify(&readme_path);
        if what == Disposition::Keep {
            kept += 1;
            if dry_run {
                println!("  keeps: {} (person-owned)", shown(&readme_path));
            }
            continue;
        }
        let content = render_readme(m);
        let verb = if what == Disposition::Scaffold {
            "scaffold"
        } else {
            "rewrite"
        };
        if dry_run {
            println!(
                "  would {}: {} ({} bytes)",
                verb,
                shown(&readme_path),
                content.len()
            );
        } else if let Err(e) = std::fs::write(&readme_path, &content) {
            eprintln!("  failed: {} -- {}", shown(&readme_path), e);
            failed += 1;
            continue;
        }
        if what == Disposition::Scaffold {
            scaffolded += 1;
        } else {
            rewritten += 1;
        }
    }

    // Top-level index.
    let index = render_index(&metas);
    let index_path = rt.join("README.md");
    if dry_run {
        println!(
            "  would write: {} (index, {} bytes)",
            shown(&index_path),
            index.len()
        );
    } else {
        match std::fs::write(&index_path, &index) {
            Ok(_) => println!("  wrote: {} (index)", shown(&index_path)),
            Err(e) => {
                eprintln!("  failed: index -- {}", e);
                failed += 1;
            }
        }
    }

    if dry_run {
        println!(
            "\n  DRY RUN: {} per-tool READMEs: {} to scaffold, {} to rewrite, {} kept (person-owned); index would be written",
            metas.len(),
            scaffolded,
            rewritten,
            kept
        );
    } else {
        println!(
            "\n  Done: {} per-tool READMEs: {} scaffolded, {} rewritten, {} kept (person-owned), {} failed; index written",
            metas.len(),
            scaffolded,
            rewritten,
            kept,
            failed
        );
        if failed > 0 {
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod count_tests {
    use super::*;

    /// The catalog counts tools, and a tool is a binary. A library crate on disk is not one --
    /// counting every crate made the catalog say 22 while 18 binaries were deployed.
    #[test]
    fn a_library_crate_is_not_a_tool() {
        let d = std::env::temp_dir().join(format!("zero-docs-count-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("src")).unwrap();
        let cargo = d.join("Cargo.toml");
        std::fs::write(&cargo, "[package]\nname = \"x\"\nversion = \"0.1.0\"\n").unwrap();
        std::fs::write(d.join("src/lib.rs"), "").unwrap();
        assert!(
            !parse_cargo(&cargo).unwrap().binary,
            "lib.rs alone is not a tool"
        );
        std::fs::write(d.join("src/main.rs"), "fn main() {}").unwrap();
        assert!(
            parse_cargo(&cargo).unwrap().binary,
            "src/main.rs is a binary"
        );
        std::fs::remove_file(d.join("src/main.rs")).unwrap();
        std::fs::write(
            &cargo,
            "[package]\nname = \"x\"\n\n[[bin]]\nname = \"x\"\npath = \"src/x.rs\"\n",
        )
        .unwrap();
        assert!(
            parse_cargo(&cargo).unwrap().binary,
            "a [[bin]] target is a binary"
        );
        let _ = std::fs::remove_dir_all(&d);
    }
}

#[cfg(test)]
mod ownership_tests {
    use super::*;

    #[test]
    fn index_staleness_names_every_changed_tool() {
        let fresh =
            "# Tools\n\n| [`alpha`](./alpha/) | AAA | a |\n| [`beta`](./beta/) | BBB | b |\n";
        let names = ["alpha", "beta"];
        assert!(stale_names(fresh, fresh, &names).is_empty());
        let one = fresh.replace("AAA", "CCC");
        assert_eq!(stale_names(fresh, &one, &names), vec!["alpha".to_string()]);
        let both = one.replace("BBB", "DDD");
        assert_eq!(
            stale_names(fresh, &both, &names),
            vec!["alpha".to_string(), "beta".to_string()]
        );
        let header = fresh.replace("# Tools", "# Old");
        assert_eq!(
            stale_names(fresh, &header, &names),
            vec!["other lines of the index".to_string()]
        );
        let extra = format!("{fresh}| [`gone`](./gone/) | 9 | x |\n");
        assert_eq!(
            stale_names(fresh, &extra, &names),
            vec!["other lines of the index".to_string()]
        );
    }

    #[test]
    fn a_file_ending_in_the_generated_footer_is_generated() {
        let footer = LEGACY_FOOTER;
        assert!(is_generated(&format!("# t\n\nbody\n\n---\n\n{footer}\n")));
        assert!(
            is_generated(&format!("# t\n{footer}\n\n\n")),
            "trailing blank lines do not matter"
        );
        assert!(
            is_generated(&format!("# t\r\n{footer}\r\n")),
            "nor does a CRLF file"
        );
    }

    #[test]
    fn a_person_owned_file_is_never_generated() {
        let footer = LEGACY_FOOTER;
        // novashell/README.md: its footer wraps across a line, so a substring test would miss it.
        let wrapped = "# nsh\n\n*HAND-MAINTAINED. This file used to claim it was generated by `zero-docs\nsync`; it is not.*\n";
        assert!(!is_generated(wrapped));
        assert!(!is_generated(
            "# prose\n\nA person wrote this and never added a marker.\n"
        ));
        assert!(!is_generated(""));
        assert!(
            !is_generated(&format!("{footer}\nA person added a line after it.\n")),
            "the footer must be the LAST line"
        );
        assert!(
            !is_generated(&format!("# t\nHAND-MAINTAINED\n{footer}\n")),
            "the marker wins over the footer"
        );
    }

    #[test]
    fn classify_proves_ownership_from_the_file_on_disk() {
        let dir = std::env::temp_dir().join(format!("zd-classify-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let put = |name: &str, bytes: &[u8]| {
            let p = dir.join(name);
            std::fs::write(&p, bytes).unwrap();
            p
        };
        let footer = LEGACY_FOOTER;
        assert_eq!(classify(&dir.join("missing.md")), Disposition::Scaffold);
        let generated = put("gen.md", format!("# t\n{footer}\n").as_bytes());
        assert_eq!(classify(&generated), Disposition::Rewrite);
        assert_eq!(classify(&put("prose.md", b"# prose\n")), Disposition::Keep);
        assert_eq!(
            classify(&put("binary.md", &[0xff, 0xfe, 0x00])),
            Disposition::Keep,
            "an unreadable file is kept, not overwritten"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
