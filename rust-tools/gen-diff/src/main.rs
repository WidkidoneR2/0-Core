//! gen-diff -- Rich visual diff between NixOS generations
//! INT-044 Phase 1-3: list, package diff, forest context (commits + intents)
//! "Every rebuild is a checkpoint."
use std::process::Command;
use std::collections::BTreeSet;
use serde::Deserialize;
use colored::*;
use chrono::{NaiveDateTime, TimeZone, Local};
use clap::Parser;

const ARROW: &str = "→";
const EMPTY: &str = "∅";
const EPS:   &str = "ε";

#[derive(Parser)]
#[command(name = "gen-diff", about = "Rich visual diff between NixOS generations -- INT-044")]
struct Cli {
    /// Older generation (default: the one before current)
    a: Option<u64>,
    /// Newer generation (default: current)
    b: Option<u64>,
    /// List all generations instead of diffing
    #[arg(short, long)]
    list: bool,
    /// Show the N most recent generations (timeline)
    #[arg(long)]
    last: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct Generation {
    generation: u64,
    date: String,
    #[serde(rename = "nixosVersion")]
    nixos_version: String,
    #[serde(rename = "kernelVersion")]
    kernel_version: String,
    #[serde(rename = "configurationRevision")]
    configuration_revision: String,
    current: bool,
}

fn repo_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    format!("{}/0-core", home)
}

fn load_generations() -> Vec<Generation> {
    let out = Command::new("nixos-rebuild")
        .args(["list-generations", "--json"])
        .output()
        .expect("failed to run `nixos-rebuild list-generations --json`");
    let mut gens: Vec<Generation> =
        serde_json::from_slice(&out.stdout).expect("failed to parse list-generations JSON");
    gens.sort_by(|a, b| b.generation.cmp(&a.generation));
    gens
}

fn load_commits() -> Vec<(String, i64)> {
    let mut commits = Vec::new();
    if let Ok(out) = Command::new("git")
        .args(["-C", &repo_dir(), "log", "--format=%H|%ct"])
        .output()
    {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if let Some((hash, ct)) = line.split_once('|') {
                if let Ok(ct) = ct.trim().parse::<i64>() {
                    commits.push((hash.chars().take(12).collect(), ct));
                }
            }
        }
    }
    commits
}

fn match_commit(gen_date: &str, commits: &[(String, i64)]) -> Option<String> {
    let naive = NaiveDateTime::parse_from_str(gen_date, "%Y-%m-%d %H:%M:%S").ok()?;
    let epoch = Local.from_local_datetime(&naive).single()?.timestamp();
    commits.iter().find(|(_, ct)| *ct <= epoch).map(|(h, _)| h.clone())
}

fn commit_for(g: &Generation, commits: &[(String, i64)]) -> String {
    if g.configuration_revision != "Unknown" {
        g.configuration_revision.chars().take(12).collect()
    } else {
        match_commit(&g.date, commits).unwrap_or_else(|| "-".into())
    }
}

/// Commits + completed intents between two attributed commits (older..newer), from git.
/// intent_commits is complete again (INT-071 restored recording + backfilled the migration gap),
/// but git is retained as the source here: it is always present and needs no DB. The table is the
/// canonical record; gen-diff intentionally re-derives from git so it works even on a fresh DB.
fn forest_context(older: &str, newer: &str) -> (usize, Vec<u32>) {
    let repo = repo_dir();
    let range = format!("{}..{}", older, newer);
    let ncommits = Command::new("git")
        .args(["-C", &repo, "rev-list", "--count", &range])
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<usize>().ok())
        .unwrap_or(0);

    let mut ids = BTreeSet::new();
    if let Ok(out) = Command::new("git")
        .args([
            "-C", &repo, "log", "--no-renames", "--diff-filter=A",
            "--name-only", "--pretty=format:", &range, "--", "intents/complete/", "faelight/intents/complete/",
        ])
        .output()
    {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("faelight/intents/complete/")
                .or_else(|| trimmed.strip_prefix("intents/complete/")) {
                let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(n) = num.parse::<u32>() {
                    ids.insert(n);
                }
            }
        }
    }
    (ncommits, ids.into_iter().collect())
}

fn list_generations(gens: &[Generation], commits: &[(String, i64)]) {
    let header = format!("{:>5}  {:<19}  {:<24}  {:<9}  {:<12}",
        "GEN", "DATE", "NIXOS", "KERNEL", "COMMIT");
    println!("{}", header.truecolor(0x78, 0x8C, 0x82));
    for g in gens {
        let commit = commit_for(g, commits);
        let mut line = format!("{:>5}  {:<19}  {:<24}  {:<9}  {:<12}",
            g.generation, g.date, g.nixos_version, g.kernel_version, commit);
        if g.current {
            line.push_str("  <- current");
            println!("{}", line.truecolor(0x39, 0xFF, 0x14).bold());
        } else {
            println!("{}", line.truecolor(0xD7, 0xE0, 0xDA));
        }
    }
    eprintln!("{}", format!("{} generations", gens.len()).truecolor(0x32, 0xDC, 0xFF));
}

