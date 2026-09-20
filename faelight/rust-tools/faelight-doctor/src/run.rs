//! Running a check set: definition + probe -> outcome.
//!
//! ⭐ THIS IS WHERE THE DECLARATION BECOMES A GATE RATHER THAN A DESCRIPTION.
//!
//! The probe says what it found. The definition says what this check is allowed to report. If
//! they disagree, THE ENGINE SAYS SO OUT LOUD -- it does not quietly pass the probe's answer
//! through, and it does not quietly downgrade it.

use crate::definition::Definition;
use crate::measurement::Measurement;
use crate::probes;
use crate::registry::Registry;
use crate::status::{Outcome, Status, Tier};

/// Run every definition in a registry.
pub fn run_all(reg: &Registry) -> Vec<Outcome> {
    reg.checks.iter().map(run_one).collect()
}

/// ⭐ QUICK SCAN IS DERIVED, NOT COPIED.
///
/// ⚠️ THE DEFECT THIS REPLACES WAS A SECOND HARDCODED LIST. The old quick scan named six checks
/// by hand beside a full scan that ran a different set, and they drifted: the short list still
/// named scripts-executable while boot errors and disk space were absent entirely. Deleting a
/// check meant editing two lists, AND ONLY THE COMPILER NOTICED THE SECOND.
///
/// Here the quick scan is the same registry filtered by the tier each definition already
/// declares. There is no second list to drift from, and adding a critical check puts it in the
/// quick scan by itself -- the same move as asking systemd which services it wants.
pub fn run_quick(reg: &Registry) -> Vec<Outcome> {
    reg.checks
        .iter()
        .filter(|d| d.tier == Tier::Critical)
        .map(run_one)
        .collect()
}

/// Run one definition.
pub fn run_one(d: &Definition) -> Outcome {
    let m = match d.probe {
        Some(p) => probes::run(p),
        // A definition with an assertion and no probe: the engine evaluates simple conditions
        // itself. Not yet implemented, and it says so rather than passing.
        None => Measurement::unknown("assertion-only definitions are not evaluated yet"),
    };
    enforce(d, m)
}

/// ⚠️ THE DECLARED RANGE IS A CONSTRAINT, AND THIS IS WHERE IT BITES.
///
/// A probe returning a severity its definition never declared is a BUG IN THE PAIR -- either the
/// probe learned to report something new, or the declaration is stale. Both are worth knowing,
/// and neither is worth hiding.
///
/// The out-of-range status becomes `Unknown` with a message naming the violation. NOT the
/// probe's status (which would make the declaration decorative) and NOT `Fail` (which would
/// report a system problem where there is a bookkeeping problem).
fn enforce(d: &Definition, m: Measurement) -> Outcome {
    if d.permits(m.status) {
        return Outcome {
            section: d.section.clone(),
            id: d.id.clone(),
            name: d.name.clone(),
            tier: d.tier,
            status: m.status,
            message: m.message,
            recovery: d.recovery.clone(),
        };
    }
    Outcome {
        section: d.section.clone(),
        id: d.id.clone(),
        name: d.name.clone(),
        tier: d.tier,
        status: Status::Unknown,
        message: format!(
            "{} reported {:?}, which its definition does not declare (declared: {:?}). \
             The probe and the declaration disagree -- one of them is wrong. It said: {}",
            d.id, m.status, d.severities, m.message
        ),
        recovery: Some(format!(
            "Either add {:?} to the severities of {} in the check set, or stop the probe \
             reporting it.",
            m.status, d.id
        )),
    }
}

/// ⭐ THE OUTPUT STATES ITS BASIS. Anti-virus never reports "97% healthy"; it reports items
/// scanned, threats found, action taken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub measured: usize,
    pub declared: usize,
    pub passing: usize,
    pub warning: usize,
    pub failing: usize,
    pub unknown: usize,
    pub critical_failing: usize,
}

