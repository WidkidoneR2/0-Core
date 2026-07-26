// faelight-deadwood -- forest-native dead-code & orphan detector (INT-094).
// CARDINAL RULE: reports, never deletes. Every finding carries a confidence level.
// "Know what's dead before you cut -- and never cut what only looks dead."
// Phase 1: dead aliases, stale .bak files, dead keybinds.

use clap::Parser;
use colored::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(
    name = "faelight-deadwood",
    version,
    about = "Forest dead-code & orphan detector (reports, never deletes)"
)]
struct Cli {
    /// Only run one check: aliases, baks, keybinds
    #[arg(long)]
    only: Option<String>,
    /// Age in days above which .bak files are flagged (default 7)
    #[arg(long, default_value = "7")]
    bak_age: u64,
    /// Output a single summary line with counts (for the health dashboard)
    #[arg(long)]
    summary: bool,
    #[arg(long)]
    purge: bool,
    #[arg(long)]
    bulk: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    fn tag(self) -> ColoredString {
        match self {
            Confidence::High => "HIGH".red().bold(),
            Confidence::Medium => "MED ".yellow(),
            Confidence::Low => "LOW ".dimmed(),
        }
    }
}

/// A safe, structured removal action. ONLY the three provably-safe checks
/// (dead aliases, stale .bak files, dead keybinds) ever attach one. Every other
/// finding has action: None and is therefore unpurgeable BY DESIGN -- purge skips
/// any finding without an action. Scripts, ghost intents, registry orphans, and
/// Nix modules stay manual, always.
#[derive(Clone)]
enum PurgeAction {
    RemoveLine { file: PathBuf, exact: String }, // dead alias / dead keybind: remove one exact line
    DeleteFile { path: PathBuf },                // stale .bak file only
}
struct Finding {
    confidence: Confidence,
    detail: String,
    action: Option<PurgeAction>,
}

fn core_root() -> PathBuf {
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join("0-core"))
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn main() {
    let cli = Cli::parse();
    let root = core_root();
    if cli.purge {
        purge(&root, cli.bak_age, cli.bulk);
        return;
    }

    if cli.summary {
        let aliases = check_dead_aliases(&root).len();
        let baks = check_stale_baks(&root, cli.bak_age)
            .iter()
            .filter(|f| f.confidence != Confidence::Low)
            .count();
        let keybinds = check_dead_keybinds(&root).len();
        let registry = check_registry_orphans(&root).len();
        let scripts = check_orphaned_scripts(&root).len();
        let modules = check_orphaned_modules(&root).len();
        let total = aliases + baks + keybinds + registry + scripts + modules;
        // machine-readable single line: TOTAL|aliases|baks|keybinds|registry|scripts|modules
        println!("{total}|{aliases}|{baks}|{keybinds}|{registry}|{scripts}|{modules}");
        return;
    }

    println!(
        "{}",
        "Faelight Deadwood -- forest hygiene report".green().bold()
    );
    println!("{}", "-".repeat(56).dimmed());
    println!(
        "{}",
        "  Reports only -- never deletes. You decide every cut.".dimmed()
    );
    println!();
    let run = |name: &str| cli.only.as_deref().map(|o| o == name).unwrap_or(true);
    if run("aliases") {
        report("Dead aliases", check_dead_aliases(&root));
    }
    if run("baks") {
        report(
            &format!("Stale .bak files (>{} days)", cli.bak_age),
            check_stale_baks(&root, cli.bak_age),
        );
    }
    if run("keybinds") {
        report("Dead keybinds (mango)", check_dead_keybinds(&root));
    }
    if run("registry") {
        report(
            "Registry orphans (deployable, no binary)",
            check_registry_orphans(&root),
        );
    }
    if run("scripts") {
        report(
            "Orphaned scripts (referenced nowhere)",
            check_orphaned_scripts(&root),
        );
    }
    if run("modules") {
        report(
            "Orphaned Nix modules (imported by no host)",
            check_orphaned_modules(&root),
        );
    }
    if run("cmdword") {
        let (found, exempt) = check_command_word_derivations(&root);
        report(
            &format!("Command-word derivation candidates (INT-195) [{exempt} author-exempt]"),
            found,
        );
    }
    println!("{}", "-".repeat(56).dimmed());
    println!("{}", "  A healthy forest sheds dead wood.".dimmed());
}

