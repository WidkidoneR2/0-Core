//! INT-313 stage 2. One reader of the intent shelves.
//!
//! The status line in the frontmatter is the state. A directory that is not
//! there is skipped. Findings are a different id scheme and are not loaded here.
//! The names match `folders::LIFECYCLE` and `folders::RECORDS`.

use std::fs;
use std::path::{Path, PathBuf};

/// Lifecycle shelves. None of these names is the status.
pub const LIFECYCLE: &[&str] = &["open"];

/// Record shelves. Not intents.
pub const RECORDS: &[&str] = &["decisions", "experiments", "incidents", "philosophy"];

/// One markdown file on a shelf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentFile {
    pub path: PathBuf,
    pub folder: String,
    pub id: String,
    pub status: String,
    pub title: String,
    pub content: String,
}

impl IntentFile {
    /// A record shelf, not an intent.
    pub fn is_record(&self) -> bool {
        RECORDS.contains(&self.folder.as_str())
    }
}

/// Intent files. Record shelves are not included.
pub fn live() -> Vec<IntentFile> {
    load()
        .into_iter()
        .filter(|file| !file.is_record())
        .collect()
}

/// Completed intent ids. The live shelf contributes files whose status is complete.
/// `closed.tsv` contributes rows whose status is complete. Git contributes every id
/// that ever lived in `zero/intents/complete/`. If git cannot answer, that archive is
/// skipped. A missing closed.tsv adds nothing. An unreadable one is reported and adds
/// nothing, which is not the same as a file that was read and was empty.
pub fn completed_ids() -> std::collections::HashSet<String> {
    let mut ids: std::collections::HashSet<String> = with_status("complete")
        .into_iter()
        .map(|file| file.id)
        .collect();
    match completed_ids_from(&crate::paths::intents_dir()) {
        Ok(closed) => {
            for warning in &closed.warnings {
                eprintln!("closed.tsv: {warning}");
            }
            ids.extend(closed.ids);
        }
        Err(err) => eprintln!("closed.tsv: {err}"),
    }
    if let Some(archived) = git_ids_in(&["complete"]) {
        ids.extend(archived);
    }
    ids
}

/// One row of `zero/intents/closed.tsv`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedRow {
    pub id: String,
    pub status: String,
    pub closed: String,
    pub last_commit: String,
    pub title: String,
    /// The sixth column, when the row has one. Empty means it does not.
    pub reason: Option<String>,
}

/// Rows read from closed.tsv, plus one warning per malformed row.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClosedIndex {
    pub rows: Vec<ClosedRow>,
    pub warnings: Vec<String>,
}

/// Complete ids from closed.tsv, plus the warnings for rows that were not rows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClosedIds {
    pub ids: std::collections::HashSet<String>,
    pub warnings: Vec<String>,
}

/// Read `closed.tsv` in `dir` (the intents directory).
///
/// A missing file is an empty index. An unreadable file is an error naming the path.
/// A header line whose first field is `id` is skipped: five columns, or six
/// ending in `reason`. A row has five fields, or six when it records a reason.
/// An empty sixth field is no reason. A malformed row is named in `warnings`
/// and is not a row; the rows around it are kept.
pub fn read_closed(dir: &Path) -> Result<ClosedIndex, String> {
    let path = dir.join("closed.tsv");
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ClosedIndex::default());
        }
        Err(e) => {
            return Err(format!(
                "closed.tsv at {} could not be read: {e}",
                path.display()
            ));
        }
    };
    Ok(parse_closed(&text))
}

fn parse_closed(text: &str) -> ClosedIndex {
    let mut index = ClosedIndex::default();
    for (n, raw) in text.lines().enumerate() {
        let line = raw.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        let first = line.split('\t').next().unwrap_or("").trim();
        if first == "id" {
            continue;
        }
        match parse_closed_row(line) {
            Ok(row) => index.rows.push(row),
            Err(why) => index
                .warnings
                .push(format!("line {}: {why}: {line}", n + 1)),
        }
    }
    index
}

