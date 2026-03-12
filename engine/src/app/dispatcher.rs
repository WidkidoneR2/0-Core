use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::cli::commands::{
    Command, AuditCommand, DecisionCommand, DoctorCommand, EventsCommand, GitCommand, IntentCommand, LauncherCommand, LinkCommand,
    NotifyCommand, PluginCommand, ProfileCommand, ReleaseCommand, SandboxCommand, SecurityCommand,
    CheckpointCommand, LedgerCommand, SimulateCommand, TraceCommand, UpdateCommand, WhyCommand, WorkspaceCommand,
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

        Command::Plugin(cmd) => match cmd {
            PluginCommand::List => crate::domains::plugins::list(ctx),
            PluginCommand::Add { name } => crate::domains::plugins::add(ctx, &name),
            PluginCommand::Remove { name } => crate::domains::plugins::remove(ctx, &name),
            PluginCommand::Status { name } => crate::domains::plugins::status(ctx, &name),
        },
        Command::Doctor(c) => {
            ctx.capabilities.require(
                "doctor",
                &[
                    Capability::OrchestratorAccess,
                    Capability::FilesystemReadHome,
                ],
            )?;
            match c {
                DoctorCommand::Run { preflight } => crate::domains::doctor::run(ctx, preflight),
                DoctorCommand::Aliases { subcmd } => {
                    crate::domains::doctor::aliases(ctx, subcmd.as_deref())
                }
                DoctorCommand::Entropy {
                    baseline,
                    trends,
                    json,
                } => crate::domains::doctor::entropy(ctx, baseline, trends, json),
                DoctorCommand::Bins { subcmd } => {
                    crate::domains::doctor::bins(ctx, subcmd.as_deref())
                }
                DoctorCommand::Trend => crate::domains::doctor::trend(ctx),
                DoctorCommand::Forecast => crate::domains::doctor::forecast(ctx),
            }
        }

        Command::Link(c) => {
            ctx.capabilities.require(
                "link",
                &[
                    Capability::FilesystemReadHome,
                    Capability::FilesystemWriteHome,
                ],
            )?;
            match c {
                LinkCommand::Status { json } => crate::domains::link::status(ctx, json),
                LinkCommand::List => crate::domains::link::list(ctx),
                LinkCommand::Audit => crate::domains::link::audit(ctx),
                LinkCommand::Plan { package } => {
                    crate::domains::link::plan(ctx, package.as_deref())
                }
                LinkCommand::Deploy {
                    package,
                    no_snapshot,
                    adopt,
                } => crate::domains::link::deploy(ctx, package.as_deref(), no_snapshot, adopt),
                LinkCommand::Undeploy { package } => crate::domains::link::undeploy(ctx, &package),
                LinkCommand::Adopt { package } => {
                    crate::domains::link::adopt(ctx, package.as_deref())
                }
                LinkCommand::Redeploy { package } => {
                    crate::domains::link::redeploy(ctx, package.as_deref())
                }
                LinkCommand::Sync { package } => {
                    crate::domains::link::sync(ctx, package.as_deref())
                }
            }
        }

        Command::Zone {
            icon,
            label,
            json,
            health,
        } => {
            ctx.capabilities
                .require("zone", &[Capability::FilesystemReadHome])?;
            crate::domains::zone::run(ctx, icon, label, json, health)
        }

        Command::Intent(c) => {
            ctx.capabilities
                .require("intent", &[Capability::FilesystemReadHome])?;
            match c {
                IntentCommand::List {
                    planned,
                    active,
                    complete,
                } => crate::domains::intent::list(ctx, planned, active, complete),
                IntentCommand::Show { id } => crate::domains::intent::show(ctx, &id),
                IntentCommand::Search { term } => crate::domains::intent::search(ctx, &term),
                IntentCommand::Stats => crate::domains::intent::stats(ctx),
                IntentCommand::Validate => crate::domains::intent::validate(ctx),
                IntentCommand::Focus { id } => crate::domains::intent::focus(ctx, &id),
                IntentCommand::Unfocus => crate::domains::intent::unfocus(ctx),
                IntentCommand::FocusStatus => crate::domains::intent::focus_status(ctx),
                IntentCommand::Drift => crate::domains::intent::drift(ctx),
                IntentCommand::Start { id } => crate::domains::intent::start(ctx, &id),
                IntentCommand::Complete { id } => crate::domains::intent::complete_intent(ctx, &id),
                IntentCommand::New { template, title } => crate::domains::intent::new_intent(ctx, &template, &title),
                IntentCommand::Deps { id } => crate::domains::intent::deps(ctx, &id),
                IntentCommand::Burndown => crate::domains::intent::burndown(ctx),
                IntentCommand::Velocity => crate::domains::intent::velocity(ctx),
                IntentCommand::Branch { id } => crate::domains::intent::branch(ctx, &id),
            }
        }

        Command::Profile(c) => {
            ctx.capabilities.require(
                "profile",
                &[
                    Capability::FilesystemReadHome,
                    Capability::FilesystemWriteHome,
                ],
            )?;
            match c {
                ProfileCommand::List => crate::domains::profile::list(ctx),
                ProfileCommand::Status => crate::domains::profile::status(ctx),
                ProfileCommand::Switch { name } => crate::domains::profile::switch(ctx, &name),
                ProfileCommand::History => crate::domains::profile::history(),
                ProfileCommand::Health => crate::domains::profile::health(ctx),
            }
        }

        Command::Security(c) => {
            ctx.capabilities.require(
                "security",
                &[
                    Capability::OrchestratorAccess,
                    Capability::FilesystemReadHome,
                    Capability::NetworkQuery,
                ],
            )?;
            match c {
                SecurityCommand::Scan => crate::domains::security::scan(ctx),
                SecurityCommand::Report { all } => crate::domains::security::report(ctx, all),
                SecurityCommand::Show { id } => crate::domains::security::show(ctx, &id),
                SecurityCommand::History => crate::domains::security::history(ctx),
                SecurityCommand::Debt => crate::domains::security::debt(ctx),
                SecurityCommand::Trend => crate::domains::security::trend(ctx),
            SecurityCommand::Advise => crate::domains::security::advise(ctx),
            }
        }

        Command::Sandbox(c) => {
            ctx.capabilities.require(
                "sandbox",
                &[
                    Capability::FilesystemReadHome,
                    Capability::FilesystemWriteHome,
                    Capability::SpawnProcess,
                ],
            )?;
            match c {
                SandboxCommand::Run { args } => crate::domains::sandbox::run(ctx, &args),
                SandboxCommand::Diff => crate::domains::sandbox::diff(ctx),
                SandboxCommand::Status => crate::domains::sandbox::status(ctx),
                SandboxCommand::Clear => crate::domains::sandbox::clear(ctx),
                SandboxCommand::Snapshot { target, name } => {
                    crate::domains::sandbox::snapshot(ctx, &target, &name)
                }
                SandboxCommand::Restore { name } => crate::domains::sandbox::restore(ctx, &name),
                SandboxCommand::Snapshots => crate::domains::sandbox::snapshots(ctx),
            }
        }

        Command::Fetch { health_check } => {
            ctx.capabilities.require(
                "fetch",
                &[Capability::FilesystemReadHome, Capability::NetworkQuery],
            )?;
            crate::domains::fetch::run(ctx, health_check)
        }

        Command::Git(c) => {
            ctx.capabilities.require(
                "git",
                &[Capability::FilesystemReadHome, Capability::SpawnProcess],
            )?;
            match c {
                GitCommand::Status => crate::domains::git::status(ctx),
                GitCommand::Risk => crate::domains::git::risk(ctx),
                GitCommand::Log { n } => crate::domains::git::log_cmd(ctx, n),
                GitCommand::Verify => crate::domains::git::verify(ctx),
                GitCommand::Delegate { subcmd, args } => {
                    crate::domains::git::delegate(&subcmd, &args)
                }
            }
        }

        Command::Workspace(c) => {
            ctx.capabilities
                .require("workspace", &[Capability::FilesystemReadHome])?;
            match c {
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
            }
        }

        Command::Release(c) => {
            ctx.capabilities.require(
                "release",
                &[
                    Capability::FilesystemReadHome,
                    Capability::FilesystemWriteHome,
                    Capability::SpawnProcess,
                ],
            )?;
            match c {
                ReleaseCommand::Get { package } => {
                    crate::domains::release::get_version(ctx, package.as_deref())
                }
                ReleaseCommand::BumpTool { args } => crate::domains::release::bump_tool(ctx, &args),
                ReleaseCommand::BumpSystem { dry_run } => {
                    crate::domains::release::bump_system(ctx, dry_run)
                }
            }
        }

        Command::Notify(c) => {
            ctx.capabilities
                .require("notify", &[Capability::SpawnProcess])?;
            match c {
                NotifyCommand::Send {
                    summary,
                    body,
                    urgency,
                } => crate::domains::notify::send(ctx, &summary, body.as_deref(), &urgency),
                NotifyCommand::Status => crate::domains::notify::status(ctx),
            }
        }

        Command::Lock { health_check } => {
            ctx.capabilities
                .require("lock", &[Capability::ControlWM])?;
            if health_check {
                crate::domains::lock::health(ctx)
            } else {
                crate::domains::lock::lock(ctx)
            }
        }

        Command::Launcher(c) => {
            ctx.capabilities.require(
                "launcher",
                &[Capability::ControlWM, Capability::SpawnProcess],
            )?;
            match c {
                LauncherCommand::Palette { dmenu, prompt } => {
                    crate::domains::launcher::palette(ctx, dmenu, prompt.as_deref())
                }
                LauncherCommand::Dmenu {
                    subcmd,
                    prompt,
                    multi,
                } => crate::domains::launcher::dmenu(
                    ctx,
                    subcmd.as_deref(),
                    prompt.as_deref(),
                    multi,
                ),
                LauncherCommand::Launch { args } => crate::domains::launcher::launcher(ctx, &args),
            }
        }

        Command::Update(c) => {
            ctx.capabilities
                .require("update", &[Capability::SpawnProcess])?;
            match c {
                UpdateCommand::Run { args } => crate::domains::update::update(ctx, &args),
                UpdateCommand::Safe { args } => crate::domains::update::safe(ctx, &args),
            }
        }
        Command::Events(c) => match c {
            EventsCommand::List => crate::domains::events::list(ctx),
            EventsCommand::Since { duration } => crate::domains::events::since(ctx, &duration),
            EventsCommand::Filter { domain } => crate::domains::events::filter(ctx, &domain),
            EventsCommand::Watch => crate::domains::events::watch(ctx),
        },
        Command::Ledger(c) => {
            ctx.capabilities.require("ledger", &[
                Capability::OrchestratorAccess,
                Capability::FilesystemReadHome,
            ])?;
            match c {
                LedgerCommand::Stats => crate::domains::events::ledger_stats(ctx),
                LedgerCommand::Query { domain } => crate::domains::events::ledger_query(ctx, &domain),
                LedgerCommand::Export => crate::domains::events::ledger_export(ctx),
                LedgerCommand::Indexes => crate::domains::events::ledger_indexes(ctx),
            }
        }
        Command::Why(c) => match c {
            WhyCommand::Summary => crate::domains::events::why_summary(ctx),
            WhyCommand::Health => crate::domains::events::why_health(ctx),
            WhyCommand::Domain { domain } => crate::domains::events::why_domain(ctx, &domain),
            WhyCommand::Visual => crate::domains::events::why_visual(ctx),
            WhyCommand::Attention => crate::domains::events::why_attention(ctx),
            WhyCommand::HealthSince { since } => crate::domains::events::why_health_since(ctx, &since),
            WhyCommand::Causal { domain } => crate::domains::events::why_causal(ctx, &domain),
            WhyCommand::Chain => crate::domains::events::why_chain(ctx),
            WhyCommand::Correlate { domain_a, domain_b } => crate::domains::events::why_correlate(ctx, &domain_a, &domain_b),
            WhyCommand::Suggest => crate::domains::events::why_suggest(ctx),
            WhyCommand::Workspace => crate::domains::events::why_workspace(ctx),
            WhyCommand::Focus => crate::domains::events::why_focus(ctx),
        },
        Command::Trace(c) => match c {
            TraceCommand::Last => crate::domains::events::trace_last(ctx),
            TraceCommand::Domain { domain } => crate::domains::events::trace_domain(ctx, &domain),
        },
        Command::Checkpoint(c) => match c {
            CheckpointCommand::Create { name, notes } => crate::domains::checkpoint::create(ctx, &name, notes.as_deref()),
            CheckpointCommand::List => crate::domains::checkpoint::list(ctx),
            CheckpointCommand::Diff { name } => crate::domains::checkpoint::diff(ctx, &name),
            CheckpointCommand::Restore { name } => crate::domains::checkpoint::restore(ctx, &name),
            CheckpointCommand::LastGood => crate::domains::checkpoint::last_good(ctx),
            CheckpointCommand::Snapshot { label } => crate::domains::checkpoint::btrfs_snapshot(ctx, &label),
            CheckpointCommand::Snapshots => crate::domains::checkpoint::btrfs_snapshots(ctx),
        },
        Command::Simulate(c) => match c {
            SimulateCommand::Doctor => crate::domains::simulate::doctor(ctx),
            SimulateCommand::Update => crate::domains::simulate::update(ctx),
            SimulateCommand::Scenario { description } => crate::domains::simulate::scenario(ctx, &description),
        },

        Command::Decision(cmd) => {
            match cmd {
                DecisionCommand::Record { description, intent } => {
                    crate::domains::decisions::decide(ctx, &description, intent.as_deref())
                }
                DecisionCommand::Outcome { id, result, notes } => {
                    crate::domains::decisions::outcome(ctx, &id, &result, notes.as_deref())
                }
                DecisionCommand::List { open } => {
                    crate::domains::decisions::list(ctx, open)
                }
                DecisionCommand::Hindsight => {
                    crate::domains::decisions::hindsight(ctx)
                }
                DecisionCommand::Show { id } => {
                    crate::domains::decisions::show(ctx, &id)
                }
                DecisionCommand::Stats => {
                    crate::domains::decisions::stats(ctx)
                }
                DecisionCommand::Advise { decision } => {
                    crate::domains::decisions::advise(ctx, decision.as_deref())
                }
                DecisionCommand::Heuristics { domain } => {
                    crate::domains::decisions::heuristics(ctx, domain.as_deref())
                }
                DecisionCommand::Lessons => {
                    crate::domains::decisions::lessons(ctx)
                }
                DecisionCommand::Story => {
                    crate::domains::decisions::story(ctx)
                }
            }
        }

        Command::Audit(cmd) => match cmd {
            AuditCommand::Scan => crate::domains::audit::scan(ctx),
            AuditCommand::Show { tool } => crate::domains::audit::show(ctx, &tool),
            AuditCommand::Stale => crate::domains::audit::stale(ctx),
            AuditCommand::Coverage => crate::domains::audit::coverage(ctx),
        },
        Command::Capabilities { json, domain } => {
            crate::domains::capabilities::list(ctx, json, domain.as_deref())
        }
    }
}