fn report(title: &str, findings: Vec<Finding>) {
    if findings.is_empty() {
        println!("  [ok] {}: clean", title);
        println!();
        return;
    }
    println!("  {} ({} flagged)", title.bright_white(), findings.len());
    for f in &findings {
        println!("    [{}] {}", f.confidence.tag(), f.detail);
    }
    println!();
}

fn config_fsh(root: &Path) -> PathBuf {
    root.join("nix/home/dotfiles/faelight-shell/.config/faelight-shell/config.fsh")
}

const BUILTINS: &[&str] = &[
    "cd",
    "ls",
    "ll",
    "la",
    "pwd",
    "which",
    "find",
    "cistart",
    "cicomplete",
    "dc",
    "ds",
    "deploy",
    "d",
    "rebuild",
    "rebuild-safe",
    "rebuild-dry",
    "rebuild-check",
    "rollback",
    "update-flake",
    "friday",
    "intent",
    "intents",
    "project",
    "experiment",
    "vm",
    "fm",
    "fmd",
    "faelight-fm",
    "gc",
    "gp",
    "fg",
    "core",
    "fsh",
    "snapshot",
    "where",
    "fsearch",
    "patch",
    "edit",
    "run",
    "query",
    "history",
    "rewind",
    "git",
    "lazygit",
    "lg",
    "cargo",
    "rustc",
    "make",
    "nix",
    "echo",
    "cat",
    "grep",
    "sed",
    "awk",
    "head",
    "tail",
    "sort",
    "uniq",
    "wc",
    "tr",
    "cut",
    "xargs",
    "tee",
    "export",
    "source",
    "exit",
    "clear",
    "c",
    "sudo",
    "rm",
    "mv",
    "cp",
    "mkdir",
    "touch",
    "chmod",
    "chown",
    "kill",
    "ps",
    "top",
    "htop",
    "systemctl",
    "journalctl",
    "env",
    "ssh",
    "curl",
    "wget",
    "tar",
    "zip",
    "unzip",
    "nvim",
    "vim",
    "hx",
    "bat",
    "less",
    "more",
    "man",
    "date",
    "uname",
    "python3",
    "python",
    "dev",
    "delete",
    "del",
    "diff",
    "list",
    "cheat",
    "it",
    "gt",
    "db",
    "ade",
    "reload",
    "help",
    "h",
];

fn on_path(cmd: &str) -> bool {
    std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|dir| Path::new(&format!("{dir}/{cmd}")).exists())
}

fn check_dead_aliases(root: &Path) -> Vec<Finding> {
    let path = config_fsh(root);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut alias_names: HashSet<String> = HashSet::new();
    for line in text.lines() {
        if let Some((n, _)) = parse_alias(line) {
            alias_names.insert(n);
        }
    }
    let mut findings = Vec::new();
    for line in text.lines() {
        if line.contains("# deadwood: skip") {
            continue;
        }
        let (name, target) = match parse_alias(line) {
            Some(x) => x,
            None => continue,
        };
        let first = target.split_whitespace().next().unwrap_or("");
        if first.is_empty() {
            continue;
        }
        let first = first.trim_matches(|c| c == '"' || c == '\'');
        let live = BUILTINS.contains(&first)
            || alias_names.contains(first)
            || on_path(first)
            || first.starts_with('~')
            || first.starts_with('/')
            || first.starts_with('$')
            || first.contains('=');
        if !live {
            findings.push(Finding {
                confidence: Confidence::Medium,
                detail: format!(
                    "alias {} -> '{}' (target '{}' not found)",
                    name.bright_white(),
                    target.dimmed(),
                    first
                ),
                action: Some(PurgeAction::RemoveLine {
                    file: path.clone(),
                    exact: line.to_string(),
                }),
            });
        }
    }
    findings
}

