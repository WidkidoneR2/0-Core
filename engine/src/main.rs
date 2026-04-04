//! core — 0-Core v2 single orchestrator binary
//! Philosophy: One binary. Five layers. Zero ambiguity.

use colored::*;

mod adapters;
mod app;
mod capabilities;
mod cli;
mod domains;
mod errors;
mod logging;
mod policy;
mod registry;
mod runtime;
#[cfg(test)]
mod test_utils;
mod utils;

fn main() {
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
    let domain = std::env::args().nth(1).unwrap_or_else(|| "unknown".to_string());
    match app::dispatcher::dispatch(cmd, &ctx) {
        Ok(()) => {
            runtime::emit_forest_event(
                &ctx.runtime.db,
                "CommandSucceeded",
                &domain,
                &cmd_name,
            );
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
