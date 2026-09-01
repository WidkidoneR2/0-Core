//! INT-191: the history domain -- what observations the shell persists about command lifecycles,
//! and what can be PROVEN from them.
//!
//! Deliberately a subsystem rather than one command implementation. `spine` did not stay "the
//! parser"; it became the namespace for the parser's domain. This is the same: `history` owns the
//! vocabulary, the classifier, and later the migration and verification, while the command layer
//! owns CLI shape and formatting.
//!
//! Dependency direction is one-way and acyclic: model <- classifier <- audit / migrate / verify.
//! Nothing here returns a `CommandResult` -- the history domain must not depend on a commands-layer
//! type.

pub mod classifier;
pub mod model;