fn parse_alias(line: &str) -> Option<(String, String)> {
    let l = line.trim();
    let rest = l.strip_prefix("alias ")?;
    let eq = rest.find('=')?;
    let name = rest[..eq].trim().to_string();
    let target = rest[eq + 1..]
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .to_string();
    if name.is_empty() {
        return None;
    }
    Some((name, target))
}

const BAK_PROTECT: &[&str] = &["regreet"];

fn check_stale_baks(root: &Path, age_days: u64) -> Vec<Finding> {
    let now = std::time::SystemTime::now();
    let mut findings = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        let name = entry.file_name().to_string_lossy();
        if !name.contains(".bak") {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let modified = match meta.modified() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let age = now
            .duration_since(modified)
            .map(|d| d.as_secs() / 86_400)
            .unwrap_or(0);
        if age < age_days {
            continue;
        }
        let lower = name.to_lowercase();
        let protected = BAK_PROTECT.iter().any(|k| lower.contains(k));
        let rel = p.strip_prefix(root).unwrap_or(p).display().to_string();
        if protected {
            findings.push(Finding {
                action: None,
                confidence: Confidence::Low,
                detail: format!("{} ({}d) -- PROTECTED (kept on purpose)", rel.dimmed(), age),
            });
        } else {
            findings.push(Finding {
                confidence: Confidence::High,
                detail: format!("{} ({}d old)", rel, age),
                action: Some(PurgeAction::DeleteFile {
                    path: p.to_path_buf(),
                }),
            });
        }
    }
    findings
}

fn check_dead_keybinds(root: &Path) -> Vec<Finding> {
    let candidates = [
        root.join("nix/home/dotfiles/mango/.config/mango/config.conf"),
        root.join("nix/home/dotfiles/mango/config.conf"),
    ];
    let path = match candidates.iter().find(|p| p.exists()) {
        Some(p) => p,
        None => return Vec::new(),
    };
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut findings = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with('#') || !l.starts_with("bind") {
            continue;
        }
        if l.contains("# deadwood: skip") {
            continue;
        }
        let cmd_field = match l.rsplit(',').next() {
            Some(c) => c.trim(),
            None => continue,
        };
        let first = cmd_field.split_whitespace().next().unwrap_or("");
        if first != "spawn" {
            continue;
        }
        let target = cmd_field
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .trim_matches(|c| c == '"' || c == '\'');
        if target.is_empty() {
            continue;
        }
        let live = BUILTINS.contains(&target)
            || on_path(target)
            || target.starts_with('~')
            || target.starts_with('/');
        if !live {
            findings.push(Finding {
                confidence: Confidence::Medium,
                detail: format!("bind -> spawn '{}' (not found)", target.bright_white()),
                action: Some(PurgeAction::RemoveLine {
                    file: path.to_path_buf(),
                    exact: line.to_string(),
                }),
            });
        }
    }
    findings
}

// ── Registry orphans ─────────────────────────────────────────────────────────
// A tool in registry/tools.toml marked deployable=true, retired=false, whose `name` binary
// isn't on PATH. (retired=true -> intentionally gone, skip.)

fn check_registry_orphans(root: &Path) -> Vec<Finding> {
    let path = root.join("registry/tools.toml");
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut findings = Vec::new();
    // Parse [[tool]] blocks by scanning name/deployable/retired per block.
    let mut name = String::new();
    let mut deployable = false;
    let mut retired = false;
    let flush = |name: &str, deployable: bool, retired: bool, findings: &mut Vec<Finding>| {
        if name.is_empty() || retired || !deployable {
            return;
        }
        if BUILTINS.contains(&name) || on_path(name) {
            return;
        }
        findings.push(Finding {
            action: None,
            confidence: Confidence::High,
            detail: format!(
                "tool {} (deployable, not retired) -- no binary on PATH",
                name.bright_white()
            ),
        });
    };
    for line in text.lines() {
        let l = line.trim();
        if l == "[[tool]]" {
            flush(&name, deployable, retired, &mut findings);
            name.clear();
            deployable = false;
            retired = false;
            continue;
        }
        if let Some(v) = l.strip_prefix("name") {
            name = v
                .trim_start_matches([' ', '='])
                .trim()
                .trim_matches('"')
                .to_string();
        } else if let Some(v) = l.strip_prefix("deployable") {
            deployable = v.contains("true");
        } else if let Some(v) = l.strip_prefix("retired") {
            retired = v.contains("true");
        }
    }
    flush(&name, deployable, retired, &mut findings); // last block
    findings
}