fn parse_closed_row(line: &str) -> Result<ClosedRow, String> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() != 5 && parts.len() != 6 {
        return Err(format!(
            "expected 5 or 6 tab-separated fields, found {}",
            parts.len()
        ));
    }
    let id = parts[0].trim();
    let status = parts[1].trim();
    let closed = parts[2].trim();
    let last_commit = parts[3].trim();
    let title = parts[4].trim();
    let reason = if parts.len() == 6 {
        let reason = parts[5].trim();
        if reason.is_empty() {
            None
        } else {
            Some(reason.to_string())
        }
    } else {
        None
    };
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("id {id:?} is not digits"));
    }
    if status != "complete" && status != "cancelled" {
        return Err(format!("status {status:?} is not complete or cancelled"));
    }
    if !is_closed_date(closed) {
        return Err(format!("closed date {closed:?} is not YYYY-MM-DD"));
    }
    if last_commit.is_empty() || last_commit.chars().any(char::is_whitespace) {
        return Err("last_commit is empty".to_string());
    }
    if title.is_empty() {
        return Err("title is empty".to_string());
    }
    Ok(ClosedRow {
        id: id.to_string(),
        status: status.to_string(),
        closed: closed.to_string(),
        last_commit: last_commit.to_string(),
        title: title.to_string(),
        reason,
    })
}

fn is_closed_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    let digits = |slice: &[u8]| slice.iter().all(|c| c.is_ascii_digit());
    if !digits(&bytes[..4]) || !digits(&bytes[5..7]) || !digits(&bytes[8..]) {
        return false;
    }
    let month: u32 = value[5..7].parse().unwrap_or(0);
    let day: u32 = value[8..].parse().unwrap_or(0);
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

/// Ids in closed.tsv whose status is complete. Cancelled rows are not ids.
/// Missing file: an empty index. Unreadable: Err. Malformed rows stay in `warnings`.
pub fn completed_ids_from(dir: &Path) -> Result<ClosedIds, String> {
    let index = read_closed(dir)?;
    let mut ids = std::collections::HashSet::new();
    for row in index.rows {
        if row.status == "complete" {
            ids.insert(row.id);
        }
    }
    Ok(ClosedIds {
        ids,
        warnings: index.warnings,
    })
}

/// Old ledgers whose ids are listed in `historical-ids.tsv`.
/// `zero/intents/` is the live ledger and is not one of these.
pub const HISTORICAL_PREFIXES: &[&str] = &["intents/", "faelight/intents/", "INTENT/"];

/// One id from `historical-ids.tsv`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalRow {
    pub id: u32,
    pub title: String,
}

/// Rows read from `historical-ids.tsv`, plus one warning per malformed line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HistoricalIndex {
    pub rows: Vec<HistoricalRow>,
    pub warnings: Vec<String>,
}

/// Read `historical-ids.tsv` in `dir` (the intents directory).
///
/// A missing file is an empty index. An unreadable file is an error naming the path.
/// A line that is empty or starts with `#` is a comment. A row is an id, or an id,
/// a tab, and a title. A duplicate id keeps the first row. `000` and `999` are not
/// special here: this reader returns the rows that are written.
pub fn read_historical(dir: &Path) -> Result<HistoricalIndex, String> {
    let path = dir.join("historical-ids.tsv");
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(HistoricalIndex::default());
        }
        Err(e) => {
            return Err(format!(
                "historical-ids.tsv at {} could not be read: {e}",
                path.display()
            ));
        }
    };
    Ok(parse_historical(&text))
}

fn parse_historical(text: &str) -> HistoricalIndex {
    let mut index = HistoricalIndex::default();
    let mut seen = std::collections::BTreeSet::new();
    for (n, raw) in text.lines().enumerate() {
        let line = raw.trim_end_matches('\r').trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (id_str, title) = match line.split_once('\t') {
            Some((id, title)) => (id.trim(), title.trim()),
            None => (line, ""),
        };
        if id_str.is_empty() || !id_str.bytes().all(|b| b.is_ascii_digit()) {
            index.warnings.push(format!(
                "line {}: id {id_str:?} is not digits: {line}",
                n + 1
            ));
            continue;
        }
        let Ok(id) = id_str.parse::<u32>() else {
            index
                .warnings
                .push(format!("line {}: id {id_str} is not a u32: {line}", n + 1));
            continue;
        };
        if !seen.insert(id) {
            continue;
        }
        index.rows.push(HistoricalRow {
            id,
            title: title.to_string(),
        });
    }
    index
}

