use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "core")]
#[command(version = env!("CARGO_PKG_VERSION"), long_version = concat!(env!("CARGO_PKG_VERSION"), "  ·  intelligence v18 (Synthesis Engine)"))]
#[command(about = "0-Core \u{2014} single orchestrator binary")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Deploy intelligence -- check, record, log
    Deploy {
        #[command(subcommand)]
        command: DeployCommands,
    },
    /// Friday -- The Living Intelligence
    Friday {
        #[command(subcommand)]
        command: FridayCommands,
    },
    /// Core v18 Synthesis Engine
    Synthesize {
        #[command(subcommand)]
        command: SynthesizeCommands,
    },
    Version,
    Doctor {
        #[command(subcommand)]
        command: DoctorCommands,
    },
    /// Manage ecosystem plugins
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
    },
    Link {
        #[command(subcommand)]
        command: LinkCommands,
    },
    Zone {
        #[arg(long)]
        icon: bool,
        #[arg(long)]
        label: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        health: bool,
    },
    Intent {
        #[command(subcommand)]
        command: IntentCommands,
    },
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    Security {
        #[command(subcommand)]
        command: SecurityCommands,
    },
    Sandbox {
        #[command(subcommand)]
        command: SandboxCommands,
    },
    Fetch {
        #[arg(long)]
        health_check: bool,
    },
    Git {
        #[command(subcommand)]
        command: GitCommands,
    },
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },
    Release {
        #[command(subcommand)]
        command: ReleaseCommands,
    },
    Notify {
        #[command(subcommand)]
        command: NotifyCommands,
    },
    Lock {
        #[arg(long)]
        health_check: bool,
    },
    Launcher {
        #[command(subcommand)]
        command: LauncherCommands,
    },
    Update {
        #[command(subcommand)]
        command: UpdateCommands,
    },
    /// Core v17 — Pattern Weight Engine
    Weight {
        #[command(subcommand)]
        command: WeightCommands,
    },
    /// Query faelight-daemon v2 — background brain
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },
    /// Core v16 — Self-Transformation: The Forest Redesigns Itself
    Self_ {
        #[command(subcommand)]
        command: SelfCommands,
    },
    /// Forest journal — the system writes its own story
    Journal {
        #[command(subcommand)]
        command: JournalCommands,
    },
    /// Forest documentation — access guides and references
    Docs {
        #[command(subcommand)]
        command: DocsCommands,
    },
    /// Declared values system — define and manage your principles
    Values {
        #[command(subcommand)]
        command: ValuesCommands,
    },
    /// Alignment checking — verify behavior matches declared values
    Align {
        #[command(subcommand)]
        command: AlignCommands,
    },
    /// Engine coordination layer — synchronize and monitor all forest engines
    Engines {
        #[command(subcommand)]
        command: EnginesCommands,
    },
    /// Query the event ledger
    Events {
        #[command(subcommand)]
        command: EventsCommands,
    },
    /// Ledger analytics — query and export the event ledger
    Ledger {
        #[command(subcommand)]
        command: LedgerCommands,
    },
    /// Causality engine — why is the system in this state?
    Why {
        #[command(subcommand)]
        command: WhyCommands,
    },
    /// Trace event history with full detail
    Trace {
        #[command(subcommand)]
        command: TraceCommands,
    },
    /// Goal engine — the forest chooses where to grow (Core v9)
    Goals {
        #[command(subcommand)]
        command: GoalsCommands,
    },
    /// Stress test engine — verify v11 before v12 builds on top (INT-152)
    Stress {
        #[command(subcommand)]
        command: StressCommands,
    },
    /// Prediction engine — the forest anticipates (Core v11)
    Predict {
        #[command(subcommand)]
        command: PredictCommands,
    },
    /// Reaction engine — the forest responds without being asked (Core v10)
    React {
        #[command(subcommand)]
        command: ReactCommands,
    },
    /// Strategy engine — the forest plans across horizons (Core v12)
    Strategy {
        #[command(subcommand)]
        command: StrategyCommands,
    },
    /// Database backup and recovery (INT-166)
    Db {
        #[command(subcommand)]
        command: DbCommands,
    },
    /// Genealogy — the forest remembers how it grew (INT-153)
    Genealogy {
        #[command(subcommand)]
        command: GenealogyCommands,
    },
    /// Integrity — consistency oracle and self-repair engine (INT-184)
    Integrity {
        #[command(subcommand)]
        command: IntegrityCommands,
    },
    /// Autonomy — mandate system and autonomous action engine (INT-156)
    Autonomy {
        #[command(subcommand)]
        command: AutonomyCommands,
    },
    /// Partner — collaborative intent creation and shared decision making (v14)
    Partner {
        #[command(subcommand)]
        command: PartnerCommands,
    },
    /// Registry — manage the tool registry (INT-183)
    Registry {
        #[command(subcommand)]
        command: RegistryCommands,
    },
    /// Task planning — break accepted goals into concrete steps (Core v9)
    Plan {
        #[command(subcommand)]
        command: PlanCommands,
    },
    /// Tradeoff engine — surface competing values in decisions (Core v9)
    Tradeoff {
        #[command(subcommand)]
        command: TradeoffCommands,
    },
    /// Dynamic prioritization — rerank goals by live conditions (Core v9)
    Prioritize {
        #[command(subcommand)]
        command: PrioritizeCommands,
    },
    /// Intent autobiography — the forest narrates its own goal history (Core v9)
    Autobiography {
        #[command(subcommand)]
        command: AutobiographyCommands,
    },
    Evolution {
        #[command(subcommand)]
        command: EvolutionCommands,
    },
    Simulate {
        #[command(subcommand)]
        command: SimulateCommands,
    },
    /// Delegation engine — trust contracts and safe autonomy simulation
    Delegate {
        #[command(subcommand)]
        command: DelegateCommands,
    },
    /// Checkpoint system state for recovery
    Checkpoint {
        #[command(subcommand)]
        command: CheckpointCommands,
    },
    /// Show capability requirements for all domains
    /// Record and track decisions with context snapshots
    Decision {
        #[command(subcommand)]
        command: DecisionCommands,
    },
    /// Auto-derived heuristics from decision corpus
    Heuristics {
        /// Filter by domain
        #[arg(short, long)]
        domain: Option<String>,
    },
    /// Dependency intelligence
    Deps {
        #[command(subcommand)]
        command: DepsCommands,
    },
    /// Forest narrative — the story of how the forest became what it is
    Narrative {
        /// Show narrative since a version (e.g. v10.0.0)
        #[arg(long)]
        since: Option<String>,
        /// Show narrative for a specific intent ID
        #[arg(long)]
        intent: Option<String>,
    },
    /// Snapshot narrative — the forest writes its own autobiography
    Snapshot {
        /// Output as machine-readable JSON reconstruction seed
        #[arg(long)]
        json: bool,
        /// Save both markdown and JSON to runtime/snapshots/
        #[arg(long)]
        save: bool,
    },
    /// Bootstrap intelligence — rebuild guidance
    Bootstrap {
        #[command(subcommand)]
        command: BootstrapCommands,
    },
    /// Detect unexpected system changes
    Anomaly {
        #[command(subcommand)]
        command: AnomalyCommands,
    },
    /// Audit tool health and intelligence
    Audit {
        #[command(subcommand)]
        command: AuditCommands,
    },
    /// What the forest has learned — heuristics summary
    Lessons,
    /// The forest narrative — 30 day story
    Story,
    /// Judgment advisory for current system state
    Advise {
        /// Optional planned decision to evaluate
        decision: Option<String>,
    },
    /// Record a decision directly (shorthand for decision record)
    Decide {
        /// Description of the decision
        description: String,
        /// Related intent ID (e.g. INT-109)
        #[arg(short, long)]
        intent: Option<String>,
    },
    /// View decision hindsight summary
    Hindsight,
    Capabilities {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show only a specific domain
        #[arg(long)]
        domain: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
pub enum DeployCommands {
    /// Pre-deploy health gate + dependency check
    Check { tool: String },
    /// Record deploy outcome to state.db
    Record {
        tool: String,
        version: String,
        outcome: String,
        #[arg(long, default_value = "0")]
        duration_ms: i64,
        #[arg(long)]
        intent: Option<String>,
    },
    /// Show recent deploy history
    Log,
    /// Rollback tool to previous version
    Rollback {
        tool: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Show full dependency graph for a tool
    #[command(name = "check-deps")]
    CheckDeps { tool: String },
}
#[derive(Subcommand, Clone)]
pub enum SynthesizeCommands {
    /// Generate synthesis snapshot now
    Now,
    /// Show current Friday brief
    Brief,
    /// Show synthesis history
    History,
}

#[derive(Subcommand, Clone)]
pub enum FridayCommands {
    /// Show what Friday has observed and learned
    Status,
    /// Ask Friday a question about the forest
    Ask { question: String },
    /// Trigger observation cycle manually
    Observe,
    /// Extract patterns from shell history
    ExtractPatterns,
    /// Get Friday's evidence-based suggestion
    Suggest,
}

#[derive(Subcommand)]
pub enum DoctorCommands {
    /// Run full health check
    Run {
        #[arg(long)]
        preflight: bool,
    },
    /// Audit aliases
    Aliases { subcmd: Option<String> },
    /// Check configuration entropy/drift
    Entropy {
        #[arg(long)]
        baseline: bool,
        #[arg(long)]
        trends: bool,
        #[arg(long)]
        json: bool,
    },
    /// Check binary manifest drift
    Bins { subcmd: Option<String> },
    /// Health trend analysis — pattern over time
    Trend,
    /// Health forecast — predicted trajectory
    Forecast,
    /// Generate deterministic rebuild plan
    Rebuild,
    /// Quick health check — critical checks only
    Quick,
    /// Show health score history over time
    History,
}

#[derive(Subcommand)]
pub enum LinkCommands {
    Status {
        #[arg(long)]
        json: bool,
    },
    List,
    Audit,
    /// Show what deploy would do without doing it
    Plan {
        /// Package name (or 'all')
        package: Option<String>,
    },
    /// Deploy a package (create symlinks)
    Deploy {
        /// Package name (or 'all')
        package: Option<String>,
        /// Skip snapshot before deploy
        #[arg(long)]
        no_snapshot: bool,
        /// Replace real files with symlinks
        #[arg(long)]
        adopt: bool,
    },
    /// Remove a package's symlinks
    Undeploy {
        /// Package name
        package: String,
    },
    /// Undeploy then redeploy atomically
    /// Adopt existing files (convert real files to managed symlinks)
    Adopt {
        /// Package name (or all packages if omitted)
        package: Option<String>,
    },
    Redeploy {
        /// Package name (or 'all')
        package: Option<String>,
    },
    /// Sync all packages — deploy clean, surface conflicts with fix commands
    Sync {
        /// Package name (or 'all')
        package: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum IntentCommands {
    /// Set focused intent
    Focus {
        id: String,
    },
    /// Clear focused intent
    Unfocus,
    /// Show current focus and workflow state
    Status,
    /// Detect if work has drifted from focused intent
    Drift,
    /// Transition intent to in-progress and set focus
    Start {
        id: String,
    },
    /// Mark intent as complete and move to complete/
    Complete {
        id: String,
    },
    /// Create a new intent from template
    New {
        /// Template type: feature, fix, arch, study
        template: String,
        /// Intent title
        title: String,
        /// Context-aware smart creation — forest suggests based on active work
        #[arg(long)]
        smart: bool,
    },
    /// Show dependency graph for an intent
    Deps {
        /// Intent ID (optional when using --critical-path)
        id: Option<String>,
        /// Show critical path — longest chain to completion
        #[arg(long)]
        critical_path: bool,
    },
    /// Show completion burndown chart
    Burndown,
    /// Show velocity metrics
    Velocity,
    /// Generate git branch name for intent
    Branch {
        id: String,
    },
    List {
        #[arg(long)]
        planned: bool,
        #[arg(long)]
        active: bool,
        #[arg(long)]
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
    /// Predict completion date based on gate velocity
    Predict { id: String },
    /// Show rich narrative autobiography of an intent
    Story { id: String },
    /// Find related intents by tag overlap
    #[command(name = "auto-link")]
    AutoLink { id: String },
    /// Show health scores for active intents
    Health {
        /// Show only stalled intents
        #[arg(long)]
        stale: bool,
    },
    /// Open intent file in $EDITOR
    Edit {
        id: String,
    },
}

#[derive(Subcommand)]
pub enum ProfileCommands {
    List,
    Status,
    Switch { name: String },
    History,
    Health,
}

#[derive(Subcommand)]
pub enum SecurityCommands {
    Scan,
    /// Show debt score — how long each finding has been present
    Debt,
    /// Show finding count trend over time
    Trend,
    Report {
        #[arg(long)]
        all: bool,
    },
    Show {
        id: String,
    },
    History,
    /// Security judgment advisory
    Advise,
    /// Simulate applying a security patch or CVE fix
    Simulate {
        /// Patch name or CVE ID to simulate
        patch: String,
    },
}

#[derive(Subcommand)]
pub enum SandboxCommands {
    Run {
        args: Vec<String>,
    },
    Diff,
    Status,
    Clear,
    Snapshot {
        #[arg(long)]
        target: String,
        #[arg(long)]
        name: String,
    },
    Restore {
        name: String,
    },
    Snapshots,
}

#[derive(Subcommand)]
pub enum GitCommands {
    Status,
    Risk,
    Log {
        #[arg(short, long, default_value = "10")]
        n: u32,
    },
    Verify,
    Commit {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    Sync {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    Quick {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    Branch {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    InstallHooks,
    RemoveHooks,
}

#[derive(Subcommand)]
pub enum WorkspaceCommands {
    View {
        #[arg(long)]
        active: bool,
        #[arg(long)]
        summary: bool,
        #[arg(long)]
        json: bool,
    },
    Recent {
        #[arg(default_value = "today")]
        range: String,
        #[arg(short, long, default_value = "10")]
        limit: u32,
        #[arg(long)]
        full_paths: bool,
    },
    Fm {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ReleaseCommands {
    Get {
        package: Option<String>,
    },
    BumpTool {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    BumpSystem {
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum NotifyCommands {
    Send {
        summary: String,
        body: Option<String>,
        #[arg(long, default_value = "normal")]
        urgency: String,
    },
    Status,
}

#[derive(Subcommand)]
pub enum LauncherCommands {
    Palette {
        #[arg(long)]
        dmenu: bool,
        #[arg(short, long)]
        prompt: Option<String>,
    },
    Dmenu {
        subcmd: Option<String>,
        #[arg(short, long)]
        prompt: Option<String>,
        #[arg(long)]
        multi: bool,
    },
    Launch {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum UpdateCommands {
    Run {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Safe {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum WeightCommands {
    /// Show all patterns ranked by weight
    List,
    /// Show Critical and Strong patterns only
    Top,
    /// Scan events and compute pattern weights
    Compute,
    /// Full weight breakdown for a pattern
    Explain { id: String },
    /// Record outcome for calibration
    Calibrate { id: String, outcome: String },
}
#[derive(Debug, Clone, Subcommand)]
pub enum DaemonCommands {
    /// Current daemon health and forest context
    Status,
    /// Full forest context snapshot from daemon
    Context,
    /// Recent engine signals
    Signals {
        /// Number of signals to show
        limit: u32,
    },
    /// Neovim context for a file
    Neovim {
        /// File path being edited
        file_path: String,
    },
    /// Health watchdog status
    Watchdog,
}
#[derive(Debug, Clone, Subcommand)]
pub enum SelfCommands {
    /// Architecture coupling analysis
    Map,
    /// Generate structural proposals with confidence + risk
    Evolve,
    /// Apply a proposal (use --dry-run first)
    Apply {
        /// Proposal ID from core self evolve
        proposal_id: i64,
        /// Show what would happen without executing
        #[arg(long)]
        dry_run: bool,
        /// Create checkpoint before applying
        #[arg(long)]
        checkpoint: bool,
    },
    /// Evolution audit trail
    History,
    /// Record outcome of a proposal
    Learn {
        /// Proposal ID
        proposal_id: i64,
        /// Outcome: success or failure
        outcome: String,
    },
    /// Proposal accuracy over time
    Accuracy,
    /// Adjust proposal thresholds based on history
    Calibrate,
    /// Prove-me-wrong mode — stress test a plan
    Challenge {
        /// Intent ID to challenge (e.g. INT-189)
        intent_id: String,
    },
}
#[derive(Debug, Clone, Subcommand)]
pub enum JournalCommands {
    /// Show today's journal entries
    Today,
    /// Show yesterday's journal entries
    Yesterday,
    /// Show this week's journal entries
    Week,
    /// Search journal by keyword
    Search { term: String },
    /// Show journal for a specific date (YYYY-MM-DD)
    Show { date: String },
    /// Write a session-start entry
    SessionStart,
    /// Write a daily summary entry
    DailySummary,
}
#[derive(Debug, Clone, Subcommand)]
pub enum DocsCommands {
    /// Show the core commands guide
    Commands,
    /// List available documentation
    List,
}
#[derive(Debug, Subcommand)]
pub enum ValuesCommands {
    /// List all declared values
    List,
    /// Declare a new value
    Define {
        /// The value statement
        statement: String,
        /// Weight 1-10 (default: 7)
        #[arg(long, default_value = "7")]
        weight: i64,
        /// Scope: all | intents | commits | deploys
        #[arg(long, default_value = "all")]
        scope: String,
    },
    /// Deactivate a declared value by ID
    Remove { id: i64 },
    /// Update the weight of a declared value
    Weight { id: i64, weight: i64 },
}

#[derive(Debug, Subcommand)]
pub enum AlignCommands {
    /// Check alignment of a subject against declared values
    Check {
        /// Subject to check (e.g. intent name or description)
        subject: String,
    },
    /// Show behavioral drift report for the last 30 days
    Drift,
    /// Show weekly alignment report
    Report {
        /// Weeks ago (0 = current week)
        #[arg(long, default_value = "0")]
        weeks_ago: i64,
    },
}

#[derive(Debug, Subcommand)]
/// Engines subcommands
pub enum EnginesCommands {
    /// Show all engines and their sync state
    Status,
    /// Acknowledge engine upgrade and update contracts
    Sync {
        /// Engine name to synchronize
        engine: String,
    },
    /// Show recent cross-engine signals
    Signals,
    /// Verify all engines are consistent
    Check,
    /// Show engine upgrade history
    UpgradeLog,
    /// Process unconsumed signals and route reactions
    Process,
}

#[derive(Debug, Subcommand)]
pub enum EventsCommands {
    /// Show event log file status and size
    Status,
    /// Archive and compress old log files
    Archive,
    /// List events from today
    List,
    /// Events since a duration (e.g. 1h, 30m, 2d)
    Since { duration: String },
    /// Filter events by domain
    Filter { domain: String },
    /// Live event stream — watch events as they happen
    Watch,
    /// Emit a validated signal to forest_events_v2
    EmitV2 {
        type_name: String,
        payload: String,
        #[arg(long)]
        caused_by: Option<i64>,
    },
    /// Replay signals from sequence range
    Replay { from_seq: i64, to_seq: i64 },
    /// Show causality chain for a signal
    Chain { seq: i64 },
    /// Show forest_events_v2 status
    StatusV2,
}

#[derive(Subcommand)]
pub enum LedgerCommands {
    /// Event counts, domains, date range, and database stats
    Stats,
    /// Query events for a specific domain
    Query { domain: String },
    /// Export all events to JSON
    Export,
    /// Create database indexes for fast time-window queries
    Indexes,
}
#[derive(Subcommand)]
pub enum WhyCommands {
    /// Why did the system do what it did today?
    Summary,
    /// Why is health at its current level?
    Health,
    /// What has a specific domain been doing?
    Domain { domain: String },
    /// Visual topology — what apps and workspaces were active today?
    Visual,
    /// Attention analysis — focus quality and fragmentation?
    Attention,
    /// Health trajectory since a specific date (YYYY-MM-DD)
    HealthSince { since: String },
    /// What events in a domain preceded health changes?
    Causal { domain: String },
    /// Full causal chain for last health drop
    Chain,
    /// Correlate two domains — find patterns between them
    Correlate { domain_a: String, domain_b: String },
    /// Proactive suggestions based on current state and learned patterns
    Suggest,
    /// Visual workspace activity — what drove workspace switches?
    Workspace,
    /// Focus quality — deep focus vs fragmentation over time
    Focus,
}

#[derive(Subcommand)]
pub enum SimulateCommands {
    /// Predict health after pending changes — no writes
    Doctor,
    /// Show what packages would be updated — no writes
    Update,
    /// Simulate risk for a planned scenario using decision history
    Scenario {
        /// Description of the planned scenario
        description: String,
    },
}

#[derive(Subcommand)]
pub enum DelegateCommands {
    /// Simulate a delegation action without executing
    Simulate {
        /// Action to simulate (e.g. "restart faelight-notify")
        action: String,
    },
    /// List all trust contracts and their status
    Contracts,
    /// Show delegation simulation history
    History,
    /// Show simulation accuracy over time
    Accuracy,
    /// Suspend all delegation instantly
    Suspend,
    /// Activate a contract after gate is met
    Activate {
        /// Contract name to activate
        contract: String,
    },    /// Show counterfactual comparison log
    Counterfactuals,
    /// Log a counterfactual (proposed vs actual action)
    LogCounterfactual {
        /// What the system proposed
        proposed: String,
        /// What you actually did
        human: String,
        /// Did the actions match?
        matched: bool,
        /// Predicted confidence (0.0-1.0)
        confidence: f64,
    },
    /// Three-dimensional accuracy report
    AccuracyReport,
}
#[derive(Subcommand)]
pub enum TraceCommands {
    /// Show last 10 events with full detail
    Last,
    /// Show full trace for a specific domain
    Domain { domain: String },
}

#[derive(Subcommand)]
pub enum PluginCommands {
    /// List all registered plugins
    List,
    /// Register a plugin
    Add {
        /// Plugin name (e.g. faelight-git)
        name: String,
    },
    /// Remove a plugin from registry
    Remove {
        /// Plugin name
        name: String,
    },
    /// Show plugin status
    Status {
        /// Plugin name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum CheckpointCommands {
    /// Restore config files from a named checkpoint
    Restore {
        /// Checkpoint name
        name: String,
    },
    /// Find and restore last checkpoint with 95%+ health
    LastGood,
    /// Create a btrfs snapshot of @home
    Snapshot {
        /// Snapshot label
        label: String,
    },
    /// List btrfs snapshots
    Snapshots,
    /// Create a named checkpoint of current system state
    Create {
        /// Checkpoint name (e.g. pre-update, pre-release)
        name: String,
        /// Optional notes about this checkpoint
        #[arg(short, long)]
        notes: Option<String>,
    },
    /// List all checkpoints
    List,
    /// Show what changed since a checkpoint
    Diff {
        /// Checkpoint name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum DecisionCommands {
    /// Record a new decision with context snapshot
    Record {
        /// Description of the decision being made
        description: String,
        /// Related intent ID (e.g. INT-109)
        #[arg(short, long)]
        intent: Option<String>,
    },
    /// Record the outcome of a decision
    Outcome {
        /// Decision ID (e.g. DEC-001)
        id: String,
        /// Outcome: success, partial, failure, unknown
        result: String,
        /// Optional notes about the outcome
        #[arg(short, long)]
        notes: Option<String>,
    },
    /// List recorded decisions
    List {
        /// Show only decisions without outcomes
        #[arg(long)]
        open: bool,
    },
    /// View hindsight summary of all decisions
    Hindsight,
    /// Show full detail for a specific decision
    Show {
        /// Decision ID (e.g. DEC-001)
        id: String,
    },
    /// Correlation stats across all decisions
    Stats,
    /// Judgment advisory for current state
    Advise {
        /// Optional planned decision to evaluate
        decision: Option<String>,
    },
    /// Auto-derived heuristics from decision corpus
    Heuristics {
        /// Filter by domain
        #[arg(short, long)]
        domain: Option<String>,
    },
    /// Human-readable lessons summary
    Lessons,
    /// 30-day narrative of computing life
    Story,
    /// Detect repeating decision patterns
    Patterns,
    /// Detect decisions requiring repeated corrections
    Friction,
    /// Detect architectural reversals in decision history
    Reversal,
}

#[derive(Subcommand)]
pub enum AuditCommands {
    /// Score all tools — full intelligence report
    Scan,
    /// Deep audit of a specific tool
    Show { tool: String },
    /// Tools below health threshold
    Stale,
    /// Tools missing documentation
    Coverage,
}

#[derive(Subcommand)]
pub enum AnomalyCommands {
    /// Detect unexpected system changes
    Scan,
    /// Show anomaly detection history
    History,
    /// Surface high-severity anomalies
    Alert,
}

#[derive(Subcommand)]
pub enum BootstrapCommands {
    /// Generate reconstruction plan
    Plan,
    /// Verify current state consistency
    Verify,
    /// Show what diverged from canonical state
    Diff,
}

#[derive(Subcommand)]
pub enum DepsCommands {
    /// Visual dependency map of all forest tools
    Graph,
    /// Which dependencies carry the most risk?
    Risk,
    /// Cross-reference deps with decision history
    Audit,
}

#[derive(Debug, clap::Subcommand)]
pub enum GoalsCommands {
    /// List all active forest goals
    List,
    /// Generate new goals from current forest evidence
    Generate,
    /// Ranked goal list with reasoning
    Priority,
    /// Accept a goal — becomes an intent record
    Accept {
        /// Goal ID (e.g. GOAL-001)
        id: String,
    },
    /// Reject a goal — logged with reason
    Reject {
        /// Goal ID
        id: String,
    },
    /// Show detail for a specific goal
    Show {
        /// Goal ID
        id: String,
    },
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum StressCommands {
    /// Event storm — inject 500 events, verify no corruption
    Events,
    /// Prediction under load — all 9 predict commands
    Predict,
    /// Reaction integrity — cooldowns and history
    React,
    /// Health trajectory integrity
    Health,
    /// Intent velocity accuracy
    Intents,
    /// Full stress report — run all tests
    Report,
    /// Chaos health report — all 5 scenarios
    HealthReport,
    /// Scenario 1 — sudden health drop
    Scenario1,
    /// Scenario 2 — slow decline detection
    Scenario2,
    /// Scenario 3 — recovery verification
    Scenario3,
    /// Scenario 4 — false alarm resistance
    Scenario4,
    /// Scenario 5 — lock/unlock cycle
    Scenario5,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum PredictCommands {
    Sessions,
    Cadence,
    Health,
    Decline,
    /// Estimated completion dates for planned intents
    Intents,
    /// What intent is most likely to ship next
    Next,
    /// Coupling forecast — which domains will hit critical coupling
    Coupling,
    /// File churn prediction — highest risk files
    Churn,
    /// Prediction confidence and accuracy tracking
    Accuracy,
    /// Verify a prediction as correct or incorrect
    Verify {
        /// Prediction ID
        id: String,
        /// Was the prediction correct?
        #[arg(long)]
        correct: bool,
    },
    /// Explain why an intent is or is not predicted
    Why { id: String },
    /// Cross-session pattern analysis
    #[command(name = "cross-session")]
    CrossSession,
    /// Memory decay — prune stale state.db entries
    #[command(name = "memory-decay")]
    MemoryDecay {
        /// Actually apply the decay
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum ReactCommands {
    /// List all reaction rules and their cooldown status
    List,
    /// Show full rule registry with TOML overrides applied
    Rules,
    /// Evaluate all rules now and surface any reactions
    Run,
    /// Show log of all triggered reactions
    History,
    /// Explain why a specific reaction fired
    Explain {
        /// Reaction ID (e.g. 1, 2, 3)
        id: String,
    },
    /// Show active cooldown timers
    Discipline,
    /// Enable a reaction rule
    Enable {
        /// Rule ID (e.g. health.advisory)
        id: String,
    },
    /// Disable a reaction rule
    Disable {
        /// Rule ID (e.g. health.advisory)
        id: String,
    },
    /// Show current reaction boundary gates and health status
    Bounds,
    /// Audit all rules against current boundaries and goals
    Audit,
    /// Show today's reaction narrative as a story
    Story,
    /// Show batched coalescing signal groups
    Coalesce,
    /// Show discipline config — decay, coalesce, escalate settings
    DisciplineShow,
    /// Add a custom reaction rule
    Add {
        /// Rule ID (e.g. git.commit_streak)
        id: String,
        /// Description of what this rule detects
        description: String,
        /// Priority: 1=high 2=medium 3=low
        #[arg(default_value = "3")]
        priority: u8,
        /// Cooldown in minutes
        #[arg(default_value = "60")]
        cooldown_m: i64,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum PlanCommands {
    /// Generate a task plan for an accepted goal
    Generate {
        /// Goal ID (e.g. GOAL-001)
        goal_id: String,
    },
    /// Review a plan in full
    Review {
        /// Plan ID (e.g. PLAN-001)
        id: String,
    },
    /// Simulate plan execution — risk analysis via scenario engine
    Simulate {
        /// Plan ID (e.g. PLAN-001)
        id: String,
    },
    /// List all plans
    List,
}

#[derive(Debug, clap::Subcommand)]
pub enum TradeoffCommands {
    /// Analyze competing values for a decision
    Analyze {
        /// Decision or change to analyze (e.g. "add faelight-vault")
        decision: String,
    },
    /// Show past tradeoff analyses
    History,
    /// Current system balance across all four axes
    Balance,
}

#[derive(Debug, clap::Subcommand)]
pub enum PrioritizeCommands {
    /// Rerank all goals given current forest state
    Run,
    /// Explain why goals are ranked as they are
    Explain,
}

#[derive(Debug, clap::Subcommand)]
pub enum AutobiographyCommands {
    /// Narrate the forest's goal history
    Narrate {
        /// Filter by version (e.g. 11.1.0)
        version: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum EvolutionCommands {
    /// Architecture map — domain structure, file counts, coupling index
    Map,
    /// Tools usage analysis — roster, age, lifecycle stage
    Tools,
    /// Architecture suggestions based on coupling, churn and tool data
    Suggest,
    /// Generate a formal evolution proposal from suggestions
    EvolvePropose,
    /// List all evolution proposals
    EvolveList,
    /// Accept a proposal — creates an intent record
    EvolveAccept {
        /// Proposal ID (e.g. PROP-001)
        id: String,
    },
    /// Reject a proposal — logged with reason
    EvolveReject {
        /// Proposal ID (e.g. PROP-001)
        id: String,
    },
    /// Simulate an architectural change — what would break?
    FutureSim {
        /// Description of the change to simulate
        change: String,
    },
    /// Risk analysis for a proposed architectural change
    FutureRisk {
        /// Description of the change
        change: String,
    },
    /// Impact analysis — which domains/tools are affected?
    FutureImpact {
        /// Description of the change
        change: String,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum StrategyCommands {
    /// What needs attention this session?
    Now,
    /// What should the next 7 days focus on?
    Week,
    /// The 90-day arc toward Jarvis
    Quarter,
    /// Optimal path to achieve a goal
    Sequence {
        /// Goal ID (e.g. GOAL-001)
        goal_id: String,
    },
    /// What is blocking the most progress?
    Unblock,
    /// What do we give up to do this action now?
    Tradeoff {
        /// The action to analyze
        action: String,
    },
    /// Which intents are pulling in opposite directions?
    Conflicts,
    /// Is the current work plan internally consistent?
    Coherence,
    /// Can these two goals be pursued together?
    Merge {
        /// First goal ID
        goal1: String,
        /// Second goal ID
        goal2: String,
    },
    /// How close is the forest to Jarvis-level capability?
    Jarvis,
    /// What evidence would justify more autonomy?
    Trust,
    /// What capabilities are missing for full Jarvis?
    Gap,
    /// Past strategies and did they help?
    History,
    /// Record outcome of a strategy
    Learn {
        /// Strategy ID or description
        strategy_id: String,
        /// Outcome: yes/no/worked/failed
        outcome: String,
    },
    /// What worked, what didn't?
    Review,
    /// What should I work on next? (INT-181)
    Next {
        /// List all ranked intents instead of top recommendation
        #[arg(long)]
        list: bool,
        /// Explain why a specific intent is ranked where it is
        #[arg(long)]
        why: Option<String>,
    },
    /// Ordered work queue for next 5 sessions (INT-181)
    Queue,
    /// What is blocking the most planned intents? (INT-181)
    Blockers,
}

#[derive(Debug, clap::Subcommand)]
pub enum GenealogyCommands {
    /// Show lineage of a specific intent
    Show {
        /// Intent ID (e.g. 148)
        id: String,
    },
    /// Show full intent family tree
    Tree,
    /// Show founding intents with no ancestors
    Roots,
}

#[derive(Debug, clap::Subcommand)]
pub enum IntegrityCommands {
    /// Full integrity scan with repair options
    Run,
    /// Show current integrity status and pending proposals
    Status,
    /// Show history of integrity issues
    Log,
    /// Apply pending proposals
    Fix,
    /// Apply a specific pending proposal by ID
    Apply { id: String },
    /// Auto-heal all safe integrity issues
    Heal {
        #[arg(long)]
        dry_run: bool,
    },
    /// Show integrity score trend over time
    Trend,
}
#[derive(Debug, clap::Subcommand)]
pub enum AutonomyCommands {
    /// List active mandates
    MandateList,
    /// Define a new mandate
    MandateSet { rule: String },
    /// Revoke a mandate by ID
    MandateRevoke { id: String },
    /// Revoke all mandates — return to manual mode
    MandateRevokeAll,
    /// Show pending autonomous actions
    Pending,
    /// Execute pending authorized actions
    Run,
    /// Show autonomy action log
    Log,
    /// Revert last autonomous action
    Revert,
    /// Show trust score
    TrustScore,
    /// Show trust history
    TrustHistory,
    /// Request expanded autonomy
    TrustExpand,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum PartnerCommands {
    /// Forest proposes a new intent based on observed patterns
    Propose,
    /// Forest shares opinion on an existing intent
    Discuss {
        /// Intent ID to discuss
        intent_id: String,
    },
    /// Forest respectfully pushes back on an intent
    Disagree {
        /// Intent ID to push back on
        intent_id: String,
    },
    /// Consult the forest before making a decision
    Consult {
        /// Question to ask the forest
        question: String,
    },
    /// What has the forest learned about your work style?
    Reflect,
    /// What patterns define how you work?
    Pattern,
    /// How has the system grown over time?
    Growth,
    /// Show recent pushback moments
    Pushback,
    /// Forest view of the optimal path forward
    Roadmap,
    /// Why does the forest recommend this roadmap?
    RoadmapWhy,
    /// How does forest roadmap differ from current plan?
    RoadmapDiff,
    /// Partner system status and readiness
    Status,
}
#[derive(Debug, clap::Subcommand)]
pub enum RegistryCommands {
    /// List all tools in the registry
    List,
    /// Show details for a specific tool
    Show {
        /// Tool name
        name: String,
    },
    /// Retire a tool — mark as retired, excluded from deploy and doctor
    Retire {
        /// Tool name
        name: String,
    },
    /// Unretire a tool — restore to active status
    Unretire {
        /// Tool name
        name: String,
    },
    /// Compare actual tool usage vs expected_usage
    RealityCheck,
}
#[derive(Debug, clap::Subcommand)]
pub enum DbCommands {
    /// Manual snapshot to timestamped file
    Backup,
    /// Restore from a backup snapshot
    Restore {
        /// Filename or path to restore from
        file: String,
    },
    /// Run integrity check
    Verify,
    /// Show db size, table counts, last backup
    Status,
    /// VACUUM to reclaim space
    Compact,
}
