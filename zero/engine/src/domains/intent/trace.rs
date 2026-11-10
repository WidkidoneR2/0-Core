//! INT-266: trace -- the questions asked of finding records and of git. Ruled by Christian,
//! 2026-10-04: findings.rs owns the records; this file owns the queries over them.
//!
//! Every git read here goes through git_read. It answers one of three things: git did not
//! start, git started and refused (its exit and stderr), or git's text. A failure never reads
//! as an empty answer (INT-192), so Ok("") is a true none.

use super::findings::Record;
use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::{CoreError, CoreResult};
use colored::*;
use std::io::Write;
use std::process::{Command, Stdio};

/// Read from git in `core_root`. Err when git could not run or exited non-zero; Ok is git's
/// stdout, untrimmed.
pub fn git_read(core_root: &str, args: &[&str]) -> Result<String, String> {
    run("git", core_root, args)
}

/// Text from the one spawn below, decoded as git_read always has.
fn run(program: &str, core_root: &str, args: &[&str]) -> Result<String, String> {
    run_bytes(program, core_root, args).map(|out| String::from_utf8_lossy(&out).into_owned())
}

/// The one place trace spawns a process. The program is a parameter so a test can prove the
/// could-not-run answer without touching the real git. INT-274: stdout comes back as bytes,
/// because a seal is a hash of exact bytes and a lossy decode could change them.
fn run_bytes(program: &str, core_root: &str, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new(program)
        .arg("-C")
        .arg(core_root)
        .args(args)
        .output()
        .map_err(|e| format!("{} could not run: {}", program, e))?;
    if !out.status.success() {
        let exit = match out.status.code() {
            Some(code) => format!("exited {}", code),
            None => "was stopped by a signal".to_string(),
        };
        return Err(format!(
            "{} {} {}: {}",
            program,
            args.first().copied().unwrap_or(""),
            exit,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}

/// Every finding id a Fixes: trailer names on the current branch, with the commit that named it.
/// Err when git could not be read.
pub fn fixes(core_root: &str) -> Result<Vec<(String, String)>, String> {
    trailers(core_root).map(|t| t.named)
}

/// Every Fixes: value on the current branch, named or not, read once. Err when git could not be
/// read.
pub fn trailers(core_root: &str) -> Result<Trailers, String> {
    let text = git_read(
        core_root,
        &[
            "log",
            "--format=%h%x09%(trailers:key=Fixes,valueonly,separator=%x2C)",
            "HEAD",
        ],
    )?;
    Ok(split_trailers(&text))
}

/// A finding id as ruled 2026-10-04: F- and exactly four digits, nothing after. INT-300 R4: a
/// Fixes value that is anything else names no finding, so prose after an id counts for nothing.
fn is_finding_id(s: &str) -> bool {
    s.len() == 6 && s.starts_with("F-") && s[2..].bytes().all(|b| b.is_ascii_digit())
}

/// One line per commit: the hash, a TAB, then the ids its Fixes: trailers name, comma-separated.
fn parse_fixes(text: &str) -> Vec<(String, String)> {
    split_trailers(text).named
}

/// The Fixes: values of every commit on the branch, split once by the one reader of them
/// (INT-300 G4, G8). `named` holds each finding id a value names, with its commit. `other` holds
/// each value that names no finding (prose, a short id), with its commit, so `core bug check`
/// can say so instead of the value vanishing.
#[derive(Debug, Default, PartialEq)]
pub struct Trailers {
    pub named: Vec<(String, String)>,
    pub other: Vec<(String, String)>,
}

/// Split the log text once: per line the hash, a TAB, then the values of its Fixes: trailers,
/// comma-separated. A value that is exactly F-NNNN is named. Any other non-empty value is kept in
/// `other` with its commit: never dropped silently and never counted as a fix (R4).
fn split_trailers(text: &str) -> Trailers {
    let mut got = Trailers::default();
    for line in text.lines() {
        if let Some((hash, values)) = line.split_once('\t') {
            for value in values.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                let pair = (value.to_string(), hash.to_string());
                if is_finding_id(value) {
                    got.named.push(pair);
                } else {
                    got.other.push(pair);
                }
            }
        }
    }
    got
}

/// A finding's state, derived when asked and never stored (decision 142).
#[derive(Debug, PartialEq)]
pub enum State {
    /// A Fixes: trailer names it; the commits that do.
    Fixed(Vec<String>),
    /// The record says it was closed without a fix, and why.
    Closed(String),
    /// Git was read and nothing fixes it.
    Open,
    /// Git could not be read, so whether it is open is not known.
    Unread(String),
}

/// INT-300 R3: a fix wins over a closure. A commit that names the finding is the stronger fact, so
/// the finding reads Fixed even when its record says closed. Closed is a stored fact: when git
/// cannot be read it still reads Closed, and Open needs git to have answered.
pub fn state_of(record: &Record, fixes: &Result<Vec<(String, String)>, String>) -> State {
    let closed = record.closed_without_fix.as_str();
    let is_closed = closed != "no" && closed != "unknown";
    let named = match fixes {
        Ok(named) => named,
        Err(e) => {
            return if is_closed {
                State::Closed(closed.to_string())
            } else {
                State::Unread(e.clone())
            };
        }
    };
    let by: Vec<String> = named
        .iter()
        .filter(|(id, _)| id == &record.id)
        .map(|(_, hash)| hash.clone())
        .collect();
    if !by.is_empty() {
        State::Fixed(by)
    } else if is_closed {
        State::Closed(closed.to_string())
    } else {
        State::Open
    }
}

fn could_not(message: String) -> CoreError {
    CoreError::Domain {
        domain: "intent".to_string(),
        message: format!("trace could not answer -- {}", message),
    }
}

/// `core intent trace INT-x` -- every finding INT-x found, and whether each is still open.
/// F-x goes to trace_finding (question b), a commit to trace_commit and a seal to trace_seal (question c).
pub fn trace(ctx: &AppContext, target: &str) -> CoreResult<()> {
    ctx.capabilities.require(
        "intent",
        &[Capability::FilesystemReadHome, Capability::SpawnProcess],
    )?;
    let t = target.trim();
    let id = match target_kind(t) {
        Target::Intent(id) => id,
        Target::Finding(f) => return trace_finding(ctx, &f),
        Target::Commit(c) => return trace_commit(ctx, &c),
        Target::SealOrCommit(s) => return trace_seal(ctx, &s),
        Target::Unknown => {
            return Err(could_not(format!(
                "{} is not an intent (INT-x), a finding (F-x), a commit (7 to 40 hex) or a seal (16 hex)",
                t
            )));
        }
    };
    if !super::load_all(ctx).iter().any(|i| i.id == id) {
        return Err(could_not(format!(
            "INT-{} names no intent in the ledger",
            id
        )));
    }
    let label = format!("INT-{}", id);
    let dir = super::intents_dir(ctx).join("findings");
    let records = super::findings::read_records(&dir).map_err(could_not)?;
    let found: Vec<&Record> = records.iter().filter(|r| r.found_by == label).collect();
    let fixed_by = fixes(&ctx.core_root);
    let states: Vec<State> = found.iter().map(|r| state_of(r, &fixed_by)).collect();
    println!(
        "  {} found {} finding{}",
        label.bright_white(),
        found.len(),
        if found.len() == 1 { "" } else { "s" }
    );
    for (r, s) in found.iter().zip(&states) {
        let state = match s {
            State::Open => "open".yellow().to_string(),
            State::Fixed(by) => format!("fixed by {}", by.join(", ")).green().to_string(),
            State::Closed(why) => format!("closed without a fix: {}", why)
                .dimmed()
                .to_string(),
            State::Unread(_) => "not known".red().to_string(),
        };
        println!("     {}  {}  {}", r.id.bright_white(), state, r.what);
        println!("             at {}", r.at.dimmed());
    }
    match &fixed_by {
        Ok(_) => {
            let open = states.iter().filter(|s| **s == State::Open).count();
            println!("     {} open", open);
            Ok(())
        }
        Err(e) => Err(could_not(format!(
            "git could not be read, so no finding above is called open: {}",
            e
        ))),
    }
}

/// The first 16 hex of sha256 over the plan text -- the Seal rule (Christian, 2026-10-04).
pub fn seal_digest(text: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(text)
        .iter()
        .take(8)
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// What the seal on one commit says, re-checked. A commit from before the Seal rule answers as
/// predating it, never as empty (INT-192).
#[derive(Debug, PartialEq)]
pub enum SealCheck {
    /// The plan text kept in refs/notes/seals hashes to the Seal trailer.
    Intact(String),
    /// The kept plan text hashes to something else.
    Broken { seal: String, got: String },
    /// A Seal trailer, but no plan text kept for this commit.
    NoNote(String),
    /// No Seal trailer; a Fingerprint trailer from before the Seal rule. Not re-checkable.
    Legacy(String),
    /// Neither trailer.
    NoSeal,
}

/// Decide the seal from the commit's trailers and the note kept for it.
pub fn check_seal(seal: Option<&str>, fingerprint: Option<&str>, note: Option<&[u8]>) -> SealCheck {
    match seal {
        Some(s) => match note {
            Some(text) => {
                let got = seal_digest(text);
                if got == s {
                    SealCheck::Intact(s.to_string())
                } else {
                    SealCheck::Broken {
                        seal: s.to_string(),
                        got,
                    }
                }
            }
            None => SealCheck::NoNote(s.to_string()),
        },
        None => match fingerprint {
            Some(f) => SealCheck::Legacy(f.to_string()),
            None => SealCheck::NoSeal,
        },
    }
}

/// One trailer's values on one commit, comma-joined. None when the commit has no such trailer.
fn trailer(core_root: &str, hash: &str, key: &str) -> Result<Option<String>, String> {
    let format = format!("--format=%(trailers:key={},valueonly,separator=%x2C)", key);
    let text = git_read(core_root, &["log", "-1", &format, hash])?;
    let value = text.trim();
    Ok(if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    })
}

/// The note blob kept for a commit, from git notes --ref=seals list: the blob, a space, the full
/// commit hash. `hash` may be abbreviated.
fn note_for<'a>(list: &'a str, hash: &str) -> Option<&'a str> {
    list.lines().find_map(|line| {
        let (blob, commit) = line.split_once(' ')?;
        if commit.trim().starts_with(hash) {
            Some(blob)
        } else {
            None
        }
    })
}