/// Lifecycle shelves an old intent file may sit in. `arch-era` is a parent of
/// these shelves, not a shelf of its own.
const HISTORICAL_LIFECYCLE: &[&str] = &[
    "future",
    "planned",
    "in-progress",
    "complete",
    "cancelled",
    "archive",
    "open",
    "deferred",
    "active",
];

/// Intent id in a numbered intent file under [`HISTORICAL_PREFIXES`].
///
/// The file's directory is a lifecycle shelf (`future`, `planned`,
/// `in-progress`, `complete`, `cancelled`, `archive`, `open`, `deferred`,
/// `active`), including `arch-era/<shelf>`. The filename matches `^[0-9]{3}-`.
/// Record shelves ([`RECORDS`]) are not intents. `000` and `999` are reserved.
/// A path under `zero/intents/` is the live ledger, not this list.
pub fn historical_id_from_path(path: &str) -> Option<u32> {
    let path = path.trim().trim_matches('"');
    let rest = HISTORICAL_PREFIXES
        .iter()
        .find_map(|prefix| path.strip_prefix(prefix))?;
    let parts: Vec<&str> = rest.split('/').collect();
    if parts.len() < 2 {
        return None;
    }
    let base = parts[parts.len() - 1];
    let dirs = &parts[..parts.len() - 1];
    if dirs.iter().any(|dir| RECORDS.contains(dir)) {
        return None;
    }
    let parent = *dirs.last()?;
    if !HISTORICAL_LIFECYCLE.contains(&parent) {
        return None;
    }
    if dirs.len() == 2 && dirs[0] != "arch-era" {
        return None;
    }
    if dirs.len() > 2 {
        return None;
    }
    let stem = base.strip_suffix(".md")?;
    let (num, slug) = stem.split_once('-')?;
    if slug.is_empty() || num.len() != 3 || !num.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n = num.parse::<u32>().ok()?;
    if n == 0 || n == 999 {
        None
    } else {
        Some(n)
    }
}

