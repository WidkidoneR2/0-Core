use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::cli::commands::{
    AnomalyCommand, AuditCommand, AutobiographyCommand, BootstrapCommand, CheckpointCommand,
    AlignCommand, Command, DocsCommand, DecisionCommand, DepsCommand, DoctorCommand, EnginesCommand, EventsCommand, EvolutionCommand,
    ValuesCommand,
    DbCommand, GitCommand, AutonomyCommand, GenealogyCommand, IntegrityCommand, RegistryCommand, GoalsCommand, PredictCommand, ReactCommand, StrategyCommand, StressCommand, IntentCommand, LauncherCommand, LedgerCommand, LinkCommand,
    NotifyCommand, PlanCommand, PluginCommand, PrioritizeCommand, ProfileCommand, ReleaseCommand,
    DelegateCommand, SandboxCommand, SecurityCommand, SimulateCommand, TraceCommand, TradeoffCommand, UpdateCommand,
    WhyCommand, WorkspaceCommand,
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
                DoctorCommand::Rebuild => crate::domains::doctor::rebuild(ctx),
                DoctorCommand::Quick   => crate::domains::doctor::run_quick(ctx),
                DoctorCommand::History => crate::domains::doctor::run_history(ctx),
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
                IntentCommand::New { template, title, smart } => {
                    if smart {
                        crate::domains::intent::new_intent_smart(ctx, &template, &title)
                    } else {
                        crate::domains::intent::new_intent(ctx, &template, &title)
                    }
                }
                IntentCommand::Deps { id, critical_path } => {
                    if critical_path {
                        crate::domains::intent::deps_critical_path(ctx)
                    } else {
                        let id_str = id.as_deref().unwrap_or("");
                        crate::domains::intent::deps(ctx, id_str)
                    }
                }
                IntentCommand::Burndown => crate::domains::intent::burndown(ctx),
                IntentCommand::Velocity => crate::domains::intent::velocity(ctx),
                IntentCommand::Branch { id } => crate::domains::intent::branch(ctx, &id),
                IntentCommand::Edit { id } => crate::domains::intent::edit(ctx, &id),
            IntentCommand::Health { stale } => crate::domains::intent::health(ctx, stale),
            IntentCommand::Predict { id } => crate::domains::intent::predict_completion(ctx, &id),
            IntentCommand::AutoLink { id } => crate::domains::intent::auto_link(ctx, &id),
            IntentCommand::Story { id } => crate::domains::intent::story(ctx, &id),
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
                SecurityCommand::Simulate { patch } => {
                    crate::domains::security::simulate(ctx, &patch)
                }
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
            ctx.capabilities.require("lock", &[Capability::ControlWM])?;
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
        Command::Docs(c) => match c {
            DocsCommand::Commands => crate::domains::docs::commands(ctx),
            DocsCommand::List => crate::domains::docs::list(ctx),
        },
        Command::Values(c) => match c {
            ValuesCommand::List => crate::domains::alignment::values_list(ctx),
            ValuesCommand::Define { statement, weight, scope } => crate::domains::alignment::values_define(ctx, &statement, weight, &scope),
            ValuesCommand::Remove { id } => crate::domains::alignment::values_remove(ctx, id),
            ValuesCommand::Weight { id, weight } => crate::domains::alignment::values_weight(ctx, id, weight),
        },
        Command::Align(c) => match c {
            AlignCommand::Check { subject } => crate::domains::alignment::align_check(ctx, &subject),
            AlignCommand::Drift => crate::domains::alignment::align_drift(ctx),
            AlignCommand::Report { weeks_ago } => crate::domains::alignment::align_report(ctx, weeks_ago),
        },
        Command::Engines(c) => match c {
            EnginesCommand::Status => crate::domains::engines::status(ctx),
            EnginesCommand::Sync { engine } => crate::domains::engines::sync(ctx, &engine),
            EnginesCommand::Signals => crate::domains::engines::signals(ctx),
            EnginesCommand::Check => crate::domains::engines::check(ctx),
            EnginesCommand::UpgradeLog => crate::domains::engines::upgrade_log(ctx),
        },
        Command::Events(c) => match c {
            EventsCommand::Status => crate::domains::events::status(ctx),
            EventsCommand::Archive => crate::domains::events::archive(ctx),
            EventsCommand::List => crate::domains::events::list(ctx),
            EventsCommand::Since { duration } => crate::domains::events::since(ctx, &duration),
            EventsCommand::Filter { domain } => crate::domains::events::filter(ctx, &domain),
            EventsCommand::Watch => crate::domains::events::watch(ctx),
        },
        Command::Ledger(c) => {
            ctx.capabilities.require(
                "ledger",
                &[
                    Capability::OrchestratorAccess,
                    Capability::FilesystemReadHome,
                ],
            )?;
            match c {
                LedgerCommand::Stats => crate::domains::events::ledger_stats(ctx),
                LedgerCommand::Query { domain } => {
                    crate::domains::events::ledger_query(ctx, &domain)
                }
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
            WhyCommand::HealthSince { since } => {
                crate::domains::events::why_health_since(ctx, &since)
            }
            WhyCommand::Causal { domain } => crate::domains::events::why_causal(ctx, &domain),
            WhyCommand::Chain => crate::domains::events::why_chain(ctx),
            WhyCommand::Correlate { domain_a, domain_b } => {
                crate::domains::events::why_correlate(ctx, &domain_a, &domain_b)
            }
            WhyCommand::Suggest => crate::domains::events::why_suggest(ctx),
            WhyCommand::Workspace => crate::domains::events::why_workspace(ctx),
            WhyCommand::Focus => crate::domains::events::why_focus(ctx),
        },
        Command::Trace(c) => match c {
            TraceCommand::Last => crate::domains::events::trace_last(ctx),
            TraceCommand::Domain { domain } => crate::domains::events::trace_domain(ctx, &domain),
        },
        Command::Checkpoint(c) => match c {
            CheckpointCommand::Create { name, notes } => {
                crate::domains::checkpoint::create(ctx, &name, notes.as_deref())
            }
            CheckpointCommand::List => crate::domains::checkpoint::list(ctx),
            CheckpointCommand::Diff { name } => crate::domains::checkpoint::diff(ctx, &name),
            CheckpointCommand::Restore { name } => crate::domains::checkpoint::restore(ctx, &name),
            CheckpointCommand::LastGood => crate::domains::checkpoint::last_good(ctx),
            CheckpointCommand::Snapshot { label } => {
                crate::domains::checkpoint::btrfs_snapshot(ctx, &label)
            }
            CheckpointCommand::Snapshots => crate::domains::checkpoint::btrfs_snapshots(ctx),
        },
        Command::Simulate(c) => match c {
            SimulateCommand::Doctor => crate::domains::simulate::doctor(ctx),
            SimulateCommand::Update => crate::domains::simulate::update(ctx),
            SimulateCommand::Scenario { description } => {
                crate::domains::simulate::scenario(ctx, &description)
            }
        },
        Command::Delegate(cmd) => match cmd {
            DelegateCommand::Simulate { action } => crate::domains::delegate::simulate(ctx, &action),
            DelegateCommand::Contracts => crate::domains::delegate::contracts(ctx),
            DelegateCommand::History => crate::domains::delegate::history(ctx),
            DelegateCommand::Accuracy => crate::domains::delegate::accuracy(ctx),
            DelegateCommand::Suspend => crate::domains::delegate::suspend(ctx),
            DelegateCommand::Activate { contract } => crate::domains::delegate::activate(ctx, &contract),
        },

        Command::Decision(cmd) => match cmd {
            DecisionCommand::Record {
                description,
                intent,
            } => crate::domains::decisions::decide(ctx, &description, intent.as_deref()),
            DecisionCommand::Outcome { id, result, notes } => {
                crate::domains::decisions::outcome(ctx, &id, &result, notes.as_deref())
            }
            DecisionCommand::List { open } => crate::domains::decisions::list(ctx, open),
            DecisionCommand::Hindsight => crate::domains::decisions::hindsight(ctx),
            DecisionCommand::Show { id } => crate::domains::decisions::show(ctx, &id),
            DecisionCommand::Stats => crate::domains::decisions::stats(ctx),
            DecisionCommand::Advise { decision } => {
                crate::domains::decisions::advise(ctx, decision.as_deref())
            }
            DecisionCommand::Heuristics { domain } => {
                crate::domains::decisions::heuristics(ctx, domain.as_deref())
            }
            DecisionCommand::Lessons => crate::domains::decisions::lessons(ctx),
            DecisionCommand::Story => crate::domains::decisions::story(ctx),
            DecisionCommand::Patterns => crate::domains::decisions::patterns(ctx),
            DecisionCommand::Friction => crate::domains::decisions::friction(ctx),
            DecisionCommand::Reversal => crate::domains::decisions::reversal(ctx),
        },

        Command::Deps(cmd) => match cmd {
            DepsCommand::Graph => crate::domains::deps::graph(ctx),
            DepsCommand::Risk => crate::domains::deps::risk(ctx),
            DepsCommand::Audit => crate::domains::deps::audit(ctx),
        },
        Command::Snapshot { json, save } => crate::domains::snapshot::narrative(ctx, json, save),
        Command::Narrative { since, intent } => {
            crate::domains::narrative::run(ctx, since.as_deref(), intent.as_deref())
        }
        Command::Bootstrap(cmd) => match cmd {
            BootstrapCommand::Plan => crate::domains::bootstrap::plan(ctx),
            BootstrapCommand::Verify => crate::domains::bootstrap::verify(ctx),
            BootstrapCommand::Diff => crate::domains::bootstrap::diff(ctx),
        },
        Command::Anomaly(cmd) => match cmd {
            AnomalyCommand::Scan => crate::domains::anomaly::scan(ctx),
            AnomalyCommand::History => crate::domains::anomaly::history(ctx),
            AnomalyCommand::Alert => crate::domains::anomaly::alert(ctx),
        },
        Command::Audit(cmd) => match cmd {
            AuditCommand::Scan => crate::domains::audit::scan(ctx),
            AuditCommand::Show { tool } => crate::domains::audit::show(ctx, &tool),
            AuditCommand::Stale => crate::domains::audit::stale(ctx),
            AuditCommand::Coverage => crate::domains::audit::coverage(ctx),
        },
        Command::Goals(c) => match c {
            GoalsCommand::List => crate::domains::goals::list(ctx),
            GoalsCommand::Generate => crate::domains::goals::generate(ctx),
            GoalsCommand::Priority => crate::domains::goals::priority(ctx),
            GoalsCommand::Accept { id } => crate::domains::goals::accept(ctx, &id),
            GoalsCommand::Reject { id } => crate::domains::goals::reject(ctx, &id),
            GoalsCommand::Show { id } => crate::domains::goals::show(ctx, &id),
        },
        Command::Stress(c) => match c {
            StressCommand::Events => crate::domains::stress::events(ctx),
            StressCommand::Predict => crate::domains::stress::predict(ctx),
            StressCommand::React => crate::domains::stress::react(ctx),
            StressCommand::Health => crate::domains::stress::health(ctx),
            StressCommand::Intents => crate::domains::stress::intents(ctx),
            StressCommand::Report => crate::domains::stress::report(ctx),
            StressCommand::HealthReport => crate::domains::stress::health_report(ctx),
            StressCommand::Scenario1 => crate::domains::stress::scenario1(ctx),
            StressCommand::Scenario2 => crate::domains::stress::scenario2(ctx),
            StressCommand::Scenario3 => crate::domains::stress::scenario3(ctx),
            StressCommand::Scenario4 => crate::domains::stress::scenario4(ctx),
            StressCommand::Scenario5 => crate::domains::stress::scenario5(ctx),
        },
        Command::Predict(c) => match c {
            PredictCommand::Sessions => crate::domains::predict::sessions(ctx),
            PredictCommand::Cadence => crate::domains::predict::cadence(ctx),
            PredictCommand::Health => crate::domains::predict::health(ctx),
            PredictCommand::Decline => crate::domains::predict::decline(ctx),
            PredictCommand::Intents => crate::domains::predict::intents(ctx),
            PredictCommand::Next => crate::domains::predict::next(ctx),
            PredictCommand::Coupling => crate::domains::predict::coupling(ctx),
            PredictCommand::Churn => crate::domains::predict::churn(ctx),
            PredictCommand::Accuracy => crate::domains::predict::accuracy(ctx),
            PredictCommand::Verify { id, correct } => crate::domains::predict::verify(ctx, &id, correct),
            PredictCommand::CrossSession => crate::domains::predict::cross_session(ctx),
            PredictCommand::MemoryDecay { apply } => crate::domains::predict::memory_decay(ctx, apply),
        },
        Command::React(c) => match c {
            ReactCommand::List => crate::domains::reaction::list(ctx),
            ReactCommand::Rules => crate::domains::reaction::rules_list(ctx),
            ReactCommand::Run => crate::domains::reaction::run(ctx),
            ReactCommand::History => crate::domains::reaction::history(ctx),
            ReactCommand::Explain { id } => crate::domains::reaction::explain(ctx, &id),
            ReactCommand::Discipline => crate::domains::reaction::discipline(ctx),
            ReactCommand::Enable { id } => crate::domains::reaction::enable(ctx, &id),
            ReactCommand::Disable { id } => crate::domains::reaction::disable(ctx, &id),
            ReactCommand::Add { id, description, priority, cooldown_m } =>
                crate::domains::reaction::add(ctx, &id, &description, priority, cooldown_m),
            ReactCommand::Bounds => crate::domains::reaction::bounds(ctx),
            ReactCommand::Audit => crate::domains::reaction::audit(ctx),
            ReactCommand::Story => crate::domains::reaction::story(ctx),
            ReactCommand::Coalesce => crate::domains::reaction::coalesce(ctx),
            ReactCommand::DisciplineShow => crate::domains::reaction::discipline_show(ctx),
        },
        Command::Db(c) => match c {
            DbCommand::Backup => crate::domains::db::backup(ctx),
            DbCommand::Restore { file } => crate::domains::db::restore(ctx, &file),
            DbCommand::Verify => crate::domains::db::verify(ctx),
            DbCommand::Status => crate::domains::db::status(ctx),
            DbCommand::Compact => crate::domains::db::compact(ctx),
        },
        Command::Genealogy(c) => match c {
            GenealogyCommand::Show { id } => crate::domains::genealogy::show(ctx, &id),
            GenealogyCommand::Tree => crate::domains::genealogy::tree(ctx),
            GenealogyCommand::Roots => crate::domains::genealogy::roots(ctx),
        },
        Command::Integrity(cmd) => match cmd {
            IntegrityCommand::Run          => crate::domains::integrity::cmd_run(ctx),
            IntegrityCommand::Status       => crate::domains::integrity::cmd_status(ctx),
            IntegrityCommand::Log          => crate::domains::integrity::cmd_log(ctx),
            IntegrityCommand::Fix          => crate::domains::integrity::cmd_fix(ctx),
            IntegrityCommand::Apply { id } => crate::domains::integrity::cmd_apply(ctx, &id),
            IntegrityCommand::Heal { dry_run } => crate::domains::integrity::cmd_heal(ctx, dry_run),
            IntegrityCommand::Trend => crate::domains::integrity::cmd_trend(ctx),
        },
        Command::Autonomy(cmd) => match cmd {
            AutonomyCommand::MandateList => crate::domains::autonomy::mandate_list(ctx),
            AutonomyCommand::MandateSet { rule } => crate::domains::autonomy::mandate_set(ctx, &rule),
            AutonomyCommand::MandateRevoke { id } => crate::domains::autonomy::mandate_revoke(ctx, &id),
            AutonomyCommand::MandateRevokeAll => crate::domains::autonomy::mandate_revoke_all(ctx),
            AutonomyCommand::Pending => crate::domains::autonomy::autonomy_pending(ctx),
            AutonomyCommand::Run => crate::domains::autonomy::autonomy_run(ctx),
            AutonomyCommand::Log => crate::domains::autonomy::autonomy_log(ctx),
            AutonomyCommand::Revert => crate::domains::autonomy::autonomy_revert(ctx),
            AutonomyCommand::TrustScore => crate::domains::autonomy::trust_score(ctx),
            AutonomyCommand::TrustHistory => crate::domains::autonomy::trust_history(ctx),
            AutonomyCommand::TrustExpand => crate::domains::autonomy::trust_expand(ctx),
        },
        Command::Partner(cmd) => crate::domains::partner::dispatch(cmd, ctx),
        Command::Registry(cmd) => match cmd {
            RegistryCommand::List => crate::domains::registry::list(ctx),
            RegistryCommand::Show { name } => crate::domains::registry::show(ctx, &name),
            RegistryCommand::Retire { name } => crate::domains::registry::retire(ctx, &name),
            RegistryCommand::Unretire { name } => crate::domains::registry::unretire(ctx, &name),
            RegistryCommand::RealityCheck => crate::domains::registry::reality_check(ctx),
        },
        Command::Strategy(c) => match c {
            StrategyCommand::Now => crate::domains::strategy::now(ctx),
            StrategyCommand::Week => crate::domains::strategy::week(ctx),
            StrategyCommand::Quarter => crate::domains::strategy::quarter(ctx),
            StrategyCommand::Sequence { goal_id } => crate::domains::strategy::sequence(ctx, &goal_id),
            StrategyCommand::Unblock => crate::domains::strategy::unblock(ctx),
            StrategyCommand::Tradeoff { action } => crate::domains::strategy::tradeoff(ctx, &action),
            StrategyCommand::Conflicts => crate::domains::strategy::conflicts(ctx),
            StrategyCommand::Coherence => crate::domains::strategy::coherence(ctx),
            StrategyCommand::Merge { goal1, goal2 } => crate::domains::strategy::merge(ctx, &goal1, &goal2),
            StrategyCommand::Jarvis => crate::domains::strategy::jarvis(ctx),
            StrategyCommand::Trust => crate::domains::strategy::trust(ctx),
            StrategyCommand::Gap => crate::domains::strategy::gap(ctx),
            StrategyCommand::History => crate::domains::strategy::history(ctx),
            StrategyCommand::Learn { strategy_id, outcome } => crate::domains::strategy::learn(ctx, &strategy_id, &outcome),
            StrategyCommand::Review => crate::domains::strategy::review(ctx),
            StrategyCommand::Next { list, why } => crate::domains::strategy::next(ctx, list, why.as_deref()),
            StrategyCommand::Queue => crate::domains::strategy::queue(ctx),
            StrategyCommand::Blockers => crate::domains::strategy::blockers(ctx),
        },
        Command::Plan(c) => match c {
            PlanCommand::Generate { goal_id } => crate::domains::planning::generate(ctx, &goal_id),
            PlanCommand::Review { id } => crate::domains::planning::review(ctx, &id),
            PlanCommand::Simulate { id } => crate::domains::planning::simulate_plan(ctx, &id),
            PlanCommand::List => crate::domains::planning::list(ctx),
        },
        Command::Tradeoff(c) => match c {
            TradeoffCommand::Analyze { decision } => {
                crate::domains::tradeoffs::analyze(ctx, &decision)
            }
            TradeoffCommand::History => crate::domains::tradeoffs::history(ctx),
            TradeoffCommand::Balance => crate::domains::tradeoffs::balance(ctx),
        },
        Command::Prioritize(c) => match c {
            PrioritizeCommand::Run => crate::domains::prioritize::prioritize(ctx),
            PrioritizeCommand::Explain => crate::domains::prioritize::explain(ctx),
        },
        Command::Autobiography(c) => match c {
            AutobiographyCommand::Narrate { version } => {
                crate::domains::autobiography::narrate(ctx, version.as_deref())
            }
        },
        Command::Evolution(c) => match c {
            EvolutionCommand::Map => crate::domains::evolution::map(ctx),
            EvolutionCommand::Tools => crate::domains::evolution::tools(ctx),
            EvolutionCommand::Suggest => crate::domains::evolution::suggest(ctx),
            EvolutionCommand::EvolvePropose => crate::domains::evolution::evolve_propose(ctx),
            EvolutionCommand::EvolveList => crate::domains::evolution::evolve_list(ctx),
            EvolutionCommand::EvolveAccept { id } => {
                crate::domains::evolution::evolve_accept(ctx, &id)
            }
            EvolutionCommand::EvolveReject { id } => {
                crate::domains::evolution::evolve_reject(ctx, &id)
            }
            EvolutionCommand::FutureSim { change } => {
                crate::domains::evolution::future_sim(ctx, &change)
            }
            EvolutionCommand::FutureRisk { change } => {
                crate::domains::evolution::future_risk(ctx, &change)
            }
            EvolutionCommand::FutureImpact { change } => {
                crate::domains::evolution::future_impact(ctx, &change)
            }
        },
        Command::Capabilities { json, domain } => {
            crate::domains::capabilities::list(ctx, json, domain.as_deref())
        }
    }
}
