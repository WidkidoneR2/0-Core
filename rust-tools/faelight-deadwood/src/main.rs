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
#[command(name = "faelight-deadwood", version, about = "Forest dead-code & orphan detector (reports, never deletes)")]
struct Cli {
    /// Only run one check: aliases, baks, keybinds
    #[arg(long)]
    only: Option<String>,
    /// Age in days above which .bak files are flagged (default 7)
    #[arg(long, default_value = "7")]
    bak_age: u64,
}

#[derive(Clone, Copy, PartialEq)]
enum Confidence { High, Medium, Low }

impl Confidence {
    fn tag(self) -> ColoredString {
        match self {
            Confidence::High => "HIGH".red().bold(),
            Confidence::Medium => "MED ".yellow(),
            Confidence::Low => "LOW ".dimmed(),
        }
    }
}

struct Finding { confidence: Confidence, detail: String }

fn core_root() -> PathBuf {
    std::env::var("HOME").map(|h| PathBuf::from(h).join("0-core")).unwrap_or_else(|_| PathBuf::from("."))
}

fn main() {
    let cli = Cli::parse();
    let root = core_root();
    println!("{}", "Faelight Deadwood -- forest hygiene report".green().bold());
    println!("{}", "-".repeat(56).dimmed());
    println!("{}", "  Reports only -- never deletes. You decide every cut.".dimmed());
    println!();
    let run = |name: &str| cli.only.as_deref().map(|o| o == name).unwrap_or(true);
    if run("aliases") { report("Dead aliases", check_dead_aliases(&root)); }
    if run("baks") { report(&format!("Stale .bak files (>{} days)", cli.bak_age), check_stale_baks(&root, cli.bak_age)); }
    if run("keybinds") { report("Dead keybinds (mango)", check_dead_keybinds(&root)); }
    if run("registry") { report("Registry orphans (deployable, no binary)", check_registry_orphans(&root)); }
    if run("scripts") { report("Orphaned scripts (referenced nowhere)", check_orphaned_scripts(&root)); }
    if run("modules") { report("Orphaned Nix modules (imported by no host)", check_orphaned_modules(&root)); }
    if run("intents") { report("Dangling intent references (ghost INT-NNN)", check_dangling_intents(&root)); }
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
    root.join("config/faelight-shell/.config/faelight-shell/config.fsh")
}

const BUILTINS: &[&str] = &[
    "cd","ls","ll","la","pwd","which","find","cistart","cicomplete","dc","ds","deploy","d",
    "rebuild","rebuild-safe","rebuild-dry","rebuild-check","rollback","update-flake","friday",
    "intent","intents","project","experiment","vm","fm","fmd","faelight-fm","gc","gp","fg",
    "core","fsh","snapshot","where","fsearch","patch","edit","run","query","history","rewind",
    "git","lazygit","lg","cargo","rustc","make","nix","echo","cat","grep","sed","awk","head",
    "tail","sort","uniq","wc","tr","cut","xargs","tee","export","source","exit","clear","c",
    "sudo","rm","mv","cp","mkdir","touch","chmod","chown","kill","ps","top","htop","systemctl",
    "journalctl","env","ssh","curl","wget","tar","zip","unzip","nvim","vim","hx","bat","less",
    "more","man","date","uname","python3","python","dev","delete","del","diff","list","cheat",
    "it","gt","db","ade","reload","help","h",
];

fn on_path(cmd: &str) -> bool {
    std::env::var("PATH").unwrap_or_default().split(':')
        .any(|dir| Path::new(&format!("{dir}/{cmd}")).exists())
}

