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
}