/// `core intent trace F-x` -- the finding, and for each commit whose Fixes: names it, that commit
/// and the seal of the plan that produced it, re-checked against refs/notes/seals.
fn trace_finding(ctx: &AppContext, id: &str) -> CoreResult<()> {
    let dir = super::intents_dir(ctx).join("findings");
    let records = super::findings::read_records(&dir).map_err(could_not)?;
    let record = match records.iter().find(|r| r.id == id) {
        Some(r) => r,
        None => {
            return Err(could_not(format!(
                "{} names no finding in {}",
                id,
                dir.display()
            )));
        }
    };
    println!("  {}  {}", record.id.bright_white(), record.what);
    println!("          at {}", record.at.dimmed());
    println!(
        "          found by {} on {}",
        record.found_by, record.found_on
    );
    let core: &str = &ctx.core_root;
    let fixed_by = fixes(core);
    match state_of(record, &fixed_by) {
        State::Open => {
            println!(
                "     {} -- no commit on HEAD carries Fixes: {}",
                "open".yellow(),
                id
            );
            Ok(())
        }
        State::Closed(why) => {
            println!("     {}: {}", "closed without a fix".dimmed(), why);
            Ok(())
        }
        State::Unread(e) => Err(could_not(format!(
            "git could not be read, so whether {} is fixed is not known: {}",
            id, e
        ))),
        State::Fixed(hashes) => {
            let notes = seal_notes(core)
                .map_err(|e| could_not(format!("the seal notes could not be read: {}", e)))?;
            for hash in &hashes {
                let subject = git_read(core, &["log", "-1", "--format=%s", hash.as_str()])
                    .map_err(could_not)?;
                let check = check_commit_seal(core, &notes, hash).map_err(could_not)?;
                println!(
                    "     {} {}  {}",
                    "fixed by".green(),
                    hash.bright_white(),
                    subject.trim()
                );
                println!("          {}", seal_line(check));
            }
            Ok(())
        }
    }
}

/// How a trace target reads, by its shape alone.
#[derive(Debug, PartialEq)]
pub enum Target {
    /// INT-x, or bare digits shorter than a commit hash.
    Intent(String),
    /// F-x.
    Finding(String),
    /// 16 hex: a seal if a commit on HEAD carries it, else tried as a commit.
    SealOrCommit(String),
    /// 7 to 40 hex.
    Commit(String),
    /// None of the above.
    Unknown,
}

pub fn target_kind(t: &str) -> Target {
    let t = t.trim();
    if t.starts_with("F-") {
        return Target::Finding(t.to_string());
    }
    let id = t.strip_prefix("INT-").unwrap_or(t);
    let digits = !id.is_empty() && id.chars().all(|c| c.is_ascii_digit());
    if digits && (t.starts_with("INT-") || id.len() < 7) {
        return Target::Intent(id.to_string());
    }
    let hex = !t.is_empty() && t.chars().all(|c| c.is_ascii_hexdigit());
    if hex && t.len() == 16 {
        return Target::SealOrCommit(t.to_lowercase());
    }
    if hex && (7..=40).contains(&t.len()) {
        return Target::Commit(t.to_string());
    }
    Target::Unknown
}

/// A trailer list as a reader writes it: one space after each comma. git joins values with a bare
/// comma.
fn spaced(list: &str) -> String {
    list.split(',')
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(", ")
}

/// One line saying what a commit's seal is, re-checked.
fn seal_line(check: SealCheck) -> String {
    match check {
        SealCheck::Intact(s) => format!(
            "seal {} intact -- the plan text kept in refs/notes/seals hashes to it",
            s
        )
        .green()
        .to_string(),
        SealCheck::Broken { seal, got } => format!(
            "seal {} BROKEN -- the plan text kept in refs/notes/seals hashes to {}",
            seal, got
        )
        .red()
        .to_string(),
        SealCheck::NoNote(s) => format!(
            "seal {} recorded, but no plan text is kept in refs/notes/seals, so it cannot be re-checked",
            s
        )
        .yellow()
        .to_string(),
        SealCheck::Legacy(f) => {
            format!("predates the Seal rule: Fingerprint {}, not re-checkable", spaced(&f))
                .dimmed()
                .to_string()
        }
        SealCheck::NoSeal => "no seal recorded on this commit".yellow().to_string(),
    }
}

/// The seal notes: one line per kept plan, the note blob, a space, the full commit hash. A repository
/// with no notes answers an empty list; git that cannot be read answers Err.
pub fn seal_notes(core_root: &str) -> Result<String, String> {
    git_read(core_root, &["notes", "--ref=seals", "list"])
}

/// A commit's seal, re-checked against the plan text kept for it.
pub fn check_commit_seal(core_root: &str, notes: &str, hash: &str) -> Result<SealCheck, String> {
    let seal = trailer(core_root, hash, "Seal")?;
    let fingerprint = trailer(core_root, hash, "Fingerprint")?;
    let note = match note_for(notes, hash) {
        Some(blob) => Some(git_read(core_root, &["cat-file", "-p", blob])?),
        None => None,
    };
    Ok(check_seal(
        seal.as_deref(),
        fingerprint.as_deref(),
        note.as_deref().map(str::as_bytes),
    ))
}

/// The abbreviated hash and subject of one commit. Err when git cannot show it.
fn commit_header(core_root: &str, rev: &str) -> Result<(String, String), String> {
    let text = git_read(core_root, &["log", "-1", "--format=%h%x09%s", rev])?;
    match text.trim().split_once('\t') {
        Some((h, s)) => Ok((h.to_string(), s.to_string())),
        None => Err(format!("git log gave no header for {}", rev)),
    }
}

/// Every seal on the current branch with the commit that carries it -- the same line shape as
/// Fixes:, so parse_fixes reads it. Err when git cannot be read.
fn seal_commits(core_root: &str) -> Result<Vec<(String, String)>, String> {
    let text = git_read(
        core_root,
        &[
            "log",
            "--format=%h%x09%(trailers:key=Seal,valueonly,separator=%x2C)",
            "HEAD",
        ],
    )?;
    Ok(parse_fixes(&text))
}