/// Ids git has stored under `zero/intents/<folder>/` for one of `folders`.
/// None when git cannot answer, so the caller does not invent an empty archive.
pub fn git_ids_in(folders: &[&str]) -> Option<std::collections::HashSet<String>> {
    let repo = crate::paths::core_dir();
    let out = std::process::Command::new("git")
        .args([
            "-C",
            repo.to_string_lossy().as_ref(),
            "log",
            "--all",
            "--name-only",
            "--format=",
            "--",
            "zero/intents",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let mut ids = std::collections::HashSet::new();
    for line in text.lines() {
        if let Some(id) = id_under(line, folders) {
            ids.insert(id);
        }
    }
    Some(ids)
}

fn id_under(path: &str, folders: &[&str]) -> Option<String> {
    let path = path.trim().trim_matches('"');
    let mut parts = path.split('/');
    if parts.next() != Some("zero") || parts.next() != Some("intents") {
        return None;
    }
    let folder = parts.next()?;
    let file = parts.next()?;
    if parts.next().is_some() || !folders.contains(&folder) || !file.ends_with(".md") {
        return None;
    }
    let stem = &file[..file.len() - 3];
    let (num, rest) = stem.split_once('-')?;
    if rest.is_empty() || num.is_empty() || !num.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(num.to_string())
}

/// Lifecycle files whose frontmatter status is `status`. Records are not included.
pub fn with_status(status: &str) -> Vec<IntentFile> {
    load()
        .into_iter()
        .filter(|file| !file.is_record() && file.status == status)
        .collect()
}

/// Every intent and record file under the live checkout. Missing shelves are skipped.
pub fn load() -> Vec<IntentFile> {
    load_from(&crate::paths::intents_dir())
}

/// The same read, against a chosen intents directory. Tests use this.
pub fn load_from(base: &Path) -> Vec<IntentFile> {
    let mut out = Vec::new();
    for folder in LIFECYCLE.iter().chain(RECORDS.iter()) {
        let dir = base.join(folder);
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "README.md" {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            let (id, status, title) = frontmatter(&content);
            let id = if id.is_empty() {
                name.split('-').next().unwrap_or("").to_string()
            } else {
                id
            };
            out.push(IntentFile {
                path,
                folder: (*folder).to_string(),
                id,
                status,
                title,
                content,
            });
        }
    }
    out
}

/// id, status, and title from the frontmatter block only. A quote in the body is not the status.
fn frontmatter(content: &str) -> (String, String, String) {
    let fm = if content.starts_with("---") {
        content.split("---").nth(1).unwrap_or("")
    } else {
        ""
    };
    let field = |key: &str| {
        fm.lines()
            .find(|line| line.trim_start().starts_with(&format!("{key}:")))
            .map(|line| {
                line.split_once(':')
                    .map(|(_, value)| value)
                    .unwrap_or("")
                    .trim()
                    .trim_matches('"')
                    .to_string()
            })
            .unwrap_or_default()
    };
    (field("id"), field("status"), field("title"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn status_comes_from_the_frontmatter_and_a_missing_shelf_is_skipped() {
        let dir = std::env::temp_dir().join(format!("ledger-load-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let future = dir.join("open");
        fs::create_dir_all(&future).unwrap();
        fs::write(
            future.join("308-quote.md"),
            "---\nid: 308\ntitle: \"quote\"\nstatus: planned\ndate: 2026-11-13\n---\n\nstatus: in-progress\n",
        )
        .unwrap();
        fs::write(
            future.join("010-live.md"),
            "---\nid: 010\ntitle: \"live\"\nstatus: in-progress\ndate: 2026-11-13\n---\n\n",
        )
        .unwrap();
        let got = load_from(&dir);
        let _ = fs::remove_dir_all(&dir);
        let live = got.iter().find(|r| r.id == "010").expect("live");
        assert_eq!(live.status, "in-progress");
        let quote = got.iter().find(|r| r.id == "308").expect("quote");
        assert_eq!(quote.status, "planned");
        assert!(got.iter().all(|r| r.folder == "open"));
        assert_eq!(got.len(), 2);
    }

    #[test]
    fn in_progress_is_one_status_line_not_every_quote() {
        let dir = std::env::temp_dir().join(format!("ledger-count-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("open")).unwrap();
        fs::write(
            dir.join("open/308-quote.md"),
            "---\nid: 308\nstatus: planned\n---\n\nstatus: in-progress\n",
        )
        .unwrap();
        fs::write(
            dir.join("open/313-live.md"),
            "---\nid: 313\nstatus: in-progress\n---\n\n",
        )
        .unwrap();
        let n = load_from(&dir)
            .into_iter()
            .filter(|file| !file.is_record() && file.status == "in-progress")
            .count();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(n, 1);
    }

    #[test]
    fn a_complete_history_path_is_an_id_and_a_record_shelf_is_not() {
        assert_eq!(
            id_under("zero/intents/complete/277-done.md", &["complete"]).as_deref(),
            Some("277")
        );
        assert_eq!(
            id_under("zero/intents/decisions/137-x.md", &["complete"]),
            None
        );
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ledger-closed-{}-{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_closed_complete_id_counts_and_a_cancelled_one_does_not() {
        let dir = scratch("counts");
        fs::write(
            dir.join("closed.tsv"),
            "id\tstatus\tclosed\tlast_commit\ttitle\n318\tcomplete\t2026-11-17\tabc1234\tfinished\n039\tcancelled\t2026-11-01\tdef5678\tdropped\n",
        )
        .unwrap();
        let got = completed_ids_from(&dir).expect("closed.tsv should be read");
        let _ = fs::remove_dir_all(&dir);
        assert!(
            got.ids.contains("318"),
            "a complete closed id counts as done: {:?}",
            got.ids
        );
        assert!(
            !got.ids.contains("039"),
            "a cancelled closed id does not count: {:?}",
            got.ids
        );
    }

    #[test]
    fn a_missing_closed_tsv_is_an_empty_index() {
        let dir = scratch("missing");
        let got =
            completed_ids_from(&dir).expect("a missing closed.tsv is an empty index, not an error");
        let _ = fs::remove_dir_all(&dir);
        assert!(got.ids.is_empty(), "{:?}", got.ids);
        assert!(got.warnings.is_empty(), "{:?}", got.warnings);
    }

    #[test]
    fn a_malformed_closed_row_is_named_and_not_dropped() {
        let dir = scratch("malformed");
        fs::write(
            dir.join("closed.tsv"),
            "id\tstatus\tclosed\tlast_commit\ttitle\n318\tcomplete\t2026-11-17\tabc1234\tkept\nbad\tbanana\tnot-a-date\t\tbroken\n",
        )
        .unwrap();
        let got = completed_ids_from(&dir)
            .expect("a malformed row is named, not a failure of the whole file");
        let _ = fs::remove_dir_all(&dir);
        assert!(got.ids.contains("318"), "a good row is kept: {:?}", got.ids);
        assert!(
            got.warnings.iter().any(|w| w.contains("banana")),
            "a malformed row is named: {:?}",
            got.warnings
        );
    }

    #[test]
    fn an_unreadable_closed_tsv_is_reported() {
        let dir = scratch("unreadable");
        fs::create_dir(dir.join("closed.tsv")).unwrap();
        let result = completed_ids_from(&dir);
        let _ = fs::remove_dir_all(&dir);
        match result {
            Err(err) => assert!(
                err.contains("could not be read"),
                "an unreadable closed.tsv must be reported, not treated as empty: {err}"
            ),
            Ok(got) => panic!(
                "an unreadable closed.tsv was treated as an empty index: {:?}",
                got.ids
            ),
        }
    }

    fn index_of(name: &str, text: &str) -> super::ClosedIndex {
        let dir = scratch(name);
        fs::write(dir.join("closed.tsv"), text).unwrap();
        let got = super::read_closed(&dir).expect("closed.tsv should be read");
        let _ = fs::remove_dir_all(&dir);
        got
    }

    #[test]
    fn five_fields_parse_and_a_sixth_is_the_reason() {
        let five = index_of(
            "five",
            "id\tstatus\tclosed\tlast_commit\ttitle\n181\tcomplete\t2026-11-19\tabc1234\tTitle\n",
        );
        assert!(five.warnings.is_empty(), "{:?}", five.warnings);
        assert_eq!(five.rows[0].reason, None, "five fields have no reason");

        let headed = index_of(
            "head6",
            "id\tstatus\tclosed\tlast_commit\ttitle\treason\n181\tcomplete\t2026-11-19\tabc1234\tTitle\n",
        );
        assert!(headed.warnings.is_empty(), "{:?}", headed.warnings);
        assert_eq!(headed.rows.len(), 1);
        assert_eq!(headed.rows[0].reason, None);

        let six = index_of(
            "six",
            "id\tstatus\tclosed\tlast_commit\ttitle\n181\tcancelled\t2026-11-19\tabc1234\tTitle\tkept\n",
        );
        let row = six
            .rows
            .iter()
            .find(|row| row.id == "181")
            .unwrap_or_else(|| {
                panic!(
                    "a 6-field row must parse and keep the reason: {:?}",
                    six.warnings
                )
            });
        assert_eq!(row.reason.as_deref(), Some("kept"));

        let empty = index_of(
            "empty6",
            "id\tstatus\tclosed\tlast_commit\ttitle\n181\tcancelled\t2026-11-19\tabc1234\tTitle\t\n",
        );
        let row = empty
            .rows
            .iter()
            .find(|row| row.id == "181")
            .unwrap_or_else(|| {
                panic!(
                    "an empty sixth field is no reason, not a malformed row: {:?}",
                    empty.warnings
                )
            });
        assert_eq!(row.reason, None);

        for (name, text) in [
            (
                "four",
                "id\tstatus\tclosed\tlast_commit\ttitle\n181\tcomplete\t2026-11-19\tabc\n",
            ),
            (
                "seven",
                "id\tstatus\tclosed\tlast_commit\ttitle\n181\tcomplete\t2026-11-19\tabc\tTitle\treason\textra\n",
            ),
        ] {
            let got = index_of(name, text);
            assert!(
                got.rows.is_empty(),
                "{name} must not be a row: {:?}",
                got.rows
            );
            assert!(
                !got.warnings.is_empty(),
                "{name} must be rejected: {:?}",
                got.warnings
            );
        }
    }

    #[test]
    fn a_missing_historical_list_is_empty() {
        let dir = scratch("hist-missing");
        let got = super::read_historical(&dir).expect("missing list is empty");
        let _ = fs::remove_dir_all(&dir);
        assert!(got.rows.is_empty(), "{:?}", got.rows);
        assert!(got.warnings.is_empty(), "{:?}", got.warnings);
    }

    #[test]
    fn historical_rows_keep_the_id_and_skip_comments() {
        let dir = scratch("hist-rows");
        fs::write(
            dir.join("historical-ids.tsv"),
            "# comment\n\n180\tsway-removal\n180\tduplicate\n075\thybrid\n",
        )
        .unwrap();
        let got = super::read_historical(&dir).expect("list should be read");
        let _ = fs::remove_dir_all(&dir);
        assert!(got.warnings.is_empty(), "{:?}", got.warnings);
        assert_eq!(got.rows.len(), 2, "a duplicate id keeps the first row");
        assert_eq!(got.rows[0].id, 180);
        assert_eq!(got.rows[0].title, "sway-removal");
        assert_eq!(got.rows[1].id, 75);
    }

    #[test]
    fn a_malformed_historical_line_is_named() {
        let dir = scratch("hist-bad");
        fs::write(dir.join("historical-ids.tsv"), "180\tsway\nnot-an-id\n").unwrap();
        let got = super::read_historical(&dir).expect("a bad line is a warning, not an error");
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(got.rows.len(), 1);
        assert!(
            got.warnings.iter().any(|w| w.contains("not-an-id")),
            "{:?}",
            got.warnings
        );
    }

    #[test]
    fn an_unreadable_historical_list_is_reported() {
        let dir = scratch("hist-unreadable");
        fs::create_dir(dir.join("historical-ids.tsv")).unwrap();
        let result = super::read_historical(&dir);
        let _ = fs::remove_dir_all(&dir);
        match result {
            Err(err) => assert!(
                err.contains("could not be read"),
                "an unreadable historical list must be reported: {err}"
            ),
            Ok(got) => panic!("an unreadable list was treated as empty: {got:?}"),
        }
    }

    #[test]
    fn legacy_paths_name_an_id_and_the_live_ledger_does_not() {
        assert_eq!(
            super::historical_id_from_path("intents/complete/180-sway-removal.md"),
            Some(180)
        );
        assert_eq!(
            super::historical_id_from_path("faelight/intents/future/351-nixos.md"),
            Some(351)
        );
        assert_eq!(
            super::historical_id_from_path("INTENT/future/075-hybrid-bar-dmenu.md"),
            Some(75)
        );
        assert_eq!(
            super::historical_id_from_path("intents/incidents/00-INDEX.md"),
            None
        );
        assert_eq!(
            super::historical_id_from_path("intents/complete/999-fixture.md"),
            None
        );
        assert_eq!(
            super::historical_id_from_path("zero/intents/open/352-live.md"),
            None
        );
        assert_eq!(
            super::historical_id_from_path("intents/arch-era/complete/349-parity.md"),
            Some(349)
        );
        assert_eq!(
            super::historical_id_from_path("intents/decisions/043-paru.md"),
            None
        );
        assert_eq!(
            super::historical_id_from_path("intents/incidents/190-login.md"),
            None
        );
        assert_eq!(
            super::historical_id_from_path("intents/philosophy/002-tool.md"),
            None
        );
        assert_eq!(
            super::historical_id_from_path("intents/experiments/001-starship.md"),
            None
        );
        assert_eq!(
            super::historical_id_from_path(
                "faelight/intents/incidents/2025-sudo-authentication-failure.md"
            ),
            None
        );
        assert_eq!(
            super::historical_id_from_path("INTENT/064-rust-tools-improvements.md"),
            None
        );
    }

    /// INT-352. The checked-in list stays inside three-digit intent ids.
    /// 000 and 999 are reserved. A year-sized incident number does not belong.
    #[test]
    fn historical_list_ids_do_not_exceed_999_or_use_000_or_999() {
        let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = loop {
            if dir.join("zero/intents/historical-ids.tsv").is_file() {
                break dir;
            }
            if !dir.pop() {
                panic!("historical-ids.tsv is not in this source tree");
            }
        };
        let listed = super::read_historical(&root.join("zero/intents"))
            .expect("historical-ids.tsv should be readable");
        assert!(
            listed.warnings.is_empty(),
            "historical-ids.tsv has warnings: {:?}",
            listed.warnings
        );
        assert!(!listed.rows.is_empty(), "the historical list is empty");
        for row in &listed.rows {
            assert!(
                (1..999).contains(&row.id),
                "id {} is outside 001-998",
                row.id
            );
        }
        let ids: std::collections::BTreeSet<u32> = listed.rows.iter().map(|row| row.id).collect();
        assert!(!ids.contains(&0));
        assert!(!ids.contains(&999));
        assert!(!ids.contains(&2025));
        assert!(!ids.contains(&2026));
        assert!(!ids.contains(&2027));
        assert!(*ids.iter().next_back().unwrap() <= 351);
    }

    /// INT-352 G3. Every intent id git names in a lifecycle shelf is in the list.
    /// Not a git checkout: print a note and skip. Record shelves are not intents.
    /// 000 and 999 are reserved.
    #[test]
    fn historical_list_covers_every_old_prefix_id_git_names() {
        let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = loop {
            if dir.join(".git").exists() {
                break Some(dir.clone());
            }
            if !dir.pop() {
                break None;
            }
        };
        let Some(root) = root else {
            eprintln!("historical-ids drift: not a git checkout, skipped");
            return;
        };
        let specs: Vec<&str> = super::HISTORICAL_PREFIXES
            .iter()
            .map(|prefix| prefix.trim_end_matches('/'))
            .collect();
        let mut args = vec![
            "-C",
            root.to_str().expect("repo path is utf-8"),
            "log",
            "--all",
            "--name-only",
            "--format=",
            "--",
        ];
        args.extend(specs);
        let output = match std::process::Command::new("git").args(&args).output() {
            Ok(output) => output,
            Err(err) => {
                eprintln!("historical-ids drift: git could not be started ({err}), skipped");
                return;
            }
        };
        assert!(
            output.status.success(),
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8(output.stdout).expect("git log is utf-8");
        let mut git_ids = std::collections::BTreeSet::new();
        for line in text.lines() {
            if let Some(id) = super::historical_id_from_path(line) {
                git_ids.insert(id);
            }
        }
        let listed = super::read_historical(&root.join("zero/intents"))
            .expect("historical-ids.tsv should be readable");
        assert!(
            listed.warnings.is_empty(),
            "historical-ids.tsv has warnings: {:?}",
            listed.warnings
        );
        let have: std::collections::BTreeSet<u32> = listed.rows.iter().map(|row| row.id).collect();
        let missing: Vec<u32> = git_ids.difference(&have).copied().collect();
        assert!(
            missing.is_empty(),
            "historical-ids.tsv lacks ids git names under the old prefixes: {missing:?}"
        );
        for id in [
            180, 322, 326, 333, 334, 341, 342, 346, 347, 348, 349, 350, 351,
        ] {
            assert!(have.contains(&id), "list lacks {id}");
            assert!(git_ids.contains(&id), "git did not name {id}");
        }
        assert!(!have.contains(&0));
        assert!(!have.contains(&999));
    }
}
