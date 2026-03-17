#[derive(Debug)]
pub enum Command {
    Anomaly(AnomalyCommand),
    Bootstrap(BootstrapCommand),
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
    Events(EventsCommand),
    Ledger(LedgerCommand),
    Why(WhyCommand),
    Trace(TraceCommand),
    Simulate(SimulateCommand),
    Checkpoint(CheckpointCommand),
    Capabilities {
        json: bool,
        domain: Option<String>,
    },
    Decision(DecisionCommand),
}

#[derive(Debug)]
pub enum DecisionCommand {
    Record { description: String, intent: Option<String> },
    Outcome { id: String, result: String, notes: Option<String> },
    List { open: bool },
    Hindsight,
    Show { id: String },
    Stats,
    Advise { decision: Option<String> },
    Heuristics { domain: Option<String> },
    Lessons,
    Story,
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
    Focus { id: String },
    Unfocus,
    FocusStatus,
    Drift,
    Start { id: String },
    Complete { id: String },
    New { template: String, title: String },
    Deps { id: String },
    Burndown,
    Velocity,
    Branch { id: String },
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
pub enum EventsCommand {
    Status,
    Archive,
    List,
    Since { duration: String },
    Filter { domain: String },
    Watch,
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