/// Print each finding id a trailer names, with what its record says. Returns how many it printed.
fn print_named(label: &str, ids: Option<&str>, records: &[Record]) -> usize {
    let mut n = 0;
    for id in ids
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        n += 1;
        let what = records
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.what.as_str())
            .unwrap_or("names a finding with no record");
        println!("     {:<8} {}  {}", label, id.bright_white(), what);
    }
    n
}

/// `core intent trace <commit>` -- question c: the intent and the findings a commit names, and its
/// seal re-checked.
fn trace_commit(ctx: &AppContext, rev: &str) -> CoreResult<()> {
    let core: &str = &ctx.core_root;
    let (hash, subject) = commit_header(core, rev)
        .map_err(|e| could_not(format!("git could not show {}: {}", rev, e)))?;
    let intent = trailer(core, &hash, "Intent").map_err(could_not)?;
    let fixes_ids = trailer(core, &hash, "Fixes").map_err(could_not)?;
    let finding_ids = trailer(core, &hash, "Finding").map_err(could_not)?;
    let notes = seal_notes(core)
        .map_err(|e| could_not(format!("the seal notes could not be read: {}", e)))?;
    let check = check_commit_seal(core, &notes, &hash).map_err(could_not)?;
    let dir = super::intents_dir(ctx).join("findings");
    let records = super::findings::read_records(&dir).map_err(could_not)?;
    println!("  {}  {}", hash.bright_white(), subject);
    match &intent {
        Some(i) => println!("     intent   {}", spaced(i)),
        None => println!(
            "     intent   {}",
            "no Intent trailer -- trailers are the rule from 8c4c4c15 (2026-09-28)".dimmed()
        ),
    }
    let named = print_named("fixes", fixes_ids.as_deref(), &records)
        + print_named("touches", finding_ids.as_deref(), &records);
    if named == 0 {
        println!("     findings none named by a Fixes: or Finding: trailer");
    }
    println!("     {}", seal_line(check));
    Ok(())
}

/// `core intent trace <seal>` -- the commit that carries the seal, traced as question c. A 16-hex
/// value no commit on HEAD carries as a seal is tried as a commit, and says so.
fn trace_seal(ctx: &AppContext, value: &str) -> CoreResult<()> {
    let core: &str = &ctx.core_root;
    let seals = seal_commits(core)
        .map_err(|e| could_not(format!("the seals on HEAD could not be read: {}", e)))?;
    let carriers: Vec<&String> = seals
        .iter()
        .filter(|(s, _)| s == value)
        .map(|(_, h)| h)
        .collect();
    if carriers.is_empty() {
        println!("  {} is no seal on HEAD; tracing it as a commit", value);
        return trace_commit(ctx, value);
    }
    for hash in carriers {
        trace_commit(ctx, hash)?;
    }
    Ok(())
}

/// INT-274: one row of `core intent seals` -- what one commit claims, and its seal re-checked.
#[derive(Debug, PartialEq)]
pub struct SealRow {
    pub hash: String,
    pub date: String,
    pub intent: Option<String>,
    pub subject: String,
    pub check: SealCheck,
}

/// The field separator in the walk's git log read (git writes it as %x1f).
const UNIT: char = '\u{1f}';

/// One commit's header from the walk's git log read.
struct Head {
    full: String,
    hash: String,
    date: String,
    intent: Option<String>,
    seal: Option<String>,
    fingerprint: Option<String>,
    subject: String,
}

/// The walk's commit read (INT-274 G7): `n` commits from HEAD, NUL between commits, the subject
/// last. The plan note is NOT read here. %N is git's display of a note, not its bytes: measured
/// 2026-10-05, it drops a trailing blank line and adds a missing final newline, and reading it
/// called four intact seals on 0-core broken. A seal is a hash of the exact bytes.
fn walk_text(core_root: &str, n: usize) -> Result<String, String> {
    let count = format!("-n{}", n);
    git_read(
        core_root,
        &[
            "log",
            "-z",
            count.as_str(),
            "--format=%H%x1f%h%x1f%cs%x1f%(trailers:key=Intent,valueonly,separator=%x2C)%x1f%(trailers:key=Seal,valueonly,separator=%x2C)%x1f%(trailers:key=Fingerprint,valueonly,separator=%x2C)%x1f%s",
            "HEAD",
        ],
    )
}

/// One header per record of walk_text. Err when a record does not have its seven fields.
fn parse_heads(text: &str) -> Result<Vec<Head>, String> {
    fn some(s: &str) -> Option<String> {
        let s = s.trim();
        if s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    }
    let mut heads = Vec::new();
    for record in text.split('\0').filter(|r| !r.trim().is_empty()) {
        let f: Vec<&str> = record.splitn(7, UNIT).collect();
        if f.len() != 7 {
            return Err(format!(
                "git log gave a record with {} fields, not 7, beginning {:?}",
                f.len(),
                record.chars().take(40).collect::<String>()
            ));
        }
        heads.push(Head {
            full: f[0].trim().to_string(),
            hash: f[1].to_string(),
            date: f[2].to_string(),
            intent: some(f[3]),
            seal: some(f[4]),
            fingerprint: some(f[5]),
            subject: f[6].to_string(),
        });
    }
    Ok(heads)
}

/// Blob sizes from `git ls-tree -r -l` of the notes tree: oid to size in bytes.
fn parse_sizes(tree: &str) -> std::collections::HashMap<String, usize> {
    tree.lines()
        .filter_map(|line| {
            let (meta, _path) = line.split_once('\t')?;
            let f: Vec<&str> = meta.split_whitespace().collect();
            if f.len() == 4 && f[1] == "blob" {
                Some((f[2].to_string(), f[3].parse().ok()?))
            } else {
                None
            }
        })
        .collect()
}

/// The exact bytes of the plan note kept for each commit, in at most three git reads whatever the
/// window: the notes list, the blob sizes in the notes tree, and one `git show` of the blobs split
/// by those sizes. `git show` of a blob is its raw bytes (measured 2026-10-05: the concatenation
/// equals `cat-file -p` of each, a trailing blank line and a missing final newline included).
fn note_bytes(core_root: &str, commits: &[&str]) -> Result<Vec<Option<Vec<u8>>>, String> {
    let list = seal_notes(core_root)?;
    let blobs: Vec<Option<String>> = commits
        .iter()
        .map(|c| note_for(&list, c).map(str::to_string))
        .collect();
    let mut wanted: Vec<&str> = Vec::new();
    for b in blobs.iter().flatten() {
        if !wanted.contains(&b.as_str()) {
            wanted.push(b.as_str());
        }
    }
    if wanted.is_empty() {
        return Ok(blobs.iter().map(|_| None).collect());
    }
    let sizes = parse_sizes(&git_read(
        core_root,
        &["ls-tree", "-r", "-l", "refs/notes/seals"],
    )?);
    let mut args: Vec<&str> = vec!["show"];
    args.extend(wanted.iter().copied());
    let shown = run_bytes("git", core_root, &args)?;
    let mut bytes: std::collections::HashMap<&str, Vec<u8>> = std::collections::HashMap::new();
    let mut at = 0;
    for blob in &wanted {
        let size = *sizes
            .get(*blob)
            .ok_or_else(|| format!("note blob {} has no size in refs/notes/seals", blob))?;
        if at + size > shown.len() {
            return Err(format!(
                "git show gave {} bytes, fewer than the notes tree sizes",
                shown.len()
            ));
        }
        bytes.insert(*blob, shown[at..at + size].to_vec());
        at += size;
    }
    if at != shown.len() {
        return Err(format!(
            "git show gave {} bytes; the notes tree sizes add to {}",
            shown.len(),
            at
        ));
    }
    Ok(blobs
        .iter()
        .map(|b| b.as_deref().and_then(|b| bytes.get(b).cloned()))
        .collect())
}

