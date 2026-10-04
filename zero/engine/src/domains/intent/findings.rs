//! INT-266: findings -- something found and not fixed, one record per file under
//! zero/intents/findings/, named F-NNNN.md (ruled by Christian, 2026-10-04).
//!
//! A record holds only facts from the moment it was filed. Whether a finding is fixed is never
//! stored here: it is derived from Fixes: trailers in git. Records are never deleted, because a
//! Fixes: trailer must always point at a real record, and an id is never reused.

use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::{CoreError, CoreResult};
use colored::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Why a record was not written. Either way, nothing on disk changed.
#[derive(Debug, PartialEq)]
pub enum WriteError {
    /// A record with this id already exists. It was left exactly as it was.
    Exists(PathBuf),
    /// The record could not be written, for the reason given.
    Io(PathBuf, String),
}

/// Write one finding record as `<dir>/<id>.md`. Refuses rather than overwrite.
pub fn write_record(dir: &Path, id: &str, body: &str) -> Result<PathBuf, WriteError> {
    let path = dir.join(format!("{}.md", id));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::AlreadyExists => WriteError::Exists(path.clone()),
            _ => WriteError::Io(path.clone(), e.to_string()),
        })?;
    file.write_all(body.as_bytes())
        .map_err(|e| WriteError::Io(path.clone(), e.to_string()))?;
    Ok(path)
}

/// The id form ruled 2026-10-04: F- and four digits.
pub fn finding_id(n: u32) -> String {
    format!("F-{:04}", n)
}

/// The highest F-number on disk. An absent folder truly holds none, so it answers 0; a folder
/// that exists but cannot be read is an error, never 0 (INT-192). Names that are not F-NNNN.md
/// are not counted.
pub fn highest_number(dir: &Path) -> Result<u32, WriteError> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(WriteError::Io(dir.to_path_buf(), e.to_string())),
    };
    let mut max = 0;
    for entry in entries {
        let entry = entry.map_err(|e| WriteError::Io(dir.to_path_buf(), e.to_string()))?;
        let name = entry.file_name().to_string_lossy().to_string();
        let n = name
            .strip_prefix("F-")
            .and_then(|r| r.strip_suffix(".md"))
            .and_then(|d| d.parse::<u32>().ok());
        if let Some(n) = n {
            max = max.max(n);
        }
    }
    Ok(max)
}

/// The record body. Every field is written: a field nobody gave reads `unknown`, never empty
/// (INT-192). Fixed is not a field -- it is derived from Fixes: trailers. The one stored part
/// of a finding's state is the closure decision, and at filing no decision has been made.
pub fn render(id: &str, what: &str, at: Option<&str>, by: Option<&str>, on: &str) -> String {
    format!(
        "---\nid: {}\nwhat: {}\nat: {}\nfound_by: {}\nfound_on: {}\nclosed_without_fix: no\n---\n",
        id,
        what,
        at.unwrap_or("unknown"),
        by.unwrap_or("unknown"),
        on
    )
}

/// One finding, read back from its record. A field the file does not hold reads `unknown`, the
/// same word render writes for a field nobody gave (INT-192).
#[derive(Debug, PartialEq)]
pub struct Record {
    pub id: String,
    pub what: String,
    pub at: String,
    pub found_by: String,
    pub found_on: String,
    pub closed_without_fix: String,
}

/// Parse a record body as render writes it. Only the frontmatter is read.
pub fn parse_record(body: &str) -> Record {
    let mut fields: Vec<(String, String)> = Vec::new();
    let mut lines = body.lines();
    if lines.next().map(str::trim) == Some("---") {
        for line in lines {
            if line.trim() == "---" {
                break;
            }
            if let Some((key, value)) = line.split_once(':') {
                fields.push((key.trim().to_string(), value.trim().to_string()));
            }
        }
    }
    let get = |name: &str| -> String {
        fields
            .iter()
            .find(|(key, _)| key.as_str() == name)
            .map(|(_, value)| value.clone())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "unknown".to_string())
    };
    Record {
        id: get("id"),
        what: get("what"),
        at: get("at"),
        found_by: get("found_by"),
        found_on: get("found_on"),
        closed_without_fix: get("closed_without_fix"),
    }
}

