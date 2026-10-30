//! INT-300 -- the bug report. A finding record stores only what a person saw or decided; every
//! other word here is derived when asked (R3): state from git, never from a stored field.
//! `core bug list` is the first reader. The rows are plain values first, so a later viewer (a
//! terminal screen, a bar, a script) reads the same rows this one prints.

use super::findings::{severity_level, with_severity, Record};
use super::trace::{
    check_commit_seal, git_read, seal_digest, seal_notes, state_of, SealCheck, State, Trailers,
};
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use std::fs;
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

/// One commit that fixed a finding, as its story tells it: the day it was committed, its subject,
/// and its seal re-checked against the plan text kept for it.
#[derive(Debug, PartialEq)]
pub struct FixFacts {
    pub hash: String,
    pub day: String,
    pub subject: String,
    pub seal: SealCheck,
}

/// What `core bug show` prints and the exit it ends with, as a value: 0 shown, 1 a seal is broken,
/// 3 the story could not be told.
pub struct Story {
    pub text: String,
    pub code: i32,
}

/// A seal in the report's own words. A fix with no Seal reads unsealed, whether it predates the
/// rule or never carried one (G7); only an intact seal checks out.
pub fn seal_phrase(check: &SealCheck) -> String {
    match check {
        SealCheck::Intact(s) => format!(
            "seal {} intact -- the plan kept in refs/notes/seals hashes to it",
            s
        ),
        SealCheck::Broken { seal, got } => format!(
            "seal {} BROKEN -- the plan kept in refs/notes/seals hashes to {}",
            seal, got
        ),
        SealCheck::NoNote(s) => format!(
            "seal {} recorded, no plan kept in refs/notes/seals -- cannot be re-checked",
            s
        ),
        SealCheck::Legacy(f) => format!(
            "unsealed -- the commit predates the Seal rule (Fingerprint {})",
            f
        ),
        SealCheck::NoSeal => "unsealed -- no Seal recorded on this commit".to_string(),
    }
}

/// One finding's whole story as a value: the record, then a timeline from found to solved to
/// sealed. Unknown is a word here: a record with no fix has no solved day, git that could not be
/// read says unknown and ends 3, and nothing prints a zero or a default (G7).
pub fn story(record: &Record, state: &State, facts: &[FixFacts]) -> Story {
    let mut sorted: Vec<&FixFacts> = facts.iter().collect();
    sorted.sort_by(|a, b| a.day.cmp(&b.day));
    let mut code = 0;
    let mut t = format!("{}  {}\n", record.id, record.what);
    t.push_str(&format!("  where     {}\n", record.at));
    t.push_str(&format!("  severity  {}\n", record.severity));
    t.push_str(&format!("  state     {}\n", state_word(state)));
    match state {
        State::Open => t.push_str(&format!(
            "  solved    none -- no commit on HEAD carries Fixes: {}\n",
            record.id
        )),
        State::Closed(why) => {
            t.push_str(&format!("  closed    {}\n", why));
            t.push_str("  solved    none -- closed without a fix\n");
        }
        State::Unread(e) => {
            t.push_str(&format!(
                "  solved    unknown -- git could not be read ({})\n",
                e
            ));
            code = 3;
        }
        State::Fixed(_) => match sorted.first() {
            Some(first) => t.push_str(&format!("  solved    {}\n", first.day)),
            None => {
                t.push_str("  solved    unknown -- the fixing commit could not be shown\n");
                code = 3;
            }
        },
    }
    t.push_str("  timeline\n");
    t.push_str(&format!(
        "    {:10}  found by {}\n",
        record.found_on, record.found_by
    ));
    for f in &sorted {
        t.push_str(&format!(
            "    {:10}  fixed by {}  {}\n",
            f.day, f.hash, f.subject
        ));
        t.push_str(&format!("    {:10}  {}\n", "", seal_phrase(&f.seal)));
        if matches!(f.seal, SealCheck::Broken { .. }) && code == 0 {
            code = 1;
        }
    }
    Story { text: t, code }
}

/// The facts of each fixing commit: its committed day and subject from git, its seal re-checked
/// by the one reader in trace.rs. Err when git cannot show one.
fn fix_facts(core: &str, hashes: &[String]) -> Result<Vec<FixFacts>, String> {
    let notes = seal_notes(core).map_err(|e| format!("the seal notes could not be read: {}", e))?;
    hashes
        .iter()
        .map(|h| {
            let head = git_read(core, &["log", "-1", "--format=%cs%x09%s", h.as_str()])?;
            let (day, subject) = head
                .trim()
                .split_once('\t')
                .ok_or_else(|| format!("git log gave no header for {}", h))?;
            Ok(FixFacts {
                hash: h.clone(),
                day: day.to_string(),
                subject: subject.to_string(),
                seal: check_commit_seal(core, &notes, h)?,
            })
        })
        .collect()
}

