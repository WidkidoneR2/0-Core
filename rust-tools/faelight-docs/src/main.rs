// faelight-docs v1.0.0 — Living Documentation Engine
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
const VERSION: &str = "1.0.0";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "--health" | "health" => { println!("faelight-docs v{} — healthy", VERSION); }
        "sync"    => { cmd_welcome(false); cmd_readme(false); println!("  {} All docs synced", "✅".normal()); }
        "check"   => { cmd_welcome(true); cmd_readme(true); }
        "welcome" => cmd_welcome(false),
        "readme"  => cmd_readme(false),
        "preview" => { cmd_welcome(true); cmd_readme(true); }
        "status"  => cmd_status(),
        _         => cmd_help(),
    }
}

fn core_root() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join("0-core")
}

struct ForestState {
    version:        String,
    theme:          String,
    tool_count:     usize,
    intent_complete: usize,
    intent_planned: usize,
    commits:        String,
    health:         String,
    core_domains:   usize,
}

fn gather_state() -> ForestState {
    let root = core_root();

    let version = std::fs::read_to_string(root.join("00-meta/VERSION"))
        .unwrap_or_else(|_| "unknown".to_string())
        .trim().to_string();

    let changelog = std::fs::read_to_string(root.join("00-meta/CHANGELOG.md"))
        .unwrap_or_default();
    let theme = changelog.lines()
        .find(|l| l.contains(&format!("[{}]", version)))
        .and_then(|l| l.split(" — ").nth(1))
        .and_then(|s| s.split('(').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "The Living Forest".to_string());

    let tool_count = std::fs::read_dir(root.join("scripts"))
        .map(|d| d.flatten().filter(|e| e.path().is_file()).count())
        .unwrap_or(0);

    let intent_complete = std::fs::read_dir(root.join("intents/complete"))
        .map(|d| d.count()).unwrap_or(0);
    let intent_planned = std::fs::read_dir(root.join("intents/future"))
        .map(|d| d.count()).unwrap_or(0);

    let commits = std::fs::read_to_string("/etc/faelight/COMMITS")
        .unwrap_or_default().trim().to_string();

    let health = std::fs::read_to_string(root.join("runtime/cache/health.txt"))
        .unwrap_or_else(|_| "95".to_string())
        .trim().trim_end_matches('%').to_string();

    let core_domains = std::fs::read_dir(root.join("engine/src/domains"))
        .map(|d| d.flatten().filter(|e| e.path().is_dir()).count())
        .unwrap_or(0);

    ForestState { version, theme, tool_count, intent_complete, intent_planned,
                  commits, health, core_domains }
}

fn cmd_welcome(dry_run: bool) {
    let state = gather_state();
    let root = core_root();

    let zshrc_path = root.join("03-interfaces/stow/shell-zsh/.zshrc");
    let content = match std::fs::read_to_string(&zshrc_path) {
        Ok(c) => c,
        Err(e) => { eprintln!("  {} Cannot read .zshrc: {}", "✗".bright_red(), e); return; }
    };

    // Find and replace the welcome line
    let new_welcome = format!(
        "    echo -e \"\\033[1;32m🌲 Welcome to Faelight Forest v{} — {}\\033[0m\"",
        state.version, state.theme
    );

    let updated = content.lines().map(|line| {
        if line.contains("Welcome to Faelight Forest") {
            new_welcome.clone()
        } else {
            line.to_string()
        }
    }).collect::<Vec<_>>().join("\n");

    // Preserve trailing newline
    let updated = if content.ends_with('\n') { updated + "\n" } else { updated };

    if content == updated {
        println!("  {} Welcome message already up to date", "✅".normal());
        return;
    }

    if dry_run {
        println!("  {} Welcome message would update to:", "→".bright_cyan());
        println!("     {}", new_welcome.bright_white());
    } else {
        match std::fs::write(&zshrc_path, &updated) {
            Ok(_) => println!("  {} Welcome message updated → v{} — {}",
                "✅".normal(), state.version.bright_green(), state.theme.dimmed()),
            Err(e) => eprintln!("  {} Cannot write .zshrc: {}", "✗".bright_red(), e),
        }
    }
}

fn cmd_readme(dry_run: bool) {
    let root = core_root();
    let readme_path = root.join("README.md");

    let content = match std::fs::read_to_string(&readme_path) {
        Ok(c) => c,
        Err(e) => { eprintln!("  {} Cannot read README: {}", "✗".bright_red(), e); return; }
    };

    // Find boundary — NEVER touch above it
    let boundary_pos = match content.find(BOUNDARY_MARKER) {
        Some(p) => p + BOUNDARY_MARKER.len(),
        None => {
            eprintln!("  {} Boundary marker not found in README — aborting",
                "✗".bright_red());
            eprintln!("     Expected: {}", BOUNDARY_MARKER);
            return;
        }
    };

    let static_header = &content[..boundary_pos];
    let state = gather_state();
    let new_static = generate_static_section(&state);

    let new_content = format!("{}\n{}", static_header, new_static);

    if content == new_content {
        println!("  {} README static section already up to date", "✅".normal());
        return;
    }

    if dry_run {
        println!("  {} README static section would update:", "→".bright_cyan());
        println!("     {} tools: {}", "·".dimmed(), state.tool_count);
        println!("     {} intents complete: {}", "·".dimmed(), state.intent_complete);
        println!("     {} version: {}", "·".dimmed(), state.version);
    } else {
        match std::fs::write(&readme_path, &new_content) {
            Ok(_) => println!("  {} README static section updated",
                "✅".normal()),
            Err(e) => eprintln!("  {} Cannot write README: {}", "✗".bright_red(), e),
        }
    }
}

