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

    // Acquire runtime lock — held until end of scope
    let _lock = match runtime::RuntimeLock::acquire(&ctx.runtime) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{} {}", "✗".bright_red(), e);
            std::process::exit(1);
        }
    };

    if let Err(e) = app::dispatcher::dispatch(cmd, &ctx) {
        eprintln!("{} {}", "✗".bright_red(), e);
        std::process::exit(1);
    }
}
