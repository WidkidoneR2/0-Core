//! The definition format, and the rules the engine ENFORCES on it.
//!
//! A definition is DATA. It declares what is checked, how much it matters, what severities it
//! may produce, and what to do when it is red. It does not contain code and cannot reach one.

use crate::probe::Probe;
use crate::status::{Status, Tier};
use serde::Deserialize;

/// A severity a definition is ALLOWED to produce.
///
/// ⭐ THIS IS THE KEY STRUCTURAL IDEA. A definition declaring only `Pass` IS A LABEL, BY
/// CONSTRUCTION -- it announces itself in the format instead of being found by hand. The
/// label/lie distinction becomes mechanical rather than editorial, and `check_dotmeta` could
/// not have hidden.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Pass,
    Warn,
    Fail,
}

impl Severity {
    fn permits(&self, s: Status) -> bool {
        matches!(
            (self, s),
            (Severity::Pass, Status::Pass)
                | (Severity::Warn, Status::Warn)
                | (Severity::Fail, Status::Fail)
        )
    }
}

/// One check, as data.
#[derive(Debug, Clone, Deserialize)]
pub struct Definition {
    pub id: String,
    pub name: String,
    pub tier: Tier,
    /// Which panel section this check appears under.
    ///
    /// REQUIRED, and that is the point. The cockpit used to hold EIGHT HARDCODED NAME LISTS
    /// deciding what was SEEN, while the registry decided what RAN, and they drifted in both
    /// directions: check_hooks ran for weeks while invisible, four names survived checks that
    /// had been deleted, and Zero Alias fell to Uncategorised from the day it was written.
    /// A check that declares its own section cannot be unclaimed, and a section that claims
    /// no check cannot exist.
    pub section: String,
    /// Which severities this check may produce. NEVER EMPTY -- see `validate`.
    pub severities: Vec<Severity>,
    /// A plain condition the engine evaluates itself. Mutually exclusive with `probe`.
    pub assertion: Option<String>,
    /// A named question from the closed registry. Mutually exclusive with `assertion`.
    pub probe: Option<Probe>,
    /// Where a threshold comes from. DERIVED, NEVER TYPED -- a typed threshold goes stale
    /// exactly the way the check count did (22 in one doc, 14 in another, 34 in the code).
    pub threshold_source: Option<String>,
    /// What to do when this is red. INT-199 shape.
    pub recovery: Option<String>,
}

/// Why a definition was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum DefinitionError {
    /// ⚠️ GATE 4. A definition with no assertion and no probe measures NOTHING, and a check
    /// that measures nothing is the defect this whole intent is named for. It is refused, not
    /// silently passed.
    NoAssertionNoProbe { id: String },
    /// Both is ambiguous: two sources of truth for one answer.
    BothAssertionAndProbe { id: String },
    /// A definition that declares no severity can produce no verdict.
    NoSeverities { id: String },
    /// An id or name nobody can act on.
    Empty { field: &'static str },
    /// ⚠️ `info` means MEASURES TRULY BUT NEVER JUDGES, and the verdict excludes it entirely.
    /// A definition in that tier declaring `warn` or `fail` is a contradiction: the severity
    /// could never reach the verdict, so it would render a colour and change nothing.
    ///
    /// ⭐ THIS IS `check_vm_state`'s SHAPE, MADE UNREPRESENTABLE. It returns Tier::Info with
    /// Status::Warn to say "could not check for running VMs" -- in a tier where Warn is inert,
    /// when `Unknown` exists for exactly that and IS always permitted.
    JudgingInfoTier {
        id: String,
        offending: Vec<Severity>,
    },
}

