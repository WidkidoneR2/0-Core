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
/// F-x goes to trace_finding (question b). A commit or a seal as the target comes next (question c).
pub fn trace(ctx: &AppContext, target: &str) -> CoreResult<()> {
    ctx.capabilities.require(
        "intent",
        &[Capability::FilesystemReadHome, Capability::SpawnProcess],
    )?;
    let t = target.trim();
    if t.starts_with("F-") {
        return trace_finding(ctx, t);
    }
    let id = t.strip_prefix("INT-").unwrap_or(t);
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
        return Err(could_not(format!(
            "{} is neither an intent nor a finding; trace answers INT-x and F-x today, and commits and seals come next",
            t
        )));
    }
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
            let notes = git_read(core, &["notes", "--ref=seals", "list"])
                .map_err(|e| could_not(format!("the seal notes could not be read: {}", e)))?;
            for hash in &hashes {
                let subject = git_read(core, &["log", "-1", "--format=%s", hash.as_str()])
                    .map_err(could_not)?;
                let seal = trailer(core, hash, "Seal").map_err(could_not)?;
                let fingerprint = trailer(core, hash, "Fingerprint").map_err(could_not)?;
                let note = match note_for(&notes, hash) {
                    Some(blob) => {
                        Some(git_read(core, &["cat-file", "-p", blob]).map_err(could_not)?)
                    }
                    None => None,
                };
                let check = check_seal(
                    seal.as_deref(),
                    fingerprint.as_deref(),
                    note.as_deref().map(str::as_bytes),
                );
                println!(
                    "     {} {}  {}",
                    "fixed by".green(),
                    hash.bright_white(),
                    subject.trim()
                );
                let line = match check {
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
                        format!("predates the Seal rule: Fingerprint {}, not re-checkable", f)
                            .dimmed()
                            .to_string()
                    }
                    SealCheck::NoSeal => "no seal recorded on this commit".yellow().to_string(),
                };
                println!("          {}", line);
            }
            Ok(())
        }
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
