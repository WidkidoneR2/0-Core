//! INT-265: one lock for every test that writes a process-wide environment variable.
//!
//! cargo runs a binary tests on parallel threads, and the environment belongs to the process,
//! not the thread. Four observe.rs tests set and cleared NSH_OBSERVE around their assertions
//! with nothing between them, so one test could clear the variable another had just set: 161
//! of 200 runs of `cargo test -p novashell --bin nsh observe` failed on 2026-10-03. A test that
//! writes the environment -- directly, or through code that does, such as next_execution_id --
//! holds this lock for its whole body.

use std::sync::{Mutex, MutexGuard};

static ENV: Mutex<()> = Mutex::new(());

/// Hold the returned guard for the whole test. A test that panicked while holding the lock
/// poisons it; the next test still gets it, so one failure does not fail the rest.
pub fn lock() -> MutexGuard<'static, ()> {
    ENV.lock().unwrap_or_else(|e| e.into_inner())
}
