use crate::app::context::AppContext;
use crate::cli::commands::Command;
use crate::errors::CoreResult;
use colored::*;

pub fn dispatch(cmd: Command, ctx: &AppContext) -> CoreResult<()> {
    match cmd {
        Command::Version => {
            println!(
                "{} {}",
                "core".bright_cyan().bold(),
                env!("CARGO_PKG_VERSION").dimmed()
            );
            println!("{}", "0-Core v2 — single orchestrator".dimmed());
            Ok(())
        }
        Command::Doctor { preflight } => crate::domains::doctor::run(ctx, preflight),
    }
}
