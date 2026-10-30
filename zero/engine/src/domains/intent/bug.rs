//! INT-300 -- the bug report. A finding record stores only what a person saw or decided; every
//! other word here is derived when asked (R3): state from git, never from a stored field.
//! `core bug list` is the first reader. The rows are plain values first, so a later viewer (a
//! terminal screen, a bar, a script) reads the same rows this one prints.

use super::findings::{severity_level, Record};
use super::trace::{state_of, State};
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use std::path::Path;

/// The four words a finding's state is ever called. `unknown` is a word, not a blank (G7).
pub const STATES: [&str; 4] = ["open", "solved", "closed", "unknown"];

/// One finding as the report reads it: the stored facts and the derived state, nothing else.
#[derive(Debug, PartialEq, Clone)]
pub struct Row {
    pub id: String,
    pub state: &'static str,
    pub severity: String,
    pub found_on: String,
    pub what: String,
    pub fixed_by: Vec<String>,
}

/// The state's report word. A fix is solved, a closure is closed, and git that could not be
/// read on an open record is unknown.
pub fn state_word(s: &State) -> &'static str {
    match s {
        State::Open => "open",
        State::Fixed(_) => "solved",
        State::Closed(_) => "closed",
        State::Unread(_) => "unknown",
    }
}

/// Every record as a row, in the order given, each state derived by the one `state_of`.
pub fn rows(records: &[Record], fixes: &Result<Vec<(String, String)>, String>) -> Vec<Row> {
    records
        .iter()
        .map(|r| {
            let state = state_of(r, fixes);
            Row {
                id: r.id.clone(),
                state: state_word(&state),
                severity: r.severity.clone(),
                found_on: r.found_on.clone(),
                what: r.what.clone(),
                fixed_by: match state {
                    State::Fixed(hashes) => hashes,
                    _ => Vec::new(),
                },
            }
        })
        .collect()
}

/// How many rows read each state.
#[derive(Debug, PartialEq, Default)]
pub struct Counts {
    pub open: usize,
    pub solved: usize,
    pub closed: usize,
    pub unknown: usize,
}

impl Counts {
    pub fn total(&self) -> usize {
        self.open + self.solved + self.closed + self.unknown
    }
}

pub fn counts(rows: &[Row]) -> Counts {
    let mut c = Counts::default();
    for r in rows {
        match r.state {
            "open" => c.open += 1,
            "solved" => c.solved += 1,
            "closed" => c.closed += 1,
            _ => c.unknown += 1,
        }
    }
    c
}

/// What a list shows. No state and no `all` is the unresolved: open and unknown.
#[derive(Debug, PartialEq)]
pub struct Filter {
    pub state: Option<&'static str>,
    pub severity: Option<&'static str>,
    pub all: bool,
}

/// Read the flags as words. A state or severity nobody ruled is refused, naming the four.
pub fn filter_from(
    state: Option<&str>,
    severity: Option<&str>,
    all: bool,
) -> Result<Filter, String> {
    let state = match state {
        None => None,
        Some(s) => {
            let s = s.trim();
            Some(STATES.iter().copied().find(|w| *w == s).ok_or_else(|| {
                format!("{:?} is not a state; the four are {}", s, STATES.join(", "))
            })?)
        }
    };
    let severity = match severity {
        None => None,
        Some(s) => Some(severity_level(s.trim())?),
    };
    Ok(Filter {
        state,
        severity,
        all,
    })
}

fn rank(severity: &str) -> usize {
    match severity {
        "high" => 0,
        "medium" => 1,
        "low" => 2,
        _ => 3,
    }
}

/// The rows a list shows: filtered, then worst first, then by id.
pub fn select(rows: Vec<Row>, f: &Filter) -> Vec<Row> {
    let mut out: Vec<Row> = rows
        .into_iter()
        .filter(|r| match f.state {
            Some(s) => r.state == s,
            None => f.all || r.state == "open" || r.state == "unknown",
        })
        .filter(|r| f.severity.map_or(true, |s| r.severity == s))
        .collect();
    out.sort_by(|a, b| {
        rank(&a.severity)
            .cmp(&rank(&b.severity))
            .then_with(|| a.id.cmp(&b.id))
    });
    out
}

/// One line per row: id, state, severity, the day it was found, what was found.
pub fn render_list(rows: &[Row]) -> String {
    rows.iter()
        .map(|r| {
            format!(
                "{}  {:<7}  {:<7}  {}  {}\n",
                r.id, r.state, r.severity, r.found_on, r.what
            )
        })
        .collect()
}

/// The count line. The total is the sum of the four, never a separate number.
pub fn render_summary(c: &Counts, shown: usize) -> String {
    format!(
        "{} findings: {} open, {} solved, {} closed, {} unknown -- showing {}",
        c.total(),
        c.open,
        c.solved,
        c.closed,
        c.unknown,
        shown
    )
}