// ── Orphaned scripts ─────────────────────────────────────────────────────────
// A script in pkgs/faelight/scripts/ whose basename is referenced nowhere else in the repo
// (no alias, no config, no other source). MED -- could be called dynamically.

fn check_orphaned_scripts(root: &Path) -> Vec<Finding> {
    let scripts_dir = root.join("pkgs/faelight/scripts");
    let entries = match std::fs::read_dir(&scripts_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    // Build a corpus of all text in the repo (config + registry + rust src), once.
    let mut corpus = String::new();
    for dir in ["config", "registry", "rust-tools", "modules", "hosts"] {
        for entry in WalkDir::new(root.join(dir))
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let n = entry.file_name().to_string_lossy();
                if n.contains(".bak") {
                    continue;
                }
                if let Ok(t) = std::fs::read_to_string(entry.path()) {
                    corpus.push_str(&t);
                    corpus.push('\n');
                }
            }
        }
    }
    let mut findings = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains(".bak") {
            continue;
        }
        if !entry.path().is_file() {
            continue;
        }
        // Referenced if its basename appears anywhere in the corpus.
        if !corpus.contains(&name) {
            findings.push(Finding {
                action: None,
                confidence: Confidence::Medium,
                detail: format!(
                    "script {} -- referenced nowhere (may be run dynamically)",
                    name.bright_white()
                ),
            });
        }
    }
    findings
}

// ── Orphaned Nix modules ─────────────────────────────────────────────────────
// A modules/**/*.nix file whose filename is referenced in no host configuration's imports.
// Empty module files are flagged HIGH (definitely dead); others MED (config may have moved
// inline, leaving a stale module).

fn check_orphaned_modules(root: &Path) -> Vec<Finding> {
    // Gather all host config text (the import sites).
    let mut host_text = String::new();
    for entry in WalkDir::new(root.join("hosts"))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let n = entry.file_name().to_string_lossy();
            if n.ends_with(".nix") && !n.contains(".bak") {
                if let Ok(t) = std::fs::read_to_string(entry.path()) {
                    host_text.push_str(&t);
                    host_text.push('\n');
                }
            }
        }
    }
    // Also count flake.nix (modules can be wired there).
    if let Ok(t) = std::fs::read_to_string(root.join("flake.nix")) {
        host_text.push_str(&t);
    }

    let mut findings = Vec::new();
    for entry in WalkDir::new(root.join("modules"))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".nix") || name.contains(".bak") {
            continue;
        }
        // Referenced if the filename appears in any host/flake text.
        if host_text.contains(&name) {
            continue;
        }
        let rel = p.strip_prefix(root).unwrap_or(p).display().to_string();
        let empty = entry.metadata().map(|m| m.len() == 0).unwrap_or(false);
        if empty {
            findings.push(Finding {
                action: None,
                confidence: Confidence::High,
                detail: format!("{} -- EMPTY and imported by no host", rel.bright_white()),
            });
        } else {
            findings.push(Finding {
                action: None,
                confidence: Confidence::Medium,
                detail: format!(
                    "{} -- imported by no host (config may have moved inline)",
                    rel
                ),
            });
        }
    }
    findings
}

