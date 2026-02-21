#[derive(Debug)]
pub enum Command {
    Version,
    Doctor {
        preflight: bool,
    },
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