/// The recount: finding files on disk, counted by name alone and not by the record reader, so
/// the two numbers can disagree and say so.
pub fn files_on_disk(dir: &Path) -> Result<usize, String> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(format!("could not read {}: {}", dir.display(), e)),
    };
    let mut n = 0;
    for entry in entries {
        let entry = entry.map_err(|e| format!("could not read {}: {}", dir.display(), e))?;
        let name = entry.file_name().to_string_lossy().to_string();
        let digits = name
            .strip_prefix("F-")
            .and_then(|rest| rest.strip_suffix(".md"));
        if let Some(d) = digits {
            if d.len() >= 4 && d.bytes().all(|b| b.is_ascii_digit()) {
                n += 1;
            }
        }
    }
    Ok(n)
}

/// What `core bug list` prints and the exit it ends with, as a value, so both can be tested
/// without a terminal: 0 clean, 1 the recount disagrees, 3 git could not be read.
pub struct Report {
    pub text: String,
    pub code: i32,
}

pub fn report(
    records: &[Record],
    fixes: &Result<Vec<(String, String)>, String>,
    filter: &Filter,
    on_disk: usize,
) -> Report {
    let all = rows(records, fixes);
    let c = counts(&all);
    let shown = select(all, filter);
    let mut text = render_list(&shown);
    text.push_str(&render_summary(&c, shown.len()));
    text.push('\n');
    let agree = on_disk == records.len();
    text.push_str(&format!(
        "recount: {} finding files on disk, {} records read -- they {}\n",
        on_disk,
        records.len(),
        if agree { "agree" } else { "disagree" }
    ));
    let mut code = if agree { 0 } else { 1 };
    if let Err(e) = fixes {
        text.push_str(&format!(
            "git could not be read ({}): open findings read unknown\n",
            e
        ));
        code = 3;
    }
    Report { text, code }
}

