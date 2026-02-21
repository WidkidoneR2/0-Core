use crate::app::context::AppContext;
use crate::cli::commands::{
    Command, GitCommand, IntentCommand, LinkCommand, ProfileCommand, ReleaseCommand,
    SandboxCommand, SecurityCommand, WorkspaceCommand,
};
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
        Command::Link(c) => match c {
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
        Command::Intent(c) => match c {
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
        Command::Profile(c) => match c {
            ProfileCommand::List => crate::domains::profile::list(ctx),
            ProfileCommand::Status => crate::domains::profile::status(ctx),
            ProfileCommand::Switch { name } => crate::domains::profile::switch(ctx, &name),
            ProfileCommand::History => crate::domains::profile::history(),
            ProfileCommand::Health => crate::domains::profile::health(ctx),
        },
        Command::Security(c) => match c {
            SecurityCommand::Scan => crate::domains::security::scan(ctx),
            SecurityCommand::Report { all } => crate::domains::security::report(ctx, all),
            SecurityCommand::Show { id } => crate::domains::security::show(ctx, &id),
            SecurityCommand::History => crate::domains::security::history(ctx),
        },
        Command::Sandbox(c) => match c {
            SandboxCommand::Run { args } => crate::domains::sandbox::run(ctx, &args),
            SandboxCommand::Diff => crate::domains::sandbox::diff(ctx),
            SandboxCommand::Status => crate::domains::sandbox::status(ctx),
            SandboxCommand::Clear => crate::domains::sandbox::clear(ctx),
            SandboxCommand::Snapshot { target, name } => {
                crate::domains::sandbox::snapshot(ctx, &target, &name)
            }
            SandboxCommand::Restore { name } => crate::domains::sandbox::restore(ctx, &name),
            SandboxCommand::Snapshots => crate::domains::sandbox::snapshots(ctx),
        },
        Command::Fetch { health_check } => crate::domains::fetch::run(ctx, health_check),
        Command::Git(c) => match c {
            GitCommand::Status => crate::domains::git::status(ctx),
            GitCommand::Risk => crate::domains::git::risk(ctx),
            GitCommand::Log { n } => crate::domains::git::log_cmd(ctx, n),
            GitCommand::Verify => crate::domains::git::verify(ctx),
            GitCommand::Delegate { subcmd, args } => crate::domains::git::delegate(&subcmd, &args),
        },
        Command::Workspace(c) => match c {
            WorkspaceCommand::View {
                active,
                summary,
                json,
            } => crate::domains::workspace::view(ctx, active, summary, json),
            WorkspaceCommand::Recent {
                range,
                limit,
                full_paths,
            } => crate::domains::workspace::recent(ctx, &range, limit, full_paths),
            WorkspaceCommand::Fm { args } => crate::domains::workspace::fm(ctx, &args),
        },
        Command::Release(c) => match c {
            ReleaseCommand::Get { package } => {
                crate::domains::release::get_version(ctx, package.as_deref())
            }
            ReleaseCommand::BumpTool { args } => crate::domains::release::bump_tool(ctx, &args),
            ReleaseCommand::BumpSystem { dry_run } => {
                crate::domains::release::bump_system(ctx, dry_run)
            }
        },
    }
}
