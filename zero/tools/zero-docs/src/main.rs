// zero-docs -- Living Documentation Engine
// INT-145, INT-291, INT-296: keeps the tools index, the public docs and the README links true.
//
// OWNERSHIP RULE: zero-docs writes only what it generates (the tools index, docs/public) and prints
// only what it measures. The root README is person-owned: zero-docs reads it to check its links and
// never writes it.
//
// Commands:
//   sync             write the tools index and the public docs, and verify the README links
//   check            what is out of date (dry run); exits 1 when the tools index is stale
//   status           what zero-docs measures: tools, domains, commits, and whether the index is true
//   links            every README link resolves
//   public           generate the curated public docs set
//   readme-preview   one tool's scaffold README (writes nothing)
//   readme-index     write the tools index
//   readme-generate  scaffold missing tool READMEs; never touches one a person owns

mod toolgen;
use colored::*;

// INT-265: read from Cargo, so --version cannot drift from the package as zero-git did.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// INT-315 Phase 3: generate public docs set -- curated, no internal content
fn cmd_public(dry_run: bool) {
    use std::fs;
    let root = zero_core::paths::core_dir();
    let docs_dir = root.join("docs");
    let public_dir = docs_dir.join("public");

    // Public docs -- curated set, no internal language
    let public_files = [
        ("COMMAND-GUIDE.md", "Command Reference"),
        ("ARCHITECTURE.md", "Architecture"),
        ("PHILOSOPHY.md", "Philosophy"),
        ("NSH-PHILOSOPHY.md", "Shell Philosophy"),
        ("NOVASHELL.md", "Shell Reference"),
    ];

    if dry_run {
        println!("  -> public/ would contain {} docs:", public_files.len());
        for (file, title) in &public_files {
            let src = docs_dir.join(file);
            let exists = if src.exists() { "✅" } else { "❌ missing" };
            println!("     {} {} ({})", exists, title, file);
        }
        return;
    }

    if !public_dir.exists() {
        if let Err(e) = fs::create_dir_all(&public_dir) {
            eprintln!("  ❌ Could not create docs/public/: {}", e);
            return;
        }
    }

    // Generate index.md. INT-296 (ruling h): no release version here. The VERSION file and
    // Cargo.toml own it, and a copy in a generated page is wrong by the next release.
    let mut index = String::from("# Project 0 Documentation\n\n## Contents\n\n");

    let mut copied = 0usize;
    for (file, title) in &public_files {
        let src = docs_dir.join(file);
        let dst = public_dir.join(file);
        if src.exists() {
            if let Ok(src_content) = fs::read_to_string(&src) {
                // Strip any lines containing internal markers
                let clean: String = src_content
                    .lines()
                    .filter(|l| !l.contains("INT-") || l.contains("INT-") && l.starts_with('#'))
                    .collect::<Vec<_>>()
                    .join("\n");
                let _ = fs::write(&dst, clean);
                index.push_str(&format!("- [{}]({})\n", title, file));
                copied += 1;
            }
        }
    }

    let _ = fs::write(public_dir.join("index.md"), index);
    println!(
        "  {} docs/public/ -- {} docs generated",
        "✅".normal(),
        copied
    );
}

fn main() {
    // INT-256: FIRST statement, before any output. `tool | head -3` must not print a panic.
    zero_core::restore_sigpipe();
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        // INT-265: --version prints the version. It fell through to the banner.
        "--version" | "-V" => println!("zero-docs {}", VERSION),
        "--health" | "health" => {
            println!("zero-docs v{} — healthy", VERSION);
        }
        "sync" => {
            toolgen::cmd_index(false);
            verify_links(false);
            cmd_public(false);
            println!("  {} All docs synced", "✅".normal());
        }
        "public" => {
            let dry = args.get(2).map(|s| s == "--dry-run").unwrap_or(false);
            cmd_public(dry);
        }
        "check" => {
            verify_links(true);
            match toolgen::check_index() {
                Ok(stale) if stale.is_empty() => {
                    println!("  \u{2705} tools/README.md matches every Cargo.toml")
                }
                Ok(stale) => {
                    eprintln!(
                        "  \u{2717} tools/README.md is stale for: {} -- run `zero-docs sync`",
                        stale.join(", ")
                    );
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("  \u{2717} {}", e);
                    std::process::exit(1);
                }
            }
        }
        "verify-links" | "links" => {
            verify_links(false);
        }
        "status" => cmd_status(),
        "readme-preview" => {
            let name = args.get(2).map(|s| s.as_str()).unwrap_or("");
            toolgen::cmd_preview_one(name);
        }
        "readme-index" => {
            let dry = args.get(2).map(|s| s == "--dry-run").unwrap_or(false);
            toolgen::cmd_index(dry);
        }
        "readme-generate" => {
            let dry = args.get(2).map(|s| s == "--dry-run").unwrap_or(false);
            toolgen::cmd_generate(dry);
        }
        _ => cmd_help(),
    }
}

