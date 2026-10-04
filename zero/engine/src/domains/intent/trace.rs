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
/// A finding, a commit or a seal as the target comes next (INT-266 questions b and c).
pub fn trace(ctx: &AppContext, target: &str) -> CoreResult<()> {
    ctx.capabilities.require(
        "intent",
        &[Capability::FilesystemReadHome, Capability::SpawnProcess],
    )?;
    let t = target.trim();
    let id = t.strip_prefix("INT-").unwrap_or(t);
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
        return Err(could_not(format!(
            "{} is not an intent; trace answers INT-x today, and findings, commits and seals come next",
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