impl std::fmt::Display for DefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefinitionError::NoAssertionNoProbe { id } => write!(
                f,
                "definition {}: declares neither an assertion nor a probe, so it measures nothing",
                id
            ),
            DefinitionError::BothAssertionAndProbe { id } => write!(
                f,
                "definition {}: declares BOTH an assertion and a probe -- one answer, two sources",
                id
            ),
            DefinitionError::NoSeverities { id } => {
                write!(
                    f,
                    "definition {}: declares no severities, so it can never report",
                    id
                )
            }
            DefinitionError::Empty { field } => write!(f, "definition: {} is empty", field),
            DefinitionError::JudgingInfoTier { id, offending } => write!(
                f,
                "definition {}: tier is info -- which never judges -- but it declares {:?}. \
                 An info check that warns is a colour nothing can act on; use unknown, which is \
                 always permitted, or give it a tier whose verdict counts.",
                id, offending
            ),
        }
    }
}

impl Definition {
    /// ⚠️ THE GATE. A definition that cannot measure is REFUSED HERE, at load, by name.
    ///
    /// This is what makes TOML acceptable. A malformed definition would otherwise be a runtime
    /// surprise where a Rust array was a compile error; refusing it loudly, by id, is the trade.
    pub fn validate(&self) -> Result<(), DefinitionError> {
        if self.id.trim().is_empty() {
            return Err(DefinitionError::Empty { field: "id" });
        }
        if self.name.trim().is_empty() {
            return Err(DefinitionError::Empty { field: "name" });
        }
        if self.severities.is_empty() {
            return Err(DefinitionError::NoSeverities {
                id: self.id.clone(),
            });
        }
        if self.tier == Tier::Info {
            let offending: Vec<Severity> = self
                .severities
                .iter()
                .copied()
                .filter(|s| *s != Severity::Pass)
                .collect();
            if !offending.is_empty() {
                return Err(DefinitionError::JudgingInfoTier {
                    id: self.id.clone(),
                    offending,
                });
            }
        }
        match (&self.assertion, &self.probe) {
            (None, None) => Err(DefinitionError::NoAssertionNoProbe {
                id: self.id.clone(),
            }),
            (Some(_), Some(_)) => Err(DefinitionError::BothAssertionAndProbe {
                id: self.id.clone(),
            }),
            _ => Ok(()),
        }
    }

    /// ⚠️ GATE 3. A definition declaring ONLY `Pass` is a LABEL: it measures truly and never
    /// judges, so it is excluded from the denominator and reported as declared, not measured.
    pub fn is_label(&self) -> bool {
        !self.severities.is_empty() && self.severities.iter().all(|s| *s == Severity::Pass)
    }