/// The last `n` commits on HEAD in `core_root`, each seal re-checked against the exact bytes of
/// its kept plan. Err when git cannot be read.
pub fn walk(core_root: &str, n: usize) -> Result<Vec<SealRow>, String> {
    let heads = parse_heads(&walk_text(core_root, n)?)?;
    let fulls: Vec<&str> = heads.iter().map(|h| h.full.as_str()).collect();
    let notes = note_bytes(core_root, &fulls)?;
    Ok(heads
        .into_iter()
        .zip(notes)
        .map(|(h, note)| SealRow {
            hash: h.hash,
            date: h.date,
            intent: h.intent,
            subject: h.subject,
            check: check_seal(h.seal.as_deref(), h.fingerprint.as_deref(), note.as_deref()),
        })
        .collect())
}

/// Does an Intent: trailer (comma-joined) name intent `id`? INT-x and bare x both count.
fn names_intent(trailer: Option<&str>, id: &str) -> bool {
    trailer
        .map(|t| {
            t.split(',')
                .map(str::trim)
                .any(|v| v.strip_prefix("INT-").unwrap_or(v) == id)
        })
        .unwrap_or(false)
}

/// INT-274 exit: 1 if any seal in the window is broken, else 2 if any names a plan that is not
/// kept, else 0. Legacy and unsealed are shown, never failed.
pub fn seals_exit(rows: &[SealRow]) -> i32 {
    if rows
        .iter()
        .any(|r| matches!(r.check, SealCheck::Broken { .. }))
    {
        1
    } else if rows.iter().any(|r| matches!(r.check, SealCheck::NoNote(_))) {
        2
    } else {
        0
    }
}

/// `core intent seals [-n N] [--intent INT-x]` -- INT-274. Every seal in the last N commits on
/// HEAD, re-checked by check_seal from one git read. Exit 0 nothing broken or unproven, 1 a seal is
/// broken, 2 a seal names a plan that is not kept, 3 could not answer. The domain owns its exit,
/// as `core fingerprint show` does, so a refusal is never read as a broken seal.
pub fn seals(ctx: &AppContext, n: &str, intent: Option<&str>) -> CoreResult<()> {
    let code = match seals_answer(ctx, n, intent) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{} {}", "\u{2717}".bright_red(), e);
            3
        }
    };
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

fn seals_answer(ctx: &AppContext, n: &str, intent: Option<&str>) -> Result<i32, String> {
    ctx.capabilities
        .require(
            "intent",
            &[Capability::FilesystemReadHome, Capability::SpawnProcess],
        )
        .map_err(|e| e.to_string())?;
    let count: usize = match n.trim().parse() {
        Ok(c) if c > 0 => c,
        _ => {
            return Err(format!(
                "seals could not answer -- -n {} is not a count of commits (1 or more)",
                n
            ))
        }
    };
    let wanted = match intent {
        None => None,
        Some(t) => {
            let t = t.trim();
            let id = t.strip_prefix("INT-").unwrap_or(t).to_string();
            if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
                return Err(format!(
                    "seals could not answer -- {} is not an intent (INT-x or x)",
                    t
                ));
            }
            if !super::load_all(ctx).iter().any(|i| i.id == id) {
                return Err(format!(
                    "seals could not answer -- INT-{} names no intent in the ledger",
                    id
                ));
            }
            Some(id)
        }
    };
    let rows = walk(&ctx.core_root, count)
        .map_err(|e| format!("seals could not answer -- git could not be read: {}", e))?;
    Ok(print_seals(rows, count, wanted.as_deref()))
}

/// Print the table, newest first, with a reason line under broken, no plan and legacy. Returns the
/// exit code for the rows printed.
fn print_seals(rows: Vec<SealRow>, count: usize, wanted: Option<&str>) -> i32 {
    let rows: Vec<SealRow> = match wanted {
        Some(id) => rows
            .into_iter()
            .filter(|r| names_intent(r.intent.as_deref(), id))
            .collect(),
        None => rows,
    };
    if rows.is_empty() {
        match wanted {
            Some(id) => println!("  no commit in the last {} on HEAD names INT-{}", count, id),
            None => println!("  no commits on HEAD"),
        }
        return 0;
    }
    let code = seals_exit(&rows);
    println!(
        "{}",
        format!(
            "  {:<10}  {:<9}  {:<9}  {:<16}  {:<8}  {}",
            "date", "commit", "intent", "seal", "verdict", "subject"
        )
        .dimmed()
    );
    for r in rows {
        let intent = r
            .intent
            .as_deref()
            .map(spaced)
            .unwrap_or_else(|| "-".to_string());
        let (seal, word) = match &r.check {
            SealCheck::Intact(s) => (s.clone(), format!("{:<8}", "sealed").green().to_string()),
            SealCheck::Broken { seal, .. } => {
                (seal.clone(), format!("{:<8}", "broken").red().to_string())
            }
            SealCheck::NoNote(s) => (s.clone(), format!("{:<8}", "no plan").yellow().to_string()),
            SealCheck::Legacy(f) => (spaced(f), format!("{:<8}", "legacy").dimmed().to_string()),
            SealCheck::NoSeal => (
                "-".to_string(),
                format!("{:<8}", "unsealed").dimmed().to_string(),
            ),
        };
        println!(
            "  {:<10}  {:<9}  {:<9}  {:<16}  {}  {}",
            r.date, r.hash, intent, seal, word, r.subject
        );
        if !matches!(r.check, SealCheck::Intact(_) | SealCheck::NoSeal) {
            println!("              {}", seal_line(r.check));
        }
    }
    code
}

/// INT-306: what a commit message's trailers require, before the commit object exists.
/// `commit-msg` applies the same rules and does not call this: the hook must not need cargo.
/// A message with no `Intent: INT-xxx` trailer is left alone.
#[derive(Debug, PartialEq, Clone)]
pub enum MessageGate {
    /// No Intent: INT-xxx trailer.
    Untouched,
    /// Intent: and a Seal: of 16 lowercase hex.
    Sealed,
    /// Intent: and a non-empty Release: trailer. The check says so.
    Exempt(String),
    /// Intent: and neither a 16-hex Seal: nor a Release:.
    Refused { intent: String },
}

/// True when a trailer value is `INT-` and digits, the form the guard requires.
fn is_intent_id(value: &str) -> bool {
    let Some(rest) = value.trim().strip_prefix("INT-") else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
}