enum Change {
    Added { name: String, ver: String },
    Removed { name: String, ver: String },
    Changed { name: String, old: String, new: String },
}

fn parse_diff(raw: &str) -> (Vec<Change>, usize, usize) {
    let mut changes = Vec::new();
    let mut system = 0usize;
    let mut resized = 0usize;
    let sep = format!(" {} ", ARROW);
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let (name, rest) = match line.split_once(": ") {
            Some(x) => x,
            None => continue,
        };
        if let Some((before, after)) = rest.split_once(&sep) {
            let after_ver = after.split(", ").next().unwrap_or(after).trim();
            let before = before.trim();
            if before == EPS || after_ver == EPS {
                system += 1;
                continue;
            }
            if before == EMPTY {
                changes.push(Change::Added { name: name.into(), ver: after_ver.into() });
            } else if after_ver == EMPTY {
                changes.push(Change::Removed { name: name.into(), ver: before.into() });
            } else {
                changes.push(Change::Changed { name: name.into(), old: before.into(), new: after_ver.into() });
            }
        } else {
            resized += 1;
        }
    }
    (changes, system, resized)
}

fn date_of(g: Option<&Generation>) -> &str {
    g.map(|x| x.date.as_str()).unwrap_or("?")
}

fn diff_generations(a: u64, b: u64, gens: &[Generation], commits: &[(String, i64)]) {
    let find = |n: u64| gens.iter().find(|g| g.generation == n);
    let (da, db) = (find(a), find(b));
    let pa = format!("/nix/var/nix/profiles/system-{}-link", a);
    let pb = format!("/nix/var/nix/profiles/system-{}-link", b);
    let out = Command::new("nix")
        .args(["store", "diff-closures", &pa, &pb])
        .output()
        .expect("failed to run `nix store diff-closures`");
    if !out.status.success() {
        eprintln!("{}", format!("diff-closures failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()).truecolor(0xFF, 0x50, 0x50));
        std::process::exit(1);
    }
    let raw = String::from_utf8_lossy(&out.stdout);
    let (changes, system, resized) = parse_diff(&raw);

    let head = format!("gen {} ({})  {}  gen {} ({})",
        a, date_of(da), ARROW, b, date_of(db));
    println!("{}", head.truecolor(0x32, 0xDC, 0xFF).bold());
    println!();

    let (mut added, mut removed, mut changed) = (0usize, 0usize, 0usize);
    for c in &changes {
        if let Change::Changed { name, old, new } = c {
            changed += 1;
            println!("{}", format!("  ~ {}  {} {} {}", name, old, ARROW, new)
                .truecolor(0xFF, 0xC8, 0x32));
        }
    }
    for c in &changes {
        if let Change::Added { name, ver } = c {
            added += 1;
            println!("{}", format!("  + {}  {}", name, ver).truecolor(0x39, 0xFF, 0x14));
        }
    }
    for c in &changes {
        if let Change::Removed { name, ver } = c {
            removed += 1;
            println!("{}", format!("  - {}  {}", name, ver).truecolor(0xFF, 0x50, 0x50));
        }
    }

    println!();
    let summary = format!(
        "{} changed  {} added  {} removed   ({} system/config, {} resized)",
        changed, added, removed, system, resized
    );
    println!("{}", summary.truecolor(0x78, 0x8C, 0x82));

    let ca = da.map(|g| commit_for(g, commits));
    let cb = db.map(|g| commit_for(g, commits));
    if let (Some(ca), Some(cb)) = (ca, cb) {
        if ca != "-" && cb != "-" {
            let (ncommits, intents) = forest_context(&ca, &cb);
            let mut ctx = format!("forest: {} commits", ncommits);
            if intents.is_empty() {
                ctx.push_str(", 0 intents completed");
            } else {
                let ids: Vec<String> = intents.iter().map(|i| format!("INT-{:03}", i)).collect();
                ctx.push_str(&format!(", {} intents completed: {}", intents.len(), ids.join(", ")));
            }
            println!("{}", ctx.truecolor(0xB4, 0x82, 0xFF));
        }
    }
}

fn main() {
    // Behave like a normal Unix tool when piped to head/less:
    // exit on SIGPIPE instead of panicking on a broken-pipe write.
    #[cfg(unix)]
    unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL); }

    let cli = Cli::parse();
    let gens = load_generations();
    let commits = load_commits();

    if let Some(n) = cli.last {
        let n = n.min(gens.len());
        list_generations(&gens[..n], &commits);
        return;
    }

    if cli.list {
        list_generations(&gens, &commits);
        return;
    }

    let current = gens.first().map(|g| g.generation);
    let previous = gens.get(1).map(|g| g.generation);
    let (a, b) = match (cli.a, cli.b) {
        (Some(a), Some(b)) => (a, b),
        (Some(a), None) => (a, current.unwrap_or(a)),
        (None, _) => (previous.unwrap_or(0), current.unwrap_or(0)),
    };
    diff_generations(a, b, &gens, &commits);
}