/// INT-195: execution-governing code must derive the command word only through
/// commands::command_word(). This is a SOURCE-structure check rather than a filesystem one --
/// the tool's charter now covers both.
///
/// RULE DETECTION is kept separate from DIAGNOSTIC RENDERING: the visitor answers only
/// "is this expression a prohibited derivation", and file:line formatting happens here.
/// Text patterns were tried first and missed three distinct classes -- rustfmt-wrapped method
/// chains, by-file scope exclusion, and alternate spellings such as splitn. An AST removes the
/// first and third as categories rather than patching them case by case.
/// A discovered derivation. `start_line` is the line where the candidate EXPRESSION begins,
/// not where the method token appears -- for a wrapped chain those differ, and the expression
/// start is the unit an author can attach a declaration to. Named rather than a bare tuple so
/// that distinction is hard to regress.
struct CommandWordCandidate {
    start_line: usize,
    method: String,
}

struct CommandWordVisitor {
    hits: Vec<CommandWordCandidate>,
}

impl<'ast> syn::visit::Visit<'ast> for CommandWordVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "next" {
            if let syn::Expr::MethodCall(inner) = node.receiver.as_ref() {
                let m = inner.method.to_string();
                // Generic .split() is NOT matched: it splits on ':', '=', ',', '::' and
                // predicates all over the codebase, and 21 of the first run's 36 findings came
                // from it -- almost none in this problem class. The signal is the whitespace
                // family, which is what a command-word derivation actually looks like.
                if matches!(
                    m.as_str(),
                    "split_whitespace" | "split_ascii_whitespace" | "splitn"
                ) {
                    // Report the start of the WHOLE derivation expression, not the inner
                    // method ident. For a wrapped chain the ident's line has only continuation
                    // lines above it, so an annotation would have to sit mid-chain where
                    // rustfmt may move it. node.span() covers the full receiver chain, which
                    // puts the reported line at the statement -- where a human writes the
                    // declaration anyway, and where the bounded window can see it.
                    use syn::spanned::Spanned;
                    self.hits.push(CommandWordCandidate {
                        start_line: node.span().start().line,
                        method: m,
                    });
                }
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

/// INT-195 phase B: resolve an author-declared exemption for a candidate.
///
/// DETECTION IS SYNTACTIC; EXEMPTION LOOKUP IS TEXTUAL. That asymmetry is deliberate, not a
/// relapse into text matching: this never searches for violations, it resolves metadata for a
/// candidate that already has a span. syn discards ordinary `//` comments, so the declaration
/// cannot be an AST node -- and that is a useful constraint, keeping "what Rust means" separate
/// from "what the project declares about that Rust" instead of overloading the compiler's
/// attribute system with project architecture.
///
/// The window is BOUNDED and ADJACENT. Scanning upward until some comment appears would let an
/// unrelated comment silently exempt code, which is the fragility annotations exist to remove.
/// A blank line is allowed through; any other non-comment line ends the window.
///
/// SAFETY PROPERTY: a missing or misplaced annotation yields a FALSE POSITIVE -- a visible
/// finding someone investigates. It can never silently erase a violation.
fn exemption_for(lines: &[&str], line: usize) -> Option<String> {
    const WINDOW: usize = 4;
    let hit = line.saturating_sub(1);
    let start = hit.saturating_sub(WINDOW);
    for raw in lines[start..hit].iter().rev() {
        let t = raw.trim();
        if let Some(rest) = t.strip_prefix("//") {
            if let Some(reason) = rest.trim().strip_prefix("deadwood: exempt") {
                // Require a TOKEN BOUNDARY, not merely a string prefix. Without this,
                // `deadwood: exempted` and `deadwood: exemptions` would both read as
                // exemptions. Harmless while the vocabulary is one directive wide, and
                // ambiguous the moment a second directive shares those eight characters --
                // so the namespace gets closed before it can be entered wrongly.
                if reason.is_empty()
                    || reason.starts_with(|c: char| c == '-' || c == ':' || c.is_whitespace())
                {
                    return Some(
                        reason
                            .trim_matches(|c: char| c == '-' || c == ':' || c.is_whitespace())
                            .to_string(),
                    );
                }
            }
            continue;
        }
        if t.is_empty() {
            continue;
        }
        break;
    }
    None
}

/// Returns (reportable findings, count of author-exempted sites). The exempt count is surfaced
/// in the report title on purpose: a silent exemption mechanism is how a check gets quietly
/// neutered, and the number makes "exempted our way to zero" visible.
fn check_command_word_derivations(root: &Path) -> (Vec<Finding>, usize) {
    // CLASSIFICATION, kept separate from discovery. The visitor answers only "is this a
    // whitespace-derived first token"; deciding whether that MATTERS is architectural knowledge
    // and lives here.
    //
    // TEMPORARY COARSE FILTER. INT-195 is defined by ROLE, not file. This list intentionally
    // OVER-APPROXIMATES the scope until per-function role annotations exist -- commands/mod.rs
    // holds the dispatcher AND every builtin body, so display-only builtins inside it are still
    // reported. Phase B does not "add suppressions"; it replaces this approximation with
    // author-declared architectural intent at the site that knows its own role.
    const IN_SCOPE: &[&str] = &[
        "main.rs",
        "commands/mod.rs",
        "exec.rs",
        "expand.rs",
        "safety_guard.rs",
        "db.rs",
    ];
    let mut findings = Vec::new();
    let mut exempted = 0usize;
    let src = root.join("faelight/rust-tools/faelight-shell/src");
    for entry in walkdir::WalkDir::new(&src)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(p) else {
            continue;
        };
        let Ok(ast) = syn::parse_file(&text) else {
            continue;
        };
        let rel = p.strip_prefix(root).unwrap_or(p).display().to_string();
        if !IN_SCOPE.iter().any(|f| rel.ends_with(f)) {
            continue;
        }
        let file_lines: Vec<&str> = text.lines().collect();
        let mut v = CommandWordVisitor { hits: Vec::new() };
        syn::visit::visit_file(&mut v, &ast);
        for c in v.hits {
            let (line, method) = (c.start_line, c.method);
            if exemption_for(&file_lines, line).is_some() {
                exempted += 1;
                continue;
            }
            findings.push(Finding {
                confidence: Confidence::High,
                // The checker knows only that a first token is derived from whitespace. Whether
                // that token IS a command word is the architectural judgement it cannot make --
                // several known candidates derive a shell name, an intent id, or a heredoc
                // delimiter. The wording claims exactly what the tool can see, no more.
                detail: format!("{rel}:{line} derives a whitespace first token via .{method}().next(), not routed through command_word()"),
                action: None,
            });
        }
    }
    // Rust macro bodies are not recursively analyzed. Findings apply to ordinary Rust syntax;
    // code embedded inside macro token streams is outside the current analysis. Verified against
    // the INT-195 census: four sites inside format!() are not reported, and all four are
    // string-building rather than execution-governing, so the boundary does not currently overlap
    // the rule. If an execution-path violation ever appears inside a macro, that is the evidence
    // that this boundary is too restrictive.
    (findings, exempted)
}

fn git_tree_clean(root: &Path) -> bool {
    match std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain"])
        .output()
    {
        Ok(o) => o.status.success() && o.stdout.is_empty(),
        Err(_) => false,
    }
}

fn purgeable(root: &Path, bak_age: u64) -> Vec<Finding> {
    let mut out = Vec::new();
    for f in check_dead_aliases(root) {
        if f.action.is_some() {
            out.push(f);
        }
    }
    for f in check_stale_baks(root, bak_age) {
        if f.action.is_some() {
            out.push(f);
        }
    }
    for f in check_dead_keybinds(root) {
        if f.action.is_some() {
            out.push(f);
        }
    }
    out
}

fn apply_action(action: &PurgeAction) -> Result<String, String> {
    match action {
        PurgeAction::RemoveLine { file, exact } => {
            let text = std::fs::read_to_string(file)
                .map_err(|e| format!("read {}: {}", file.display(), e))?;
            let lines: Vec<&str> = text.lines().collect();
            let hits: Vec<usize> = lines
                .iter()
                .enumerate()
                .filter(|(_, l)| **l == exact.as_str())
                .map(|(i, _)| i)
                .collect();
            if hits.len() != 1 {
                return Err(format!(
                    "line no longer matches exactly once ({} hits) in {} -- skipped for safety",
                    hits.len(),
                    file.display()
                ));
            }
            let idx = hits[0];
            let kept: Vec<&str> = lines
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != idx)
                .map(|(_, l)| *l)
                .collect();
            let mut new_text = kept.join("\n");
            if text.ends_with('\n') {
                new_text.push('\n');
            }
            std::fs::write(file, new_text)
                .map_err(|e| format!("write {}: {}", file.display(), e))?;
            Ok(format!("removed line in {}", file.display()))
        }
        PurgeAction::DeleteFile { path } => {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if !name.contains(".bak") {
                return Err(format!(
                    "refusing to delete non-.bak file {} -- skipped",
                    path.display()
                ));
            }
            std::fs::remove_file(path).map_err(|e| format!("delete {}: {}", path.display(), e))?;
            Ok(format!("deleted {}", path.display()))
        }
    }
}

