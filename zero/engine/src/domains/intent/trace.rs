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
use std::process::Command;

/// Read from git in `core_root`. Err when git could not run or exited non-zero; Ok is git's
/// stdout, untrimmed.
pub fn git_read(core_root: &str, args: &[&str]) -> Result<String, String> {
    run("git", core_root, args)
}

/// The one place trace spawns a process. The program is a parameter so a test can prove the
/// could-not-run answer without touching the real git.
fn run(program: &str, core_root: &str, args: &[&str]) -> Result<String, String> {
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
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Every finding id a Fixes: trailer names on the current branch, with the commit that named it.
/// Err when git could not be read.
pub fn fixes(core_root: &str) -> Result<Vec<(String, String)>, String> {
    let text = git_read(
        core_root,
        &[
            "log",
            "--format=%h%x09%(trailers:key=Fixes,valueonly,separator=%x2C)",
            "HEAD",
        ],
    )?;
    Ok(parse_fixes(&text))
}

/// One line per commit: the hash, a TAB, then the ids its Fixes: trailers name, comma-separated.
fn parse_fixes(text: &str) -> Vec<(String, String)> {
    let mut named = Vec::new();
    for line in text.lines() {
        if let Some((hash, ids)) = line.split_once('\t') {
            for id in ids.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                named.push((id.to_string(), hash.to_string()));
            }
        }
    }
    named
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

/// Closed is a stored fact and needs no git. Open and Fixed need git to have answered.
pub fn state_of(record: &Record, fixes: &Result<Vec<(String, String)>, String>) -> State {
    let closed = record.closed_without_fix.as_str();
    if closed != "no" && closed != "unknown" {
        return State::Closed(closed.to_string());
    }
    let named = match fixes {
        Ok(named) => named,
        Err(e) => return State::Unread(e.clone()),
    };
    let by: Vec<String> = named
        .iter()
        .filter(|(id, _)| id == &record.id)
        .map(|(_, hash)| hash.clone())
        .collect();
    if by.is_empty() {
        State::Open
    } else {
        State::Fixed(by)
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
fn seal_notes(core_root: &str) -> Result<String, String> {
    git_read(core_root, &["notes", "--ref=seals", "list"])
}

/// A commit's seal, re-checked against the plan text kept for it.
fn check_commit_seal(core_root: &str, notes: &str, hash: &str) -> Result<SealCheck, String> {
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
    use super::{parse_fixes, state_of, State};
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
