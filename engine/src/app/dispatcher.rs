use crate::app::context::AppContext;
use crate::cli::commands::{Command, LinkCommand};
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
        Command::Link(link_cmd) => match link_cmd {
            LinkCommand::Status { json } => crate::domains::link::status(ctx, json),
            LinkCommand::List => crate::domains::link::list(ctx),
            LinkCommand::Audit => crate::domains::link::audit(ctx),
        },
        Command::Zone {
            icon,
            label,
            json,
            health,
        } => crate::domains::zone::run(ctx, icon, label, json, health),
    }
}
