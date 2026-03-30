pub mod commands;
pub mod parser;

use clap::Parser;
use commands::{
    AnomalyCommand, AuditCommand, AutobiographyCommand, BootstrapCommand, CheckpointCommand,
    Command, DecisionCommand, DepsCommand, DoctorCommand, EventsCommand, EvolutionCommand,
    GitCommand, GoalsCommand, PredictCommand, ReactCommand, StrategyCommand, StressCommand, IntentCommand, LauncherCommand, LedgerCommand, LinkCommand,
    NotifyCommand, PlanCommand, PluginCommand, PrioritizeCommand, ProfileCommand, ReleaseCommand,
    SandboxCommand, SecurityCommand, SimulateCommand, TraceCommand, TradeoffCommand, UpdateCommand,
    WhyCommand, WorkspaceCommand,
};
use parser::{
    AnomalyCommands, AuditCommands, AutobiographyCommands, BootstrapCommands, CheckpointCommands,
    Cli, Commands, DecisionCommands, DepsCommands, DoctorCommands, EventsCommands,
    EvolutionCommands, GitCommands, GoalsCommands, PredictCommands, ReactCommands, StrategyCommands, StressCommands, IntentCommands, LauncherCommands,
    LedgerCommands, LinkCommands, NotifyCommands, PlanCommands, PluginCommands, PrioritizeCommands,
    ProfileCommands, ReleaseCommands, SandboxCommands, SecurityCommands, SimulateCommands,
    TraceCommands, TradeoffCommands, UpdateCommands, WhyCommands, WorkspaceCommands,
};