impl Summary {
    pub fn of(outcomes: &[Outcome], labels: usize) -> Summary {
        let judging: Vec<&Outcome> = outcomes.iter().filter(|o| o.tier != Tier::Info).collect();
        Summary {
            measured: judging.len(),
            declared: labels,
            passing: judging.iter().filter(|o| o.status == Status::Pass).count(),
            warning: judging.iter().filter(|o| o.status == Status::Warn).count(),
            failing: judging.iter().filter(|o| o.status == Status::Fail).count(),
            unknown: judging
                .iter()
                .filter(|o| matches!(o.status, Status::Unknown | Status::Blocked))
                .count(),
            critical_failing: judging
                .iter()
                .filter(|o| o.tier == Tier::Critical && o.status == Status::Fail)
                .count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::Severity;
    use crate::probe::Probe;

    fn def(id: &str, tier: Tier, sev: Vec<Severity>) -> Definition {
        Definition {
            section: "Test".into(),
            id: id.into(),
            name: id.into(),
            tier,
            severities: sev,
            assertion: None,
            probe: Some(Probe::PackageCache),
            threshold_source: None,
            recovery: None,
        }
    }

    #[test]
    fn the_quick_scan_is_the_critical_tier_and_nothing_else() {
        // ⭐ DERIVED, NOT COPIED. Every outcome the quick scan produces must be one the
        // registry declared critical -- there is no second list it could disagree with.
        let text =
            std::fs::read_to_string("/home/christian/0-core/faelight/registry/doctor/checks.toml")
                .expect("the check set is readable");
        let reg = Registry::parse(&text, "checks.toml").expect("it parses");
        let quick = run_quick(&reg);
        assert!(
            !quick.is_empty(),
            "a quick scan of nothing is not a quick scan"
        );
        for o in &quick {
            assert_eq!(o.tier, Tier::Critical, "{} is not critical", o.id);
        }
        // And it is a strict subset: the full run covers everything the quick one did.
        assert!(quick.len() < reg.checks.len());
    }

    #[test]
    fn an_out_of_range_status_becomes_unknown_and_says_why() {
        // ⭐ A pass-only definition, handed a warn. The engine must not pass it through.
        let d = def("thing", Tier::User, vec![Severity::Pass]);
        let o = enforce(&d, Measurement::warn("something happened"));
        assert_eq!(
            o.status,
            Status::Unknown,
            "an undeclared severity is not reported as-is"
        );
        assert!(o.message.contains("does not declare"));
        assert!(
            o.message.contains("something happened"),
            "the probe's finding is not lost"
        );
        assert!(
            o.recovery.is_some(),
            "and it says how to resolve the disagreement"
        );
    }

    #[test]
    fn a_declared_status_passes_through_with_the_declarations_attached() {
        let mut d = def("thing", Tier::System, vec![Severity::Pass, Severity::Warn]);
        d.recovery = Some("do the thing".into());
        let o = enforce(&d, Measurement::warn("a bit off"));
        assert_eq!(o.status, Status::Warn);
        assert_eq!(o.message, "a bit off");
        assert_eq!(o.tier, Tier::System);
        assert_eq!(o.recovery.as_deref(), Some("do the thing"));
    }

    #[test]
    fn unknown_is_always_allowed_through() {
        let d = def("thing", Tier::Info, vec![Severity::Pass]);
        let o = enforce(&d, Measurement::unknown("could not look"));
        assert_eq!(o.status, Status::Unknown);
        assert_eq!(
            o.message, "could not look",
            "no violation message -- unknown is permitted"
        );
    }

    #[test]
    fn the_summary_excludes_labels_from_the_denominator() {
        let outcomes = vec![
            Outcome {
                section: "Test".into(),
                id: "a".into(),
                name: "A".into(),
                tier: Tier::System,
                status: Status::Pass,
                message: String::new(),
                recovery: None,
            },
            Outcome {
                section: "Test".into(),
                id: "b".into(),
                name: "B".into(),
                tier: Tier::Info,
                status: Status::Pass,
                message: String::new(),
                recovery: None,
            },
        ];
        let s = Summary::of(&outcomes, 1);
        assert_eq!(s.measured, 1, "the info-tier outcome is not measured");
        assert_eq!(s.declared, 1);
    }
}
