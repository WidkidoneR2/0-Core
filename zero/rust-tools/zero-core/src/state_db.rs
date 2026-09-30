//! The ledger's opener for every tool that does not own it. INT-247.
//!
//! `rusqlite::Connection::open` CREATES a missing file. A tool that opened state.db that way and
//! found it absent got an empty database, and every query then answered "no such table" as zero,
//! as empty, or as healthy -- the silent empty ledger this intent exists to prevent. These
//! functions never create anything: an absent ledger, an unreadable one and a file with no ledger
//! schema are each an error with a name, so a caller can say what it could not read.
//!
//! The one exception is NovaShell's `StateDb::open`, which OWNS the schema and creates it on a
//! fresh machine. Every other tool, reader or writer, opens the ledger here.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

/// A table only the ledger has. A file without it is a database, but not this one.
const SENTINEL_TABLE: &str = "shell_history";

/// Why the ledger could not be opened. Each variant names the path it was looking at.
#[derive(Debug, thiserror::Error)]
pub enum StateDbError {
    /// Nothing exists at the path. Nothing was created there either.
    #[error("state.db absent at {}", .0.display())]
    Absent(PathBuf),
    /// Something is there, but it could not be opened or read.
    #[error("state.db unreadable at {}: {}", .0.display(), .1)]
    Unreadable(PathBuf, String),
    /// A database is there, but it holds no ledger.
    #[error("state.db at {} has no ledger schema (no shell_history table)", .0.display())]
    NoSchema(PathBuf),
}

/// Open the ledger for reading and writing. Never creates it.
pub fn open_state_db() -> Result<Connection, StateDbError> {
    open_at(&crate::paths::state_db(), OpenFlags::SQLITE_OPEN_READ_WRITE)
}

/// Open the ledger read-only. Never creates it.
pub fn open_state_db_readonly() -> Result<Connection, StateDbError> {
    open_at(&crate::paths::state_db(), OpenFlags::SQLITE_OPEN_READ_ONLY)
}

/// The rule itself, on an explicit path, so it can be tested without the real ledger.
///
/// `mode` is `SQLITE_OPEN_READ_WRITE` or `SQLITE_OPEN_READ_ONLY`. `SQLITE_OPEN_CREATE` is removed
/// whatever the caller passes: this function must not be able to make the file it reports.
pub fn open_at(path: &Path, mode: OpenFlags) -> Result<Connection, StateDbError> {
    match std::fs::metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(StateDbError::Absent(path.to_path_buf()))
        }
        Err(e) => return Err(StateDbError::Unreadable(path.to_path_buf(), e.to_string())),
        Ok(_) => {}
    }
    let mut flags = mode | OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    flags.remove(OpenFlags::SQLITE_OPEN_CREATE);
    let unreadable =
        |e: rusqlite::Error| StateDbError::Unreadable(path.to_path_buf(), e.to_string());
    let conn = Connection::open_with_flags(path, flags).map_err(unreadable)?;
    let has_schema: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
            [SENTINEL_TABLE],
            |r| r.get(0),
        )
        .map_err(unreadable)?;
    if !has_schema {
        return Err(StateDbError::NoSchema(path.to_path_buf()));
    }
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch path per test and per process, removed before and after use.
    fn scratch(name: &str) -> PathBuf {
        let p =
            std::env::temp_dir().join(format!("zero-state-db-{}-{}.db", std::process::id(), name));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[test]
    fn absent_is_reported_absent_and_nothing_is_created() {
        let p = scratch("absent");
        let r = open_at(&p, OpenFlags::SQLITE_OPEN_READ_WRITE);
        assert!(matches!(r, Err(StateDbError::Absent(_))), "{:?}", r.err());
        assert!(
            !p.exists(),
            "opening an absent ledger created {}",
            p.display()
        );
    }

    #[test]
    fn a_zero_byte_file_is_no_schema_not_an_empty_ledger() {
        let p = scratch("zero");
        std::fs::write(&p, b"").unwrap();
        let r = open_at(&p, OpenFlags::SQLITE_OPEN_READ_WRITE);
        let _ = std::fs::remove_file(&p);
        assert!(matches!(r, Err(StateDbError::NoSchema(_))), "{:?}", r.err());
    }

    #[test]
    fn a_file_that_is_not_a_database_is_unreadable() {
        let p = scratch("garbage");
        std::fs::write(&p, vec![b'x'; 4096]).unwrap();
        let r = open_at(&p, OpenFlags::SQLITE_OPEN_READ_ONLY);
        let _ = std::fs::remove_file(&p);
        assert!(
            matches!(r, Err(StateDbError::Unreadable(..))),
            "{:?}",
            r.err()
        );
    }

    #[test]
    fn a_file_without_permission_is_unreadable() {
        // Root reads through mode 000, so the case cannot be staged as root.
        if unsafe { libc::geteuid() } == 0 {
            return;
        }
        let p = scratch("perm");
        Connection::open(&p)
            .unwrap()
            .execute_batch("CREATE TABLE shell_history (id INTEGER)")
            .unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o000)).unwrap();
        let r = open_at(&p, OpenFlags::SQLITE_OPEN_READ_ONLY);
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600)).unwrap();
        let _ = std::fs::remove_file(&p);
        assert!(
            matches!(r, Err(StateDbError::Unreadable(..))),
            "{:?}",
            r.err()
        );
    }

    #[test]
    fn the_ledger_opens_in_both_modes() {
        let p = scratch("ledger");
        Connection::open(&p)
            .unwrap()
            .execute_batch("CREATE TABLE shell_history (id INTEGER)")
            .unwrap();
        let rw = open_at(&p, OpenFlags::SQLITE_OPEN_READ_WRITE).map(|_| ());
        let ro = open_at(&p, OpenFlags::SQLITE_OPEN_READ_ONLY).map(|_| ());
        let _ = std::fs::remove_file(&p);
        assert!(
            rw.is_ok() && ro.is_ok(),
            "rw {:?} ro {:?}",
            rw.err(),
            ro.err()
        );
    }

    #[test]
    fn errors_name_the_path_they_looked_at() {
        let p = scratch("named");
        let msg = open_at(&p, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .err()
            .unwrap()
            .to_string();
        assert!(
            msg.contains(&p.display().to_string()) && msg.contains("absent"),
            "{msg}"
        );
    }
}
