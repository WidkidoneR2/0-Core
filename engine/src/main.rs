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

    if let Err(e) = app::dispatcher::dispatch(cmd, &ctx) {
        eprintln!("{} {}", "✗".bright_red(), e);
        std::process::exit(1);
    }
}
