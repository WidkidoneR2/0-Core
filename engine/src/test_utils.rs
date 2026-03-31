// faelight-forest — Test Utilities
// INT-168 — Test Suite Foundation
//
// Provides isolated test context — no production state reads.
// Uses in-memory SQLite — never touches production state.db.

#[cfg(test)]
pub mod test_support {
    use rusqlite::Connection;
    use crate::runtime::Runtime;
    use crate::capabilities::CapabilityContext;
    use crate::app::context::AppContext;
    use std::path::PathBuf;

    /// Build an isolated Runtime using in-memory SQLite
    pub fn test_runtime() -> Runtime {
        let db = Connection::open_in_memory()
            .expect("Failed to create in-memory test db");

        db.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;
            CREATE TABLE IF NOT EXISTS domain_state (
                domain TEXT NOT NULL, key TEXT NOT NULL,
                value TEXT NOT NULL, updated_at INTEGER NOT NULL,
                PRIMARY KEY (domain, key)
            );
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                domain TEXT NOT NULL, action TEXT NOT NULL,
                payload TEXT, timestamp INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS capabilities_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                domain TEXT NOT NULL, capability TEXT NOT NULL,
                granted INTEGER NOT NULL, timestamp INTEGER NOT NULL
            );"
        ).expect("Failed to init test db schema");

        let tmp = PathBuf::from("/tmp/faelight-test");
        std::fs::create_dir_all(&tmp).ok();

        Runtime {
            root:      tmp.clone(),
            logs:      tmp.join("logs"),
            cache:     tmp.join("cache"),
            snapshots: tmp.join("snapshots"),
            locks:     tmp.join("locks"),
            db,
        }
    }

    /// Build an isolated AppContext for testing
    pub fn test_context() -> AppContext {
        AppContext {
            runtime:      test_runtime(),
            capabilities: CapabilityContext::unprivileged(),
            home:         "/tmp".to_string(),
            core_root:    "/tmp/faelight-test".to_string(),
        }
    }
}
