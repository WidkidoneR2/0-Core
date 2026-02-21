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