fn check_dead_aliases(root: &Path) -> Vec<Finding> {
    let path = config_fsh(root);
    let text = match std::fs::read_to_string(&path) { Ok(t) => t, Err(_) => return Vec::new() };
    let mut alias_names: HashSet<String> = HashSet::new();
    for line in text.lines() {
        if let Some((n, _)) = parse_alias(line) { alias_names.insert(n); }
    }
    let mut findings = Vec::new();
    for line in text.lines() {
        if line.contains("# deadwood: skip") { continue; }
        let (name, target) = match parse_alias(line) { Some(x) => x, None => continue };
        let first = target.split_whitespace().next().unwrap_or("");
        if first.is_empty() { continue; }
        let first = first.trim_matches(|c| c == '"' || c == '\'');
        let live = BUILTINS.contains(&first) || alias_names.contains(first) || on_path(first)
            || first.starts_with('~') || first.starts_with('/') || first.starts_with('$') || first.contains('=');
        if !live {
            findings.push(Finding {
                confidence: Confidence::Medium,
                detail: format!("alias {} -> '{}' (target '{}' not found)", name.bright_white(), target.dimmed(), first),
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
    let target = rest[eq+1..].trim().trim_matches(|c| c == '"' || c == '\'').to_string();
    if name.is_empty() { return None; }
    Some((name, target))
}

const BAK_PROTECT: &[&str] = &["regreet"];

fn check_stale_baks(root: &Path, age_days: u64) -> Vec<Finding> {
    let now = std::time::SystemTime::now();
    let mut findings = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        let name = entry.file_name().to_string_lossy();
        if !name.contains(".bak") { continue; }
        let meta = match entry.metadata() { Ok(m) => m, Err(_) => continue };
        let modified = match meta.modified() { Ok(m) => m, Err(_) => continue };
        let age = now.duration_since(modified).map(|d| d.as_secs() / 86_400).unwrap_or(0);
        if age < age_days { continue; }
        let lower = name.to_lowercase();
        let protected = BAK_PROTECT.iter().any(|k| lower.contains(k));
        let rel = p.strip_prefix(root).unwrap_or(p).display().to_string();
        if protected {
            findings.push(Finding { confidence: Confidence::Low, detail: format!("{} ({}d) -- PROTECTED (kept on purpose)", rel.dimmed(), age) });
        } else {
            findings.push(Finding { confidence: Confidence::High, detail: format!("{} ({}d old)", rel, age) });
        }
    }
    findings
}

fn check_dead_keybinds(root: &Path) -> Vec<Finding> {
    let candidates = [
        root.join("config/mango/.config/mango/config.conf"),
        root.join("config/mango/config.conf"),
    ];
    let path = match candidates.iter().find(|p| p.exists()) { Some(p) => p, None => return Vec::new() };
    let text = match std::fs::read_to_string(path) { Ok(t) => t, Err(_) => return Vec::new() };
    let mut findings = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with('#') || !l.starts_with("bind") { continue; }
        if l.contains("# deadwood: skip") { continue; }
        let cmd_field = match l.rsplit(',').next() { Some(c) => c.trim(), None => continue };
        let first = cmd_field.split_whitespace().next().unwrap_or("");
        if first != "spawn" { continue; }
        let target = cmd_field.split_whitespace().nth(1).unwrap_or("").trim_matches(|c| c == '"' || c == '\'');
        if target.is_empty() { continue; }
        let live = BUILTINS.contains(&target) || on_path(target) || target.starts_with('~') || target.starts_with('/');
        if !live {
            findings.push(Finding { confidence: Confidence::Medium, detail: format!("bind -> spawn '{}' (not found)", target.bright_white()) });
        }
    }
    findings
}


// ── Registry orphans ─────────────────────────────────────────────────────────
// A tool in registry/tools.toml marked deployable=true, retired=false, whose `name` binary
// isn't on PATH. (retired=true -> intentionally gone, skip.)

fn check_registry_orphans(root: &Path) -> Vec<Finding> {
    let path = root.join("registry/tools.toml");
    let text = match std::fs::read_to_string(&path) { Ok(t) => t, Err(_) => return Vec::new() };
    let mut findings = Vec::new();
    // Parse [[tool]] blocks by scanning name/deployable/retired per block.
    let mut name = String::new();
    let mut deployable = false;
    let mut retired = false;
    let flush = |name: &str, deployable: bool, retired: bool, findings: &mut Vec<Finding>| {
        if name.is_empty() || retired || !deployable { return; }
        if BUILTINS.contains(&name) || on_path(name) { return; }
        findings.push(Finding {
            confidence: Confidence::High,
            detail: format!("tool {} (deployable, not retired) -- no binary on PATH", name.bright_white()),
        });
    };
    for line in text.lines() {
        let l = line.trim();
        if l == "[[tool]]" {
            flush(&name, deployable, retired, &mut findings);
            name.clear(); deployable = false; retired = false;
            continue;
        }
        if let Some(v) = l.strip_prefix("name") {
            name = v.trim_start_matches([' ', '=']).trim().trim_matches('"').to_string();
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
    let entries = match std::fs::read_dir(&scripts_dir) { Ok(e) => e, Err(_) => return Vec::new() };
    // Build a corpus of all text in the repo (config + registry + rust src), once.
    let mut corpus = String::new();
    for dir in ["config", "registry", "rust-tools", "modules", "hosts"] {
        for entry in WalkDir::new(root.join(dir)).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let n = entry.file_name().to_string_lossy();
                if n.contains(".bak") { continue; }
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
        if name.contains(".bak") { continue; }
        if !entry.path().is_file() { continue; }
        // Referenced if its basename appears anywhere in the corpus.
        if !corpus.contains(&name) {
            findings.push(Finding {
                confidence: Confidence::Medium,
                detail: format!("script {} -- referenced nowhere (may be run dynamically)", name.bright_white()),
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
    for entry in WalkDir::new(root.join("hosts")).into_iter().filter_map(|e| e.ok()) {
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
    for entry in WalkDir::new(root.join("modules")).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if !entry.file_type().is_file() { continue; }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".nix") || name.contains(".bak") { continue; }
        // Referenced if the filename appears in any host/flake text.
        if host_text.contains(&name) { continue; }
        let rel = p.strip_prefix(root).unwrap_or(p).display().to_string();
        let empty = entry.metadata().map(|m| m.len() == 0).unwrap_or(false);
        if empty {
            findings.push(Finding {
                confidence: Confidence::High,
                detail: format!("{} -- EMPTY and imported by no host", rel.bright_white()),
            });
        } else {
            findings.push(Finding {
                confidence: Confidence::Medium,
                detail: format!("{} -- imported by no host (config may have moved inline)", rel),
            });
        }
    }
    findings
}


// ── Dangling intent references ───────────────────────────────────────────────
// An INT-NNN referenced in an intent file where no intent file with that number exists
// (the "ghost INT-260" class). Walks intents/, collects real intent numbers from filenames,
// then flags references to non-existent numbers. HIGH -- a dangling ref is a real doc error.

fn check_dangling_intents(root: &Path) -> Vec<Finding> {
    use once_cell::sync::Lazy;
    use regex::Regex;
    static INT_REF: Lazy<Regex> = Lazy::new(|| Regex::new(r"INT-(\d{3})").unwrap());
    static FILE_NUM: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d{3})-").unwrap());

    let intents_dir = root.join("intents");
    // Collect real intent numbers from filenames (NNN-*.md in any subdir).
    let mut real: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut ref_sites: Vec<(String, String)> = Vec::new(); // (int_num, file)
    for entry in WalkDir::new(&intents_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() { continue; }
        let fname = entry.file_name().to_string_lossy().to_string();
        if !fname.ends_with(".md") || fname.contains(".bak") { continue; }
        if let Some(c) = FILE_NUM.captures(&fname) {
            real.insert(c[1].to_string());
        }
        if let Ok(text) = std::fs::read_to_string(entry.path()) {
            for c in INT_REF.captures_iter(&text) {
                ref_sites.push((c[1].to_string(), fname.clone()));
            }
        }
    }
    // Flag references to numbers with no matching file. Dedup (num,file).
    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    let mut findings = Vec::new();
    for (num, file) in ref_sites {
        if real.contains(&num) { continue; }
        if !seen.insert((num.clone(), file.clone())) { continue; }
        findings.push(Finding {
            confidence: Confidence::High,
            detail: format!("INT-{} referenced in {} -- no such intent file", num.bright_white(), file.dimmed()),
        });
    }
    findings
}
