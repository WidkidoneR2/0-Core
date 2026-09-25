#![allow(clippy::type_complexity)]
#![allow(clippy::unnecessary_filter_map)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
//! core — 0-Core v2 single orchestrator binary
//! Philosophy: One binary. Five layers. Zero ambiguity.

use colored::*;

mod app;
mod capabilities;
mod cli;
mod domains;
mod errors;
mod logging;
mod runtime;

fn main() {
    // INT-256, correcting INT-249b. The signal stays; THE PANIC HOOK IS GONE, and the comment
    // that justified it was measured FALSE on 2026-09-23.
    //
    // It claimed "SIG_DFL alone is insufficient because Rust stdio panics on EPIPE before SIGPIPE
    // fires". Three identical programs printing 100,000 lines into `head -2`:
    //
    //     nothing                   exit 101   panicked at io/stdio.rs
    //     signal(SIGPIPE, SIG_DFL)  exit 141   silent
    //     signal + this hook        exit 141   silent -- THE HOOK NEVER RAN
    //     seq 1 100000              exit 141   the reference
    //
    // The process is killed by the signal before stdio reports EPIPE, so the hook's exit(0) was
    // unreachable -- and had it run it would have been WRONG: exit 0 claims the command finished
    // when `head` cut it off. 141 is what seq and yes report.
    //
    // ⚠️ THE HOOK ALSO HANDLED EVERY OTHER PANIC (eprintln then exit 101). That is what Rust does
    // by default, so nothing is lost -- but it was a second behaviour riding along inside a
    // broken-pipe fix, and removing it is a deliberate change rather than a side effect.
    faelight_core::restore_sigpipe();
    let cmd = cli::parse();

    let ctx = match app::context::AppContext::init() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} Failed to initialize: {}", "✗".bright_red(), e);
            std::process::exit(1);
        }
    };

    // Acquire runtime lock only for write operations
    // Skip for read-only commands that run constantly (zone, fetch, version)
    // Acquire runtime lock — warn but don't fail on contention
    // This prevents corruption without blocking the bar/prompt polling
    let _lock = runtime::RuntimeLock::acquire(&ctx.runtime).ok();

    // Emit forest event for insightd to observe
    let cmd_name = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let domain = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "unknown".to_string());
    match app::dispatcher::dispatch(cmd, &ctx) {
        Ok(()) => {
            runtime::emit_forest_event(&ctx.runtime.db, "CommandSucceeded", &domain, &cmd_name);
        }
        Err(e) => {
            runtime::emit_forest_event(
                &ctx.runtime.db,
                "CommandFailed",
                &domain,
                &format!("{}: {}", cmd_name, e),
            );
            eprintln!("{} {}", "✗".bright_red(), e);
            std::process::exit(1);
        }
    }
}

/// ⚠️⚠️ EARNED, NOT DECORATION. Tests that set HOME exist in two modules -- the intent validator's
/// and the doctor's -- and cargo runs tests in PARALLEL. One restored the real HOME while the other
/// was mid-flight: each passed alone, together they failed.
///
/// ⭐ `--test-threads=1` WOULD HAVE HIDDEN IT and slowed every unrelated test. This serialises only
/// the tests that mutate process-wide state. It lives at the crate root because both modules can
/// reach it here without making a private module public to satisfy a test.
///
/// ANY future test that touches HOME must take this lock.
#[cfg(test)]
pub static HOME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
