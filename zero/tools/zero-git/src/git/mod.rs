//! Git operations module

pub mod repo;

pub use repo::{CommitEntry, FileEntry, FileState, GitRepo, WorkingTreeStatus};