    /// Can this definition legitimately report `s`?
    ///
    /// ⭐ THE CONSTRAINT `Tier::Info` NEVER HAD. `check_vm_state` returns Info + Warn today --
    /// a warning the verdict filters out, saying "could not check" in the one state that cannot
    /// be heard. Under this format that shape is REFUSED rather than merely discouraged.
    ///
    /// Unknown and Blocked are always permitted: they say the check did not run, which is not a
    /// severity it chose to report.
    pub fn permits(&self, s: Status) -> bool {
        match s {
            Status::Unknown | Status::Blocked => true,
            other => self.severities.iter().any(|sev| sev.permits(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Definition {
        Definition {
            section: "Test".into(),
            id: "example".into(),
            name: "Example".into(),
            tier: Tier::System,
            severities: vec![Severity::Pass, Severity::Fail],
            assertion: Some("something is true".into()),
            probe: None,
            threshold_source: None,
            recovery: None,
        }
    }

    // ── GATE 4: a definition that measures nothing is REFUSED ────────────────────────────

    #[test]
    fn no_assertion_and_no_probe_is_refused() {
        let mut d = base();
        d.assertion = None;
        d.probe = None;
        assert_eq!(
            d.validate(),
            Err(DefinitionError::NoAssertionNoProbe {
                id: "example".into()
            })
        );
    }

    #[test]
    fn completing_it_makes_it_accepted() {
        // The other half of "proven by watching it fail": watch it PASS once completed.
        let mut d = base();
        d.assertion = None;
        d.probe = None;
        assert!(d.validate().is_err());
        d.probe = Some(Probe::DiskSpace);
        assert!(d.validate().is_ok());
    }

    #[test]
    fn both_assertion_and_probe_is_refused() {
        let mut d = base();
        d.probe = Some(Probe::BootTime);
        assert_eq!(
            d.validate(),
            Err(DefinitionError::BothAssertionAndProbe {
                id: "example".into()
            })
        );
    }

    #[test]
    fn no_severities_is_refused() {
        let mut d = base();
        d.severities = vec![];
        assert_eq!(
            d.validate(),
            Err(DefinitionError::NoSeverities {
                id: "example".into()
            })
        );
    }

    // ── GATE 3: pass-only is a LABEL, and the range is a CONSTRAINT ──────────────────────

    #[test]
    fn an_info_tier_definition_that_judges_is_refused() {
        // ⭐ check_vm_state's ACTUAL shape: info tier, declaring warn.
        let mut d = base();
        d.tier = Tier::Info;
        d.severities = vec![Severity::Pass, Severity::Warn];
        assert_eq!(
            d.validate(),
            Err(DefinitionError::JudgingInfoTier {
                id: "example".into(),
                offending: vec![Severity::Warn],
            })
        );
    }

    #[test]
    fn an_info_tier_label_is_accepted() {
        // check_package_cache's shape -- the same tier, done correctly.
        let mut d = base();
        d.tier = Tier::Info;
        d.severities = vec![Severity::Pass];
        assert!(d.validate().is_ok());
        assert!(d.is_label());
        assert!(
            d.permits(Status::Unknown),
            "a label may still be unable to run"
        );
    }

    #[test]
    fn pass_only_is_a_label() {
        let mut d = base();
        d.severities = vec![Severity::Pass];
        assert!(d.is_label(), "a definition that can only pass is a label");
    }

    #[test]
    fn anything_that_can_fail_is_not_a_label() {
        assert!(!base().is_label());
    }

    #[test]
    fn a_label_cannot_report_warn_or_fail() {
        // ⭐ THE check_vm_state SHAPE, MADE IMPOSSIBLE.
        // That check returns Tier::Info with Status::Warn -- a warning verdict() filters out.
        // Here the declared range REFUSES it.
        let mut d = base();
        d.severities = vec![Severity::Pass];
        assert!(d.permits(Status::Pass));
        assert!(
            !d.permits(Status::Warn),
            "a pass-only definition must not warn"
        );
        assert!(
            !d.permits(Status::Fail),
            "a pass-only definition must not fail"
        );
    }

    #[test]
    fn unknown_is_always_permitted() {
        // "could not run" is not a severity the check chose. Even a label may be Unknown --
        // that is the state INT-192 exists for, and forbidding it would recreate the silence.
        let mut d = base();
        d.severities = vec![Severity::Pass];
        assert!(d.permits(Status::Unknown));
        assert!(d.permits(Status::Blocked));
    }

    #[test]
    fn a_warn_only_definition_cannot_fail() {
        let mut d = base();
        d.severities = vec![Severity::Pass, Severity::Warn];
        assert!(d.permits(Status::Warn));
        assert!(
            !d.permits(Status::Fail),
            "a dirty tree is never an emergency"
        );
    }

    // ── the probe registry is finite and its names are stable ────────────────────────────

    #[test]
    fn every_probe_has_a_stable_name_and_the_registry_is_closed() {
        let all = Probe::all();
        assert_eq!(
            all.len(),
            27,
            "one probe per measurement -- 27 checks in all_checks(), 27 probes",
        );
        let mut seen = std::collections::HashSet::new();
        for p in all {
            assert!(!p.as_str().is_empty());
            assert!(
                seen.insert(p.as_str()),
                "probe names must be unique: {}",
                p.as_str()
            );
        }
    }
}
