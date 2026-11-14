//! INT-300 -- the bug report. A finding record stores only what a person saw or decided; every
//! other word here is derived when asked (R3): state from git, never from a stored field.
//! `core bug list` is the first reader. The rows are plain values first, so a later viewer (a
//! terminal screen, a bar, a script) reads the same rows this one prints.

use super::findings::{severity_level, with_field, with_severity, Record};
use super::trace::{
    check_commit_seal, git_read, seal_digest, seal_notes, state_of, SealCheck, State, Trailers,
};
use super::{create_in, target_in, Seed, TEMPLATES};
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

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
    if let Some(to) = &record.promoted_to {
        t.push_str(&format!("  promoted  {}\n", to));
    }
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
    write_body(path, &plan.new_body)
}

/// Write `body` beside the record at `path`, move it into place and read it back. A record is
/// never left half written (INT-300 G9, G11).
fn write_body(path: &Path, body: &str) -> Result<(), String> {
    let tmp = path.with_extension("md.tmp");
    fs::write(&tmp, body).map_err(|e| format!("could not write {}: {}", tmp.display(), e))?;
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("could not replace {}: {}", path.display(), e)
    })?;
    let back = fs::read_to_string(path)
        .map_err(|e| format!("could not read back {}: {}", path.display(), e))?;
    if back != body {
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

/// One promotion as a plan: the intent it would file, the change to the bug's record, and the
/// Seal of that text (INT-300 G11, R5). Nothing is written until the same plan is applied by its Seal.
#[derive(Debug, PartialEq)]
pub struct PromotePlan {
    pub text: String,
    pub seal: String,
    pub id: u32,
    pub path: PathBuf,
    pub new_record: String,
}

/// What `core bug promote` did: printed a plan, filed the intent, or refused. A refusal writes nothing.
#[derive(Debug, PartialEq)]
pub enum Promoted {
    Planned(PromotePlan),
    Applied(PromotePlan),
    Refused(String),
}

/// What the promoted intent is called: `Fix F-NNNN: ` and the finding's own words, cut at sixty
/// characters, with no quote or backslash to break the title line.
pub fn promote_title(record: &Record) -> String {
    let said: String = record
        .what
        .chars()
        .map(|c| match c {
            '"' => '\'',
            '\\' => '/',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let said = said.split_whitespace().collect::<Vec<_>>().join(" ");
    let short = if said.chars().count() <= 60 {
        said
    } else {
        let cut: String = said.chars().take(60).collect();
        match cut.rfind(' ') {
            Some(i) if i > 30 => format!("{}...", &cut[..i]),
            _ => format!("{}...", cut),
        }
    };
    format!("Fix {}: {}", record.id, short)
}

/// The words the new intent starts with in place of the template's placeholders: the finding as
/// The Problem, and one gate that a sealed commit carries `Fixes: F-NNNN`.
pub fn promote_seed(record: &Record) -> Seed {
    Seed {
        problem: format!(
            "{}\n\nWhere: {}. Found {} by {}; severity {}.\n\
             This is finding {}: `core bug show {}` tells its whole story.",
            record.what,
            record.at,
            record.found_on,
            record.found_by,
            record.severity,
            record.id,
            record.id
        ),
        gate: format!(
            "- [ ] A sealed commit carries `Fixes: {}`, so `core bug show {}` reads solved \
             with that commit and its Seal checking out",
            record.id, record.id
        ),
    }
}

/// The fix template as `core intent new fix` reads it: its type, its status and its tags.
fn fix_template() -> Result<(&'static str, &'static str, Vec<String>), String> {
    let (_, type_tag, tags, status) = TEMPLATES
        .iter()
        .find(|(t, ..)| *t == "fix")
        .copied()
        .ok_or_else(|| "the fix template is missing".to_string())?;
    let tags = tags.split(',').map(|t| t.trim().to_string()).collect();
    Ok((type_tag, status, tags))
}

/// The plan for promoting one bug: refuses a bug already promoted and a bug that is not open,
/// otherwise names the intent, its file, the words it starts with, the change to the bug's record
/// and the Seal of that text. `state` is the word `state_word` gives the bug.
pub fn promote_plan(
    base: &Path,
    record: &Record,
    record_path: &str,
    old_body: &str,
    state: &str,
) -> Result<PromotePlan, String> {
    if let Some(to) = &record.promoted_to {
        return Err(format!("{} is already promoted to {}", record.id, to));
    }
    if state != "open" {
        return Err(format!(
            "{} reads {}, not open; there is nothing to promote",
            record.id, state
        ));
    }
    let (type_tag, status, tags) = fix_template()?;
    let (id, path, title) =
        target_in(base, "open", &promote_title(record)).map_err(|e| e.to_string())?;
    let name = format!("INT-{:03}", id);
    let (new_record, _) = with_field(old_body, "promoted_to", &name)?;
    let seed = promote_seed(record);
    let text = format!(
        "promote {}: a new intent, {}\n\
         title    {}\n\
         folder   open (template fix: type {}, status {}, tags {})\n\
         file     {}\n\
         problem  {}\n\
         gate     {}\n\
         record   {}  sha256 {} -> {}\n\
         ++ promoted_to: {}",
        record.id,
        name,
        title,
        type_tag,
        status,
        tags.join(", "),
        path.display(),
        record.what,
        seed.gate,
        record_path,
        seal_digest(old_body.as_bytes()),
        seal_digest(new_record.as_bytes()),
        name
    );
    let seal = seal_digest(text.as_bytes());
    Ok(PromotePlan {
        text,
        seal,
        id,
        path,
        new_record,
    })
}

/// `core bug promote` as a value. With no words after the id it plans; with `apply SEAL` it
/// rebuilds the plan from the ledger as it is now and files the intent only if that plan still has
/// the given Seal. Refused means nothing was written; Err means the ledger could not be read or
/// written, and a promotion that failed half way takes the new intent back.
pub fn promote(
    base: &Path,
    id: &str,
    fixes: &Result<Vec<(String, String)>, String>,
    apply: &[String],
) -> Result<Promoted, String> {
    let seal = match apply {
        [] => None,
        [word, seal] if word == "apply" => Some(seal.as_str()),
        _ => {
            return Ok(Promoted::Refused(
                "after the id say nothing to see the plan, or: apply SEAL".to_string(),
            ));
        }
    };
    let dir = base.join("findings");
    let records = super::findings::read_records(&dir)?;
    let record = records
        .iter()
        .find(|r| r.id == id)
        .ok_or_else(|| format!("{} names no finding in {}", id, dir.display()))?;
    let path = dir.join(format!("{}.md", id));
    let old_body = fs::read_to_string(&path)
        .map_err(|e| format!("could not read {}: {}", path.display(), e))?;
    let state = state_word(&state_of(record, fixes));
    let plan = match promote_plan(base, record, &path.display().to_string(), &old_body, state) {
        Ok(p) => p,
        Err(e) => return Ok(Promoted::Refused(e)),
    };
    match seal {
        None => Ok(Promoted::Planned(plan)),
        Some(given) if given == plan.seal => {
            file_promotion(base, record, &path, &plan)?;
            Ok(Promoted::Applied(plan))
        }
        Some(given) => Ok(Promoted::Refused(format!(
            "the plan now hashes to {}, not {}: the ledger or the bug changed",
            plan.seal, given
        ))),
    }
}

/// Files the planned intent through `create_in`, the code behind `core intent new`, then marks the
/// bug. If the bug cannot be written the new intent is removed, so nothing is left half done.
fn file_promotion(
    base: &Path,
    record: &Record,
    record_path: &Path,
    plan: &PromotePlan,
) -> Result<(), String> {
    let (type_tag, status, tags) = fix_template()?;
    let seed = promote_seed(record);
    let made = create_in(
        base,
        "open",
        type_tag,
        &promote_title(record),
        status,
        &tags,
        Some(&seed),
    )
    .map_err(|e| e.to_string())?;
    if made != plan.path {
        let _ = fs::remove_file(&made);
        return Err(format!(
            "the intent was filed at {}, not at {}; it was removed",
            made.display(),
            plan.path.display()
        ));
    }
    if let Err(e) = write_body(record_path, &plan.new_record) {
        let _ = fs::remove_file(&made);
        return Err(format!(
            "{}; the new intent {} was removed",
            e,
            made.display()
        ));
    }
    Ok(())
}

/// What `core bug promote` prints and the exit it ends with: the plan with its Seal and the line
/// that files it, the filed intent, or a refusal.
pub fn promoted_text(got: &Promoted, id: &str) -> (String, i32) {
    match got {
        Promoted::Planned(p) => (
            format!(
                "{}\nDRY RUN -- nothing written. Seal: {}\nto write it: core bug promote {} apply {}\n",
                p.text, p.seal, id, p.seal
            ),
            0,
        ),
        Promoted::Applied(p) => (
            format!(
                "{}\nOK {} promoted to INT-{:03}, verified on disk\nfiled {}\n",
                p.text,
                id,
                p.id,
                p.path.display()
            ),
            0,
        ),
        Promoted::Refused(why) => (format!("REFUSE {}. Nothing was written.\n", why), 1),
    }
}

/// `core bug promote F-NNNN [apply SEAL]` -- INT-300 G11. The domain owns its exit: 0 planned or
/// filed, 1 refused (nothing written), 3 the ledger could not be read or written.
pub fn promote_ledger(ctx: &AppContext, id: &str, apply: &[String]) -> CoreResult<()> {
    let base = super::intents_dir(ctx);
    let fixes = super::trace::fixes(&ctx.core_root);
    let code = match promote(&base, id, &fixes, apply) {
        Ok(got) => {
            let (text, code) = promoted_text(&got, id);
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
/// A civil day as a count of days since 1970-01-01, or None for anything that is not a real
/// YYYY-MM-DD day (INT-300 G12). The one place this file does date arithmetic.
pub fn day_number(day: &str) -> Option<i64> {
    let b = day.as_bytes();
    if !day.is_ascii() || b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let num = |s: &str| -> Option<i64> {
        if s.bytes().all(|c| c.is_ascii_digit()) {
            s.parse().ok()
        } else {
            None
        }
    };
    let (y, m, d) = (num(&day[0..4])?, num(&day[5..7])?, num(&day[8..10])?);
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let in_month = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    if !(1..=in_month).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe - 719468)
}

/// The YYYY-MM-DD text of a day number, the inverse of `day_number`.
pub fn day_text(n: i64) -> String {
    let z = n + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Twice the median of `days`, so a half day stays exact; None when there are no days.
pub fn median_twice(days: &[i64]) -> Option<i64> {
    if days.is_empty() {
        return None;
    }
    let mut sorted = days.to_vec();
    sorted.sort();
    let n = sorted.len();
    Some(if n % 2 == 1 {
        sorted[n / 2] * 2
    } else {
        sorted[n / 2 - 1] + sorted[n / 2]
    })
}

/// Twice a number of days, said in days: `3` or `2.5`.
fn half_days(twice: i64) -> String {
    if twice % 2 == 0 {
        format!("{}", twice / 2)
    } else {
        format!("{}.5", twice / 2)
    }
}

/// The earliest day any of `hashes` was committed, as git dates it (`%cs`). A commit git cannot
/// date is an error, never a guess.
pub fn fix_day(core: &str, hashes: &[String]) -> Result<String, String> {
    let mut best: Option<String> = None;
    for hash in hashes {
        let out = git_read(core, &["log", "-1", "--format=%cs", hash.as_str()])?;
        let day = out.trim().to_string();
        if day_number(&day).is_none() {
            return Err(format!("git gave no day for {}", hash));
        }
        let earlier = match &best {
            Some(b) => day < *b,
            None => true,
        };
        if earlier {
            best = Some(day);
        }
    }
    best.ok_or_else(|| "there is no fixing commit to date".to_string())
}

/// Today as the machine says it, YYYY-MM-DD.
pub fn today_text() -> Result<String, String> {
    let out = std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .map_err(|e| format!("could not run date: {}", e))?;
    let day = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !out.status.success() || day_number(&day).is_none() {
        return Err("date gave no day".to_string());
    }
    Ok(day)
}

/// The levels the dashboard counts, worst first. A severity outside these is counted as `other`.
const LEVELS: [&str; 4] = ["high", "medium", "low", "unrated"];

/// How many days "the last 7 days" covers, today included.
const WINDOW: i64 = 7;

/// `core bug` with no arguments, as a value (INT-300 G12): open bugs by severity, the oldest open
/// bug and its age, the median days from found to fixed, and the bugs fixed in the last 7 days.
/// `fixed_on` is each solved bug's earliest fixing commit day. Exit: 0 clean, 1 a recount
/// disagrees, 3 git or the date could not be read. Nothing prints a zero for a number it could
/// not measure.
pub fn dashboard_report(
    records: &[Record],
    fixes: &Result<Vec<(String, String)>, String>,
    fixed_on: &[(String, String)],
    today: &str,
    on_disk: usize,
) -> Report {
    let now = match day_number(today) {
        Some(n) => n,
        None => {
            return Report {
                text: format!("today could not be read: {:?} is not a day\n", today),
                code: 3,
            }
        }
    };
    let all = rows(records, fixes);
    let c = counts(&all);
    let mut text = format!(
        "{} -- {} findings: {} open, {} solved, {} closed, {} unknown\n",
        today,
        c.total(),
        c.open,
        c.solved,
        c.closed,
        c.unknown
    );
    let open: Vec<&Row> = all.iter().filter(|r| r.state == "open").collect();
    let mut summed = 0;
    match fixes {
        Err(e) => text.push_str(&format!(
            "open by severity: unknown -- git could not be read ({})\n",
            e
        )),
        Ok(_) => {
            text.push_str(
                "open by severity (core bug list --state open --severity LEVEL shows each)\n",
            );
            for level in LEVELS {
                let ids: Vec<&str> = open
                    .iter()
                    .filter(|r| r.severity == level)
                    .map(|r| r.id.as_str())
                    .collect();
                summed += ids.len();
                let named = if (level == "high" || level == "medium") && !ids.is_empty() {
                    format!("  {}", ids.join(" "))
                } else {
                    String::new()
                };
                text.push_str(&format!("  {:<8} {:>3}{}\n", level, ids.len(), named));
            }
            let other: Vec<&str> = open
                .iter()
                .filter(|r| !LEVELS.contains(&r.severity.as_str()))
                .map(|r| r.id.as_str())
                .collect();
            if !other.is_empty() {
                summed += other.len();
                text.push_str(&format!(
                    "  {:<8} {:>3}  {}\n",
                    "other",
                    other.len(),
                    other.join(" ")
                ));
            }
        }
    }
    let dated: Vec<(i64, &Row)> = open
        .iter()
        .filter_map(|r| day_number(&r.found_on).map(|n| (n, *r)))
        .collect();
    let oldest = dated
        .iter()
        .min_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.id.cmp(&b.1.id)));
    match (oldest, fixes) {
        (_, Err(_)) => text.push_str("oldest open: unknown -- git could not be read\n"),
        (Some((n, r)), Ok(_)) if *n <= now => text.push_str(&format!(
            "oldest open   {}  found {}  {} days  {}\n",
            r.id,
            r.found_on,
            now - n,
            r.what
        )),
        (Some((_, r)), Ok(_)) => text.push_str(&format!(
            "oldest open   {}  found {}  after today  {}\n",
            r.id, r.found_on, r.what
        )),
        (None, Ok(_)) if open.is_empty() => text.push_str("oldest open   none -- no bug is open\n"),
        (None, Ok(_)) => {
            text.push_str("oldest open   unknown -- no open bug has a readable found day\n")
        }
    }
    if fixes.is_ok() && !dated.is_empty() && dated.len() < open.len() {
        text.push_str(&format!(
            "              left out: {} open without a readable found day\n",
            open.len() - dated.len()
        ));
    }
    let mut timed: Vec<i64> = Vec::new();
    let mut left: Vec<(String, &str)> = Vec::new();
    let mut recent: Vec<String> = Vec::new();
    for r in all.iter().filter(|r| r.state == "solved") {
        let fixed = fixed_on
            .iter()
            .find(|(id, _)| *id == r.id)
            .and_then(|(_, day)| day_number(day));
        match (fixed, day_number(&r.found_on)) {
            (None, _) => left.push((r.id.clone(), "no readable fix day")),
            (Some(f), _) if f > now => left.push((r.id.clone(), "fixed after today")),
            (Some(f), found) => {
                if now - f < WINDOW {
                    recent.push(r.id.clone());
                }
                match found {
                    Some(o) if f >= o => timed.push(f - o),
                    Some(_) => left.push((r.id.clone(), "fixed before it was found")),
                    None => left.push((r.id.clone(), "no readable found day")),
                }
            }
        }
    }
    let solved_known = fixes.is_ok();
    match (median_twice(&timed), solved_known) {
        (_, false) => text.push_str("median fix: unknown -- git could not be read\n"),
        (Some(m), true) => text.push_str(&format!(
            "median fix    {} days, found day to the earliest fixing commit day, over {} solved\n",
            half_days(m),
            timed.len()
        )),
        (None, true) => text.push_str("median fix    unknown -- no solved bug can be timed\n"),
    }
    if solved_known && !left.is_empty() {
        let said: Vec<String> = left
            .iter()
            .map(|(id, why)| format!("{} ({})", id, why))
            .collect();
        text.push_str(&format!("              left out: {}\n", said.join(", ")));
    }
    let from = day_text(now - (WINDOW - 1));
    if solved_known {
        text.push_str(&format!(
            "solved in the last {} days ({} to {})  {}{}\n",
            WINDOW,
            from,
            today,
            recent.len(),
            if recent.is_empty() {
                String::new()
            } else {
                format!(": {}", recent.join(" "))
            }
        ));
    } else {
        text.push_str(&format!(
            "solved in the last {} days ({} to {}): unknown -- git could not be read\n",
            WINDOW, from, today
        ));
    }
    let files_agree = on_disk == records.len();
    let sums_agree = fixes.is_err() || summed == c.open;
    text.push_str(&format!(
        "recount: {} finding files on disk, {} records read -- they {}; open by severity adds up to {} of {} open -- {}\n",
        on_disk,
        records.len(),
        if files_agree { "agree" } else { "disagree" },
        summed,
        c.open,
        if sums_agree { "agree" } else { "disagree" }
    ));
    let mut code = if files_agree && sums_agree { 0 } else { 1 };
    if let Err(e) = fixes {
        text.push_str(&format!(
            "git could not be read ({}): open findings read unknown\n",
            e
        ));
        code = 3;
    }
    Report { text, code }
}

/// `core bug` with no arguments -- INT-300 G12. The domain owns its exit: 0 clean, 1 a recount
/// disagrees, 3 git or the date could not be read.
pub fn dashboard(ctx: &AppContext) -> CoreResult<()> {
    let code = match dashboard_answer(ctx) {
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

fn dashboard_answer(ctx: &AppContext) -> Result<i32, String> {
    let dir = super::intents_dir(ctx).join("findings");
    let records = super::findings::read_records(&dir)?;
    let on_disk = files_on_disk(&dir)?;
    let core: &str = &ctx.core_root;
    let named = super::trace::fixes(core);
    let today = today_text()?;
    let mut fixed_on = Vec::new();
    for row in rows(&records, &named) {
        if row.state == "solved" {
            fixed_on.push((row.id.clone(), fix_day(core, &row.fixed_by)?));
        }
    }
    let got = dashboard_report(&records, &named, &fixed_on, &today, on_disk);
    print!("{}", got.text);
    Ok(got.code)
}
/// The one line the shell banner prints about bugs, as a value (INT-300 G13): `bugs: N open (H
/// high)`, counted by the same rows as the dashboard. When git could not be read, or a finding
/// file was not read, it says `bugs: unknown` and ends 3; a measured zero is `0 open`, not unknown.
pub fn welcome_line(
    records: &[Record],
    fixes: &Result<Vec<(String, String)>, String>,
    on_disk: usize,
) -> Report {
    if fixes.is_err() || on_disk != records.len() {
        return Report {
            text: "bugs: unknown\n".to_string(),
            code: 3,
        };
    }
    let all = rows(records, fixes);
    let open: Vec<&Row> = all.iter().filter(|r| r.state == "open").collect();
    let high = open.iter().filter(|r| r.severity == "high").count();
    Report {
        text: format!("bugs: {} open ({} high)\n", open.len(), high),
        code: 0,
    }
}

/// `core bug welcome` -- INT-300 G13. The domain owns its exit: 0 the line, 3 it could not tell.
pub fn welcome(ctx: &AppContext) -> CoreResult<()> {
    let code = match welcome_answer(ctx) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{} {}", "\u{2717}".bright_red(), e);
            print!("bugs: unknown\n");
            3
        }
    };
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

fn welcome_answer(ctx: &AppContext) -> Result<i32, String> {
    let dir = super::intents_dir(ctx).join("findings");
    let records = super::findings::read_records(&dir)?;
    let on_disk = files_on_disk(&dir)?;
    let core: &str = &ctx.core_root;
    let named = super::trace::fixes(core);
    let got = welcome_line(&records, &named, on_disk);
    print!("{}", got.text);
    Ok(got.code)
}

#[cfg(test)]
mod bug_tests {
    use super::{
        check, counts, dashboard_report, day_number, day_text, files_on_disk, filter_from, fix_day,
        median_twice, promote, promote_title, promoted_text, rate, rate_plan, rated_text,
        render_list, render_summary, report, rows, select, state_word, story, welcome_line, Counts,
        FixFacts, Promoted, Rated, Row,
    };
    use crate::domains::intent::findings::{
        parse_record, read_records, with_field, with_severity, Record,
    };
    use crate::domains::intent::trace::{seal_digest, SealCheck, State, Trailers};
    use crate::domains::intent::{create_in, Seed};

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

    const FRESH: &str = "---\nid: F-0001\nwhat: the \"parser\" drops a line\nat: a.rs:1\nfound_by: INT-1\nfound_on: 2026-10-30\nclosed_without_fix: no\nseverity: high\n---\n";

    fn ledger(name: &str, body: &str) -> std::path::PathBuf {
        let base =
            std::env::temp_dir().join(format!("core-promote-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("findings")).unwrap();
        std::fs::write(base.join("findings").join("F-0001.md"), body).unwrap();
        base
    }

    fn nothing_fixed() -> Result<Vec<(String, String)>, String> {
        Ok(Vec::new())
    }

    fn filed_in(base: &std::path::Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(base.join("open"))
            .map(|d| {
                d.flatten()
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }

    fn plan_of(base: &std::path::Path) -> (String, String) {
        match promote(base, "F-0001", &nothing_fixed(), &[]).unwrap() {
            Promoted::Planned(p) => (p.text, p.seal),
            other => panic!("expected a plan, got {:?}", other),
        }
    }

    /// INT-300 G11: the one rewriter of a record sets any frontmatter key, inserting it before the
    /// closing line when the record has none and replacing it when it has, and every other byte stays.
    #[test]
    fn with_field_sets_any_key_and_keeps_every_other_byte() {
        let (new, old) = with_field(FRESH, "promoted_to", "INT-042").unwrap();
        let want = format!("{}promoted_to: INT-042\n---\n", &FRESH[..FRESH.len() - 4]);
        assert_eq!(new, want);
        assert_eq!(old, None);
        let (again, was) = with_field(&new, "promoted_to", "INT-043").unwrap();
        assert_eq!(again, want.replace("INT-042", "INT-043"));
        assert_eq!(was, Some("promoted_to: INT-042".to_string()));
    }

    /// INT-300 G11: a value of more than one line, and a body with no frontmatter, are refused.
    #[test]
    fn with_field_refuses_a_two_line_value_and_a_body_with_no_frontmatter() {
        let two = with_field(FRESH, "promoted_to", "INT-1\nsecret: x").unwrap_err();
        assert!(two.contains("must be one line"), "{}", two);
        let bare = with_field("just text\n", "promoted_to", "INT-1").unwrap_err();
        assert!(bare.contains("no promoted_to can be set"), "{}", bare);
    }

    /// INT-300 G11: a record reads promoted_to when it has one and none when it does not, so every
    /// record filed before G11 keeps working.
    #[test]
    fn a_record_reads_promoted_to_only_when_it_has_one() {
        assert_eq!(parse_record(FRESH).promoted_to, None);
        let (with, _) = with_field(FRESH, "promoted_to", "INT-042").unwrap();
        assert_eq!(parse_record(&with).promoted_to, Some("INT-042".to_string()));
        let (empty, _) = with_field(FRESH, "promoted_to", "").unwrap();
        assert_eq!(parse_record(&empty).promoted_to, None);
    }

    /// INT-300 G11: the promoted title is "Fix F-NNNN: " and the finding in at most sixty
    /// characters, with no quote or backslash that could break the title line.
    #[test]
    fn the_promoted_title_is_the_finding_in_sixty_characters_with_no_quote() {
        let short = promote_title(&parse_record(FRESH));
        assert_eq!(short, "Fix F-0001: the 'parser' drops a line");
        let long = FRESH.replace("the \"parser\" drops a line", &"word \\ ".repeat(30));
        let title = promote_title(&parse_record(&long));
        let said = title.strip_prefix("Fix F-0001: ").unwrap();
        assert!(said.ends_with("..."), "{}", title);
        assert!(said.chars().count() <= 63, "{}", title);
        assert!(!title.contains('"') && !title.contains('\\'), "{}", title);
    }

    /// INT-300 G11: the plan names the intent, its folder and template, its file, the finding as the
    /// problem, the first gate, the record's hashes and the Seal, and the Seal is the digest of the
    /// text.
    #[test]
    fn a_promotion_plan_names_the_intent_the_file_the_gate_the_record_and_the_seal() {
        let base = ledger("plan", FRESH);
        let (text, seal) = plan_of(&base);
        let file = base
            .join("open")
            .join("001-fix-f-0001-the-parser-drops-a-line.md");
        for want in [
            "promote F-0001: a new intent, INT-001".to_string(),
            "title    Fix F-0001: the 'parser' drops a line".to_string(),
            "folder   open (template fix: type fix, status planned, tags fix, bugfix)".to_string(),
            format!("file     {}", file.display()),
            "problem  the \"parser\" drops a line".to_string(),
            "gate     - [ ] A sealed commit carries `Fixes: F-0001`".to_string(),
            format!(
                "record   {}  sha256 ",
                base.join("findings").join("F-0001.md").display()
            ),
            "\n++ promoted_to: INT-001".to_string(),
        ] {
            assert!(text.contains(&want), "missing {:?} in {}", want, text);
        }
        assert_eq!(seal, seal_digest(text.as_bytes()));
    }

    /// INT-300 G11, R5: showing the plan writes nothing, no intent and no change to the bug.
    #[test]
    fn a_promotion_plan_writes_nothing() {
        let base = ledger("plan-only", FRESH);
        let _ = plan_of(&base);
        assert!(filed_in(&base).is_empty());
        let on_disk = std::fs::read_to_string(base.join("findings").join("F-0001.md")).unwrap();
        assert_eq!(on_disk, FRESH);
    }

    /// INT-300 G11: apply with the plan's own Seal files a real intent whose Problem and first
    /// criterion are the finding's, with no placeholder left, and marks the bug.
    #[test]
    fn apply_with_the_plans_seal_files_the_intent_and_marks_the_bug() {
        let base = ledger("apply", FRESH);
        let (_, seal) = plan_of(&base);
        let words = vec!["apply".to_string(), seal];
        let got = promote(&base, "F-0001", &nothing_fixed(), &words).unwrap();
        assert!(matches!(got, Promoted::Applied(_)), "{:?}", got);
        assert_eq!(
            filed_in(&base),
            vec!["001-fix-f-0001-the-parser-drops-a-line.md".to_string()]
        );
        let intent = std::fs::read_to_string(
            base.join("open")
                .join("001-fix-f-0001-the-parser-drops-a-line.md"),
        )
        .unwrap();
        for want in [
            "id: 001\n",
            "type: fix\n",
            "status: planned\n",
            "title: \"Fix F-0001: the 'parser' drops a line\"\n",
            "## The Problem\nthe \"parser\" drops a line\n\nWhere: a.rs:1. Found 2026-10-30 by INT-1; severity high.\n",
            "## Success Criteria\n- [ ] A sealed commit carries `Fixes: F-0001`",
        ] {
            assert!(intent.contains(want), "missing {:?} in {}", want, intent);
        }
        assert!(
            !intent.contains("[What problem does this solve?]"),
            "{}",
            intent
        );
        assert!(!intent.contains("- [ ] ...\n"), "{}", intent);
        let record = std::fs::read_to_string(base.join("findings").join("F-0001.md")).unwrap();
        assert_eq!(
            record,
            with_field(FRESH, "promoted_to", "INT-001").unwrap().0
        );
        let read = read_records(&base.join("findings")).unwrap();
        assert_eq!(read[0].promoted_to, Some("INT-001".to_string()));
    }

    /// INT-300 G11: apply with a Seal the plan does not have is refused and nothing is written.
    #[test]
    fn promote_with_another_seal_is_refused_and_writes_nothing() {
        let base = ledger("wrong-seal", FRESH);
        let words = vec!["apply".to_string(), "0000000000000000".to_string()];
        match promote(&base, "F-0001", &nothing_fixed(), &words).unwrap() {
            Promoted::Refused(why) => assert!(why.contains("hashes to"), "{}", why),
            other => panic!("expected a refusal, got {:?}", other),
        }
        assert!(filed_in(&base).is_empty());
        let on_disk = std::fs::read_to_string(base.join("findings").join("F-0001.md")).unwrap();
        assert_eq!(on_disk, FRESH);
    }

    /// INT-300 G11: a bug already promoted is refused naming the intent, and no second intent is
    /// filed.
    #[test]
    fn a_bug_already_promoted_is_refused_and_no_second_intent_is_filed() {
        let base = ledger("twice", FRESH);
        let (_, seal) = plan_of(&base);
        let words = vec!["apply".to_string(), seal];
        assert!(matches!(
            promote(&base, "F-0001", &nothing_fixed(), &words).unwrap(),
            Promoted::Applied(_)
        ));
        for again in [Vec::new(), words.clone()] {
            match promote(&base, "F-0001", &nothing_fixed(), &again).unwrap() {
                Promoted::Refused(why) => {
                    assert!(why.contains("already promoted to INT-001"), "{}", why)
                }
                other => panic!("expected a refusal, got {:?}", other),
            }
        }
        assert_eq!(filed_in(&base).len(), 1);
    }

    /// INT-300 G11, G7: only an open bug is promoted. A solved one, a closed one and one whose git
    /// could not be read are each refused naming the word they read.
    #[test]
    fn only_an_open_bug_is_promoted() {
        let base = ledger("not-open", FRESH);
        let solved: Result<Vec<(String, String)>, String> =
            Ok(vec![("F-0001".to_string(), "abc1234".to_string())]);
        let unread: Result<Vec<(String, String)>, String> =
            Err("git log could not run".to_string());
        for (fixes, word) in [(solved, "solved"), (unread, "unknown")] {
            match promote(&base, "F-0001", &fixes, &[]).unwrap() {
                Promoted::Refused(why) => {
                    assert!(
                        why.contains(&format!("reads {}, not open", word)),
                        "{}",
                        why
                    )
                }
                other => panic!("expected a refusal, got {:?}", other),
            }
        }
        let closed = FRESH.replace("closed_without_fix: no", "closed_without_fix: duplicate");
        let base = ledger("closed", &closed);
        match promote(&base, "F-0001", &nothing_fixed(), &[]).unwrap() {
            Promoted::Refused(why) => assert!(why.contains("reads closed, not open"), "{}", why),
            other => panic!("expected a refusal, got {:?}", other),
        }
        assert!(filed_in(&base).is_empty());
    }

    /// INT-300 G11: the new intent takes the next number the ledger gives, the one numberer.
    #[test]
    fn the_new_intent_takes_the_next_number() {
        let base = ledger("numbering", FRESH);
        std::fs::create_dir_all(base.join("open")).unwrap();
        std::fs::write(base.join("open").join("041-old.md"), "x").unwrap();
        let (text, _) = plan_of(&base);
        assert!(text.contains("a new intent, INT-042"), "{}", text);
        assert!(text.contains("042-fix-f-0001-"), "{}", text);
    }

    /// INT-300 G11: the words after the id are nothing, or apply and a Seal; an id with no record
    /// could not be answered.
    #[test]
    fn the_words_after_the_bug_are_nothing_or_apply_and_a_seal() {
        let base = ledger("words", FRESH);
        for bad in [
            vec!["abc".to_string()],
            vec!["apply".to_string()],
            vec!["apply".to_string(), "a".to_string(), "b".to_string()],
        ] {
            let got = promote(&base, "F-0001", &nothing_fixed(), &bad).unwrap();
            assert!(matches!(got, Promoted::Refused(_)), "{:?}", bad);
        }
        let err = promote(&base, "F-0099", &nothing_fixed(), &[]).unwrap_err();
        assert!(err.contains("names no finding"), "{}", err);
        assert!(filed_in(&base).is_empty());
    }

    /// INT-300 G11: when the bug cannot be written after the intent was filed, the intent is taken
    /// back and the call ends in an error, so nothing is left half done.
    #[test]
    fn a_bug_that_cannot_be_written_takes_the_new_intent_back() {
        let base = ledger("rollback", FRESH);
        let (_, seal) = plan_of(&base);
        std::fs::create_dir(base.join("findings").join("F-0001.md.tmp")).unwrap();
        let words = vec!["apply".to_string(), seal];
        let err = promote(&base, "F-0001", &nothing_fixed(), &words).unwrap_err();
        assert!(err.contains("was removed"), "{}", err);
        assert!(filed_in(&base).is_empty());
        let on_disk = std::fs::read_to_string(base.join("findings").join("F-0001.md")).unwrap();
        assert_eq!(on_disk, FRESH);
    }

    /// INT-300 G11: the creator without a seed writes the template exactly as before; with one it
    /// replaces the Problem and the first criterion and nothing else; an unknown category is refused.
    #[test]
    fn create_in_without_a_seed_keeps_the_template_and_with_one_replaces_two_lines() {
        let base = std::env::temp_dir().join(format!("core-create-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let tags = vec!["fix".to_string(), "bugfix".to_string()];
        let plain = create_in(&base, "open", "fix", "A title", "planned", &tags, None).unwrap();
        let plain = std::fs::read_to_string(plain).unwrap();
        assert!(
            plain.contains("## The Problem\n[What problem does this solve?]\n\n## The Solution")
        );
        assert!(plain.contains("## Success Criteria\n- [ ] ...\n\n<!-- INT-158"));
        let seed = Seed {
            problem: "P".to_string(),
            gate: "- [ ] G".to_string(),
        };
        let seeded = create_in(
            &base,
            "open",
            "fix",
            "A title",
            "planned",
            &tags,
            Some(&seed),
        )
        .unwrap();
        let seeded = std::fs::read_to_string(seeded).unwrap();
        assert_eq!(
            seeded
                .replace(
                    "## The Problem\nP\n\n",
                    "## The Problem\n[What problem does this solve?]\n\n"
                )
                .replace(
                    "## Success Criteria\n- [ ] G\n\n",
                    "## Success Criteria\n- [ ] ...\n\n"
                )
                .replace("id: 002", "id: 001"),
            plain
        );
        let bad = create_in(&base, "nowhere", "fix", "A title", "planned", &tags, None);
        assert!(bad.is_err());
    }

    /// INT-300 G11: the plan prints its Seal and the line that files it, a filed promotion says
    /// so, and a refusal ends 1 saying nothing was written.
    #[test]
    fn the_plan_the_filing_and_the_refusal_print_their_words() {
        let base = ledger("words-out", FRESH);
        let (_, seal) = plan_of(&base);
        let planned = promote(&base, "F-0001", &nothing_fixed(), &[]).unwrap();
        let (text, code) = promoted_text(&planned, "F-0001");
        assert_eq!(code, 0);
        assert!(
            text.contains(&format!("DRY RUN -- nothing written. Seal: {}", seal)),
            "{}",
            text
        );
        assert!(
            text.contains(&format!("core bug promote F-0001 apply {}", seal)),
            "{}",
            text
        );
        let words = vec!["apply".to_string(), seal];
        let filed = promote(&base, "F-0001", &nothing_fixed(), &words).unwrap();
        let (text, code) = promoted_text(&filed, "F-0001");
        assert_eq!(code, 0);
        assert!(
            text.contains("OK F-0001 promoted to INT-001, verified on disk"),
            "{}",
            text
        );
        let (text, code) = promoted_text(&Promoted::Refused("why".to_string()), "F-0001");
        assert_eq!(code, 1);
        assert!(text.contains("Nothing was written"), "{}", text);
    }

    /// INT-300 G11: `core bug show` tells where a bug was promoted to, and says nothing about it
    /// for a bug that was not.
    #[test]
    fn show_tells_the_intent_a_bug_was_promoted_to() {
        let (with, _) = with_field(FRESH, "promoted_to", "INT-042").unwrap();
        let said = story(&parse_record(&with), &State::Open, &[]);
        assert!(said.text.contains("  promoted  INT-042\n"), "{}", said.text);
        let plain = story(&parse_record(FRESH), &State::Open, &[]);
        assert!(!plain.text.contains("promoted"), "{}", plain.text);
    }

    /// The words on the first line of `text` that starts with `first`.
    fn words_of(text: &str, first: &str) -> Vec<String> {
        text.lines()
            .map(|l| {
                l.split_whitespace()
                    .map(|w| w.to_string())
                    .collect::<Vec<_>>()
            })
            .find(|w| w.first().map(|f| f == first).unwrap_or(false))
            .unwrap_or_default()
    }

    fn fixed_by(id: &str) -> (String, String) {
        (id.to_string(), "abc1234".to_string())
    }

    fn on_day(id: &str, d: &str) -> (String, String) {
        (id.to_string(), d.to_string())
    }

    /// INT-300 G12: day_number reads a real YYYY-MM-DD day, leap days included, and refuses every
    /// other text with None, never a guess.
    #[test]
    fn day_number_reads_real_days_and_refuses_the_rest() {
        assert_eq!(day_number("1970-01-01"), Some(0));
        assert_eq!(day_number("2000-03-01"), Some(11017));
        assert!(day_number("2024-02-29").is_some());
        for bad in [
            "2025-02-29",
            "2026-13-01",
            "2026-00-10",
            "2026-04-31",
            "2026-1-5",
            "26-10-30",
            "2026/10/30",
            "",
            "unknown",
            "2026-10-3x",
            "2026-10-300",
            "\u{ff12}026-10-30",
        ] {
            assert_eq!(day_number(bad), None, "{:?}", bad);
        }
    }

    /// INT-300 G12: the difference of two day numbers is the days between, across month and year
    /// ends and a leap year.
    #[test]
    fn the_difference_of_two_day_numbers_is_the_days_between() {
        let n = |d: &str| day_number(d).unwrap();
        assert_eq!(n("2026-10-30") - n("2026-10-24"), 6);
        assert_eq!(n("2026-03-01") - n("2026-02-28"), 1);
        assert_eq!(n("2025-01-01") - n("2024-01-01"), 366);
        assert_eq!(n("2027-01-01") - n("2026-12-31"), 1);
    }

    /// INT-300 G12: day_text is the inverse of day_number over a wide run of days.
    #[test]
    fn day_text_is_the_inverse_of_day_number() {
        assert_eq!(day_text(0), "1970-01-01");
        assert_eq!(day_text(11017), "2000-03-01");
        for n in (-800..60000).step_by(7) {
            assert_eq!(day_number(&day_text(n)), Some(n), "{}", n);
        }
    }

    /// INT-300 G12: the median is exact for an odd count, the mean of the middle two for an even
    /// count (kept as twice the value), and nothing for no days.
    #[test]
    fn the_median_of_days_is_exact_and_empty_is_none() {
        assert_eq!(median_twice(&[]), None);
        assert_eq!(median_twice(&[5]), Some(10));
        assert_eq!(median_twice(&[3, 1, 2]), Some(4));
        assert_eq!(median_twice(&[1, 2]), Some(3));
        assert_eq!(median_twice(&[4, 1, 3, 2]), Some(5));
    }

    /// INT-300 G12: fix_day is the earliest committer day among the fixing commits, read from
    /// git; a commit git does not have is an error.
    #[test]
    fn fix_day_is_the_earliest_commit_day_and_an_unknown_commit_is_an_error() {
        let dir = std::env::temp_dir().join(format!("core-fixday-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let git = |args: &[&str], when: &str| -> String {
            let out = std::process::Command::new("git")
                .arg("-C")
                .arg(&dir)
                .args(["-c", "user.name=t", "-c", "user.email=t@t"])
                .args([
                    "-c",
                    "commit.gpgsign=false",
                    "-c",
                    "core.hooksPath=/dev/null",
                ])
                .args(args)
                .env("GIT_COMMITTER_DATE", when)
                .env("GIT_AUTHOR_DATE", when)
                .output()
                .unwrap();
            assert!(out.status.success(), "{:?}", out);
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        git(&["init", "-q"], "2026-10-01T12:00:00+0000");
        git(
            &["commit", "-q", "--allow-empty", "-m", "late"],
            "2026-10-25T12:00:00+0000",
        );
        let late = git(&["rev-parse", "HEAD"], "2026-10-25T12:00:00+0000");
        git(
            &["commit", "-q", "--allow-empty", "-m", "early"],
            "2026-10-21T12:00:00+0000",
        );
        let early = git(&["rev-parse", "HEAD"], "2026-10-21T12:00:00+0000");
        let core = dir.to_string_lossy().to_string();
        assert_eq!(
            fix_day(&core, &[late.clone(), early.clone()]),
            Ok("2026-10-21".to_string())
        );
        assert_eq!(fix_day(&core, &[late]), Ok("2026-10-25".to_string()));
        assert!(fix_day(
            &core,
            &["0000000000000000000000000000000000000000".to_string()]
        )
        .is_err());
        assert!(fix_day(&core, &[]).is_err());
    }

    /// INT-300 G12: open bugs are counted by severity, worst first, with the ids of the high and
    /// the medium ones named; solved bugs are not counted as open.
    #[test]
    fn the_dashboard_counts_open_bugs_by_severity_worst_first() {
        let records = vec![
            rec("F-0001", "no", "high", "2026-10-01", "a"),
            rec("F-0002", "no", "medium", "2026-10-02", "b"),
            rec("F-0003", "no", "low", "2026-10-03", "c"),
            rec("F-0004", "no", "unrated", "2026-10-04", "d"),
            rec("F-0005", "no", "unrated", "2026-10-05", "e"),
            rec("F-0006", "no", "low", "2026-10-06", "f"),
        ];
        let fixes = Ok(vec![fixed_by("F-0006")]);
        let got = dashboard_report(
            &records,
            &fixes,
            &[on_day("F-0006", "2026-10-29")],
            "2026-10-30",
            6,
        );
        assert_eq!(got.code, 0, "{}", got.text);
        assert_eq!(words_of(&got.text, "high"), ["high", "1", "F-0001"]);
        assert_eq!(words_of(&got.text, "medium"), ["medium", "1", "F-0002"]);
        assert_eq!(words_of(&got.text, "low"), ["low", "1"]);
        assert_eq!(words_of(&got.text, "unrated"), ["unrated", "2"]);
        assert!(
            got.text.contains("6 findings: 5 open, 1 solved"),
            "{}",
            got.text
        );
        let high = got.text.find("  high").unwrap();
        let unrated = got.text.find("  unrated").unwrap();
        assert!(high < unrated, "{}", got.text);
    }

    /// INT-300 G12: a severity outside the four words is counted as other, named, and still adds
    /// up to the open count.
    #[test]
    fn a_severity_outside_the_four_words_is_counted_as_other() {
        let records = vec![
            rec("F-0001", "no", "urgent", "2026-10-01", "a"),
            rec("F-0002", "no", "low", "2026-10-02", "b"),
        ];
        let got = dashboard_report(&records, &Ok(Vec::new()), &[], "2026-10-30", 2);
        assert_eq!(words_of(&got.text, "other"), ["other", "1", "F-0001"]);
        assert!(
            got.text.contains("adds up to 2 of 2 open -- agree"),
            "{}",
            got.text
        );
        assert_eq!(got.code, 0, "{}", got.text);
    }

    /// INT-300 G12: the oldest open bug is the earliest found day among the open ones only, ties
    /// go to the lower id, and its age is the days from that day to today.
    #[test]
    fn the_oldest_open_bug_ignores_solved_ones_and_breaks_ties_by_id() {
        let records = vec![
            rec("F-0001", "no", "low", "2026-09-01", "old but solved"),
            rec("F-0002", "no", "low", "2026-10-10", "first of two"),
            rec("F-0003", "no", "low", "2026-10-10", "second of two"),
        ];
        let got = dashboard_report(
            &records,
            &Ok(vec![fixed_by("F-0001")]),
            &[on_day("F-0001", "2026-09-05")],
            "2026-10-30",
            3,
        );
        let w = words_of(&got.text, "oldest");
        assert_eq!(
            &w[..7],
            [
                "oldest",
                "open",
                "F-0002",
                "found",
                "2026-10-10",
                "20",
                "days"
            ]
        );
        assert!(got.text.contains("first of two"), "{}", got.text);
    }

    /// INT-300 G12: with no open bug the oldest line says none; an open bug found after today says
    /// so rather than a negative age; open bugs with no readable day are named as left out.
    #[test]
    fn the_oldest_open_line_says_none_after_today_or_left_out() {
        let none = dashboard_report(
            &[rec("F-0001", "duplicate", "low", "2026-10-01", "a")],
            &Ok(Vec::new()),
            &[],
            "2026-10-30",
            1,
        );
        assert!(none.text.contains("oldest open   none"), "{}", none.text);
        let later = dashboard_report(
            &[rec("F-0001", "no", "low", "2026-11-02", "a")],
            &Ok(Vec::new()),
            &[],
            "2026-10-30",
            1,
        );
        assert!(later.text.contains("after today"), "{}", later.text);
        assert!(!later.text.contains("-3 days"), "{}", later.text);
        let mixed = dashboard_report(
            &[
                rec("F-0001", "no", "low", "unknown", "a"),
                rec("F-0002", "no", "low", "2026-10-20", "b"),
            ],
            &Ok(Vec::new()),
            &[],
            "2026-10-30",
            2,
        );
        assert!(mixed.text.contains("F-0002"), "{}", mixed.text);
        assert!(
            mixed
                .text
                .contains("left out: 1 open without a readable found day"),
            "{}",
            mixed.text
        );
    }

    /// INT-300 G12: the median is over the solved bugs that can be timed, a fix dated before the
    /// bug was found is left out and named, and the line says how many it is over.
    #[test]
    fn the_median_fix_time_leaves_out_a_fix_dated_before_the_find() {
        let records = vec![
            rec("F-0001", "no", "low", "2026-10-01", "a"),
            rec("F-0002", "no", "low", "2026-10-01", "b"),
            rec("F-0003", "no", "low", "2026-10-10", "c"),
            rec("F-0004", "no", "low", "2026-10-01", "d"),
        ];
        let fixes = Ok(vec![
            fixed_by("F-0001"),
            fixed_by("F-0002"),
            fixed_by("F-0003"),
            fixed_by("F-0004"),
        ]);
        let fixed_on = [
            on_day("F-0001", "2026-10-04"),
            on_day("F-0002", "2026-10-02"),
            on_day("F-0003", "2026-10-05"),
            on_day("F-0004", "2026-10-08"),
        ];
        let got = dashboard_report(&records, &fixes, &fixed_on, "2026-10-30", 4);
        let w = words_of(&got.text, "median");
        assert_eq!(&w[..4], ["median", "fix", "3", "days,"]);
        assert!(got.text.contains("over 3 solved"), "{}", got.text);
        assert!(
            got.text.contains("F-0003 (fixed before it was found)"),
            "{}",
            got.text
        );
    }

    /// INT-300 G12: an even count takes the mean of the middle two, which can be a half day, and
    /// no solved bug at all reads unknown, never 0 days.
    #[test]
    fn the_median_can_be_a_half_day_and_no_solved_bug_reads_unknown() {
        let records = vec![
            rec("F-0001", "no", "low", "2026-10-01", "a"),
            rec("F-0002", "no", "low", "2026-10-01", "b"),
        ];
        let fixes = Ok(vec![fixed_by("F-0001"), fixed_by("F-0002")]);
        let both = [
            on_day("F-0001", "2026-10-04"),
            on_day("F-0002", "2026-10-03"),
        ];
        let half = dashboard_report(&records, &fixes, &both, "2026-10-30", 2);
        assert!(
            half.text.contains("median fix    2.5 days"),
            "{}",
            half.text
        );
        let none = dashboard_report(&records, &Ok(Vec::new()), &[], "2026-10-30", 2);
        assert!(
            none.text
                .contains("median fix    unknown -- no solved bug can be timed"),
            "{}",
            none.text
        );
        assert!(!none.text.contains("0 days"), "{}", none.text);
    }

    /// INT-300 G12: the last 7 days are today and the six before it. A fix six days ago is in, one
    /// seven days ago is out, a fix dated after today is left out and named, and the window is
    /// printed.
    #[test]
    fn the_last_seven_days_include_today_and_the_six_before() {
        let records = vec![
            rec("F-0001", "no", "low", "2026-10-01", "today"),
            rec("F-0002", "no", "low", "2026-10-01", "six ago"),
            rec("F-0003", "no", "low", "2026-10-01", "seven ago"),
            rec("F-0004", "no", "low", "2026-10-01", "tomorrow"),
        ];
        let fixes = Ok(vec![
            fixed_by("F-0001"),
            fixed_by("F-0002"),
            fixed_by("F-0003"),
            fixed_by("F-0004"),
        ]);
        let fixed_on = [
            on_day("F-0001", "2026-10-30"),
            on_day("F-0002", "2026-10-24"),
            on_day("F-0003", "2026-10-23"),
            on_day("F-0004", "2026-10-31"),
        ];
        let got = dashboard_report(&records, &fixes, &fixed_on, "2026-10-30", 4);
        let line = got
            .text
            .lines()
            .find(|l| l.starts_with("solved in the last 7 days"))
            .unwrap();
        assert!(line.contains("(2026-10-24 to 2026-10-30)"), "{}", line);
        assert!(line.contains(": F-0001 F-0002"), "{}", line);
        assert!(
            !line.contains("F-0003") && !line.contains("F-0004"),
            "{}",
            line
        );
        assert!(
            got.text.contains("F-0004 (fixed after today)"),
            "{}",
            got.text
        );
    }

    /// INT-300 G12: a solved bug whose fix day was not given is left out of the median and the
    /// window and named, not counted as today.
    #[test]
    fn a_solved_bug_with_no_fix_day_is_left_out_and_named() {
        let records = vec![rec("F-0001", "no", "low", "2026-10-01", "a")];
        let got = dashboard_report(
            &records,
            &Ok(vec![fixed_by("F-0001")]),
            &[],
            "2026-10-30",
            1,
        );
        assert!(
            got.text.contains("F-0001 (no readable fix day)"),
            "{}",
            got.text
        );
        assert!(got.text.contains("median fix    unknown"), "{}", got.text);
        assert!(
            got.text.contains("(2026-10-24 to 2026-10-30)  0\n"),
            "{}",
            got.text
        );
    }

    /// INT-300 G12, G7: when git could not be read nothing is counted as open, no zero is printed
    /// for a number that is unknown, and the exit is 3.
    #[test]
    fn the_dashboard_says_unknown_and_exits_3_when_git_cannot_be_read() {
        let records = vec![
            rec("F-0001", "no", "high", "2026-10-01", "a"),
            rec("F-0002", "duplicate", "low", "2026-10-02", "b"),
        ];
        let got = dashboard_report(
            &records,
            &Err("git log could not run".to_string()),
            &[],
            "2026-10-30",
            2,
        );
        assert_eq!(got.code, 3, "{}", got.text);
        assert!(
            got.text.contains("open by severity: unknown"),
            "{}",
            got.text
        );
        assert!(got.text.contains("oldest open: unknown"), "{}", got.text);
        assert!(got.text.contains("median fix: unknown"), "{}", got.text);
        assert!(
            got.text.contains("(2026-10-24 to 2026-10-30): unknown"),
            "{}",
            got.text
        );
        assert!(words_of(&got.text, "high").is_empty(), "{}", got.text);
        assert!(got.text.contains("1 unknown"), "{}", got.text);
    }

    /// INT-300 G12, G5: the dashboard ends with the recount; when the files on disk and the records
    /// read disagree it says so and exits 1.
    #[test]
    fn the_dashboard_recount_disagrees_and_exits_1_when_a_file_is_not_read() {
        let records = vec![rec("F-0001", "no", "low", "2026-10-01", "a")];
        let agree = dashboard_report(&records, &Ok(Vec::new()), &[], "2026-10-30", 1);
        assert_eq!(agree.code, 0, "{}", agree.text);
        assert!(
            agree
                .text
                .contains("1 finding files on disk, 1 records read -- they agree"),
            "{}",
            agree.text
        );
        let off = dashboard_report(&records, &Ok(Vec::new()), &[], "2026-10-30", 2);
        assert_eq!(off.code, 1, "{}", off.text);
        assert!(off.text.contains("they disagree"), "{}", off.text);
    }

    /// INT-300 G12: a day that is not a day for today is a refusal that ends 3, never a dashboard
    /// built on a guess.
    #[test]
    fn a_today_that_is_not_a_day_ends_3() {
        let got = dashboard_report(&[], &Ok(Vec::new()), &[], "yesterday", 0);
        assert_eq!(got.code, 3);
        assert!(got.text.contains("today could not be read"), "{}", got.text);
    }

    /// INT-300 G12: the header line is the G5 count line's numbers, the same rows counted.
    #[test]
    fn the_dashboard_header_counts_the_same_rows_as_the_list() {
        let records = vec![
            rec("F-0001", "no", "low", "2026-10-01", "a"),
            rec("F-0002", "no", "low", "2026-10-02", "b"),
            rec("F-0003", "duplicate", "low", "2026-10-03", "c"),
        ];
        let fixes = Ok(vec![fixed_by("F-0002")]);
        let got = dashboard_report(
            &records,
            &fixes,
            &[on_day("F-0002", "2026-10-20")],
            "2026-10-30",
            3,
        );
        assert!(
            got.text
                .starts_with("2026-10-30 -- 3 findings: 1 open, 1 solved, 1 closed, 0 unknown\n"),
            "{}",
            got.text
        );
        let all = rows(&records, &fixes);
        assert_eq!(
            render_summary(&counts(&all), 0)
                .split(" -- ")
                .next()
                .unwrap(),
            "3 findings: 1 open, 1 solved, 1 closed, 0 unknown"
        );
    }

    /// INT-300 G13: the welcome line counts open bugs and the high ones among them; solved and
    /// closed bugs are not open, and a bug that is only unrated is not high.
    #[test]
    fn the_welcome_line_counts_open_and_high() {
        let records = vec![
            rec("F-0001", "no", "high", "2026-10-01", "a"),
            rec("F-0002", "no", "high", "2026-10-02", "b"),
            rec("F-0003", "no", "unrated", "2026-10-03", "c"),
            rec("F-0004", "no", "high", "2026-10-04", "solved"),
            rec("F-0005", "duplicate", "high", "2026-10-05", "closed"),
        ];
        let got = welcome_line(&records, &Ok(vec![fixed_by("F-0004")]), 5);
        assert_eq!(got.text, "bugs: 3 open (2 high)\n");
        assert_eq!(got.code, 0);
    }

    /// INT-300 G13: the welcome line and the dashboard are one derivation: for the same bugs the
    /// open count and the high count are the same numbers.
    #[test]
    fn the_welcome_line_agrees_with_the_dashboard() {
        let records = vec![
            rec("F-0001", "no", "high", "2026-10-01", "a"),
            rec("F-0002", "no", "medium", "2026-10-02", "b"),
            rec("F-0003", "no", "low", "2026-10-03", "c"),
            rec("F-0004", "no", "high", "2026-10-04", "d"),
            rec("F-0005", "no", "high", "2026-10-05", "e"),
        ];
        let fixes = Ok(vec![fixed_by("F-0005")]);
        let line = welcome_line(&records, &fixes, 5);
        let board = dashboard_report(
            &records,
            &fixes,
            &[on_day("F-0005", "2026-10-29")],
            "2026-10-30",
            5,
        );
        assert!(board.text.contains("4 open"), "{}", board.text);
        assert_eq!(words_of(&board.text, "high")[1], "2");
        assert_eq!(line.text, "bugs: 4 open (2 high)\n");
    }

    /// INT-300 G13, G7: when git cannot be read, or a finding file was not read, the line says
    /// unknown and ends 3. It never prints a zero for a number it could not measure.
    #[test]
    fn the_welcome_line_says_unknown_when_it_cannot_answer() {
        let records = vec![rec("F-0001", "no", "high", "2026-10-01", "a")];
        let git = welcome_line(&records, &Err("git log could not run".to_string()), 1);
        assert_eq!(git.text, "bugs: unknown\n");
        assert_eq!(git.code, 3);
        let short = welcome_line(&records, &Ok(Vec::new()), 2);
        assert_eq!(short.text, "bugs: unknown\n");
        assert_eq!(short.code, 3);
        assert!(!git.text.contains("0 open") && !short.text.contains("0 open"));
    }

    /// INT-300 G13, G7: a measured zero is a number. With every bug solved or closed the line says
    /// 0 open, not unknown.
    #[test]
    fn a_measured_zero_is_a_number_not_unknown() {
        let records = vec![
            rec("F-0001", "no", "high", "2026-10-01", "solved"),
            rec("F-0002", "duplicate", "low", "2026-10-02", "closed"),
        ];
        let got = welcome_line(&records, &Ok(vec![fixed_by("F-0001")]), 2);
        assert_eq!(got.text, "bugs: 0 open (0 high)\n");
        assert_eq!(got.code, 0);
    }
}
