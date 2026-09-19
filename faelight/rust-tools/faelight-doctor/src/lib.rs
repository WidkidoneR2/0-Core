//! The doctor engine. INT-222 Phase 2 step 1.
//!
//! THE RUNNER IS CODE, THE CHECKS ARE DATA. Updating the check set does not rebuild this.
//!
//! ⚠️ THIS CRATE OWNS NO CHECKS. `core doctor` checks the SYSTEM and `nsh doctor` checks the
//! SHELL; they want different tiers and different subjects, and each brings its own registry.
//! What is shared is the FORMAT and the RULES -- because both doctors were wrong at a similar
//! rate the first time anyone read them (27 of 34, and 3 of 7), and the defects were structural
//! rather than domain-specific.
//!
//! ★ IT LIVES OUTSIDE `core` BY NECESSITY, NOT TIDINESS. novashell has no dependency edge to
//! `core`, and giving it one would pull in ~57 domains including Intelligence -- the exact edge
//! INT-223 exists to remove and decision 147 built its ownership model around not creating.

pub mod definition;
pub mod measurement;
pub mod probe;
pub mod probes;
pub mod registry;
pub mod render;
pub mod run;
pub mod status;

pub use definition::{Definition, DefinitionError, Severity};
pub use measurement::Measurement;
pub use probe::Probe;
pub use registry::{LoadError, Registry};
pub use render::{is_red, render};
pub use run::{run_all, run_one, run_quick, Summary};
pub use status::{verdict, Status, Tier, Verdict};