/// How many commits the repository holds. INT-250: git, not the old commit-count file.
fn commit_count() -> Option<usize> {
    std::process::Command::new("git")
        .args([
            "-C",
            &zero_core::paths::core_dir().to_string_lossy(),
            "rev-list",
            "--count",
            "HEAD",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
}

/// The engine's domain directories (directories, not `ls` entries: mod.rs is not a domain).
fn domain_count() -> Result<usize, String> {
    let path = zero_core::paths::core_dir().join("zero/engine/src/domains");
    std::fs::read_dir(&path)
        .map(|d| d.flatten().filter(|e| e.path().is_dir()).count())
        .map_err(|e| format!("cannot read {} ({})", path.display(), e))
}

fn verify_links(dry_run: bool) -> bool {
    let root = zero_core::paths::core_dir();
    let readme_path = root.join("README.md");
    let content = match std::fs::read_to_string(&readme_path) {
        Ok(c) => c,
        Err(_) => return true,
    };

    let mut broken: Vec<String> = vec![];

    // Scan for markdown links [text](path)
    let mut pos = 0;
    let bytes = content.as_bytes();
    while pos < content.len() {
        if let Some(start) = content[pos..].find("](") {
            let link_start = pos + start + 2;
            if let Some(end) = content[link_start..].find(')') {
                let link = &content[link_start..link_start + end];
                // Only check relative links (not http/https/mailto/#)
                if !link.starts_with("http")
                    && !link.starts_with('#')
                    && !link.starts_with("mailto")
                    && !link.is_empty()
                {
                    let full_path = root.join(link);
                    if !full_path.exists() {
                        broken.push(link.to_string());
                    }
                }
                pos = link_start + end + 1;
            } else {
                break;
            }
        } else {
            break;
        }
        let _ = bytes; // suppress warning
    }

    if broken.is_empty() {
        if !dry_run {
            println!(
                "  {} README links verified — all resolve correctly",
                "✅".normal()
            );
        }
        return true;
    }

    for link in &broken {
        println!(
            "  {} Broken link: {}",
            "✗".bright_red(),
            link.bright_white()
        );
    }
    if dry_run {
        println!(
            "  {} {} broken link(s) found — fix before release",
            "⚠️ ".normal(),
            broken.len()
        );
    }
    false
}

fn cmd_status() {
    let unknown = |why: String| format!("unknown -- {}", why).yellow().to_string();
    let row = |label: &str, value: String| {
        println!("  {} {}", format!("{:<14}", label).dimmed(), value);
    };

    println!();
    println!("  {} zero-docs status", "📋".normal());
    println!("{}", "  ─────────────────────────────────────".dimmed());
    row(
        "Tools:",
        toolgen::active_tool_count()
            .to_string()
            .bright_white()
            .to_string(),
    );
    row(
        "Core domains:",
        match domain_count() {
            Ok(n) => n.to_string().bright_white().to_string(),
            Err(e) => unknown(e),
        },
    );
    row(
        "Commits:",
        match commit_count() {
            Some(n) => n.to_string().bright_white().to_string(),
            None => unknown("git rev-list failed".to_string()),
        },
    );
    row(
        "Index:",
        match toolgen::check_index() {
            Ok(stale) if stale.is_empty() => "✅ tools/README.md matches every Cargo.toml"
                .bright_green()
                .to_string(),
            Ok(stale) => format!(
                "⚠  out of date: {} -- run: zero-docs sync",
                stale.join(", ")
            )
            .yellow()
            .to_string(),
            Err(e) => format!("⚠  {}", e).yellow().to_string(),
        },
    );
    println!();
}

fn cmd_help() {
    println!();
    println!("  {} {}", "📋".normal(), "zero-docs".bright_green().bold());
    println!("  {}", "Living Documentation Engine".dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();
    let cmds = [
        (
            "sync",
            "Write the tools index and the public docs, and verify the README links",
        ),
        (
            "check",
            "Show what is out of date (dry run); exit 1 if the index is stale",
        ),
        ("links", "Verify all README links resolve"),
        ("status", "Show doc sync status"),
        ("public", "Generate the curated public docs set"),
        (
            "readme-preview",
            "Show one tool's scaffold README (writes nothing)",
        ),
        ("readme-index", "Write the tools index"),
        (
            "readme-generate",
            "Scaffold missing tool READMEs; keep any a person owns",
        ),
    ];
    for (c, d) in &cmds {
        println!("  {:16} {}", c.bright_cyan(), d.dimmed());
    }
    println!();
}

#[cfg(test)]
mod hint_tests {
    // INT-296: a message that tells a reader to run a subcommand must name one the dispatch
    // accepts. INT-291 retired readme and swept the retired tool names, not these hints.

    /// Every word that starts a match arm of main(), read from this file's own text.
    fn dispatch_words() -> Vec<String> {
        let src = include_str!("main.rs");
        let start = src.find("match cmd {").expect("main() has no dispatch");
        let end = start
            + src[start..]
                .find("_ => cmd_help()")
                .expect("the dispatch has no fallback arm");
        let mut words = Vec::new();
        for line in src[start..end].lines() {
            let t = line.trim_start();
            if !t.starts_with('"') {
                continue;
            }
            if let Some(arrow) = t.find("=>") {
                for w in t[..arrow].split('"').skip(1).step_by(2) {
                    words.push(w.to_string());
                }
            }
        }
        words
    }

    #[test]
    fn every_hint_names_a_subcommand_the_dispatch_accepts() {
        let words = dispatch_words();
        for known in ["sync", "check", "status"] {
            assert!(
                words.iter().any(|w| w == known),
                "the dispatch reader lost {known}: {words:?}"
            );
        }
        let sources = [
            ("main.rs", include_str!("main.rs")),
            ("toolgen.rs", include_str!("toolgen.rs")),
        ];
        let mut bad = Vec::new();
        for (file, text) in sources {
            for (n, line) in text.lines().enumerate() {
                for lead in ["`zero-docs ", "run: zero-docs "] {
                    let mut rest = line;
                    while let Some(i) = rest.find(lead) {
                        rest = &rest[i + lead.len()..];
                        let word: String = rest
                            .chars()
                            .take_while(|c| c.is_ascii_lowercase() || *c == '-')
                            .collect();
                        if !word.is_empty() && !words.contains(&word) {
                            bad.push(format!("{file}:{}: zero-docs {word}", n + 1));
                        }
                    }
                }
            }
        }
        assert!(
            bad.is_empty(),
            "hints name a subcommand the dispatch does not accept: {bad:?}"
        );
    }
}
