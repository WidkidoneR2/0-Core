#[derive(Debug)]
pub enum Command {
    Version,
    Doctor(DoctorCommand),
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
    Capabilities {
        json: bool,
        domain: Option<String>,
    },
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
}

#[derive(Debug)]
pub enum LinkCommand {
    Status { json: bool },
    List,
    Audit,
}

#[derive(Debug)]
pub enum IntentCommand {
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
    Report { all: bool },
    Show { id: String },
    History,
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
