//! INT-313 stage 2. One reader of the intent shelves.
//!
//! The status line in the frontmatter is the state. A directory that is not
//! there is skipped. Findings are a different id scheme and are not loaded here.
//! The names match `folders::LIFECYCLE` and `folders::RECORDS` until stage 4.

use std::fs;
use std::path::{Path, PathBuf};

/// Lifecycle shelves. None of these names is the status.
pub const LIFECYCLE: &[&str] = &["future", "planned", "in-progress", "complete", "cancelled"];

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
        let future = dir.join("future");
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
        assert!(got.iter().all(|r| r.folder == "future"));
        assert_eq!(got.len(), 2);
    }

    #[test]
    fn in_progress_is_one_status_line_not_every_quote() {
        let dir = std::env::temp_dir().join(format!("ledger-count-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("future")).unwrap();
        fs::create_dir_all(dir.join("in-progress")).unwrap();
        fs::write(
            dir.join("future/308-quote.md"),
            "---\nid: 308\nstatus: planned\n---\n\nstatus: in-progress\n",
        )
        .unwrap();
        fs::write(
            dir.join("in-progress/313-live.md"),
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
}