fn read_line_prompt(prompt: &str) -> String {
    use std::io::Write;
    print!("{}", prompt);
    let _ = std::io::stdout().flush();
    let mut s = String::new();
    let _ = std::io::stdin().read_line(&mut s);
    s.trim().to_string()
}

fn purge(root: &Path, bak_age: u64, bulk: bool) {
    println!(
        "{}",
        "Faelight Deadwood -- purge (safe dead weight only)"
            .green()
            .bold()
    );
    println!("{}", "-".repeat(56).dimmed());
    println!(
        "{}",
        "  Purges ONLY: dead aliases, stale .bak files, dead keybinds.".dimmed()
    );
    println!(
        "{}",
        "  Scripts, ghost intents, registry, modules: never touched here.".dimmed()
    );
    println!();
    if !git_tree_clean(root) {
        println!(
            "  {}",
            "Refusing to purge: git tree is not clean.".red().bold()
        );
        println!(
            "  {}",
            "Commit or stash first, so every deletion is a reviewable diff.".dimmed()
        );
        return;
    }
    let items = purgeable(root, bak_age);
    if items.is_empty() {
        println!(
            "  {}",
            "Nothing safe to purge. The forest is tidy.".dimmed()
        );
        return;
    }
    let mut done = 0usize;
    let mut skipped = 0usize;
    if bulk {
        println!("  Manifest -- {} item(s) would be removed:", items.len());
        for (i, f) in items.iter().enumerate() {
            println!("    {}. [{}] {}", i + 1, f.confidence.tag(), f.detail);
        }
        println!();
        let phrase = format!("purge {}", items.len());
        let ans = read_line_prompt(&format!(
            "  Type '{}' to confirm, anything else to abort: ",
            phrase
        ));
        if ans != phrase {
            println!("  {}", "Aborted. Nothing removed.".dimmed());
            return;
        }
        for f in &items {
            if let Some(a) = &f.action {
                match apply_action(a) {
                    Ok(msg) => {
                        println!("    {} {}", "[done]".green(), msg);
                        done += 1;
                    }
                    Err(e) => {
                        println!("    {} {}", "[skip]".yellow(), e);
                        skipped += 1;
                    }
                }
            }
        }
    } else {
        for f in &items {
            println!("  [{}] {}", f.confidence.tag(), f.detail);
            let ans = read_line_prompt("    [d]elete / [s]kip / [q]uit (default skip): ");
            match ans.as_str() {
                "d" | "D" => {
                    if let Some(a) = &f.action {
                        match apply_action(a) {
                            Ok(msg) => {
                                println!("    {} {}", "[done]".green(), msg);
                                done += 1;
                            }
                            Err(e) => {
                                println!("    {} {}", "[skip]".yellow(), e);
                                skipped += 1;
                            }
                        }
                    }
                }
                "q" | "Q" => {
                    println!("    {}", "stopped.".dimmed());
                    break;
                }
                _ => {
                    println!("    {}", "skipped.".dimmed());
                    skipped += 1;
                }
            }
        }
    }
    println!();
    println!("  {} removed, {} skipped.", done, skipped);
    if done > 0 {
        println!(
            "  {}",
            "Review the git diff, then commit the cuts.".dimmed()
        );
    }
}
