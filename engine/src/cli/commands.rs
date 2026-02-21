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
}

#[derive(Debug)]
pub enum LinkCommand {
    Status { json: bool },
    List,
    Audit,
}
