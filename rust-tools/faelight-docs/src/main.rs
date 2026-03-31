// faelight-docs v2.0.0 — Living Documentation Engine
// INT-145 — Keeps README and welcome message in sync with forest state
//
// BOUNDARY RULE: faelight-release owns README lines 1-37 (dynamic section)
//                faelight-docs owns README lines 38+ (static section)
//                These two tools NEVER cross this boundary.
//
// Commands:
//   sync     — update all docs from forest state
//   check    — show what is out of date (dry run)
//   welcome  — regenerate zshrc welcome message only
//   readme   — regenerate README static section only
//   preview  — show what would change without writing
//   status   — what docs exist, last updated

use colored::*;
use std::path::PathBuf;

const BOUNDARY_MARKER: &str = "<!-- END DYNAMIC SECTION -->";
const VERSION: &str = "2.0.0";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "--health" | "health" => {
            println!("faelight-docs v{} — healthy", VERSION);
        }
        "sync" => {
            cmd_welcome(false);
            cmd_readme(false);
            verify_links(false);
            println!("  {} All docs synced", "✅".normal());
        }
        "check" => {
            cmd_welcome(true);
            cmd_readme(true);
            verify_links(true);
        }
        "verify-links" | "links" => {
            verify_links(false);
        }
        "welcome" => cmd_welcome(false),
        "readme" => cmd_readme(false),
        "preview" => {
            cmd_welcome(true);
            cmd_readme(true);
        }
        "status" => cmd_status(),
        _ => cmd_help(),
    }
}

fn core_root() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join("0-core")
}

struct ForestState {
    version: String,
    theme: String,
    tool_count: usize,
    intent_complete: usize,
    intent_planned: usize,
    commits: String,
    health: String,
    core_domains: usize,
}

fn gather_state() -> ForestState {
    let root = core_root();

    let version = std::fs::read_to_string(root.join("00-meta/VERSION"))
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    let changelog = std::fs::read_to_string(root.join("00-meta/CHANGELOG.md")).unwrap_or_default();
    let theme = changelog
        .lines()
        .find(|l| l.contains(&format!("[{}]", version)))
        .and_then(|l| l.split(" — ").nth(1))
        .and_then(|s| s.split('(').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "The Living Forest".to_string());

    // Count registered tools from tools.toml — matches doctor path resilience check
    let tool_count = std::fs::read_to_string(root.join("01-registry/tools.toml"))
        .map(|t| t.lines().filter(|l| l.starts_with("name = ")).count())
        .unwrap_or(0);

    // Count intents by scanning all categories — mirrors doctor check_intents logic
    let (intent_complete, intent_planned) = {
        let intent_dir = root.join("intents");
        let categories = ["complete", "decisions", "experiments", "philosophy",
                          "future", "cancelled", "deferred", "incidents", "active"];
        let mut complete = 0usize;
        let mut planned = 0usize;
        for cat in &categories {
            if let Ok(entries) = std::fs::read_dir(intent_dir.join(cat)) {
                for entry in entries.flatten() {
                    if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                        if let Ok(text) = std::fs::read_to_string(entry.path()) {
                            if text.contains("status: complete") { complete += 1; }
                            else if text.contains("status: planned") { planned += 1; }
                        }
                    }
                }
            }
        }
        (complete, planned)
    };

    let commits = std::fs::read_to_string("/etc/faelight/COMMITS")
        .unwrap_or_default()
        .trim()
        .to_string();

    // Read live health — prefer ~/.cache/faelight/health-status, fall back to state.db cache
    let health = {
        let home = std::env::var("HOME").unwrap_or_default();
        let cache_path = std::path::PathBuf::from(&home).join(".cache/faelight/health-status");
        std::fs::read_to_string(&cache_path)
            .or_else(|_| std::fs::read_to_string(root.join("runtime/cache/health.txt")))
            .unwrap_or_else(|_| "100".to_string())
            .trim()
            .trim_end_matches('%')
            .to_string()
    };

    let core_domains = std::fs::read_dir(root.join("engine/src/domains"))
        .map(|d| d.flatten().filter(|e| e.path().is_dir()).count())
        .unwrap_or(0);

    ForestState {
        version,
        theme,
        tool_count,
        intent_complete,
        intent_planned,
        commits,
        health,
        core_domains,
    }
}

