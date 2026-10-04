use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum CoreError {
    #[error("Domain error [{domain}]: {message}")]
    Domain { domain: String, message: String },
    #[error("Capability denied: {0}")]
    CapabilityDenied(String),
    #[error("Registry error: {0}")]
    Registry(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("{0}")]
    StateDb(#[from] zero_core::state_db::StateDbError),
    /// INT-272: the code that refused has already told the reader, in full. The top level keeps
    /// the non-zero exit and the event, and prints nothing more.
    #[error("{0}")]
    Reported(String),
}

pub type CoreResult<T> = Result<T, CoreError>;

/// INT-272: the line the top level prints for an error, or None when the error was already
/// printed where it happened. One owner, so a refusal cannot reach the reader twice.
pub fn top_level_line(e: &CoreError) -> Option<String> {
    match e {
        CoreError::Reported(_) => None,
        other => Some(other.to_string()),
    }
}

#[cfg(test)]
mod top_level_line_tests {
    use super::{top_level_line, CoreError};

    // INT-272: the class. A Reported error is silent at the top level; every other variant is
    // still printed, with its wording unchanged.
    #[test]
    fn a_reported_error_is_not_printed_twice() {
        assert_eq!(top_level_line(&CoreError::Reported("r".into())), None);
        let printed = [
            CoreError::Runtime("x".into()),
            CoreError::Registry("x".into()),
            CoreError::CapabilityDenied("x".into()),
            CoreError::Io(std::io::Error::other("x")),
        ];
        for e in printed {
            assert!(top_level_line(&e).is_some(), "{:?}", e);
        }
        let line = top_level_line(&CoreError::Runtime("x".into()));
        assert_eq!(line.as_deref(), Some("Runtime error: x"));
    }
}