fn cmd_status() {
    let state = gather_state();
    let root = core_root();

    println!();
    println!("  {} faelight-docs status", "📋".normal());
    println!("{}", "  ─────────────────────────────────────".dimmed());
    println!("  {}  v{} — {}", "Version:".dimmed(),
        state.version.bright_green(), state.theme.dimmed());
    println!("  {}    {}", "Tools:".dimmed(),
        state.tool_count.to_string().bright_white());
    println!("  {}  {} complete, {} planned",
        "Intents:".dimmed(),
        state.intent_complete.to_string().bright_white(),
        state.intent_planned.to_string().dimmed());
    println!("  {}  {}", "Commits:".dimmed(),
        state.commits.bright_white());
    println!("  {}   {}%", "Health:".dimmed(),
        state.health.bright_green());
    println!("  {} {}", "Core domains:".dimmed(),
        state.core_domains.to_string().bright_white());
    println!();

    // Check welcome message
    let zshrc = std::fs::read_to_string(
        root.join("03-interfaces/stow/shell-zsh/.zshrc")
    ).unwrap_or_default();
    let welcome_ok = zshrc.contains(&format!("v{}", state.version));
    println!("  {}  {}", "Welcome msg:".dimmed(),
        if welcome_ok { "✅ up to date".bright_green().to_string() }
        else { "⚠  outdated — run: faelight-docs welcome".yellow().to_string() }
    );

    // Check README
    let readme = std::fs::read_to_string(root.join("README.md"))
        .unwrap_or_default();
    let readme_ok = readme.contains(&format!("v{}", state.version));
    println!("  {}      {}", "README:".dimmed(),
        if readme_ok { "✅ up to date".bright_green().to_string() }
        else { "⚠  may be outdated — run: faelight-docs readme".yellow().to_string() }
    );
    println!();
}

fn cmd_help() {
    println!();
    println!("  {} {}", "📋".normal(), "faelight-docs".bright_green().bold());
    println!("  {}", "Living Documentation Engine".dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();
    let cmds = [
        ("sync",    "Update all docs from forest state"),
        ("check",   "Show what is out of date (dry run)"),
        ("welcome", "Regenerate zshrc welcome message"),
        ("readme",  "Regenerate README static section"),
        ("preview", "Preview what would change"),
        ("status",  "Show doc sync status"),
    ];
    for (c, d) in &cmds {
        println!("  {:12} {}", c.bright_cyan(), d.dimmed());
    }
    println!();
    println!("  {} faelight-release calls faelight-docs sync automatically",
        "Note:".dimmed());
    println!("  {} README lines 1-37 are owned by faelight-release",
        "     ".dimmed());
    println!();
}

fn generate_static_section(state: &ForestState) -> String {
    let root = core_root();

    // Read journey from CHANGELOG
    let changelog = std::fs::read_to_string(root.join("00-meta/CHANGELOG.md"))
        .unwrap_or_default();
    let journey_entries: Vec<String> = changelog.lines()
        .filter(|l| l.starts_with("## ["))
        .take(8)
        .map(|l| {
            let version = l.split('[').nth(1).and_then(|s| s.split(']').next()).unwrap_or("?");
            let theme = l.split(" — ").nth(1).and_then(|s| s.split('(').next()).unwrap_or("").trim();
            format!("| v{} | {} |", version, theme)
        })
        .collect();

    format!(r#"
## 🤔 What is 0-Core?

**0-Core** is a completely custom Linux environment built on vanilla Arch Linux, where every component is understood, controlled, and intentionally chosen. Not a dotfiles collection — a **personal operating system built from scratch in Rust**.

### For Everyday Users

Like **building a custom motorcycle** instead of buying one from a dealer. You know every bolt, every wire, every piece.

**You get:**
- 🎨 Custom everything (terminal, bar, launcher, login screen, notifications, compositor)
- 🦀 {} Rust tools you fully understand
- 🛡️ Security through comprehension (no mystery packages)
- ⚡ Lightning fast (no bloat, no hidden automation)
- 💎 Complete ownership and control
- 🌲 A shell that knows it is a forest — and speaks to you

### For Technical People

- **`core` v2.0.0** — single orchestrator binary with {}+ native Rust domains
- **faelight-shell** — forest-native structured shell with SQL queries, joins, NL translation, scripting
- **faelight-notify v4.0.0** — D-Bus compliant notifications, fontdue renderer
- **faelight-vault v1.0.0** — forest-native credential manager, Argon2id encryption
- **faelight-compositor** — custom Wayland compositor, renders forest green on real DRM hardware
- **{} complete intents** — every architectural decision documented

---

## 🗺️ The Journey

| Version | Theme |
|---------|-------|
{}

---

## 🔒 Security Philosophy
```
Nothing runs without explicit human authorization.
Every change is intentional. Every tool is understood.
```

- UFW firewall + fail2ban
- faelight-vault — forest-native credential manager
- faelight-sandbox v3 — policy engine, namespace isolation, seccomp
- Immutable core (chattr +i) — cannot modify without explicit unlock
- 24-check health monitoring

---

## 🚀 Quick Start
```bash
git clone https://github.com/WidkidoneR2/0-Core.git ~/0-core
cd ~/0-core && cargo build --release --workspace
cp target/release/* scripts/
cd 03-interfaces/stow && stow */
core doctor run
```

---

*"The forest that speaks is the forest that connects."* 🌲

*Auto-generated by faelight-docs v{} — last sync: {}*
"#,
        state.tool_count,
        state.core_domains,
        state.intent_complete,
        journey_entries.join("\n"),
        VERSION,
        chrono::Local::now().format("%Y-%m-%d %H:%M")
    )
}