fn cmd_welcome(dry_run: bool) {
    let state = gather_state();
    let root = core_root();

    let zshrc_path = root.join("03-interfaces/stow/shell-zsh/.zshrc");
    let content = match std::fs::read_to_string(&zshrc_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  {} Cannot read .zshrc: {}", "✗".bright_red(), e);
            return;
        }
    };

    // Find and replace the welcome line
    let new_welcome = format!(
        "    echo -e \"\\033[1;32m🌲 Welcome to Faelight Forest v{} — {}\\033[0m\"",
        state.version, state.theme
    );

    let updated = content
        .lines()
        .map(|line| {
            if line.contains("Welcome to Faelight Forest") {
                new_welcome.clone()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Preserve trailing newline
    let updated = if content.ends_with('\n') {
        updated + "\n"
    } else {
        updated
    };

    if content == updated {
        println!("  {} Welcome message already up to date", "✅".normal());
        return;
    }

    if dry_run {
        println!("  {} Welcome message would update to:", "→".bright_cyan());
        println!("     {}", new_welcome.bright_white());
    } else {
        match std::fs::write(&zshrc_path, &updated) {
            Ok(_) => println!(
                "  {} Welcome message updated → v{} — {}",
                "✅".normal(),
                state.version.bright_green(),
                state.theme.dimmed()
            ),
            Err(e) => eprintln!("  {} Cannot write .zshrc: {}", "✗".bright_red(), e),
        }
    }
}

fn cmd_readme(dry_run: bool) {
    let root = core_root();
    let readme_path = root.join("README.md");

    let content = match std::fs::read_to_string(&readme_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  {} Cannot read README: {}", "✗".bright_red(), e);
            return;
        }
    };

    // Find boundary — NEVER touch above it
    let boundary_pos = match content.find(BOUNDARY_MARKER) {
        Some(p) => p + BOUNDARY_MARKER.len(),
        None => {
            eprintln!(
                "  {} Boundary marker not found in README — aborting",
                "✗".bright_red()
            );
            return;
        }
    };

    let state = gather_state();

    // ── Targeted patch strategy ───────────────────────────────────────────────
    // Only patch specific live numbers — never replace prose or structure
    let header = content[..boundary_pos].to_string();
    let mut body = content[boundary_pos..].to_string();

    // 1. Tool count: "68 custom Rust tools"
    let tool_pattern = regex_replace_first(
        &body,
        r"\d+ custom Rust tools",
        &format!("{} custom Rust tools", state.tool_count),
    );
    body = tool_pattern;

    // 2. Domain count: "33+ native domains"
    let domain_pattern = regex_replace_first(
        &body,
        r"\d+\+ native domains",
        &format!("{}+ native domains", state.core_domains),
    );
    body = domain_pattern;

    // 3. Intent count: "104 complete intents"
    let intent_pattern = regex_replace_first(
        &body,
        r"\d+ complete intents",
        &format!("{} complete intents", state.intent_complete),
    );
    body = intent_pattern;

    // 4. Timestamp
    let timestamp = format!(
        "*Auto-generated by faelight-docs v{} — last sync: {}*",
        VERSION,
        chrono::Local::now().format("%Y-%m-%d %H:%M")
    );
    body = if let Some(start) = body.find("*Auto-generated by faelight-docs") {
        let end = body[start..].find('\n').map(|i| start + i).unwrap_or(body.len());
        format!("{}{}{}", &body[..start], timestamp, &body[end..])
    } else {
        format!("{}\n{}\n", body.trim_end(), timestamp)
    };

    let new_content = format!("{}{}", header, body);

    // Data-only diff (ignore timestamp line) — drives check/status
    let data_changed = strip_timestamp(&body) != strip_timestamp(&content[boundary_pos..]);

    // Full diff including timestamp — drives actual write
    if content == new_content {
        println!("  {} README already up to date", "✅".normal());
        return;
    }

    if dry_run {
        if data_changed {
            println!("  {} README would update:", "→".bright_cyan());
            println!("     tools: {}  domains: {}+  intents: {}  version: {}",
                state.tool_count.to_string().bright_white(),
                state.core_domains.to_string().bright_white(),
                state.intent_complete.to_string().bright_white(),
                state.version.bright_green(),
            );
        } else {
            println!("  {} README already up to date", "✅".normal());
        }
        return;
    } else {
        match std::fs::write(&readme_path, &new_content) {
            Ok(_) => println!(
                "  {} README updated — tools: {}  domains: {}+  intents: {}",
                "✅".normal(),
                state.tool_count.to_string().bright_white(),
                state.core_domains.to_string().bright_white(),
                state.intent_complete.to_string().bright_white(),
            ),
            Err(e) => eprintln!("  {} Cannot write README: {}", "✗".bright_red(), e),
        }
    }
}

// Strip auto-generated timestamp line for data-only comparisons
fn strip_timestamp(s: &str) -> String {
    s.lines()
        .filter(|l| !l.starts_with("*Auto-generated by faelight-docs"))
        .collect::<Vec<_>>()
        .join("\n")
}

// Simple regex-free pattern replacer for "digits + suffix" patterns
fn regex_replace_first(text: &str, pattern_kind: &str, replacement: &str) -> String {
    // Parse the pattern kind to find prefix digits and literal suffix
    let suffix = match pattern_kind {
        r"\d+ custom Rust tools"  => " custom Rust tools",
        r"\d+\+ native domains"  => "+ native domains",
        r"\d+ complete intents"   => " complete intents",
        _ => return text.to_string(),
    };
    // Find the suffix in text, walk back over preceding digits
    if let Some(suffix_pos) = text.find(suffix) {
        let before = &text[..suffix_pos];
        let digit_start = before
            .rfind(|c: char| !c.is_ascii_digit())
            .map(|i| i + 1)
            .unwrap_or(0);
        format!("{}{}{}", &text[..digit_start], replacement, &text[suffix_pos + suffix.len()..])
    } else {
        text.to_string()
    }
}

fn verify_links(dry_run: bool) -> bool {
    let root = core_root();
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
                if !link.starts_with("http") && !link.starts_with('#') && !link.starts_with("mailto") && !link.is_empty() {
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
            println!("  {} README links verified — all resolve correctly", "✅".normal());
        }
        return true;
    }

    for link in &broken {
        println!("  {} Broken link: {}", "✗".bright_red(), link.bright_white());
    }
    if dry_run {
        println!("  {} {} broken link(s) found — fix before release", "⚠️ ".normal(), broken.len());
    }
    false
}

