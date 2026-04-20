#[derive(Debug)]
pub enum KnowledgeCommand {
    Search { term: String },
    Patterns { domain: Option<String> },
    Accuracy,
    Add { domain: String, description: String, resolution: String },
    Seed,
    Show { id: String },
    Outcome { id: String, correct: String },  // "yes"/"no" or "true"/"false"
}

#[derive(Debug)]
pub enum FridayArchCommand {
    Run,
    Models,
    Proposals,
    Contradictions,
}

#[derive(Debug)]
pub enum SynthesizeCommand {
    Now,
    Brief,
    History,
}

#[derive(Debug)]
pub enum FridayCommand {
    Status,
    Ask { question: String },
    Observe,
    ExtractPatterns,
    Suggest,
    UpdatePersonality,
    SeedKnowledge,
    LearningLoop,
    NameAbstraction { name: String, description: String },
    Vocabulary,
    ProposeIntent,
    // INT-219 Phase 2
    Phase2Init,
    Phase2Status,
    Plan,
    TemporalModels,
}

pub enum Command {
    Anomaly(AnomalyCommand),
    Bootstrap(BootstrapCommand),
    Snapshot {
        json: bool,
        save: bool,
    },
    Narrative {
        since: Option<String>,
        intent: Option<String>,
    },
    Deps(DepsCommand),
    Audit(AuditCommand),
    Version,
    Doctor(DoctorCommand),
    Plugin(PluginCommand),
    Link(LinkCommand),
    Zone {
        icon: bool,
        label: bool,
        json: bool,
        health: bool,
    },
    Intent(IntentCommand),
    Profile(ProfileCommand),
    Security(SecurityCommand),
    Sandbox(SandboxCommand),
    Fetch {
        health_check: bool,
    },
    Git(GitCommand),
    Workspace(WorkspaceCommand),
    Release(ReleaseCommand),
    Notify(NotifyCommand),
    Lock {
        health_check: bool,
    },
    Launcher(LauncherCommand),
    Update(UpdateCommand),
    Weight(WeightCommand),
    Daemon(DaemonCommand),
    Self_(SelfCommand),
    Journal(JournalCommand),
    Docs(DocsCommand),
    Values(ValuesCommand),
    Align(AlignCommand),
    Engines(EnginesCommand),
    Events(EventsCommand),
    Ledger(LedgerCommand),
    Why(WhyCommand),
    Trace(TraceCommand),
    Simulate(SimulateCommand),
    Delegate(DelegateCommand),
    Evolution(EvolutionCommand),
    Goals(GoalsCommand),
    Stress(StressCommand),
    Predict(PredictCommand),
    React(ReactCommand),
    Strategy(StrategyCommand),
    Genealogy(GenealogyCommand),
    Integrity(IntegrityCommand),
    Autonomy(AutonomyCommand),
    Partner(PartnerCommand),
    Registry(RegistryCommand),
    Db(DbCommand),
    Plan(PlanCommand),
    Tradeoff(TradeoffCommand),
    Prioritize(PrioritizeCommand),
    Autobiography(AutobiographyCommand),
    Checkpoint(CheckpointCommand),
    Capabilities {
        json: bool,
        domain: Option<String>,
    },
    Decision(DecisionCommand),
    Deploy(DeployCommand),
    Friday(FridayCommand),
    Synthesize(SynthesizeCommand),
    FridayArch(FridayArchCommand),
    Knowledge(KnowledgeCommand),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum DeployCommand {
    Check { tool: String },
    Record { tool: String, version: String, outcome: String, duration_ms: i64, intent: Option<String> },
    Log,
    Rollback { tool: Option<String>, dry_run: bool },
    CheckDeps { tool: String },
}
#[derive(Debug)]
pub enum DecisionCommand {
    Record {
        description: String,
        intent: Option<String>,
    },
    Outcome {
        id: String,
        result: String,
        notes: Option<String>,
    },
    List {
        open: bool,
    },
    Hindsight,
    Show {
        id: String,
    },
    Stats,
    Advise {
        decision: Option<String>,
    },
    Heuristics {
        domain: Option<String>,
    },
    Lessons,
    Story,
    Patterns,
    Friction,
    Reversal,
}

#[derive(Debug)]
pub enum DoctorCommand {
    Run {
        preflight: bool,
    },
    Aliases {
        subcmd: Option<String>,
    },
    Entropy {
        baseline: bool,
        trends: bool,
        json: bool,
    },
    Bins {
        subcmd: Option<String>,
    },
    Trend,
    Forecast,
    Rebuild,
    Quick,
    History,
}

#[derive(Debug)]
pub enum LinkCommand {
    Status {
        json: bool,
    },
    List,
    Audit,
    Plan {
        package: Option<String>,
    },
    Deploy {
        package: Option<String>,
        no_snapshot: bool,
        adopt: bool,
    },
    Undeploy {
        package: String,
    },
    Adopt {
        package: Option<String>,
    },
    Redeploy {
        package: Option<String>,
    },
    Sync {
        package: Option<String>,
    },
}

#[derive(Debug)]
pub enum IntentCommand {
    Focus {
        id: String,
    },
    Unfocus,
    FocusStatus,
    Drift,
    Start {
        id: String,
    },
    Complete {
        id: String,
    },
    New {
        template: String,
        title: String,
        smart: bool,
    },
    Deps {
        id: Option<String>,
        critical_path: bool,
    },
    Burndown,
    Velocity,
    Branch {
        id: String,
    },
    List {
        planned: bool,
        active: bool,
        complete: bool,
    },
    Show {
        id: String,
    },
    Search {
        term: String,
    },
    Stats,
    Validate,
    Edit {
        id: String,
    },
    Health { stale: bool },
    Predict { id: String },
    AutoLink { id: String },
    Story { id: String },
}
#[derive(Debug)]
pub enum ProfileCommand {
    List,
    Status,
    Switch { name: String },
    History,
    Health,
}

#[derive(Debug)]
pub enum SecurityCommand {
    Scan,
    Advise,
    Debt,
    Trend,
    Report { all: bool },
    Show { id: String },
    History,
    Simulate { patch: String },
}

#[derive(Debug)]
pub enum SandboxCommand {
    Run { args: Vec<String> },
    Diff,
    Status,
    Clear,
    Snapshot { target: String, name: String },
    Restore { name: String },
    Snapshots,
}

#[derive(Debug)]
pub enum GitCommand {
    Status,
    Risk,
    Log { n: u32 },
    Verify,
    Delegate { subcmd: String, args: Vec<String> },
}

#[derive(Debug)]
pub enum WorkspaceCommand {
    View {
        active: bool,
        summary: bool,
        json: bool,
    },
    Recent {
        range: String,
        limit: u32,
        full_paths: bool,
    },
    Fm {
        args: Vec<String>,
    },
}

#[derive(Debug)]
pub enum ReleaseCommand {
    Get { package: Option<String> },
    BumpTool { args: Vec<String> },
    BumpSystem { dry_run: bool },
}

#[derive(Debug)]
pub enum NotifyCommand {
    Send {
        summary: String,
        body: Option<String>,
        urgency: String,
    },
    Status,
}

#[derive(Debug)]
pub enum LauncherCommand {
    Palette {
        dmenu: bool,
        prompt: Option<String>,
    },
    Dmenu {
        subcmd: Option<String>,
        prompt: Option<String>,
        multi: bool,
    },
    Launch {
        args: Vec<String>,
    },
}

#[derive(Debug)]
pub enum UpdateCommand {
    Run { args: Vec<String> },
    Safe { args: Vec<String> },
}

#[derive(Debug)]
pub enum LedgerCommand {
    Stats,
    Query { domain: String },
    Export,
    Indexes,
}
#[derive(Debug)]
pub enum ValuesCommand {
    List,
    Define { statement: String, weight: i64, scope: String },
    Remove { id: i64 },
    Weight { id: i64, weight: i64 },
}

#[derive(Debug)]
pub enum WeightCommand {
    List,
    Top,
    Compute,
    Explain { id: String },
    Calibrate { id: String, outcome: String },
}
#[derive(Debug)]
pub enum DaemonCommand {
    Status,
    Context,
    Signals { limit: u32 },
    Neovim { file_path: String },
    Watchdog,
}
#[derive(Debug)]
pub enum SelfCommand {
    Map,
    Evolve,
    Apply { proposal_id: i64, dry_run: bool, checkpoint: bool },
    History,
    Learn { proposal_id: i64, outcome: String },
    Accuracy,
    Calibrate,
    Challenge { intent_id: String },
}
#[derive(Debug)]
pub enum JournalCommand {
    Today,
    Yesterday,
    Week,
    Search { term: String },
    Show { date: String },
    SessionStart,
    DailySummary,
}
#[derive(Debug)]
pub enum DocsCommand {
    Commands,
    List,
}
#[derive(Debug)]
pub enum AlignCommand {
    Check { subject: String },
    Drift,
    Report { weeks_ago: i64 },
}

#[derive(Debug)]
pub enum EnginesCommand {
    Status,
    Sync { engine: String },
    Signals,
    Check,
    UpgradeLog,
    Process,
}

#[derive(Debug)]
pub enum EventsCommand {
    Status,
    Archive,
    List,
    Since { duration: String },
    Filter { domain: String },
    Watch,
    EmitV2 { type_name: String, payload: String, caused_by: Option<i64> },
    Replay { from_seq: i64, to_seq: i64 },
    Chain { seq: i64 },
    StatusV2,
}

#[derive(Debug)]
pub enum WhyCommand {
    Summary,
    Health,
    Domain { domain: String },
    Visual,
    Attention,
    HealthSince { since: String },
    Causal { domain: String },
    Chain,
    Correlate { domain_a: String, domain_b: String },
    Suggest,
    Workspace,
    Focus,
}

#[derive(Debug)]
pub enum SimulateCommand {
    Doctor,
    Update,
    Scenario { description: String },
}

#[derive(Debug)]
pub enum DelegateCommand {
    Simulate { action: String },
    Contracts,
    History,
    Accuracy,
    Suspend,
    Activate { contract: String },    Counterfactuals,
    LogCounterfactual { proposed: String, human: String, matched: bool, confidence: f64 },
    AccuracyReport,
}
#[derive(Debug)]
pub enum TraceCommand {
    Last,
    Domain { domain: String },
}

#[derive(Debug)]
pub enum PluginCommand {
    List,
    Add { name: String },
    Remove { name: String },
    Status { name: String },
}

#[derive(Debug)]
pub enum CheckpointCommand {
    Restore { name: String },
    LastGood,
    Snapshot { label: String },
    Snapshots,
    Create { name: String, notes: Option<String> },
    List,
    Diff { name: String },
}

#[derive(Debug)]
pub enum AuditCommand {
    Scan,
    Show { tool: String },
    Stale,
    Coverage,
}

#[derive(Debug)]
pub enum AnomalyCommand {
    Scan,
    History,
    Alert,
}

#[derive(Debug)]
pub enum BootstrapCommand {
    Plan,
    Verify,
    Diff,
}

#[derive(Debug)]
pub enum DepsCommand {
    Graph,
    Risk,
    Audit,
}

#[derive(Debug, Clone)]
pub enum GoalsCommand {
    List,
    Generate,
    Priority,
    Accept { id: String },
    Reject { id: String },
    Show { id: String },
}

#[derive(Debug, Clone)]
pub enum StressCommand {
    Events,
    Predict,
    React,
    Health,
    Intents,
    Report,
    HealthReport,
    Scenario1,
    Scenario2,
    Scenario3,
    Scenario4,
    Scenario5,
}

#[derive(Debug, Clone)]
pub enum PredictCommand {
    Sessions,
    Cadence,
    Health,
    Decline,
    Intents,
    Next,
    Coupling,
    Churn,
    Accuracy,
    Verify { id: String, correct: bool },
    Why { id: String },
    CrossSession,
    MemoryDecay { apply: bool },
}

#[derive(Debug, Clone)]
pub enum ReactCommand {
    List,
    Rules,
    Run,
    History,
    Explain { id: String },
    Discipline,
    Enable { id: String },
    Disable { id: String },
    Add { id: String, description: String, priority: u8, cooldown_m: i64 },
    Bounds,
    Audit,
    Story,
    Coalesce,
    DisciplineShow,
}

#[derive(Debug, Clone)]
pub enum PlanCommand {
    Generate { goal_id: String },
    Review { id: String },
    Simulate { id: String },
    List,
}

#[derive(Debug, Clone)]
pub enum TradeoffCommand {
    Analyze { decision: String },
    History,
    Balance,
}

#[derive(Debug, Clone)]
pub enum PrioritizeCommand {
    Run,
    Explain,
}

#[derive(Debug, Clone)]
pub enum AutobiographyCommand {
    Narrate { version: Option<String> },
}

#[derive(Debug, Clone)]
pub enum EvolutionCommand {
    Map,
    Tools,
    Suggest,
    EvolvePropose,
    EvolveList,
    EvolveAccept { id: String },
    EvolveReject { id: String },
    FutureSim { change: String },
    FutureRisk { change: String },
    FutureImpact { change: String },
}

#[derive(Debug, Clone)]
pub enum StrategyCommand {
    Now,
    Week,
    Quarter,
    Sequence { goal_id: String },
    Unblock,
    Tradeoff { action: String },
    Conflicts,
    Coherence,
    Merge { goal1: String, goal2: String },
    Jarvis,
    Trust,
    Gap,
    History,
    Learn { strategy_id: String, outcome: String },
    Review,
    Next { list: bool, why: Option<String> },
    Queue,
    Blockers,
}

#[derive(Debug, Clone)]
pub enum GenealogyCommand {
    Show { id: String },
    Tree,
    Roots,
}
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum IntegrityCommand { Run, Status, Log, Fix, Apply { id: String }, Heal { dry_run: bool }, Trend }
#[derive(Debug, Clone)]
pub enum AutonomyCommand {
    MandateList,
    MandateSet { rule: String },
    MandateRevoke { id: String },
    MandateRevokeAll,
    Pending,
    Run,
    Log,
    Revert,
    TrustScore,
    TrustHistory,
    TrustExpand,
}

#[derive(Debug, Clone)]
pub enum PartnerCommand {
    // Phase 1 — Collaborative Intent Creation
    Propose,
    Discuss { intent_id: String },
    Disagree { intent_id: String },
    // Phase 2 — Shared Decision Making
    Consult { question: String },
    // Phase 3 — Longitudinal Memory
    Reflect,
    Pattern,
    Growth,
    // Phase 4 — Honest Disagreement
    Pushback,
    // Phase 5 — Co-Authored Roadmap
    Roadmap,
    RoadmapWhy,
    RoadmapDiff,
    // Status
    Status,
}
#[derive(Debug, Clone)]
pub enum RegistryCommand {
    List,
    Show { name: String },
    Retire { name: String },
    Unretire { name: String },
    RealityCheck,
}

#[derive(Debug, Clone)]
pub enum DbCommand {
    Backup,
    Restore { file: String },
    Verify,
    Status,
    Compact,
}
