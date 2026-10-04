//! INT-266: findings -- something found and not fixed, one record per file under
//! zero/intents/findings/, named F-NNNN.md (ruled by Christian, 2026-10-04).
//!
//! A record holds only facts from the moment it was filed. Whether a finding is fixed is never
//! stored here: it is derived from Fixes: trailers in git. Records are never deleted, because a
//! Fixes: trailer must always point at a real record, and an id is never reused.

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
