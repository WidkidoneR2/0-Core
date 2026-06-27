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
