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
    // INT-249b: SIGPIPE handling -- exit silently when piped to head/grep/etc
    // SIG_DFL alone is insufficient because Rust stdio panics on EPIPE before SIGPIPE fires.
    // Combine: signal handler + panic hook for broken-pipe writes.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
    std::panic::set_hook(Box::new(|info| {
        let msg = info.payload().downcast_ref::<String>()
            .map(|s| s.as_str())
            .or_else(|| info.payload().downcast_ref::<&str>().copied())
            .unwrap_or("");
        if msg.contains("Broken pipe") || msg.contains("os error 32") {
            std::process::exit(0);
        }
        // Other panics: print and exit non-zero like default
        eprintln!("{}", info);
        std::process::exit(101);
    }));
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

    // Emit forest event for contextd to observe
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