/// True when a trailer value is exactly 16 lowercase hex.
fn is_seal_token(value: &str) -> bool {
    let value = value.trim();
    value.len() == 16
        && value
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

/// Trailers git would keep, as `git interpret-trailers --parse` prints them.
fn interpret_trailers(text: &str) -> Result<Vec<(String, String)>, String> {
    let mut child = Command::new("git")
        .args(["interpret-trailers", "--parse"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("git interpret-trailers could not run: {e}"))?;
    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or("git interpret-trailers took no stdin")?;
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("could not write the message: {e}"))?;
    }
    let out = child
        .wait_with_output()
        .map_err(|e| format!("git interpret-trailers could not be read: {e}"))?;
    if !out.status.success() {
        let code = out
            .status
            .code()
            .map_or("by signal".to_string(), |c| c.to_string());
        return Err(format!(
            "git interpret-trailers exited {}: {}",
            code,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let mut pairs = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if let Some((key, value)) = line.split_once(':') {
            pairs.push((key.trim().to_string(), value.trim().to_string()));
        }
    }
    Ok(pairs)
}

/// What the commit-msg hook decides. The hook is a shell script and does not call this;
/// the two must agree, and g5_commit_msg_hook runs the script.
pub fn commit_message_gate(text: &str) -> Result<MessageGate, String> {
    let mut intent: Option<String> = None;
    let mut sealed = false;
    let mut release: Option<String> = None;
    for (key, value) in interpret_trailers(text)? {
        match key.as_str() {
            "Intent" if intent.is_none() && is_intent_id(&value) => {
                intent = Some(value.trim().to_string());
            }
            "Seal" if is_seal_token(&value) => sealed = true,
            "Release" if release.is_none() && !value.trim().is_empty() => {
                release = Some(value.trim().to_string());
            }
            _ => {}
        }
    }
    let Some(intent) = intent else {
        return Ok(MessageGate::Untouched);
    };
    if let Some(release) = release {
        return Ok(MessageGate::Exempt(release));
    }
    if sealed {
        return Ok(MessageGate::Sealed);
    }
    Ok(MessageGate::Refused { intent })
}

/// One commit a range check looked at, because it carries Intent:.
#[derive(Debug, PartialEq, Clone)]
pub struct IntentRangeRow {
    pub hash: String,
    pub subject: String,
    pub verdict: IntentRangeVerdict,
}

/// The check_seal verdict, plus the release exemption. Legacy is a Fingerprint with no Seal.
#[derive(Debug, PartialEq, Clone)]
pub enum IntentRangeVerdict {
    Intact,
    Exempt,
    NoSeal,
    NoNote,
    Broken,
    Legacy,
}

/// The rows a range produced, and whether any of them fail the push.
#[derive(Debug, PartialEq)]
pub struct IntentRangeReport {
    pub rows: Vec<IntentRangeRow>,
    pub failed: bool,
}

fn names_an_intent(intent: &Option<String>) -> bool {
    intent
        .as_deref()
        .map(|value| value.split(',').map(str::trim).any(is_intent_id))
        .unwrap_or(false)
}

/// Each commit in a git rev-list range that carries Intent:. Intact passes. A Release:
/// trailer is exempt and does not ask the note. NoSeal, NoNote, Broken and Legacy fail.
/// Commits with no Intent: are left out. The verdict is check_seal; this does not hash.
pub fn check_intent_range(core_root: &str, rev_args: &[&str]) -> Result<IntentRangeReport, String> {
    if rev_args.is_empty() {
        return Err("seals check needs a rev-list range".to_string());
    }
    let mut args = vec!["rev-list", "--reverse"];
    args.extend(rev_args);
    let list = git_read(core_root, &args)?;
    let notes = seal_notes(core_root)?;
    let mut rows = Vec::new();
    let mut failed = false;
    for hash in list.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if !names_an_intent(&trailer(core_root, hash, "Intent")?) {
            continue;
        }
        let (abbr, subject) = commit_header(core_root, hash)?;
        let release = trailer(core_root, hash, "Release")?;
        let verdict = if release
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        {
            IntentRangeVerdict::Exempt
        } else {
            match check_commit_seal(core_root, &notes, hash)? {
                SealCheck::Intact(_) => IntentRangeVerdict::Intact,
                SealCheck::NoSeal => {
                    failed = true;
                    IntentRangeVerdict::NoSeal
                }
                SealCheck::NoNote(_) => {
                    failed = true;
                    IntentRangeVerdict::NoNote
                }
                SealCheck::Broken { .. } => {
                    failed = true;
                    IntentRangeVerdict::Broken
                }
                SealCheck::Legacy(_) => {
                    failed = true;
                    IntentRangeVerdict::Legacy
                }
            }
        };
        rows.push(IntentRangeRow {
            hash: abbr,
            subject,
            verdict,
        });
    }
    Ok(IntentRangeReport { rows, failed })
}

/// The line `core intent seals check` prints. An exemption says so.
pub fn intent_range_line(row: &IntentRangeRow) -> String {
    let word = match row.verdict {
        IntentRangeVerdict::Intact => "intact",
        IntentRangeVerdict::Exempt => "exempt (release)",
        IntentRangeVerdict::NoSeal => "NoSeal",
        IntentRangeVerdict::NoNote => "NoNote",
        IntentRangeVerdict::Broken => "Broken",
        IntentRangeVerdict::Legacy => "Legacy",
    };
    format!("{} {}", row.hash, word)
}

/// `core intent seals check <range>`. Exit 0 when every Intent commit is intact or
/// exempt, 1 when one is not, 3 when the range could not be read.
pub fn seals_check(ctx: &AppContext, range: &[String]) -> CoreResult<()> {
    let args: Vec<&str> = range.iter().map(String::as_str).collect();
    let report = (|| {
        ctx.capabilities
            .require(
                "intent",
                &[Capability::FilesystemReadHome, Capability::SpawnProcess],
            )
            .map_err(|e| e.to_string())?;
        check_intent_range(&ctx.core_root, &args)
    })();
    match report {
        Ok(report) => {
            if report.rows.is_empty() {
                println!("  no commit in the range carries Intent:");
            }
            for row in &report.rows {
                let line = intent_range_line(row);
                match row.verdict {
                    IntentRangeVerdict::Intact => println!("{}", line.green()),
                    IntentRangeVerdict::Exempt => println!("{}", line.yellow()),
                    _ => println!("{}", line.red()),
                }
            }
            if report.failed {
                std::process::exit(1);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("{} {}", "\u{2717}".bright_red(), e);
            std::process::exit(3);
        }
    }
}

/// INT-274 G3 and G6: a real repository in a temp directory, one commit per verdict, walked by the
/// same function the command uses. The real refs/notes/seals is never touched.
#[cfg(test)]
mod seals_tests {
    use super::{names_intent, parse_heads, seal_digest, seals_exit, walk, SealCheck};

    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn git(dir: &str, args: &[&str]) {
        let st = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "user.name=int274",
                "-c",
                "user.email=int274@localhost",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .args(args)
            .status()
            .expect("git runs");
        assert!(st.success(), "fixture git {:?} failed", args);
    }

    fn fixture() -> Fixture {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("int274-fixture-{}-{}", std::process::id(), nanos));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let f = Fixture(dir);
        let d = f.0.to_str().expect("utf-8 temp path").to_string();
        git(&d, &["init", "-q"]);
        let plan = b"plan A\n\ta tab and a \x1f unit separator\n";
        let altered = b"plan C, altered after review\n";
        std::fs::write(f.0.join("plan-a"), plan).expect("plan a");
        std::fs::write(f.0.join("plan-c"), altered).expect("plan c");
        let seal_a = format!("Seal: {}", seal_digest(plan));
        let seal_b = format!("Seal: {}", seal_digest(b"plan B\n"));
        let seal_c = format!("Seal: {}", seal_digest(b"plan C\n"));
        let plan_a = f.0.join("plan-a").to_str().unwrap().to_string();
        let plan_c = f.0.join("plan-c").to_str().unwrap().to_string();
        git(&d, &["commit", "-q", "--allow-empty", "-m", "unsealed"]);
        git(
            &d,
            &[
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "legacy",
                "--trailer",
                "Fingerprint: ce3dd6c2176c585a",
            ],
        );
        git(
            &d,
            &[
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "no plan",
                "--trailer",
                &seal_b,
            ],
        );
        git(
            &d,
            &[
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "broken",
                "--trailer",
                &seal_c,
                "--trailer",
                "Intent: INT-7",
            ],
        );
        git(&d, &["notes", "--ref=seals", "add", "-F", &plan_c, "HEAD"]);
        git(
            &d,
            &[
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "sealed",
                "--trailer",
                &seal_a,
                "--trailer",
                "Intent: INT-7",
            ],
        );
        git(&d, &["notes", "--ref=seals", "add", "-F", &plan_a, "HEAD"]);
        f
    }

    #[test]
    fn the_walk_gives_all_five_verdicts_newest_first() {
        let f = fixture();
        let rows = walk(f.0.to_str().unwrap(), 10).expect("fixture walks");
        let subjects: Vec<&str> = rows.iter().map(|r| r.subject.as_str()).collect();
        assert_eq!(
            subjects,
            ["sealed", "broken", "no plan", "legacy", "unsealed"]
        );
        assert!(
            matches!(rows[0].check, SealCheck::Intact(_)),
            "{:?}",
            rows[0]
        );
        assert!(
            matches!(rows[1].check, SealCheck::Broken { .. }),
            "{:?}",
            rows[1]
        );
        assert!(
            matches!(rows[2].check, SealCheck::NoNote(_)),
            "{:?}",
            rows[2]
        );
        assert_eq!(
            rows[3].check,
            SealCheck::Legacy("ce3dd6c2176c585a".to_string())
        );
        assert_eq!(rows[4].check, SealCheck::NoSeal);
        assert_eq!(rows[0].intent.as_deref(), Some("INT-7"));
    }

    #[test]
    fn the_exit_is_the_worst_row() {
        let f = fixture();
        let rows = walk(f.0.to_str().unwrap(), 10).expect("fixture walks");
        assert_eq!(seals_exit(&rows), 1, "a broken seal exits 1");
        let rows = walk(f.0.to_str().unwrap(), 10).expect("fixture walks");
        let without_broken: Vec<_> = rows.into_iter().filter(|r| r.subject != "broken").collect();
        assert_eq!(
            seals_exit(&without_broken),
            2,
            "a seal with no kept plan exits 2"
        );
        let rows = walk(f.0.to_str().unwrap(), 10).expect("fixture walks");
        let clean: Vec<_> = rows
            .into_iter()
            .filter(|r| r.subject != "broken" && r.subject != "no plan")
            .collect();
        assert_eq!(seals_exit(&clean), 0, "sealed, legacy and unsealed exit 0");
    }

    fn git_out(dir: &str, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("git runs");
        assert!(out.status.success(), "fixture git {:?} failed", args);
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    /// INT-274 regression -- the class, not the example. git's %N is a display of a note, not its
    /// bytes: it drops a trailing blank line and adds a missing final newline. A walk that hashed %N
    /// called four intact seals on 0-core broken (20804eaf, 3492c26f, cb1dbe90, d5a65703). These
    /// notes are written raw (hash-object --no-filters, notes add -C) so git cannot clean them first.
    #[test]
    fn a_note_git_would_display_differently_still_hashes_exactly() {
        let f = fixture();
        let d = f.0.to_str().unwrap().to_string();
        let bodies: [&[u8]; 3] = [
            b"ends in a blank line\n\n",
            b"no final newline",
            b"crlf\r\nand a blank line\r\n\r\n",
        ];
        for (i, body) in bodies.iter().enumerate() {
            let p = f.0.join(format!("raw-{}", i));
            std::fs::write(&p, body).expect("raw note");
            let blob = git_out(
                &d,
                &["hash-object", "-w", "--no-filters", p.to_str().unwrap()],
            );
            let seal = format!("Seal: {}", seal_digest(body));
            let subject = format!("raw {}", i);
            git(
                &d,
                &[
                    "commit",
                    "-q",
                    "--allow-empty",
                    "-m",
                    &subject,
                    "--trailer",
                    &seal,
                ],
            );
            git(
                &d,
                &["notes", "--ref=seals", "add", "-C", blob.trim(), "HEAD"],
            );
        }
        let rows = walk(&d, 3).expect("fixture walks");
        assert_eq!(rows.len(), 3);
        for r in &rows {
            assert!(
                matches!(r.check, SealCheck::Intact(_)),
                "{} read as {:?}",
                r.subject,
                r.check
            );
        }
    }

    #[test]
    fn n_limits_the_window() {
        let f = fixture();
        let rows = walk(f.0.to_str().unwrap(), 2).expect("fixture walks");
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn an_unreadable_repo_refuses() {
        assert!(walk("/nonexistent/int274-no-repo", 5).is_err());
    }

    #[test]
    fn a_record_short_of_fields_refuses_rather_than_guessing() {
        assert!(parse_heads("abc1234\u{1f}2026-10-05\0").is_err());
    }

    #[test]
    fn an_intent_trailer_matches_by_id() {
        assert!(names_intent(Some("INT-266"), "266"));
        assert!(names_intent(Some("INT-272,INT-266"), "266"));
        assert!(!names_intent(Some("INT-2660"), "266"));
        assert!(!names_intent(None, "266"));
    }
}

/// Gate 9's class: did not start, started and refused, answered nothing. Each is its own test.
#[cfg(test)]
mod reader_tests {
    use super::run;

    fn repo() -> String {
        env!("CARGO_MANIFEST_DIR").to_string()
    }

    #[test]
    fn a_program_that_cannot_start_answers_could_not_run() {
        match run("zero-int266-no-such-program", &repo(), &["log", "-1"]) {
            Err(e) => assert!(e.contains("could not run"), "wrong refusal: {}", e),
            Ok(text) => panic!(
                "a program that cannot start must answer Err, got Ok({:?})",
                text
            ),
        }
    }

    #[test]
    fn git_that_refuses_answers_its_exit() {
        match run(
            "git",
            &repo(),
            &["log", "-1", "zero-int266-no-such-revision"],
        ) {
            Err(e) => assert!(e.contains("exited"), "wrong refusal: {}", e),
            Ok(text) => panic!(
                "git refusing a revision must answer Err, got Ok({:?})",
                text
            ),
        }
    }

    #[test]
    fn git_that_answers_nothing_is_a_true_none() {
        let got = run("git", &repo(), &["log", "-1", "--format="]);
        assert_eq!(got.map(|t| t.trim().to_string()), Ok(String::new()));
    }
}

/// INT-266 question a. Gate 9 at the query: git that cannot be read never makes a finding open.
#[cfg(test)]
mod state_tests {
    use super::{parse_fixes, split_trailers, state_of, State};
    use crate::domains::intent::findings::parse_record;

    fn record(closed: &str) -> super::Record {
        parse_record(&format!(
            "---\nid: F-0001\nwhat: x\nat: a.rs:1\nfound_by: INT-266\nfound_on: 2026-10-04\nclosed_without_fix: {}\n---\n",
            closed
        ))
    }

    #[test]
    fn git_that_cannot_be_read_never_reads_open() {
        let got = state_of(&record("no"), &Err("git log could not run".to_string()));
        assert_eq!(got, State::Unread("git log could not run".to_string()));
    }

    #[test]
    fn a_fixes_trailer_marks_it_fixed_by_that_commit() {
        let named = Ok(vec![("F-0001".to_string(), "abc1234".to_string())]);
        assert_eq!(
            state_of(&record("no"), &named),
            State::Fixed(vec!["abc1234".to_string()])
        );
        assert_eq!(state_of(&record("no"), &Ok(Vec::new())), State::Open);
    }

    #[test]
    fn a_closed_record_reads_closed_with_its_reason() {
        assert_eq!(
            state_of(&record("duplicate of F-0002"), &Ok(Vec::new())),
            State::Closed("duplicate of F-0002".to_string())
        );
    }

    #[test]
    fn a_fixes_line_names_every_id_it_lists() {
        let got = parse_fixes("abc1234\tF-0001,F-0002\ndef5678\t\n");
        assert_eq!(
            got,
            vec![
                ("F-0001".to_string(), "abc1234".to_string()),
                ("F-0002".to_string(), "abc1234".to_string()),
            ]
        );
    }

    /// INT-300 R3: a fix wins over a closure. A finding closed without a fix that a commit then
    /// fixed reads fixed, because the commit is the stronger fact.
    #[test]
    fn a_fix_wins_over_a_closure() {
        let named = Ok(vec![("F-0001".to_string(), "abc1234".to_string())]);
        assert_eq!(
            state_of(&record("duplicate of F-0002"), &named),
            State::Fixed(vec!["abc1234".to_string()])
        );
    }

    /// INT-300 R4: a Fixes value counts only when it is exactly F-NNNN. A value with prose after
    /// the id counts for nothing, so a filing commit cannot mark a finding fixed.
    #[test]
    fn a_fixes_value_with_prose_counts_for_nothing() {
        let got = parse_fixes("abc1234\tF-0046 later\ndef5678\tF-0002\n");
        assert_eq!(got, vec![("F-0002".to_string(), "def5678".to_string())]);
    }

    /// INT-300 G8: the one reader keeps what it does not count. A value that is not exactly
    /// F-NNNN lands in `other` with its commit, a value that is lands in `named`, and an empty
    /// value is nothing.
    #[test]
    fn values_that_name_no_finding_are_kept_not_dropped() {
        let got = split_trailers("abc1234\tF-0046 later\ndef5678\tF-0002, junk\nfff0000\t\n");
        assert_eq!(
            got.named,
            vec![("F-0002".to_string(), "def5678".to_string())]
        );
        assert_eq!(
            got.other,
            vec![
                ("F-0046 later".to_string(), "abc1234".to_string()),
                ("junk".to_string(), "def5678".to_string()),
            ]
        );
    }

    /// INT-300 G8: lower case, a short number and a long number are not finding ids either, and a
    /// stray comma makes no empty value.
    #[test]
    fn near_misses_are_other_and_empty_values_are_nothing() {
        let got = split_trailers("a1\t,F-0001,f-0002,F-3,F-00045,\n");
        assert_eq!(got.named, vec![("F-0001".to_string(), "a1".to_string())]);
        let other: Vec<&str> = got.other.iter().map(|(v, _)| v.as_str()).collect();
        assert_eq!(other, vec!["f-0002", "F-3", "F-00045"]);
    }

    /// INT-300 G4, G8: parse_fixes is the named half of the one reader, never a second parse.
    #[test]
    fn parse_fixes_is_the_named_half_of_the_one_reader() {
        let text = "abc1234\tF-0001,F-0046 later\ndef5678\tF-0002\n";
        assert_eq!(parse_fixes(text), split_trailers(text).named);
    }
}

/// INT-266 question b. A seal is re-checked, never assumed: a note that hashes to something
/// else is broken, and a commit from before the Seal rule answers legacy, never empty.
#[cfg(test)]
mod seal_tests {
    use super::{check_seal, note_for, seal_digest, SealCheck};

    #[test]
    fn the_digest_is_the_first_16_hex_of_sha256() {
        assert_eq!(seal_digest(b"abc"), "ba7816bf8f01cfea");
    }

    #[test]
    fn a_note_that_hashes_to_the_seal_is_intact() {
        let seal = seal_digest(b"plan\n");
        assert_eq!(
            check_seal(Some(seal.as_str()), None, Some(&b"plan\n"[..])),
            SealCheck::Intact(seal.clone())
        );
    }

    #[test]
    fn a_note_that_does_not_hash_to_the_seal_is_broken() {
        assert_eq!(
            check_seal(Some("0000000000000000"), None, Some(&b"plan\n"[..])),
            SealCheck::Broken {
                seal: "0000000000000000".to_string(),
                got: seal_digest(b"plan\n"),
            }
        );
    }

    #[test]
    fn a_commit_before_the_seal_rule_answers_legacy_never_empty() {
        assert_eq!(
            check_seal(None, Some("ce3dd6c2176c585a"), None),
            SealCheck::Legacy("ce3dd6c2176c585a".to_string())
        );
    }

    #[test]
    fn a_seal_with_no_kept_plan_says_so() {
        assert_eq!(
            check_seal(Some("abc"), None, None),
            SealCheck::NoNote("abc".to_string())
        );
        assert_eq!(check_seal(None, None, None), SealCheck::NoSeal);
    }

    #[test]
    fn a_note_is_found_by_an_abbreviated_hash() {
        let list = "blob1 abcdef0123456789\nblob2 1234567890abcdef\n";
        assert_eq!(note_for(list, "1234567"), Some("blob2"));
        assert_eq!(note_for(list, "fffffff"), None);
    }
}

/// INT-266 gate 9: every reader a trace query uses answers Err when git cannot be read -- never an
/// empty answer that would print as no findings or no commits. NOWHERE is a path git cannot enter,
/// so the answer does not depend on GIT_DIR or the current directory.
#[cfg(test)]
mod unreadable_tests {
    use super::{commit_header, fixes, seal_commits, seal_notes, target_kind, trailer, Target};

    const NOWHERE: &str = "/nonexistent/int266-no-repo";

    #[test]
    fn the_question_a_and_b_readers_refuse_an_unreadable_repo() {
        assert!(fixes(NOWHERE).is_err(), "fixes");
        assert!(trailer(NOWHERE, "HEAD", "Seal").is_err(), "trailer");
        assert!(seal_notes(NOWHERE).is_err(), "seal notes");
    }

    #[test]
    fn the_commit_reader_refuses_an_unreadable_repo() {
        let got = commit_header(NOWHERE, "HEAD");
        assert!(got.is_err(), "commit_header must answer Err, got {:?}", got);
    }

    #[test]
    fn the_seal_reader_refuses_an_unreadable_repo() {
        let got = seal_commits(NOWHERE);
        assert!(got.is_err(), "seal_commits must answer Err, got {:?}", got);
    }

    #[test]
    fn a_target_is_read_by_its_shape() {
        assert_eq!(target_kind("INT-266"), Target::Intent("266".to_string()));
        assert_eq!(target_kind("266"), Target::Intent("266".to_string()));
        assert_eq!(target_kind("F-0010"), Target::Finding("F-0010".to_string()));
        assert_eq!(
            target_kind("369dd33f83163a00"),
            Target::SealOrCommit("369dd33f83163a00".to_string())
        );
        assert_eq!(
            target_kind("87e17913"),
            Target::Commit("87e17913".to_string())
        );
        assert_eq!(target_kind("not-a-thing"), Target::Unknown);
    }
}

/// Trailer lists print with a space after each comma, the way a reader writes them.
#[cfg(test)]
mod display_tests {
    use super::spaced;

    #[test]
    fn a_trailer_list_reads_with_a_space_after_each_comma() {
        assert_eq!(spaced("INT-272,INT-273"), "INT-272, INT-273");
        assert_eq!(spaced("INT-266"), "INT-266");
    }
}

/// INT-306 G5. Ignored until the guard exists, so a bare cargo test stays green.
/// The red run is `.githooks/test-g5-red.sh`.
#[cfg(test)]
mod g5_seal_guard_tests {
    use super::{
        check_intent_range, commit_message_gate, intent_range_line, seal_digest, IntentRangeReport,
        IntentRangeRow, IntentRangeVerdict, MessageGate,
    };
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    fn gate(text: &str) -> MessageGate {
        commit_message_gate(text).expect("git interpret-trailers runs")
    }

    #[test]
    fn g5_commit_message_gate_leaves_a_message_with_no_intent() {
        assert_eq!(gate("just a subject\n"), MessageGate::Untouched);
        assert_eq!(
            gate("mentioned\n\nThe prose names Intent: INT-9 but not as a trailer line.\n"),
            MessageGate::Untouched
        );
        // The first line is the subject. Git does not treat it as a trailer.
        assert_eq!(gate("Intent: INT-306\n"), MessageGate::Untouched);
        assert_eq!(gate("fix\n\nIntent: not-an-id\n"), MessageGate::Untouched);
    }

    #[test]
    fn g5_commit_message_gate_refuses_intent_without_a_seal() {
        assert_eq!(
            gate("fix\n\nIntent: INT-306\n"),
            MessageGate::Refused {
                intent: "INT-306".to_string()
            }
        );
    }

    #[test]
    fn g5_commit_message_gate_accepts_sixteen_lowercase_hex() {
        assert_eq!(
            gate("fix\n\nIntent: INT-306\nSeal: 0123456789abcdef\n"),
            MessageGate::Sealed
        );
    }

    #[test]
    fn g5_commit_message_gate_rejects_a_seal_that_is_not_sixteen_lowercase_hex() {
        for seal in [
            "0123456789ABCDEF",
            "0123456789abcde",
            "0123456789abcdef0",
            "0123456789abcdeg",
        ] {
            assert_eq!(
                gate(&format!("fix\n\nIntent: INT-306\nSeal: {seal}\n")),
                MessageGate::Refused {
                    intent: "INT-306".to_string()
                },
                "{seal}"
            );
        }
    }

    #[test]
    fn g5_commit_message_gate_exempts_a_release() {
        assert_eq!(
            gate("INT-295: complete; nsh 5.3.12\n\nIntent: INT-295\nRelease: nsh 5.3.12, nsh-test 2.0.15\n"),
            MessageGate::Exempt("nsh 5.3.12, nsh-test 2.0.15".to_string())
        );
        // A Release trailer is the exemption even when a Seal is also present.
        assert_eq!(
            gate("bump\n\nIntent: INT-9\nSeal: 0123456789abcdef\nRelease: nsh 1.2.3\n"),
            MessageGate::Exempt("nsh 1.2.3".to_string())
        );
    }

    #[test]
    fn g5_exempt_line_says_exempt_release() {
        let row = IntentRangeRow {
            hash: "abc1234".to_string(),
            subject: "bump".to_string(),
            verdict: IntentRangeVerdict::Exempt,
        };
        let line = intent_range_line(&row);
        assert!(
            line.contains("exempt (release)"),
            "the exemption has to be visible, got {line:?}"
        );
    }

    struct Fixture(PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn git(dir: &str, args: &[&str]) {
        let st = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "user.name=int306",
                "-c",
                "user.email=int306@localhost",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .args(args)
            .status()
            .expect("git runs");
        assert!(st.success(), "fixture git {args:?} failed");
    }

    fn git_out(dir: &str, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("git runs");
        assert!(out.status.success(), "fixture git {args:?} failed");
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    fn commit(dir: &str, subject: &str, trailers: &[&str]) {
        let mut args = vec!["commit", "-q", "--allow-empty", "-m", subject];
        let owned: Vec<String> = trailers.iter().map(|t| (*t).to_string()).collect();
        for t in &owned {
            args.push("--trailer");
            args.push(t);
        }
        git(dir, &args);
    }

    fn note(dir: &Path, name: &str, bytes: &[u8]) {
        let path = dir.join(name);
        std::fs::write(&path, bytes).expect("note bytes");
        git(
            dir.to_str().unwrap(),
            &[
                "notes",
                "--ref=seals",
                "add",
                "--no-stripspace",
                "-F",
                path.to_str().unwrap(),
                "HEAD",
            ],
        );
    }

    fn hash_of(dir: &str, subject: &str) -> String {
        git_out(dir, &["log", "--format=%H%x09%s"])
            .lines()
            .find_map(|line| {
                let (hash, sub) = line.split_once('\t')?;
                (sub == subject).then(|| hash.to_string())
            })
            .unwrap_or_else(|| panic!("no commit {subject}"))
    }

    fn between(dir: &str, from: &str, to: &str) -> IntentRangeReport {
        let spec = format!("{from}..{to}");
        check_intent_range(dir, &[&spec]).expect("range")
    }

    #[test]
    fn g5_range_check_intact_noseal_nonote_broken_release_and_ignores_no_intent() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("int306-g5-{}-{}", std::process::id(), nanos));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let fix = Fixture(dir);
        let d = fix.0.to_str().expect("utf-8").to_string();
        git(&d, &["init", "-q"]);
        let plan = b"plan intact\n";
        let other = b"plan other\n";
        let seal = format!("Seal: {}", seal_digest(plan));

        commit(&d, "plain", &[]);
        commit(&d, "mentioned", &[]);
        // The body names an intent and is not a trailer. Amend keeps the subject.
        git(
            &d,
            &[
                "commit",
                "-q",
                "--amend",
                "--allow-empty",
                "-m",
                "mentioned\n\nThe prose names Intent: INT-9 but not as a trailer line.",
            ],
        );
        commit(&d, "intact", &["Intent: INT-9", &seal]);
        note(&fix.0, "plan-intact", plan);
        commit(&d, "noseal", &["Intent: INT-9"]);
        commit(&d, "nonote", &["Intent: INT-9", &seal]);
        commit(&d, "broken", &["Intent: INT-9", &seal]);
        note(&fix.0, "plan-other", other);
        commit(&d, "release", &["Intent: INT-9", "Release: nsh 1.2.3"]);
        commit(
            &d,
            "release-broken",
            &["Intent: INT-9", &seal, "Release: nsh 1.2.3"],
        );
        note(&fix.0, "plan-other-again", other);

        let subjects = git_out(&d, &["log", "--format=%s"]);
        assert!(
            subjects.lines().any(|s| s == "plain"),
            "the no-intent commit has to be in the repo to be ignored, not absent"
        );

        let plain = hash_of(&d, "plain");
        let mentioned = hash_of(&d, "mentioned");
        let intact = hash_of(&d, "intact");
        let noseal = hash_of(&d, "noseal");
        let nonote = hash_of(&d, "nonote");
        let broken = hash_of(&d, "broken");
        let release = hash_of(&d, "release");
        let release_broken = hash_of(&d, "release-broken");

        let one = between(&d, &mentioned, &intact);
        assert!(!one.failed, "intact passes: {one:?}");
        assert_eq!(one.rows.len(), 1);
        assert_eq!(one.rows[0].verdict, IntentRangeVerdict::Intact);
        assert!(intent_range_line(&one.rows[0]).contains("intact"));

        let one = between(&d, &intact, &noseal);
        assert!(one.failed);
        assert_eq!(one.rows[0].verdict, IntentRangeVerdict::NoSeal);
        assert!(intent_range_line(&one.rows[0]).contains("NoSeal"));

        let one = between(&d, &noseal, &nonote);
        assert!(one.failed);
        assert_eq!(one.rows[0].verdict, IntentRangeVerdict::NoNote);
        assert!(intent_range_line(&one.rows[0]).contains("NoNote"));

        let one = between(&d, &nonote, &broken);
        assert!(one.failed);
        assert_eq!(one.rows[0].verdict, IntentRangeVerdict::Broken);
        assert!(intent_range_line(&one.rows[0]).contains("Broken"));

        let one = between(&d, &broken, &release);
        assert!(!one.failed, "release is exempt: {one:?}");
        assert_eq!(one.rows[0].verdict, IntentRangeVerdict::Exempt);
        assert!(
            intent_range_line(&one.rows[0]).contains("exempt (release)"),
            "{}",
            intent_range_line(&one.rows[0])
        );

        let one = between(&d, &release, &release_broken);
        assert!(
            !one.failed,
            "Release exempts a commit whose note would be Broken: {one:?}"
        );
        assert_eq!(one.rows[0].verdict, IntentRangeVerdict::Exempt);

        let all = check_intent_range(&d, &["HEAD"]).expect("HEAD");
        assert!(all.failed);
        let seen: Vec<&str> = all.rows.iter().map(|r| r.subject.as_str()).collect();
        assert!(
            !seen.contains(&"plain") && !seen.contains(&"mentioned"),
            "no-Intent commits are ignored, saw {seen:?}"
        );
        assert!(seen.contains(&"intact") && seen.contains(&"noseal"));
        let _ = plain;
    }

    #[test]
    fn g5_a_range_check_without_a_range_refuses() {
        let err = check_intent_range("/tmp", &[]).expect_err("no range");
        assert!(err.contains("range"), "{err}");
    }

    #[test]
    fn g5_commit_msg_hook() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let root = root.canonicalize().expect("repo root");
        let script = root.join(".githooks/test-commit-msg.sh");
        let out = Command::new("bash")
            .arg(&script)
            .current_dir(&root)
            .output()
            .expect("bash runs");
        assert!(
            out.status.success(),
            "commit-msg script failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    #[test]
    fn g5_pre_push_asks_seals_check_for_each_pushed_ref() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let root = root.canonicalize().expect("repo root");
        let script = root.join(".githooks/pre-push");
        let text = std::fs::read_to_string(&script).expect("pre-push");
        assert!(
            text.contains("intent seals check"),
            "pre-push does not call core intent seals check"
        );
        let dir = std::env::temp_dir().join(format!("g5-push-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let log = dir.join("calls");
        let fake = dir.join("core");
        std::fs::write(&fake, "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$CALLS\"\n").unwrap();
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        let zeros = "0000000000000000000000000000000000000000";
        let local = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let remote = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let newb = "cccccccccccccccccccccccccccccccccccccccc";
        let stdin = format!(
            "refs/heads/main {local} refs/heads/main {remote}\nrefs/heads/topic {newb} refs/heads/topic {zeros}\nrefs/heads/old {zeros} refs/heads/old {remote}\n"
        );
        let mut child = Command::new("bash")
            .arg("-c")
            .arg("source \"$1\"; check_pushed_refs \"$2\"")
            .arg("bash")
            .arg(&script)
            .arg(&fake)
            .current_dir(&root)
            .env("CALLS", &log)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("bash");
        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        let calls = std::fs::read_to_string(&log).unwrap_or_default();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            out.status.success(),
            "pre-push source failed\nstderr:\n{}\ncalls:\n{calls}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            calls,
            format!(
                "intent seals check {remote}..{local}\nintent seals check {newb} --not --remotes\n"
            )
        );
        let stdin_test = root.join(".githooks/test-pre-push.sh");
        let hooked = Command::new("bash")
            .arg(&stdin_test)
            .current_dir(&root)
            .output()
            .expect("bash runs");
        assert!(
            hooked.status.success(),
            "pre-push stdin test failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&hooked.stdout),
            String::from_utf8_lossy(&hooked.stderr)
        );
    }
}
