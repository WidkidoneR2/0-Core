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
/// Git contributes every id that ever lived in `zero/intents/complete/`. If git
/// cannot answer, only the live shelf is returned.
pub fn completed_ids() -> std::collections::HashSet<String> {
    let mut ids: std::collections::HashSet<String> = with_status("complete")
        .into_iter()
        .map(|file| file.id)
        .collect();
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
/// INT-320 red: the file is not read yet, so every directory looks empty.
/// A missing file and an unreadable file are not told apart.
pub fn read_closed(dir: &Path) -> Result<ClosedIndex, String> {
    let _ = dir;
    Ok(ClosedIndex::default())
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
        let dir = std::env::temp_dir().join(format!(
            "ledger-closed-{}-{}-{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
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
}