/// `core bug list [--state S] [--severity L] [--all]` -- INT-300 G5. The findings ledger as a
/// report: the unresolved first, worst first, with the count line and its recount. The domain
/// owns its exit, as `core intent seals` does: 0 clean, 1 the recount disagrees, 3 could not
/// answer.
pub fn list(
    ctx: &AppContext,
    state: Option<&str>,
    severity: Option<&str>,
    all: bool,
) -> CoreResult<()> {
    let code = match list_answer(ctx, state, severity, all) {
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

fn list_answer(
    ctx: &AppContext,
    state: Option<&str>,
    severity: Option<&str>,
    all: bool,
) -> Result<i32, String> {
    let filter = filter_from(state, severity, all)?;
    let dir = super::intents_dir(ctx).join("findings");
    let records = super::findings::read_records(&dir)?;
    let on_disk = files_on_disk(&dir)?;
    let core: &str = &ctx.core_root;
    let named = super::trace::fixes(core);
    let got = report(&records, &named, &filter, on_disk);
    print!("{}", got.text);
    Ok(got.code)
}

#[cfg(test)]
mod bug_tests {
    use super::{
        counts, files_on_disk, filter_from, render_list, render_summary, report, rows, select,
        state_word, Counts, Row,
    };
    use crate::domains::intent::findings::{parse_record, read_records, Record};
    use crate::domains::intent::trace::State;

    fn rec(id: &str, closed: &str, severity: &str, day: &str, what: &str) -> Record {
        parse_record(&format!(
            "---\nid: {}\nwhat: {}\nat: unknown\nfound_by: INT-1\nfound_on: {}\nclosed_without_fix: {}\nseverity: {}\n---\n",
            id, what, day, closed, severity
        ))
    }

    fn row(id: &str, state: &'static str, severity: &str) -> Row {
        Row {
            id: id.to_string(),
            state,
            severity: severity.to_string(),
            found_on: "2026-10-30".to_string(),
            what: "w".to_string(),
            fixed_by: Vec::new(),
        }
    }

    /// INT-300 G5: the report's four state words, one per derived state. A fix is solved, a
    /// closure is closed, git that could not be read on an open record is unknown.
    #[test]
    fn every_state_has_its_word() {
        assert_eq!(state_word(&State::Open), "open");
        assert_eq!(state_word(&State::Fixed(vec!["abc".into()])), "solved");
        assert_eq!(state_word(&State::Closed("dup".into())), "closed");
        assert_eq!(state_word(&State::Unread("no git".into())), "unknown");
    }

    /// INT-300 G5: a row carries the stored facts and the state state_of derives, and a solved
    /// row names the commits that fixed it.
    #[test]
    fn rows_carry_the_facts_and_the_derived_state() {
        let records = vec![
            rec("F-0001", "no", "high", "2026-10-01", "broke"),
            rec("F-0002", "no", "unrated", "2026-10-02", "fixed one"),
            rec("F-0003", "duplicate of F-0001", "low", "2026-10-03", "dup"),
        ];
        let fixes = Ok(vec![("F-0002".to_string(), "abc1234".to_string())]);
        let got = rows(&records, &fixes);
        assert_eq!(got.len(), 3);
        assert_eq!(got[0].state, "open");
        assert_eq!(got[0].severity, "high");
        assert_eq!(got[0].found_on, "2026-10-01");
        assert_eq!(got[0].what, "broke");
        assert_eq!(got[1].state, "solved");
        assert_eq!(got[1].fixed_by, vec!["abc1234".to_string()]);
        assert_eq!(got[2].state, "closed");
        assert!(got[2].fixed_by.is_empty());
    }

    /// INT-300 G7: git that cannot be read never turns an open record into an answer. The open
    /// record reads unknown and the closed one still reads closed.
    #[test]
    fn unreadable_git_reads_unknown_for_open_and_closed_for_closed() {
        let records = vec![
            rec("F-0001", "no", "unrated", "2026-10-01", "a"),
            rec("F-0002", "duplicate", "unrated", "2026-10-02", "b"),
        ];
        let got = rows(&records, &Err("git could not run".to_string()));
        assert_eq!(got[0].state, "unknown");
        assert_eq!(got[1].state, "closed");
    }

    /// INT-300 G5: the four counts add up to the rows, and the summary line says the total as
    /// their sum with the number shown beside it.
    #[test]
    fn the_counts_add_up_and_the_summary_says_so() {
        let all = vec![
            row("F-0001", "open", "high"),
            row("F-0002", "open", "low"),
            row("F-0003", "solved", "low"),
            row("F-0004", "closed", "unrated"),
            row("F-0005", "unknown", "unrated"),
        ];
        let c = counts(&all);
        assert_eq!(
            c,
            Counts {
                open: 2,
                solved: 1,
                closed: 1,
                unknown: 1
            }
        );
        assert_eq!(c.total(), all.len());
        assert_eq!(
            render_summary(&c, 3),
            "5 findings: 2 open, 1 solved, 1 closed, 1 unknown -- showing 3"
        );
    }

    /// INT-300 G5: no flag shows the unresolved (open and unknown), worst first and then by id.
    /// --all shows every row. --state picks one state. --severity narrows.
    #[test]
    fn the_default_list_is_the_unresolved_worst_first() {
        let all = || {
            vec![
                row("F-0001", "open", "low"),
                row("F-0002", "solved", "high"),
                row("F-0003", "open", "high"),
                row("F-0004", "closed", "medium"),
                row("F-0005", "unknown", "medium"),
                row("F-0006", "open", "unrated"),
            ]
        };
        let ids = |v: Vec<Row>| v.into_iter().map(|r| r.id).collect::<Vec<_>>();
        let default = filter_from(None, None, false).expect("no flags is a filter");
        assert_eq!(
            ids(select(all(), &default)),
            vec!["F-0003", "F-0005", "F-0001", "F-0006"]
        );
        let every = filter_from(None, None, true).expect("all is a filter");
        assert_eq!(select(all(), &every).len(), 6);
        let solved = filter_from(Some("solved"), None, false).expect("solved is a state");
        assert_eq!(ids(select(all(), &solved)), vec!["F-0002"]);
        let high = filter_from(None, Some("high"), true).expect("high is a level");
        assert_eq!(ids(select(all(), &high)), vec!["F-0002", "F-0003"]);
    }

    /// INT-300 G5: a state or a severity nobody ruled is refused and the refusal names the words
    /// that are.
    #[test]
    fn a_word_nobody_ruled_is_refused_naming_the_real_ones() {
        let state = filter_from(Some("done"), None, false).expect_err("done is not a state");
        assert!(state.contains("open, solved, closed, unknown"), "{}", state);
        let sev = filter_from(None, Some("urgent"), false).expect_err("urgent is not a level");
        assert!(sev.contains("unrated, low, medium, high"), "{}", sev);
        assert!(
            filter_from(Some(" open "), Some(" low "), false).is_ok(),
            "words are trimmed"
        );
    }

    /// INT-300 G5: a list line carries every field of its row, whole, and one line per row.
    #[test]
    fn a_list_line_carries_every_field_of_its_row() {
        let r = Row {
            id: "F-0111".to_string(),
            state: "open",
            severity: "low".to_string(),
            found_on: "2026-10-30".to_string(),
            what: "cargo doc reports one warning, from the doc comment on core intent gates"
                .to_string(),
            fixed_by: Vec::new(),
        };
        let text = render_list(&[r.clone(), row("F-0112", "unknown", "high")]);
        assert_eq!(text.lines().count(), 2);
        let first = text.lines().next().unwrap();
        for part in [
            "F-0111",
            "open",
            "low",
            "2026-10-30",
            "cargo doc reports one warning, from the doc comment on core intent gates",
        ] {
            assert!(first.contains(part), "{} missing from {:?}", part, first);
        }
    }

    fn temp(tag: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("int300-{}-{}-{}", tag, std::process::id(), nanos));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    /// INT-300 G5: the recount counts finding files by name alone: F- and four or more digits
    /// and .md. Notes, drafts and other names are not findings.
    #[test]
    fn the_recount_counts_only_finding_files() {
        let dir = temp("recount");
        for name in [
            "F-0001.md",
            "F-0002.md",
            "F-10000.md",
            "notes.md",
            "F-12.md",
            "F-0003.txt",
            "F-00a4.md",
        ] {
            std::fs::write(dir.join(name), "x").expect("seed");
        }
        let got = files_on_disk(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(got, Ok(3));
        assert_eq!(
            files_on_disk(std::path::Path::new("/nonexistent/int300-nowhere")),
            Ok(0)
        );
    }

    /// INT-300 G5: the recount holds on the real ledger. The finding files counted by name and
    /// the records read by the record reader are the same number.
    #[test]
    fn the_real_ledger_recounts_to_the_records_read() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../intents/findings");
        let read = read_records(&dir)
            .expect("the real findings folder reads")
            .len();
        assert!(read > 0, "the real folder holds records");
        assert_eq!(files_on_disk(&dir), Ok(read));
    }

    /// INT-300 G5: the report lists the unresolved, then the count line over all of them, then
    /// the recount. A clean answer ends 0.
    #[test]
    fn the_report_lists_counts_and_recounts() {
        let records = vec![
            rec("F-0001", "no", "high", "2026-10-01", "broke"),
            rec("F-0002", "no", "unrated", "2026-10-02", "fixed one"),
            rec("F-0003", "duplicate of F-0001", "low", "2026-10-03", "dup"),
        ];
        let fixes = Ok(vec![("F-0002".to_string(), "abc1234".to_string())]);
        let filter = filter_from(None, None, false).expect("no flags is a filter");
        let got = report(&records, &fixes, &filter, 3);
        assert_eq!(got.code, 0, "{}", got.text);
        assert!(got.text.starts_with("F-0001  open "), "{}", got.text);
        assert!(
            !got.text.contains("F-0002  "),
            "a solved row is not unresolved"
        );
        assert!(
            got.text
                .contains("3 findings: 1 open, 1 solved, 1 closed, 0 unknown -- showing 1"),
            "{}",
            got.text
        );
        assert!(
            got.text
                .contains("recount: 3 finding files on disk, 3 records read -- they agree"),
            "{}",
            got.text
        );
    }

    /// INT-300 G5: when the files on disk and the records read are not the same number the
    /// report says so and ends 1, because a count that cannot explain itself is a finding.
    #[test]
    fn the_report_says_so_when_the_recount_disagrees() {
        let records = vec![rec("F-0001", "no", "low", "2026-10-01", "a")];
        let filter = filter_from(None, None, false).expect("filter");
        let got = report(&records, &Ok(Vec::new()), &filter, 2);
        assert_eq!(got.code, 1, "{}", got.text);
        assert!(
            got.text
                .contains("recount: 2 finding files on disk, 1 records read -- they disagree"),
            "{}",
            got.text
        );
    }

    /// INT-300 G7: git that cannot be read is said, the open finding reads unknown in its row,
    /// and the report ends 3, could not answer, never 0.
    #[test]
    fn the_report_with_unreadable_git_says_unknown_and_ends_3() {
        let records = vec![rec("F-0001", "no", "low", "2026-10-01", "a")];
        let filter = filter_from(None, None, false).expect("filter");
        let got = report(&records, &Err("git could not run".to_string()), &filter, 1);
        assert_eq!(got.code, 3, "{}", got.text);
        assert!(got.text.starts_with("F-0001  unknown"), "{}", got.text);
        assert!(got.text.contains("git could not be read"), "{}", got.text);
    }

    /// INT-300 G5: an empty ledger is a true answer: zero findings, zero shown, exit 0.
    #[test]
    fn the_report_of_an_empty_ledger_is_zero_and_clean() {
        let filter = filter_from(None, None, true).expect("filter");
        let got = report(&[], &Ok(Vec::new()), &filter, 0);
        assert_eq!(got.code, 0, "{}", got.text);
        assert!(
            got.text
                .contains("0 findings: 0 open, 0 solved, 0 closed, 0 unknown -- showing 0"),
            "{}",
            got.text
        );
    }
}