/// Every record in `dir`, in id order. An absent folder truly holds none; a folder or record
/// that cannot be read is an error, never an empty list (INT-192). Only F-NNNN.md is read.
pub fn read_records(dir: &Path) -> Result<Vec<Record>, String> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("could not read {}: {}", dir.display(), e)),
    };
    let mut records = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("could not read {}: {}", dir.display(), e))?;
        let name = entry.file_name().to_string_lossy().to_string();
        let is_record = name
            .strip_prefix("F-")
            .and_then(|r| r.strip_suffix(".md"))
            .and_then(|d| d.parse::<u32>().ok())
            .is_some();
        if !is_record {
            continue;
        }
        let path = entry.path();
        let body = fs::read_to_string(&path)
            .map_err(|e| format!("could not read {}: {}", path.display(), e))?;
        records.push(parse_record(&body));
    }
    records.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(records)
}

fn refuse(message: String) -> CoreError {
    CoreError::Domain {
        domain: "intent".to_string(),
        message: format!("nothing was filed -- {}", message),
    }
}

/// `core intent find "<what>" [--at <file:line>] [--by <intent>]` -- file one finding.
/// `--by` defaults to the focused intent, read the way `core intent status` reads it.
pub fn find(ctx: &AppContext, what: &str, at: Option<&str>, by: Option<&str>) -> CoreResult<()> {
    ctx.capabilities.require(
        "intent",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;
    let what = what.split_whitespace().collect::<Vec<_>>().join(" ");
    if what.is_empty() {
        return Err(refuse("a finding must say what was found".to_string()));
    }
    let at = at.map(str::trim).filter(|s| !s.is_empty());
    let (by_id, from_focus) = match by.map(str::trim).filter(|s| !s.is_empty()) {
        Some(b) => (Some(b.trim_start_matches("INT-").to_string()), false),
        None => (super::read_focus().map(|f| f.id), true),
    };
    if let Some(id) = &by_id {
        if !super::load_all(ctx).iter().any(|i| &i.id == id) {
            return Err(refuse(format!("INT-{} names no intent in the ledger", id)));
        }
    }
    let by_label = by_id.as_ref().map(|id| format!("INT-{}", id));
    let dir = super::intents_dir(ctx).join("findings");
    fs::create_dir_all(&dir)
        .map_err(|e| refuse(format!("could not create {}: {}", dir.display(), e)))?;
    let on = chrono::Local::now().format("%Y-%m-%d").to_string();
    for _ in 0..5 {
        let n = highest_number(&dir).map_err(|e| refuse(format!("{:?}", e)))? + 1;
        let id = finding_id(n);
        let body = render(&id, &what, at, by_label.as_deref(), &on);
        match write_record(&dir, &id, &body) {
            Ok(path) => {
                println!("  {} {} filed", "✅".green(), id.bright_white());
                match &by_label {
                    Some(b) if from_focus => println!("     found by {} (focused)", b),
                    Some(b) => println!("     found by {}", b),
                    None => println!(
                        "     found by unknown -- no --by given and no focused intent could be read"
                    ),
                }
                println!("     at {}", at.unwrap_or("unknown"));
                println!("     {}", path.display().to_string().dimmed());
                return Ok(());
            }
            Err(WriteError::Exists(_)) => continue,
            Err(e) => return Err(refuse(format!("{:?}", e))),
        }
    }
    Err(refuse(
        "five ids in a row were taken while filing; run it again".to_string(),
    ))
}

#[cfg(test)]
mod collision_tests {
    /// INT-266 gate 4: A FINDING ID CANNOT COLLIDE. The id is the whole point of a record: a
    /// Fixes: trailer in git names it, and trace follows it. If a second filing could reuse an
    /// id, the first record would be overwritten and every commit that named it would silently
    /// point at someone else's finding.
    ///
    /// Proven red first, the honest way: the first writer used File::create, which overwrites,
    /// and this test caught it. The fix is the filesystem's own refusal -- create_new -- so two
    /// filings that race cannot both win.
    #[test]
    fn a_second_write_of_the_same_id_is_refused_and_the_first_survives() {
        let dir = std::env::temp_dir().join("core_finding_collision_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        let first = super::write_record(&dir, "F-0001", "first\n");
        let second = super::write_record(&dir, "F-0001", "second\n");
        let on_disk = std::fs::read(dir.join("F-0001.md")).expect("read back");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(first.is_ok(), "the first write must succeed: {:?}", first);
        assert!(
            matches!(second, Err(super::WriteError::Exists(_))),
            "a second write of F-0001 must be refused, got {:?}",
            second
        );
        assert_eq!(
            on_disk, b"first\n",
            "and the first record must survive byte for byte"
        );
    }
}

/// INT-266 question a: records read back. A missing field reads unknown, and a folder that
/// cannot be read is an error, never an empty list (INT-192).
#[cfg(test)]
mod read_tests {
    use super::{parse_record, read_records, render, Record};

    #[test]
    fn a_record_reads_back_as_it_was_written() {
        let body = render(
            "F-0001",
            "something broke: here",
            None,
            Some("INT-266"),
            "2026-10-04",
        );
        assert_eq!(
            parse_record(&body),
            Record {
                id: "F-0001".to_string(),
                what: "something broke: here".to_string(),
                at: "unknown".to_string(),
                found_by: "INT-266".to_string(),
                found_on: "2026-10-04".to_string(),
                closed_without_fix: "no".to_string(),
            }
        );
    }

    #[test]
    fn a_field_missing_from_the_file_reads_unknown() {
        let got = parse_record("---\nid: F-0003\n---\n");
        assert_eq!(got.id, "F-0003");
        assert_eq!(got.what, "unknown", "a missing what must read unknown");
        assert_eq!(got.closed_without_fix, "unknown");
    }

    #[test]
    fn a_folder_that_cannot_be_read_is_an_error_never_none() {
        let base = std::env::temp_dir();
        let absent = base.join(format!("core_finding_absent_{}", std::process::id()));
        assert_eq!(
            read_records(&absent),
            Ok(Vec::new()),
            "an absent folder holds none"
        );
        let file = base.join(format!("core_finding_not_a_dir_{}", std::process::id()));
        std::fs::write(&file, "x").expect("seed");
        let got = read_records(&file);
        let _ = std::fs::remove_file(&file);
        assert!(
            got.is_err(),
            "a file where the folder should be must answer Err, got {:?}",
            got
        );
    }
}

#[cfg(test)]
mod record_tests {
    /// INT-266 gate 5: A FIELD THAT IS NOT KNOWN SAYS SO. A record filed without --at or --by
    /// must read `unknown` in that field, never an empty value that looks like a parse error or
    /// a blank someone forgot (INT-192). Proven red first: the first render used an empty
    /// default, and this test caught it.
    #[test]
    fn a_missing_field_reads_unknown_never_empty() {
        let body = super::render("F-0001", "something broke", None, None, "2026-10-04");
        for field in [
            "id",
            "what",
            "at",
            "found_by",
            "found_on",
            "closed_without_fix",
        ] {
            let line = body
                .lines()
                .find(|l| l.starts_with(&format!("{}:", field)))
                .unwrap_or_else(|| panic!("the record has no {} field:\n{}", field, body));
            let value = line[field.len() + 1..].trim();
            assert!(!value.is_empty(), "{} is empty:\n{}", field, body);
        }
        assert!(body.contains("\nat: unknown\n"), "{}", body);
        assert!(body.contains("\nfound_by: unknown\n"), "{}", body);
    }

    /// The allocator counts F-NNNN.md and nothing else, and an absent folder holds none.
    #[test]
    fn the_highest_number_counts_only_finding_names() {
        let dir = std::env::temp_dir().join("core_finding_highest_test");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(
            super::highest_number(&dir),
            Ok(0),
            "an absent folder holds none"
        );
        std::fs::create_dir_all(&dir).expect("temp dir");
        for name in [
            "F-0001.md",
            "F-0007.md",
            "F-abc.md",
            "notes.md",
            "F-0009.txt",
        ] {
            std::fs::write(dir.join(name), "x").expect("seed");
        }
        let got = super::highest_number(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(got, Ok(7));
    }
}
