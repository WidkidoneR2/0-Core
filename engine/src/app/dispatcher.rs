use crate::app::context::AppContext;
use crate::cli::commands::{Command, IntentCommand, LinkCommand, ProfileCommand};
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
        Command::Intent(intent_cmd) => match intent_cmd {
            IntentCommand::List {
                planned,
                active,
                complete,
            } => crate::domains::intent::list(ctx, planned, active, complete),
            IntentCommand::Show { id } => crate::domains::intent::show(ctx, &id),
            IntentCommand::Search { term } => crate::domains::intent::search(ctx, &term),
            IntentCommand::Stats => crate::domains::intent::stats(ctx),
            IntentCommand::Validate => crate::domains::intent::validate(ctx),
        },
        Command::Profile(profile_cmd) => match profile_cmd {
            ProfileCommand::List => crate::domains::profile::list(ctx),
            ProfileCommand::Status => crate::domains::profile::status(ctx),
            ProfileCommand::Switch { name } => crate::domains::profile::switch(ctx, &name),
            ProfileCommand::History => crate::domains::profile::history(),
            ProfileCommand::Health => crate::domains::profile::health(ctx),
        },
    }
}
