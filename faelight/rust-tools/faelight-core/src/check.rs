//! Whether a check could reach an answer at all.
//!
//! INT-192. A check that could not run must not report a result. Vec::new() on an error
//! path makes "nothing found" and "could not look" THE SAME VALUE, and every
//! consumer downstream reads the first meaning.
//!
//! PROVEN LIVE 2026-09-04: config.nsh was moved aside and faelight-deadwood reported
//! Dead aliases: clean, 0|0|0|0. It could not read the file and said clean. That number
//! feeds the doctor health score, so a missing config reads as a healthy system.
//!
//! NOT IN error.rs, DELIBERATELY. An error is a failure; this is an ABSENCE OF
//! KNOWLEDGE, and the whole point of the intent is that those are different. Filing it
//! beside FontLoad and GlyphRasterize would encode the confusion this exists to remove.
//! The doctor already keeps Unknown distinct from Fail for the same reason (INT-148).

use std::fmt;

/// Why a check could not produce an answer. The reason is CARRIED, not discarded --
/// Err(_) throws away the one thing that makes the report actionable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    /// What could not be reached: a path, a command, a table.
    pub subject: String,
    /// Why, in the words of whatever failed.
    pub reason: String,
}

impl Skipped {
    pub fn new(subject: impl Into<String>, reason: impl fmt::Display) -> Self {
        Self {
            subject: subject.into(),
            reason: reason.to_string(),
        }
    }
}

impl fmt::Display for Skipped {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not check {}: {}", self.subject, self.reason)
    }
}

/// A check outcome. The type is the enforcement: a caller cannot read the value without
/// handling the case where there is none, so the collapse this intent is about becomes a
/// COMPILE ERROR rather than a discipline that drifts across 28 tools.
///
/// Same move as IoPlan being a reserved type rather than fake fields, and lower()
/// returning Result rather than fabricating a plan: make the wrong state unrepresentable.
pub type Checked<T> = std::result::Result<T, Skipped>;