pub fn parse() -> Command {
    let cli = Cli::parse();
    match cli.command {
        Commands::Version => Command::Version,
        Commands::Plugin { command } => Command::Plugin(match command {
            PluginCommands::List => PluginCommand::List,
            PluginCommands::Add { name } => PluginCommand::Add { name },
            PluginCommands::Remove { name } => PluginCommand::Remove { name },
            PluginCommands::Status { name } => PluginCommand::Status { name },
        }),
        Commands::Doctor { command } => Command::Doctor(match command {
            DoctorCommands::Run { preflight } => DoctorCommand::Run { preflight },
            DoctorCommands::Aliases { subcmd } => DoctorCommand::Aliases { subcmd },
            DoctorCommands::Entropy {
                baseline,
                trends,
                json,
            } => DoctorCommand::Entropy {
                baseline,
                trends,
                json,
            },
            DoctorCommands::Bins { subcmd } => DoctorCommand::Bins { subcmd },
            DoctorCommands::Trend => DoctorCommand::Trend,
            DoctorCommands::Forecast => DoctorCommand::Forecast,
            DoctorCommands::Rebuild => DoctorCommand::Rebuild,
        }),
        Commands::Link { command } => Command::Link(match command {
            LinkCommands::Status { json } => LinkCommand::Status { json },
            LinkCommands::List => LinkCommand::List,
            LinkCommands::Audit => LinkCommand::Audit,
            LinkCommands::Plan { package } => LinkCommand::Plan { package },
            LinkCommands::Deploy {
                package,
                no_snapshot,
                adopt,
            } => LinkCommand::Deploy {
                package,
                no_snapshot,
                adopt,
            },
            LinkCommands::Undeploy { package } => LinkCommand::Undeploy { package },
            LinkCommands::Adopt { package } => LinkCommand::Adopt { package },
            LinkCommands::Redeploy { package } => LinkCommand::Redeploy { package },
            LinkCommands::Sync { package } => LinkCommand::Sync { package },
        }),
        Commands::Zone {
            icon,
            label,
            json,
            health,
        } => Command::Zone {
            icon,
            label,
            json,
            health,
        },
        Commands::Intent { command } => Command::Intent(match command {
            IntentCommands::List {
                planned,
                active,
                complete,
            } => IntentCommand::List {
                planned,
                active,
                complete,
            },
            IntentCommands::Show { id } => IntentCommand::Show { id },
            IntentCommands::Search { term } => IntentCommand::Search { term },
            IntentCommands::Stats => IntentCommand::Stats,
            IntentCommands::Validate => IntentCommand::Validate,
            IntentCommands::Focus { id } => IntentCommand::Focus { id },
            IntentCommands::Unfocus => IntentCommand::Unfocus,
            IntentCommands::Status => IntentCommand::FocusStatus,
            IntentCommands::Drift => IntentCommand::Drift,
            IntentCommands::Start { id } => IntentCommand::Start { id },
            IntentCommands::Complete { id } => IntentCommand::Complete { id },
            IntentCommands::New { template, title } => IntentCommand::New { template, title },
            IntentCommands::Deps { id } => IntentCommand::Deps { id },
            IntentCommands::Burndown => IntentCommand::Burndown,
            IntentCommands::Velocity => IntentCommand::Velocity,
            IntentCommands::Branch { id } => IntentCommand::Branch { id },
        }),
        Commands::Profile { command } => Command::Profile(match command {
            ProfileCommands::List => ProfileCommand::List,
            ProfileCommands::Status => ProfileCommand::Status,
            ProfileCommands::Switch { name } => ProfileCommand::Switch { name },
            ProfileCommands::History => ProfileCommand::History,
            ProfileCommands::Health => ProfileCommand::Health,
        }),
        Commands::Security { command } => Command::Security(match command {
            SecurityCommands::Scan => SecurityCommand::Scan,
            SecurityCommands::Report { all } => SecurityCommand::Report { all },
            SecurityCommands::Show { id } => SecurityCommand::Show { id },
            SecurityCommands::History => SecurityCommand::History,
            SecurityCommands::Simulate { patch } => SecurityCommand::Simulate { patch },
            SecurityCommands::Debt => SecurityCommand::Debt,
            SecurityCommands::Trend => SecurityCommand::Trend,
            SecurityCommands::Advise => SecurityCommand::Advise,
        }),
        Commands::Sandbox { command } => Command::Sandbox(match command {
            SandboxCommands::Run { args } => SandboxCommand::Run { args },
            SandboxCommands::Diff => SandboxCommand::Diff,
            SandboxCommands::Status => SandboxCommand::Status,
            SandboxCommands::Clear => SandboxCommand::Clear,
            SandboxCommands::Snapshot { target, name } => SandboxCommand::Snapshot { target, name },
            SandboxCommands::Restore { name } => SandboxCommand::Restore { name },
            SandboxCommands::Snapshots => SandboxCommand::Snapshots,
        }),
        Commands::Fetch { health_check } => Command::Fetch { health_check },
        Commands::Git { command } => Command::Git(match command {
            GitCommands::Status => GitCommand::Status,
            GitCommands::Risk => GitCommand::Risk,
            GitCommands::Log { n } => GitCommand::Log { n },
            GitCommands::Verify => GitCommand::Verify,
            GitCommands::Commit { args } => GitCommand::Delegate {
                subcmd: "commit".to_string(),
                args,
            },
            GitCommands::Sync { args } => GitCommand::Delegate {
                subcmd: "sync".to_string(),
                args,
            },
            GitCommands::Quick { args } => GitCommand::Delegate {
                subcmd: "quick".to_string(),
                args,
            },
            GitCommands::Branch { args } => GitCommand::Delegate {
                subcmd: "branch".to_string(),
                args,
            },
            GitCommands::InstallHooks => GitCommand::Delegate {
                subcmd: "install-hooks".to_string(),
                args: vec![],
            },
            GitCommands::RemoveHooks => GitCommand::Delegate {
                subcmd: "remove-hooks".to_string(),
                args: vec![],
            },
        }),
        Commands::Workspace { command } => Command::Workspace(match command {
            WorkspaceCommands::View {
                active,
                summary,
                json,
            } => WorkspaceCommand::View {
                active,
                summary,
                json,
            },
            WorkspaceCommands::Recent {
                range,
                limit,
                full_paths,
            } => WorkspaceCommand::Recent {
                range,
                limit,
                full_paths,
            },
            WorkspaceCommands::Fm { args } => WorkspaceCommand::Fm { args },
        }),
        Commands::Release { command } => Command::Release(match command {
            ReleaseCommands::Get { package } => ReleaseCommand::Get { package },
            ReleaseCommands::BumpTool { args } => ReleaseCommand::BumpTool { args },
            ReleaseCommands::BumpSystem { dry_run } => ReleaseCommand::BumpSystem { dry_run },
        }),
        Commands::Notify { command } => Command::Notify(match command {
            NotifyCommands::Send {
                summary,
                body,
                urgency,
            } => NotifyCommand::Send {
                summary,
                body,
                urgency,
            },
            NotifyCommands::Status => NotifyCommand::Status,
        }),
        Commands::Lock { health_check } => Command::Lock { health_check },
        Commands::Launcher { command } => Command::Launcher(match command {
            LauncherCommands::Palette { dmenu, prompt } => {
                LauncherCommand::Palette { dmenu, prompt }
            }
            LauncherCommands::Dmenu {
                subcmd,
                prompt,
                multi,
            } => LauncherCommand::Dmenu {
                subcmd,
                prompt,
                multi,
            },
            LauncherCommands::Launch { args } => LauncherCommand::Launch { args },
        }),
        Commands::Update { command } => Command::Update(match command {
            UpdateCommands::Run { args } => UpdateCommand::Run { args },
            UpdateCommands::Safe { args } => UpdateCommand::Safe { args },
        }),
        Commands::Events { command } => Command::Events(match command {
            EventsCommands::Status => EventsCommand::Status,
            EventsCommands::Archive => EventsCommand::Archive,
            EventsCommands::List => EventsCommand::List,
            EventsCommands::Since { duration } => EventsCommand::Since { duration },
            EventsCommands::Filter { domain } => EventsCommand::Filter { domain },
            EventsCommands::Watch => EventsCommand::Watch,
        }),
        Commands::Ledger { command } => Command::Ledger(match command {
            LedgerCommands::Stats => LedgerCommand::Stats,
            LedgerCommands::Query { domain } => LedgerCommand::Query { domain },
            LedgerCommands::Export => LedgerCommand::Export,
            LedgerCommands::Indexes => LedgerCommand::Indexes,
        }),
        Commands::Why { command } => Command::Why(match command {
            WhyCommands::Summary => WhyCommand::Summary,
            WhyCommands::Health => WhyCommand::Health,
            WhyCommands::Domain { domain } => WhyCommand::Domain { domain },
            WhyCommands::Visual => WhyCommand::Visual,
            WhyCommands::Attention => WhyCommand::Attention,
            WhyCommands::HealthSince { since } => WhyCommand::HealthSince { since },
            WhyCommands::Causal { domain } => WhyCommand::Causal { domain },
            WhyCommands::Chain => WhyCommand::Chain,
            WhyCommands::Correlate { domain_a, domain_b } => {
                WhyCommand::Correlate { domain_a, domain_b }
            }
            WhyCommands::Suggest => WhyCommand::Suggest,
            WhyCommands::Workspace => WhyCommand::Workspace,
            WhyCommands::Focus => WhyCommand::Focus,
        }),
        Commands::Trace { command } => Command::Trace(match command {
            TraceCommands::Last => TraceCommand::Last,
            TraceCommands::Domain { domain } => TraceCommand::Domain { domain },
        }),
        Commands::Goals { command } => Command::Goals(match command {
            GoalsCommands::List => GoalsCommand::List,
            GoalsCommands::Generate => GoalsCommand::Generate,
            GoalsCommands::Priority => GoalsCommand::Priority,
            GoalsCommands::Accept { id } => GoalsCommand::Accept { id },
            GoalsCommands::Reject { id } => GoalsCommand::Reject { id },
            GoalsCommands::Show { id } => GoalsCommand::Show { id },
        }),
        Commands::Stress { command } => Command::Stress(match command {
            StressCommands::Events => StressCommand::Events,
            StressCommands::Predict => StressCommand::Predict,
            StressCommands::React => StressCommand::React,
            StressCommands::Health => StressCommand::Health,
            StressCommands::Intents => StressCommand::Intents,
            StressCommands::Report => StressCommand::Report,
            StressCommands::HealthReport => StressCommand::HealthReport,
            StressCommands::Scenario1 => StressCommand::Scenario1,
            StressCommands::Scenario2 => StressCommand::Scenario2,
            StressCommands::Scenario3 => StressCommand::Scenario3,
            StressCommands::Scenario4 => StressCommand::Scenario4,
            StressCommands::Scenario5 => StressCommand::Scenario5,
        }),
        Commands::Predict { command } => Command::Predict(match command {
            PredictCommands::Sessions => PredictCommand::Sessions,
            PredictCommands::Cadence => PredictCommand::Cadence,
            PredictCommands::Health => PredictCommand::Health,
            PredictCommands::Decline => PredictCommand::Decline,
            PredictCommands::Intents => PredictCommand::Intents,
            PredictCommands::Next => PredictCommand::Next,
            PredictCommands::Coupling => PredictCommand::Coupling,
            PredictCommands::Churn => PredictCommand::Churn,
            PredictCommands::Accuracy => PredictCommand::Accuracy,
        }),
        Commands::React { command } => Command::React(match command {
            ReactCommands::List => ReactCommand::List,
            ReactCommands::Rules => ReactCommand::Rules,
            ReactCommands::Run => ReactCommand::Run,
            ReactCommands::History => ReactCommand::History,
            ReactCommands::Explain { id } => ReactCommand::Explain { id },
            ReactCommands::Discipline => ReactCommand::Discipline,
            ReactCommands::Enable { id } => ReactCommand::Enable { id },
            ReactCommands::Disable { id } => ReactCommand::Disable { id },
            ReactCommands::Add { id, description, priority, cooldown_m } =>
                ReactCommand::Add { id, description, priority, cooldown_m },
            ReactCommands::Bounds => ReactCommand::Bounds,
            ReactCommands::Audit => ReactCommand::Audit,
            ReactCommands::Story => ReactCommand::Story,
            ReactCommands::Coalesce => ReactCommand::Coalesce,
            ReactCommands::DisciplineShow => ReactCommand::DisciplineShow,
        }),
        Commands::Strategy { command } => Command::Strategy(match command {
            StrategyCommands::Now => StrategyCommand::Now,
            StrategyCommands::Week => StrategyCommand::Week,
            StrategyCommands::Quarter => StrategyCommand::Quarter,
        }),
        Commands::Plan { command } => Command::Plan(match command {
            PlanCommands::Generate { goal_id } => PlanCommand::Generate { goal_id },
            PlanCommands::Review { id } => PlanCommand::Review { id },
            PlanCommands::Simulate { id } => PlanCommand::Simulate { id },
            PlanCommands::List => PlanCommand::List,
        }),
        Commands::Tradeoff { command } => Command::Tradeoff(match command {
            TradeoffCommands::Analyze { decision } => TradeoffCommand::Analyze { decision },
            TradeoffCommands::History => TradeoffCommand::History,
            TradeoffCommands::Balance => TradeoffCommand::Balance,
        }),
        Commands::Prioritize { command } => Command::Prioritize(match command {
            PrioritizeCommands::Run => PrioritizeCommand::Run,
            PrioritizeCommands::Explain => PrioritizeCommand::Explain,
        }),
        Commands::Autobiography { command } => Command::Autobiography(match command {
            AutobiographyCommands::Narrate { version } => AutobiographyCommand::Narrate { version },
        }),
        Commands::Evolution { command } => Command::Evolution(match command {
            EvolutionCommands::Map => EvolutionCommand::Map,
            EvolutionCommands::Tools => EvolutionCommand::Tools,
            EvolutionCommands::Suggest => EvolutionCommand::Suggest,
            EvolutionCommands::EvolvePropose => EvolutionCommand::EvolvePropose,
            EvolutionCommands::EvolveList => EvolutionCommand::EvolveList,
            EvolutionCommands::EvolveAccept { id } => EvolutionCommand::EvolveAccept { id },
            EvolutionCommands::EvolveReject { id } => EvolutionCommand::EvolveReject { id },
            EvolutionCommands::FutureSim { change } => EvolutionCommand::FutureSim { change },
            EvolutionCommands::FutureRisk { change } => EvolutionCommand::FutureRisk { change },
            EvolutionCommands::FutureImpact { change } => EvolutionCommand::FutureImpact { change },
        }),
        Commands::Simulate { command } => Command::Simulate(match command {
            SimulateCommands::Doctor => SimulateCommand::Doctor,
            SimulateCommands::Update => SimulateCommand::Update,
            SimulateCommands::Scenario { description } => SimulateCommand::Scenario { description },
        }),
        Commands::Checkpoint { command } => Command::Checkpoint(match command {
            CheckpointCommands::Create { name, notes } => CheckpointCommand::Create { name, notes },
            CheckpointCommands::List => CheckpointCommand::List,
            CheckpointCommands::Diff { name } => CheckpointCommand::Diff { name },
            CheckpointCommands::Restore { name } => CheckpointCommand::Restore { name },
            CheckpointCommands::LastGood => CheckpointCommand::LastGood,
            CheckpointCommands::Snapshot { label } => CheckpointCommand::Snapshot { label },
            CheckpointCommands::Snapshots => CheckpointCommand::Snapshots,
        }),
        Commands::Decision { command } => Command::Decision(match command {
            DecisionCommands::Record {
                description,
                intent,
            } => DecisionCommand::Record {
                description,
                intent,
            },
            DecisionCommands::Outcome { id, result, notes } => {
                DecisionCommand::Outcome { id, result, notes }
            }
            DecisionCommands::List { open } => DecisionCommand::List { open },
            DecisionCommands::Hindsight => DecisionCommand::Hindsight,
            DecisionCommands::Show { id } => DecisionCommand::Show { id },
            DecisionCommands::Stats => DecisionCommand::Stats,
            DecisionCommands::Advise { decision } => DecisionCommand::Advise { decision },
            DecisionCommands::Heuristics { domain } => DecisionCommand::Heuristics { domain },
            DecisionCommands::Lessons => DecisionCommand::Lessons,
            DecisionCommands::Story => DecisionCommand::Story,
            DecisionCommands::Patterns => DecisionCommand::Patterns,
            DecisionCommands::Friction => DecisionCommand::Friction,
            DecisionCommands::Reversal => DecisionCommand::Reversal,
        }),
        Commands::Heuristics { domain } => {
            Command::Decision(DecisionCommand::Heuristics { domain })
        }
        Commands::Deps { command } => Command::Deps(match command {
            DepsCommands::Graph => DepsCommand::Graph,
            DepsCommands::Risk => DepsCommand::Risk,
            DepsCommands::Audit => DepsCommand::Audit,
        }),
        Commands::Narrative { since, intent } => Command::Narrative { since, intent },
        Commands::Snapshot { json, save } => Command::Snapshot { json, save },
        Commands::Bootstrap { command } => Command::Bootstrap(match command {
            BootstrapCommands::Plan => BootstrapCommand::Plan,
            BootstrapCommands::Verify => BootstrapCommand::Verify,
            BootstrapCommands::Diff => BootstrapCommand::Diff,
        }),
        Commands::Anomaly { command } => Command::Anomaly(match command {
            AnomalyCommands::Scan => AnomalyCommand::Scan,
            AnomalyCommands::History => AnomalyCommand::History,
            AnomalyCommands::Alert => AnomalyCommand::Alert,
        }),
        Commands::Audit { command } => Command::Audit(match command {
            AuditCommands::Scan => AuditCommand::Scan,
            AuditCommands::Show { tool } => AuditCommand::Show { tool },
            AuditCommands::Stale => AuditCommand::Stale,
            AuditCommands::Coverage => AuditCommand::Coverage,
        }),
        Commands::Lessons => Command::Decision(DecisionCommand::Lessons),
        Commands::Story => Command::Decision(DecisionCommand::Story),
        Commands::Advise { decision } => Command::Decision(DecisionCommand::Advise { decision }),
        Commands::Decide {
            description,
            intent,
        } => Command::Decision(DecisionCommand::Record {
            description,
            intent,
        }),
        Commands::Hindsight => Command::Decision(DecisionCommand::Hindsight),
        Commands::Capabilities { json, domain } => Command::Capabilities { json, domain },
    }
}
