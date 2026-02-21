#[derive(Debug)]
pub enum Command {
    Version,
    Doctor { preflight: bool },
}
