#[derive(Debug)]
pub enum Command {
    Version,
    Doctor { preflight: bool },
    Link(LinkCommand),
}

#[derive(Debug)]
pub enum LinkCommand {
    Status { json: bool },
    List,
    Audit,
}
