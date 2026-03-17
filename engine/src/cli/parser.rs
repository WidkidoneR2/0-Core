use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "core")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "0-Core v2 — single orchestrator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
    /// Dry-run simulation — predict outcomes without changing anything
    Simulate {
        #[command(subcommand)]
        command: SimulateCommands,
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
    Focus { id: String },
    /// Clear focused intent
    Unfocus,
    /// Show current focus and workflow state
    Status,
    /// Detect if work has drifted from focused intent
    Drift,
    /// Transition intent to in-progress and set focus
    Start { id: String },
    /// Mark intent as complete and move to complete/
    Complete { id: String },
    /// Create a new intent from template
    New {
        /// Template type: feature, fix, arch, study
        template: String,
        /// Intent title
        title: String,
    },
    /// Show dependency graph for an intent
    Deps { id: String },
    /// Show completion burndown chart
    Burndown,
    /// Show velocity metrics
    Velocity,
    /// Generate git branch name for intent
    Branch { id: String },
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

#[derive(Subcommand)]
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