fn cmd_status() {
    let state = gather_state();
    let root = core_root();

    println!();
    println!("  {} faelight-docs status", "📋".normal());
    println!("{}", "  ─────────────────────────────────────".dimmed());
    println!(
        "  {}  v{} — {}",
        "Version:".dimmed(),
        state.version.bright_green(),
        state.theme.dimmed()
    );
    println!(
        "  {}    {}",
        "Tools:".dimmed(),
        state.tool_count.to_string().bright_white()
    );
    println!(
        "  {}  {} complete, {} planned",
        "Intents:".dimmed(),
        state.intent_complete.to_string().bright_white(),
        state.intent_planned.to_string().dimmed()
    );
    println!(
        "  {}  {}",
        "Commits:".dimmed(),
        state.commits.bright_white()
    );
    println!(
        "  {}   {}%",
        "Health:".dimmed(),
        state.health.bright_green()
    );
    println!(
        "  {} {}",
        "Core domains:".dimmed(),
        state.core_domains.to_string().bright_white()
    );
    println!();

    // Check welcome message
    let zshrc = std::fs::read_to_string(root.join("03-interfaces/stow/shell-zsh/.zshrc"))
        .unwrap_or_default();
    let welcome_ok = zshrc.contains(&format!("v{}", state.version));
    println!(
        "  {}  {}",
        "Welcome msg:".dimmed(),
        if welcome_ok {
            "✅ up to date".bright_green().to_string()
        } else {
            "⚠  outdated — run: faelight-docs welcome"
                .yellow()
                .to_string()
        }
    );

    // Check README — use actual diff logic, not shallow version check
    let readme_ok = {
        let readme_path = root.join("README.md");
        if let Ok(content) = std::fs::read_to_string(&readme_path) {
            if let Some(boundary_pos) = content.find(BOUNDARY_MARKER).map(|p| p + BOUNDARY_MARKER.len()) {
                let mut body = content[boundary_pos..].to_string();
                body = regex_replace_first(&body, r"\d+ custom Rust tools", &format!("{} custom Rust tools", state.tool_count));
                body = regex_replace_first(&body, r"\d+\+ native domains", &format!("{}+ native domains", state.core_domains));
                body = regex_replace_first(&body, r"\d+ complete intents", &format!("{} complete intents", state.intent_complete));
                // Compare data only — ignore timestamp line
                let strip = |s: &str| s.lines()
                    .filter(|l| !l.starts_with("*Auto-generated by faelight-docs"))
                    .collect::<Vec<_>>().join("\n");
                strip(&content[boundary_pos..]) == strip(&body)
            } else { false }
        } else { false }
    };
    println!(
        "  {}      {}",
        "README:".dimmed(),
        if readme_ok {
            "✅ up to date".bright_green().to_string()
        } else {
            "⚠  numbers outdated — run: faelight-docs readme"
                .yellow()
                .to_string()
        }
    );
    println!();
}

fn cmd_help() {
    println!();
    println!(
        "  {} {}",
        "📋".normal(),
        "faelight-docs".bright_green().bold()
    );
    println!("  {}", "Living Documentation Engine".dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();
    let cmds = [
        ("sync", "Update all docs from forest state"),
        ("check", "Show what is out of date (dry run)"),
        ("welcome", "Regenerate zshrc welcome message"),
        ("readme", "Regenerate README static section"),
        ("preview", "Preview what would change"),
        ("links", "Verify all README links resolve"),
        ("status", "Show doc sync status"),
    ];
    for (c, d) in &cmds {
        println!("  {:12} {}", c.bright_cyan(), d.dimmed());
    }
    println!();
    println!(
        "  {} faelight-release calls faelight-docs sync automatically",
        "Note:".dimmed()
    );
    println!(
        "  {} README lines 1-37 are owned by faelight-release",
        "     ".dimmed()
    );
    println!();
}
