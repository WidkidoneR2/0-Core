//! What a probe returns, and the only thing it is allowed to decide.
//!
//! ⭐ A PROBE RETURNS A MEASUREMENT, NOT A RESULT. It reports what it found and how bad that is.
//! It does NOT know its own id, name, tier or recovery text -- those are DECLARED, in TOML, and
//! the engine supplies them.
//!
//! That division is the whole of INT-222. Every defect the intent was filed against lived in a
//! hand-written `CheckResult` literal where the tier and the status were re-decided together,
//! at every construction site, with nothing checking the answer:
//!
//! ```text
//! check_dotmeta     status: Status::Pass, typed, unconditional
//! check_vm_state    tier: Tier::Info beside status: Status::Warn
//! ```
//!
//! A probe cannot write either of those, because a probe cannot write a tier at all.

use crate::status::Status;

/// What a probe found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measurement {
    /// How bad it is. The engine REFUSES this if the definition's declared range forbids it.
    pub status: Status,
    /// What to say, ALREADY FORMATTED.
    ///
    /// The probe owns the wording because only the probe knows the units: that bytes should
    /// render as GB, that a kernel version is a string, that "3/5 services running" is the
    /// useful shape. An engine formatting these would need to know each measurement's meaning,
    /// which is exactly the knowledge that belongs in the probe.
    pub message: String,
}

impl Measurement {
    pub fn pass(message: impl Into<String>) -> Self {
        Measurement {
            status: Status::Pass,
            message: message.into(),
        }
    }
    pub fn warn(message: impl Into<String>) -> Self {
        Measurement {
            status: Status::Warn,
            message: message.into(),
        }
    }
    pub fn fail(message: impl Into<String>) -> Self {
        Measurement {
            status: Status::Fail,
            message: message.into(),
        }
    }
    /// ⚠️ THE CHECK COULD NOT RUN. Not a failure, and never a quiet pass.
    ///
    /// Use this wherever a measurement was attempted and could not be taken. `check_vm_state`
    /// reaches for `warn` here and lands in a tier where warn is inert; `check_package_cache`
    /// gets it right. The engine ALWAYS permits unknown, whatever the declared range says,
    /// because "could not run" is not a severity the check chose to report.
    pub fn unknown(message: impl Into<String>) -> Self {
        Measurement {
            status: Status::Unknown,
            message: message.into(),
        }
    }
    /// Deliberately not run: a precondition was absent by design, not by failure.
    pub fn blocked(message: impl Into<String>) -> Self {
        Measurement {
            status: Status::Blocked,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_measurement_carries_no_tier_and_no_id() {
        // ⭐ THE POINT, AS A TEST. There is no field here for a tier, an id, a name or a
        // recovery string -- so `check_vm_state`'s defect (info tier beside a warn status)
        // cannot be expressed by a probe at all. It needs two things a probe cannot touch.
        let m = Measurement::warn("something is off");
        assert_eq!(m.status, Status::Warn);
        assert_eq!(m.message, "something is off");
    }

    #[test]
    fn unknown_says_it_could_not_run() {
        let m = Measurement::unknown("could not read /var/cache/pacman/pkg");
        assert_eq!(m.status, Status::Unknown);
    }
}