/// `core bug show F-NNNN` -- INT-300 G6. One finding's whole story. The domain owns its exit:
/// 0 shown, 1 a seal is broken, 3 the finding or git could not be read.
pub fn show(ctx: &AppContext, id: &str) -> CoreResult<()> {
    let code = match show_answer(ctx, id) {
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

fn show_answer(ctx: &AppContext, id: &str) -> Result<i32, String> {
    let dir = super::intents_dir(ctx).join("findings");
    let records = super::findings::read_records(&dir)?;
    let record = records
        .iter()
        .find(|r| r.id == id)
        .ok_or_else(|| format!("{} names no finding in {}", id, dir.display()))?;
    let core: &str = &ctx.core_root;
    let named = super::trace::fixes(core);
    let state = state_of(record, &named);
    let facts = match &state {
        State::Fixed(hashes) => fix_facts(core, hashes)?,
        _ => Vec::new(),
    };
    let got = story(record, &state, &facts);
    print!("{}", got.text);
    Ok(got.code)
}

/// The kinds of trouble `core bug check` reports, in the order its totals line counts them.
pub const KINDS: [&str; 5] = [
    "malformed-fixes",
    "fixes-no-record",
    "closed-and-fixed",
    "missing-field",
    "no-location",
];

/// One thing `core bug check` found wrong: the finding or commit it is about, its kind, and the
/// detail that lets a person go and look.
#[derive(Debug, PartialEq)]
pub struct Problem {
    pub subject: String,
    pub kind: &'static str,
    pub detail: String,
}

/// What `core bug check` prints and the exit it ends with, as a value: 0 clean, 1 problems
/// found, 3 the ledger could not be checked.
pub struct Checked {
    pub problems: Vec<Problem>,
    pub text: String,
    pub code: i32,
}

/// INT-300 G8. The ledger checked against itself and git: every Fixes value is exactly one
/// finding id, every id a Fixes names has a record, no closed finding also has a fix (R3: the fix
/// wins, so the closure is stale), and every record holds its fields and a location. Unknown is a
/// word: git that could not be read ends 3 and says so, never a clean ledger.
pub fn check(records: &[Record], trailers: &Result<Trailers, String>) -> Checked {
    let t = match trailers {
        Ok(t) => t,
        Err(e) => {
            return Checked {
                problems: Vec::new(),
                text: format!(
                    "unknown -- git could not be read ({}); nothing was checked\n",
                    e
                ),
                code: 3,
            };
        }
    };
    let mut problems = Vec::new();
    for (value, hash) in &t.other {
        problems.push(Problem {
            subject: hash.clone(),
            kind: "malformed-fixes",
            detail: format!(
                "Fixes: {} is not exactly F-NNNN, so it names no finding",
                value
            ),
        });
    }
    for (id, hash) in &t.named {
        if !records.iter().any(|r| &r.id == id) {
            problems.push(Problem {
                subject: hash.clone(),
                kind: "fixes-no-record",
                detail: format!("Fixes: {} names a finding with no record", id),
            });
        }
    }
    let nothing_fixed: Result<Vec<(String, String)>, String> = Ok(Vec::new());
    for r in records {
        let fixed = t.named.iter().any(|(id, _)| id == &r.id);
        if fixed && matches!(state_of(r, &nothing_fixed), State::Closed(_)) {
            problems.push(Problem {
                subject: r.id.clone(),
                kind: "closed-and-fixed",
                detail: "closed without a fix, yet a commit fixes it; the fix wins".to_string(),
            });
        }
        for (name, value) in [
            ("what", &r.what),
            ("found_by", &r.found_by),
            ("found_on", &r.found_on),
        ]
        .iter()
        {
            if value.as_str() == "unknown" {
                problems.push(Problem {
                    subject: r.id.clone(),
                    kind: "missing-field",
                    detail: format!("{} is missing from the record", name),
                });
            }
        }
        if r.at == "unknown" {
            problems.push(Problem {
                subject: r.id.clone(),
                kind: "no-location",
                detail: "no location (at) recorded".to_string(),
            });
        }
    }
    let mut text = String::new();
    for p in &problems {
        text.push_str(&format!("{:16}  {:8}  {}\n", p.kind, p.subject, p.detail));
    }
    if problems.is_empty() {
        text.push_str(&format!(
            "clean -- {} records and {} Fixes values read, nothing wrong\n",
            records.len(),
            t.named.len() + t.other.len()
        ));
    } else {
        let per: Vec<String> = KINDS
            .iter()
            .map(|k| format!("{} {}", problems.iter().filter(|p| p.kind == *k).count(), k))
            .collect();
        let noun = if problems.len() == 1 {
            "problem"
        } else {
            "problems"
        };
        text.push_str(&format!(
            "{} {} -- {}\n",
            problems.len(),
            noun,
            per.join(", ")
        ));
    }
    let code = if problems.is_empty() { 0 } else { 1 };
    Checked {
        problems,
        text,
        code,
    }
}

/// `core bug check` -- INT-300 G8. The domain owns its exit: 0 clean, 1 problems found, 3 the
/// ledger or git could not be read.
pub fn check_ledger(ctx: &AppContext) -> CoreResult<()> {
    let code = match check_answer(ctx) {
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

fn check_answer(ctx: &AppContext) -> Result<i32, String> {
    let dir = super::intents_dir(ctx).join("findings");
    let records = super::findings::read_records(&dir)?;
    let core: &str = &ctx.core_root;
    let got = check(&records, &super::trace::trailers(core));
    print!("{}", got.text);
    Ok(got.code)
}

/// One rating as a plan: the exact change, the hashes of the file before and after, and the Seal
/// of that text (INT-300 G9, R5). Nothing is written until the same plan is applied by its Seal.
#[derive(Debug, PartialEq)]
pub struct RatePlan {
    pub text: String,
    pub seal: String,
    pub new_body: String,
}

/// What `core bug rate` did: printed a plan, applied it, or refused. A refusal writes nothing.
#[derive(Debug, PartialEq)]
pub enum Rated {
    Planned(RatePlan),
    Applied(RatePlan),
    Refused(String),
}

/// The plan for rating one finding: refuses a level nobody ruled and a level the record already
/// has, otherwise names the change, the file's hash before and after, and the Seal of that text.
pub fn rate_plan(
    path: &str,
    record: &Record,
    old_body: &str,
    level: &str,
) -> Result<RatePlan, String> {
    let level = severity_level(level.trim())?;
    if record.severity == level {
        return Err(format!(
            "{} already reads {}; there is nothing to rate",
            record.id, level
        ));
    }
    let (new_body, old_line) = with_severity(old_body, level)?;
    let minus = old_line.unwrap_or_else(|| "(no severity line -- it reads unrated)".to_string());
    let text = format!(
        "rate {}: severity {} -> {}\nfile {}  sha256 {} -> {}\n-- {}\n++ severity: {}",
        record.id,
        record.severity,
        level,
        path,
        seal_digest(old_body.as_bytes()),
        seal_digest(new_body.as_bytes()),
        minus,
        level
    );
    let seal = seal_digest(text.as_bytes());
    Ok(RatePlan {
        text,
        seal,
        new_body,
    })
}

/// `core bug rate` as a value. With no words after the level it plans; with `apply SEAL` it
/// rebuilds the plan from the file as it is now and writes only if that plan still has the given
/// Seal. Refused means nothing was written; Err means the ledger could not be read or written.
pub fn rate(dir: &Path, id: &str, level: &str, apply: &[String]) -> Result<Rated, String> {
    let seal = match apply {
        [] => None,
        [word, seal] if word == "apply" => Some(seal.as_str()),
        _ => {
            return Ok(Rated::Refused(
                "after the level say nothing to see the plan, or: apply SEAL".to_string(),
            ));
        }
    };
    let level = match severity_level(level.trim()) {
        Ok(l) => l,
        Err(e) => return Ok(Rated::Refused(e)),
    };
    let records = super::findings::read_records(dir)?;
    let record = records
        .iter()
        .find(|r| r.id == id)
        .ok_or_else(|| format!("{} names no finding in {}", id, dir.display()))?;
    let path = dir.join(format!("{}.md", id));
    let old_body = fs::read_to_string(&path)
        .map_err(|e| format!("could not read {}: {}", path.display(), e))?;
    let plan = match rate_plan(&path.display().to_string(), record, &old_body, level) {
        Ok(p) => p,
        Err(e) => return Ok(Rated::Refused(e)),
    };
    match seal {
        None => Ok(Rated::Planned(plan)),
        Some(given) if given == plan.seal => {
            write_rated(&path, &plan)?;
            Ok(Rated::Applied(plan))
        }
        Some(given) => Ok(Rated::Refused(format!(
            "the plan now hashes to {}, not {}: the record or the level changed",
            plan.seal, given
        ))),
    }
}

/// Write the planned body beside the record, move it into place and read it back. A record is
/// never left half written.
fn write_rated(path: &Path, plan: &RatePlan) -> Result<(), String> {
    let tmp = path.with_extension("md.tmp");
    fs::write(&tmp, &plan.new_body)
        .map_err(|e| format!("could not write {}: {}", tmp.display(), e))?;
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("could not replace {}: {}", path.display(), e)
    })?;
    let back = fs::read_to_string(path)
        .map_err(|e| format!("could not read back {}: {}", path.display(), e))?;
    if back != plan.new_body {
        return Err(format!(
            "{} does not read back as the plan wrote it",
            path.display()
        ));
    }
    Ok(())
}

/// What `core bug rate` prints and the exit it ends with: the plan with its Seal and the line
/// that applies it, the applied change, or a refusal.
pub fn rated_text(got: &Rated, id: &str, level: &str) -> (String, i32) {
    match got {
        Rated::Planned(p) => (
            format!(
                "{}\nDRY RUN -- nothing written. Seal: {}\nto write it: core bug rate {} {} apply {}\n",
                p.text,
                p.seal,
                id,
                level.trim(),
                p.seal
            ),
            0,
        ),
        Rated::Applied(p) => (
            format!("{}\nOK {} rated, verified on disk\n", p.text, id),
            0,
        ),
        Rated::Refused(why) => (format!("REFUSE {}. Nothing was written.\n", why), 1),
    }
}

/// `core bug rate F-NNNN LEVEL [apply SEAL]` -- INT-300 G9. The domain owns its exit: 0 planned
/// or applied, 1 refused (nothing written), 3 the finding or ledger could not be read or written.
pub fn rate_ledger(ctx: &AppContext, id: &str, level: &str, apply: &[String]) -> CoreResult<()> {
    let dir = super::intents_dir(ctx).join("findings");
    let code = match rate(&dir, id, level, apply) {
        Ok(got) => {
            let (text, code) = rated_text(&got, id, level);
            if code == 0 {
                print!("{}", text);
            } else {
                eprint!("{}", text);
            }
            code
        }
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

#[cfg(test)]
mod bug_tests {
    use super::{
        check, counts, files_on_disk, filter_from, rate, rate_plan, rated_text, render_list,
        render_summary, report, rows, select, state_word, story, Counts, FixFacts, Rated, Row,
    };
    use crate::domains::intent::findings::{parse_record, read_records, with_severity, Record};
    use crate::domains::intent::trace::{seal_digest, SealCheck, State, Trailers};

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

    fn full(id: &str, closed: &str, at: &str) -> Record {
        parse_record(&format!(
            "---\nid: {}\nwhat: w\nat: {}\nfound_by: INT-1\nfound_on: 2026-10-30\nclosed_without_fix: {}\nseverity: low\n---\n",
            id, at, closed
        ))
    }

    fn trailers(named: &[(&str, &str)], other: &[(&str, &str)]) -> Result<Trailers, String> {
        let own = |v: &[(&str, &str)]| -> Vec<(String, String)> {
            v.iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect()
        };
        Ok(Trailers {
            named: own(named),
            other: own(other),
        })
    }

    /// INT-300 G8: a ledger with nothing wrong ends 0 and says what it read, so clean is a count
    /// and never silence.
    #[test]
    fn a_clean_ledger_ends_0_and_counts_what_it_read() {
        let records = vec![
            full("F-0001", "no", "a.rs:1"),
            full("F-0002", "no", "b.rs:2"),
        ];
        let got = check(&records, &trailers(&[("F-0001", "abc1234")], &[]));
        assert_eq!(got.code, 0, "{}", got.text);
        assert!(got.problems.is_empty());
        assert!(
            got.text
                .contains("clean -- 2 records and 1 Fixes values read, nothing wrong"),
            "{}",
            got.text
        );
    }

    /// INT-300 G8, R4: a Fixes value that is not exactly F-NNNN is reported with the commit that
    /// carries it, as 23c7310e carries F-0046 with prose after it.
    #[test]
    fn a_malformed_fixes_value_is_reported_with_its_commit() {
        let records = vec![full("F-0046", "no", "a.rs:1")];
        let got = check(&records, &trailers(&[], &[("F-0046 later", "23c7310e")]));
        assert_eq!(got.code, 1);
        assert_eq!(got.problems.len(), 1);
        assert_eq!(got.problems[0].kind, "malformed-fixes");
        assert_eq!(got.problems[0].subject, "23c7310e");
        assert!(got.text.contains("F-0046 later"), "{}", got.text);
    }

    /// INT-300 G8: a Fixes that names a finding nobody filed is reported with its commit.
    #[test]
    fn a_fixes_naming_a_finding_with_no_record_is_reported() {
        let records = vec![full("F-0001", "no", "a.rs:1")];
        let got = check(&records, &trailers(&[("F-0099", "abc1234")], &[]));
        assert_eq!(got.code, 1);
        assert_eq!(got.problems.len(), 1);
        assert_eq!(got.problems[0].kind, "fixes-no-record");
        assert_eq!(got.problems[0].subject, "abc1234");
    }

    /// INT-300 G8, R3: a closure that also has a fix is reported. A closure with no fix, and a
    /// fix on an open record, are not.
    #[test]
    fn a_closure_that_also_has_a_fix_is_reported_and_a_plain_closure_is_not() {
        let records = vec![
            full("F-0001", "duplicate of F-0002", "a.rs:1"),
            full("F-0002", "not a bug", "b.rs:2"),
            full("F-0003", "no", "c.rs:3"),
        ];
        let named = [("F-0001", "abc1234"), ("F-0003", "def5678")];
        let got = check(&records, &trailers(&named, &[]));
        assert_eq!(got.problems.len(), 1, "{}", got.text);
        assert_eq!(got.problems[0].kind, "closed-and-fixed");
        assert_eq!(got.problems[0].subject, "F-0001");
    }

    /// INT-300 G8: a record that lacks a field says which, and a record with no location is its
    /// own kind. Nothing is filled in with a default.
    #[test]
    fn a_record_missing_a_field_or_a_location_is_reported() {
        let bare = parse_record("---\nid: F-0001\nat: a.rs:1\n---\n");
        let records = vec![bare, full("F-0002", "no", "unknown")];
        let got = check(&records, &trailers(&[], &[]));
        let kinds: Vec<(&str, &str)> = got
            .problems
            .iter()
            .map(|p| (p.subject.as_str(), p.kind))
            .collect();
        assert_eq!(
            kinds,
            vec![
                ("F-0001", "missing-field"),
                ("F-0001", "missing-field"),
                ("F-0001", "missing-field"),
                ("F-0002", "no-location"),
            ]
        );
        assert!(got.text.contains("what is missing"), "{}", got.text);
        assert_eq!(got.code, 1);
    }

    /// INT-300 G7, G8: git that could not be read is not a clean ledger. It says unknown and
    /// ends 3, and finds no problems it did not see.
    #[test]
    fn an_unreadable_ledger_is_unknown_not_clean_and_ends_3() {
        let records = vec![full("F-0001", "no", "a.rs:1")];
        let got = check(&records, &Err("no git here".to_string()));
        assert_eq!(got.code, 3);
        assert!(got.problems.is_empty());
        assert!(got.text.contains("unknown"), "{}", got.text);
        assert!(got.text.contains("no git here"), "{}", got.text);
        assert!(!got.text.contains("clean"), "{}", got.text);
    }

    /// INT-300 G8: the totals line counts each kind, so a number always explains itself.
    #[test]
    fn the_totals_line_counts_each_kind() {
        let records = vec![full("F-0001", "no", "a.rs:1")];
        let got = check(
            &records,
            &trailers(&[("F-0099", "abc1234")], &[("F-0046 later", "23c7310e")]),
        );
        assert!(
            got.text.contains(
                "2 problems -- 1 malformed-fixes, 1 fixes-no-record, 0 closed-and-fixed, 0 missing-field, 0 no-location"
            ),
            "{}",
            got.text
        );
    }

    fn fact(hash: &str, day: &str, subject: &str, seal: SealCheck) -> FixFacts {
        FixFacts {
            hash: hash.to_string(),
            day: day.to_string(),
            subject: subject.to_string(),
            seal,
        }
    }

    fn at(text: &str, needle: &str) -> usize {
        text.find(needle)
            .unwrap_or_else(|| panic!("{:?} is not in:\n{}", needle, text))
    }

    /// INT-300 G6: a fixed finding tells found, then fixed, then sealed, in that order, with the
    /// fixing commit's subject and the day it was solved.
    #[test]
    fn a_fixed_finding_tells_found_then_solved_then_sealed() {
        let r = rec("F-0049", "no", "unrated", "2026-10-21", "a thing");
        let state = State::Fixed(vec!["825e4803".to_string()]);
        let facts = vec![fact(
            "825e4803",
            "2026-10-22",
            "fix the thing",
            SealCheck::Intact("eb4f6f578bcc9321".to_string()),
        )];
        let got = story(&r, &state, &facts);
        assert_eq!(got.code, 0, "{}", got.text);
        assert!(got.text.starts_with("F-0049  a thing\n"), "{}", got.text);
        assert!(got.text.contains("  state     solved\n"), "{}", got.text);
        assert!(
            got.text.contains("  solved    2026-10-22\n"),
            "{}",
            got.text
        );
        let found = at(&got.text, "2026-10-21  found by INT-1");
        let fixed = at(&got.text, "2026-10-22  fixed by 825e4803  fix the thing");
        let sealed = at(&got.text, "seal eb4f6f578bcc9321 intact");
        assert!(found < fixed && fixed < sealed, "{}", got.text);
    }

    /// INT-300 G6, G7: an unrated bug reads unrated, and a bug with no fix has no solved day: no
    /// date, no zero, no default after the word solved.
    #[test]
    fn an_open_unrated_finding_has_no_solved_day() {
        let r = rec("F-0001", "no", "unrated", "2026-10-04", "w");
        let got = story(&r, &State::Open, &[]);
        assert_eq!(got.code, 0, "{}", got.text);
        assert!(got.text.contains("  severity  unrated\n"), "{}", got.text);
        assert!(got.text.contains("  solved    none -- "), "{}", got.text);
        assert!(!got.text.contains("fixed by"), "{}", got.text);
        assert!(!got.text.contains("medium"), "{}", got.text);
    }

    /// INT-300 G7: a fix that carries no Seal reads unsealed, whether it predates the rule or
    /// never had one, and does not read intact.
    #[test]
    fn a_fix_without_a_seal_reads_unsealed() {
        let r = rec("F-0002", "no", "low", "2026-10-04", "w");
        let state = State::Fixed(vec!["abc1234".to_string(), "def5678".to_string()]);
        let facts = vec![
            fact("abc1234", "2026-10-05", "old", SealCheck::NoSeal),
            fact(
                "def5678",
                "2026-10-06",
                "older",
                SealCheck::Legacy("a b c".to_string()),
            ),
        ];
        let got = story(&r, &state, &facts);
        assert_eq!(got.code, 0, "{}", got.text);
        assert_eq!(got.text.matches("unsealed").count(), 2, "{}", got.text);
        assert!(!got.text.contains("intact"), "{}", got.text);
    }

    /// INT-300 G6: a seal whose kept plan hashes to something else reads BROKEN and the story
    /// ends 1; a seal with no kept plan says it cannot be re-checked and does not end 1.
    #[test]
    fn a_broken_seal_ends_1_and_a_missing_plan_says_so() {
        let r = rec("F-0003", "no", "low", "2026-10-04", "w");
        let state = State::Fixed(vec!["abc1234".to_string()]);
        let broken = vec![fact(
            "abc1234",
            "2026-10-05",
            "s",
            SealCheck::Broken {
                seal: "aaaa".to_string(),
                got: "bbbb".to_string(),
            },
        )];
        let got = story(&r, &state, &broken);
        assert_eq!(got.code, 1, "{}", got.text);
        assert!(got.text.contains("BROKEN"), "{}", got.text);
        let missing = vec![fact(
            "abc1234",
            "2026-10-05",
            "s",
            SealCheck::NoNote("aaaa".to_string()),
        )];
        let got = story(&r, &state, &missing);
        assert_eq!(got.code, 0, "{}", got.text);
        assert!(got.text.contains("cannot be re-checked"), "{}", got.text);
    }

    /// INT-300 G7: git that cannot be read says unknown, never open and never a date, and the
    /// story ends 3.
    #[test]
    fn git_that_cannot_be_read_says_unknown_and_ends_3() {
        let r = rec("F-0004", "no", "low", "2026-10-04", "w");
        let got = story(&r, &State::Unread("boom".to_string()), &[]);
        assert_eq!(got.code, 3, "{}", got.text);
        assert!(got.text.contains("  state     unknown\n"), "{}", got.text);
        assert!(
            got.text
                .contains("  solved    unknown -- git could not be read (boom)\n"),
            "{}",
            got.text
        );
        assert!(!got.text.contains("none"), "{}", got.text);
    }

    /// INT-300 G6: a closed finding shows its reason and has no solved day.
    #[test]
    fn a_closed_finding_shows_its_reason_and_no_solved_day() {
        let r = rec("F-0005", "duplicate of F-0002", "low", "2026-10-04", "w");
        let got = story(&r, &State::Closed("duplicate of F-0002".to_string()), &[]);
        assert_eq!(got.code, 0, "{}", got.text);
        assert!(
            got.text.contains("  closed    duplicate of F-0002\n"),
            "{}",
            got.text
        );
        assert!(got.text.contains("  solved    none -- "), "{}", got.text);
    }

    /// INT-300 G6: with more than one fixing commit the solved day is the earliest, and the
    /// timeline lists them oldest first whatever order they came in.
    #[test]
    fn the_solved_day_is_the_earliest_fix_and_the_timeline_runs_oldest_first() {
        let r = rec("F-0006", "no", "low", "2026-10-04", "w");
        let state = State::Fixed(vec!["bbbb".to_string(), "aaaa".to_string()]);
        let facts = vec![
            fact("bbbb", "2026-10-25", "second", SealCheck::NoSeal),
            fact("aaaa", "2026-10-22", "first", SealCheck::NoSeal),
        ];
        let got = story(&r, &state, &facts);
        assert!(
            got.text.contains("  solved    2026-10-22\n"),
            "{}",
            got.text
        );
        assert!(
            at(&got.text, "fixed by aaaa") < at(&got.text, "fixed by bbbb"),
            "{}",
            got.text
        );
    }

    const OLD: &str = "---\nid: F-0001\nwhat: w\nat: a.rs:1\nfound_by: INT-1\nfound_on: 2026-10-30\nclosed_without_fix: no\n---\n";
    const LOW: &str = "---\nid: F-0001\nwhat: w\nat: a.rs:1\nfound_by: INT-1\nfound_on: 2026-10-30\nclosed_without_fix: no\nseverity: low\n---\n";

    fn filed(name: &str, body: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("core-rate-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("F-0001.md"), body).unwrap();
        dir
    }

    fn words(w: &[&str]) -> Vec<String> {
        w.iter().map(|s| s.to_string()).collect()
    }

    /// INT-300 G9: the one rewriter of a record replaces the severity line and every other byte
    /// stays.
    #[test]
    fn with_severity_replaces_the_line_and_keeps_every_other_byte() {
        let (new, old) = with_severity(LOW, "high").unwrap();
        assert_eq!(new, LOW.replace("severity: low", "severity: high"));
        assert_eq!(old, Some("severity: low".to_string()));
    }

    /// INT-300 G9: a record filed before G3 has no severity line, so one is inserted before the
    /// closing line and the old line reads None.
    #[test]
    fn with_severity_inserts_before_the_closing_line_when_there_is_none() {
        let (new, old) = with_severity(OLD, "medium").unwrap();
        let want = format!("{}severity: medium\n---\n", &OLD[..OLD.len() - 4]);
        assert_eq!(new, want);
        assert_eq!(old, None);
        assert_eq!(parse_record(&new).severity, "medium");
    }

    /// INT-300 G9: only the frontmatter is touched. A line in the prose below that happens to read
    /// like a severity stays as written.
    #[test]
    fn with_severity_touches_only_the_frontmatter() {
        let body = format!("{}severity: low in prose\n", LOW);
        let (new, _) = with_severity(&body, "high").unwrap();
        assert_eq!(new, body.replacen("severity: low", "severity: high", 1));
        assert!(new.ends_with("severity: low in prose\n"), "{}", new);
    }

    /// INT-300 G9: a body with no frontmatter, and a level nobody ruled, are refused.
    #[test]
    fn with_severity_refuses_no_frontmatter_and_an_unknown_level() {
        let bare = with_severity("just text\n", "high").unwrap_err();
        assert!(bare.contains("no frontmatter"), "{}", bare);
        let level = with_severity(LOW, "urgent").unwrap_err();
        assert!(level.contains("the four are"), "{}", level);
    }

    /// INT-300 G9: the plan names the change, the file's hashes and a Seal that is the digest of
    /// the plan text, and the new body carries the level.
    #[test]
    fn a_rating_plan_shows_the_change_the_hashes_and_the_seal() {
        let rec = parse_record(LOW);
        let plan = rate_plan("/x/F-0001.md", &rec, LOW, "high").unwrap();
        assert!(
            plan.text.contains("rate F-0001: severity low -> high"),
            "{}",
            plan.text
        );
        assert!(
            plan.text.contains("\n-- severity: low\n++ severity: high"),
            "{}",
            plan.text
        );
        assert!(
            plan.text.contains("file /x/F-0001.md  sha256 "),
            "{}",
            plan.text
        );
        assert_eq!(plan.seal.len(), 16);
        assert_eq!(plan.seal, seal_digest(plan.text.as_bytes()));
        assert!(plan.new_body.contains("severity: high\n"));
        assert!(!plan.new_body.contains("severity: low"));
    }

    /// INT-300 G7, G9: a record with no severity line reads unrated, and its plan says so rather
    /// than showing a line that was never there.
    #[test]
    fn a_record_with_no_severity_line_gets_one_and_the_plan_says_so() {
        let rec = parse_record(OLD);
        let plan = rate_plan("/x/F-0001.md", &rec, OLD, "medium").unwrap();
        assert!(
            plan.text.contains("severity unrated -> medium"),
            "{}",
            plan.text
        );
        assert!(plan.text.contains("-- (no severity line"), "{}", plan.text);
        assert_eq!(parse_record(&plan.new_body).severity, "medium");
    }

    /// INT-300 G9: rating a level the record already has is refused, with no plan.
    #[test]
    fn rating_the_level_a_record_already_has_is_refused() {
        let low = rate_plan("/x", &parse_record(LOW), LOW, "low").unwrap_err();
        assert!(low.contains("already reads low"), "{}", low);
        let unrated = rate_plan("/x", &parse_record(OLD), OLD, "unrated").unwrap_err();
        assert!(unrated.contains("already reads unrated"), "{}", unrated);
    }

    /// INT-300 G9: a level nobody ruled is refused, names the four, and writes nothing.
    #[test]
    fn an_unknown_level_is_refused_naming_the_four() {
        let dir = filed("unknown-level", LOW);
        match rate(&dir, "F-0001", "urgent", &[]).unwrap() {
            Rated::Refused(why) => assert!(why.contains("the four are"), "{}", why),
            other => panic!("expected a refusal, got {:?}", other),
        }
        assert_eq!(std::fs::read_to_string(dir.join("F-0001.md")).unwrap(), LOW);
    }

    /// INT-300 G9, R5: showing the plan writes nothing.
    #[test]
    fn a_plan_writes_nothing() {
        let dir = filed("plan-only", LOW);
        assert!(matches!(
            rate(&dir, "F-0001", "high", &[]).unwrap(),
            Rated::Planned(_)
        ));
        assert_eq!(std::fs::read_to_string(dir.join("F-0001.md")).unwrap(), LOW);
    }

    /// INT-300 G9, R5: apply with the plan's own Seal writes the level, the record then shows it,
    /// and every other line is as it was.
    #[test]
    fn apply_with_the_plans_seal_writes_the_level() {
        let dir = filed("apply", LOW);
        let seal = match rate(&dir, "F-0001", "high", &[]).unwrap() {
            Rated::Planned(p) => p.seal,
            other => panic!("expected a plan, got {:?}", other),
        };
        let got = rate(&dir, "F-0001", "high", &words(&["apply", &seal])).unwrap();
        assert!(matches!(got, Rated::Applied(_)), "{:?}", got);
        let body = std::fs::read_to_string(dir.join("F-0001.md")).unwrap();
        assert_eq!(body, LOW.replace("severity: low", "severity: high"));
        assert_eq!(read_records(&dir).unwrap()[0].severity, "high");
    }

    /// INT-300 G9: apply with a Seal the plan does not have is refused and the record stays.
    #[test]
    fn apply_with_another_seal_is_refused_and_writes_nothing() {
        let dir = filed("wrong-seal", LOW);
        let got = rate(
            &dir,
            "F-0001",
            "high",
            &words(&["apply", "0000000000000000"]),
        )
        .unwrap();
        match got {
            Rated::Refused(why) => assert!(why.contains("hashes to"), "{}", why),
            other => panic!("expected a refusal, got {:?}", other),
        }
        assert_eq!(std::fs::read_to_string(dir.join("F-0001.md")).unwrap(), LOW);
    }

    /// INT-300 G9: the same plan applied twice is refused the second time, because the record
    /// already reads the level; the file is not touched again.
    #[test]
    fn a_second_apply_of_the_same_plan_is_refused() {
        let dir = filed("twice", LOW);
        let seal = match rate(&dir, "F-0001", "high", &[]).unwrap() {
            Rated::Planned(p) => p.seal,
            other => panic!("expected a plan, got {:?}", other),
        };
        let apply = words(&["apply", &seal]);
        assert!(matches!(
            rate(&dir, "F-0001", "high", &apply).unwrap(),
            Rated::Applied(_)
        ));
        let after_first = std::fs::read_to_string(dir.join("F-0001.md")).unwrap();
        match rate(&dir, "F-0001", "high", &apply).unwrap() {
            Rated::Refused(why) => assert!(why.contains("already reads high"), "{}", why),
            other => panic!("expected a refusal, got {:?}", other),
        }
        assert_eq!(
            std::fs::read_to_string(dir.join("F-0001.md")).unwrap(),
            after_first
        );
    }

    /// INT-300 G9: the words after the level are nothing, or apply and a Seal; anything else is
    /// refused.
    #[test]
    fn the_words_after_the_level_are_nothing_or_apply_and_a_seal() {
        let dir = filed("words", LOW);
        for bad in [
            words(&["abc"]),
            words(&["apply"]),
            words(&["apply", "a", "b"]),
        ] {
            assert!(
                matches!(
                    rate(&dir, "F-0001", "high", &bad).unwrap(),
                    Rated::Refused(_)
                ),
                "{:?}",
                bad
            );
        }
        assert_eq!(std::fs::read_to_string(dir.join("F-0001.md")).unwrap(), LOW);
    }

    /// INT-300 G9: an id with no record could not be answered, which is an error and not a
    /// refusal.
    #[test]
    fn an_id_with_no_record_could_not_be_answered() {
        let dir = filed("no-record", LOW);
        let err = rate(&dir, "F-0099", "high", &[]).unwrap_err();
        assert!(err.contains("names no finding"), "{}", err);
    }

    /// INT-300 G9: the plan prints its Seal and the line that applies it, an applied change says
    /// so, and a refusal ends 1 saying nothing was written.
    #[test]
    fn the_plan_the_apply_and_the_refusal_print_their_words() {
        let rec = parse_record(LOW);
        let plan = rate_plan("/x/F-0001.md", &rec, LOW, "high").unwrap();
        let seal = plan.seal.clone();
        let (text, code) = rated_text(&Rated::Planned(plan), "F-0001", "high");
        assert_eq!(code, 0);
        assert!(
            text.contains(&format!("DRY RUN -- nothing written. Seal: {}", seal)),
            "{}",
            text
        );
        assert!(
            text.contains(&format!("core bug rate F-0001 high apply {}", seal)),
            "{}",
            text
        );
        let plan = rate_plan("/x/F-0001.md", &rec, LOW, "high").unwrap();
        let (text, code) = rated_text(&Rated::Applied(plan), "F-0001", "high");
        assert_eq!(code, 0);
        assert!(text.contains("OK F-0001 rated"), "{}", text);
        let (text, code) = rated_text(&Rated::Refused("why".to_string()), "F-0001", "high");
        assert_eq!(code, 1);
        assert!(text.contains("Nothing was written"), "{}", text);
    }
}
