//! INT-169 spine: the execution-semantics comparison engine. See docs/rfc-169-parser-spine.md.
//!
//! Increment 10 Phase 1. Answers ONE question about two ExecutionPlans: "would these execute
//! the same way?" -- NOT "are they byte-equal". Pure and reusable (no I/O, no db); the migration
//! audit (Phase 2) is just one consumer.
//!
//! ★ SEMANTIC, not structural (Christian): compare what the EXECUTOR OBSERVES, not the fields'
//! structure. `cwd: None` (inherit current) and `cwd: Some(current_dir)` describe the SAME
//! execution intent in the same process context -- equivalent, not a difference. Structural
//! PartialEq would flag a false difference on every command.
//!
//! ★ SOURCE IS EVIDENCE, not input to recompute (Christian): an ExecutionPlan has intentionally
//! thrown away syntax (it is the executor contract, not a record of how it was produced). From
//! two argv vectors alone you CANNOT justify "this difference is quoting" -- it could be
//! escaping, a tokenizer bug, whitespace. So the comparator takes the SOURCE LINE as evidence
//! and only classifies a difference the source DIRECTLY supports (source has a quote char +
//! argv differs -> QuoteHandling). It never reparses; it uses the source the way a human
//! debugger would -- as metadata to explain an observed divergence. No heuristic beyond what
//! the source directly supports; anything unexplained is honestly Unexpected.
//!
//! ★ FACT vs POLICY (Christian): KnownDifference is a STABLE fact about how two models differ.
//! Whether a difference is migration-safe is a CHANGING policy -- that lives in migration_status
//! (the prescriptive layer), NOT baked into the enum. When a feature lands, the comparator stops
//! PRODUCING that difference (the plans converge); the fact's meaning never gets redefined.

use super::plan::ExecutionPlan;

/// A stable, descriptive fact about how two execution models differ. NOT a policy judgment --
/// see migration_status for "is this safe to flip". The set grows as language features land
/// (each new variant arrives with its own source-evidence rule): VariableExpansion when `$`
/// is modeled, RedirectParsing when `>`/`<` are, etc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnownDifference {
    /// Legacy lowercases the command word; the spine preserves case. Evidence: argv elements
    /// differ ONLY by ASCII case, and argv[0] is not an assignment. (Intended improvement.)
    CommandCasePreservation,
    /// MATERIALLY IMPORTANT: the first word is an environment-assignment prefix (`SHELL=...`)
    /// and legacy lowercased it. An assignment prefix is part of the shell LANGUAGE, and env
    /// var names are case-sensitive on Unix -- so legacy silently CORRUPTS the variable name.
    /// Evidence: case-only argv difference where argv[0] contains '='. (A real bug the spine fixes.)
    EnvironmentAssignmentCase,
    /// Legacy's tokenizer strips quotes and joins quoted spans; the spine (pre-step-2) treats
    /// quotes as literal characters. Evidence: argv differs AND the source contains a quote
    /// character. (A capability gap -- the spine is not yet correct here.)
    QuoteHandling,
}

/// Which executor-observable field diverged (for an Unexpected difference).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DifferenceField {
    Argv,
    Cwd,
    Env,
    Io,
}

/// An unexplained divergence -- data, not a boolean. Carries both sides so a human can see
/// exactly what diverged and investigate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Difference {
    pub field: DifferenceField,
    pub legacy: String,
    pub spine: String,
}

/// The outcome of comparing two plans' execution semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonResult {
    /// The plans would execute identically.
    Equivalent,
    /// They differ, but in a way we can explain and have classified.
    Known(KnownDifference),
    /// They differ and the comparator has no evidence-backed explanation -- investigate.
    Unexpected(Difference),
}

/// True if two cwd fields describe the same execution intent. `None` means "inherit the caller's
/// current directory"; evaluated in the same process context (as the audit does), that is the
/// same directory legacy captured explicitly. So None and Some(_) are equivalent here; two
/// DIFFERENT explicit dirs would not be (that only arises once `cd`/subshell cwd is modeled).
fn cwd_equivalent(legacy: &ExecutionPlan, spine: &ExecutionPlan) -> bool {
    match (&legacy.cwd, &spine.cwd) {
        (None, None) => true,
        (Some(_), None) | (None, Some(_)) => true, // one inherits, one is explicit-current
        (Some(a), Some(b)) => a == b,
    }
}

/// Compare the execution semantics of a legacy plan and a spine plan for the same source line.
pub fn compare_execution_semantics(
    source: &str,
    legacy: &ExecutionPlan,
    spine: &ExecutionPlan,
) -> ComparisonResult {
    // env / io: only one shape exists today (Inherit / Simple). A mismatch is unexpected.
    if legacy.env != spine.env {
        return ComparisonResult::Unexpected(Difference {
            field: DifferenceField::Env,
            legacy: format!("{:?}", legacy.env),
            spine: format!("{:?}", spine.env),
        });
    }
    if legacy.io != spine.io {
        return ComparisonResult::Unexpected(Difference {
            field: DifferenceField::Io,
            legacy: format!("{:?}", legacy.io),
            spine: format!("{:?}", spine.io),
        });
    }
    if !cwd_equivalent(legacy, spine) {
        return ComparisonResult::Unexpected(Difference {
            field: DifferenceField::Cwd,
            legacy: format!("{:?}", legacy.cwd),
            spine: format!("{:?}", spine.cwd),
        });
    }

    // argv is where the real classification happens.
    if legacy.argv == spine.argv {
        return ComparisonResult::Equivalent;
    }

    // Same length, and every differing element differs ONLY by ASCII case -> CasePreservation.
    // This is provable from the vectors alone (no source needed).
    if legacy.argv.len() == spine.argv.len() {
        let all_case_only = legacy.argv.iter().zip(spine.argv.iter()).all(|(l, s)| {
            match (l.to_str(), s.to_str()) {
                (Some(l), Some(s)) => l.eq_ignore_ascii_case(s),
                _ => l == s, // non-UTF8: fall back to exact equality
            }
        });
        if all_case_only {
            // Legacy lowercases only the command word. If that word is an assignment prefix
            // (NAME=value), lowercasing changes SEMANTICS -- classify it distinctly.
            let is_assignment = legacy
                .argv
                .first()
                .and_then(|a| a.to_str())
                .map(|s| s.contains('='))
                .unwrap_or(false);
            return ComparisonResult::Known(if is_assignment {
                KnownDifference::EnvironmentAssignmentCase
            } else {
                KnownDifference::CommandCasePreservation
            });
        }
    }

    // Otherwise the argv differs in shape. Use the SOURCE as evidence: if it contains a quote
    // character, this matches our known quote-handling capability gap. This is classification
    // from directly-supported evidence, NOT reparsing.
    if source.contains('"') || source.contains('\'') {
        return ComparisonResult::Known(KnownDifference::QuoteHandling);
    }

    // No evidence-backed explanation -> honestly Unexpected.
    ComparisonResult::Unexpected(Difference {
        field: DifferenceField::Argv,
        legacy: format!("{:?}", legacy.argv),
        spine: format!("{:?}", spine.argv),
    })
}

/// The current migration's judgment about a comparison result. PRESCRIPTIVE and changing --
/// separate from the descriptive KnownDifference. As features land, a KnownDifference stops
/// being produced (plans converge), so its status here is naturally retired rather than edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationStatus {
    /// Plans execute identically -- flip-safe.
    Equivalent,
    /// An intentional, approved improvement (the spine is correct, legacy had the bug) -- flip-safe.
    SafeImprovement,
    /// A known capability the spine does not yet implement -- NOT flip-safe until it lands.
    FeatureGap,
    /// An unexplained difference -- NOT flip-safe; investigate.
    Unexpected,
}

/// Map a comparison result to the current migration policy.
pub fn migration_status(result: &ComparisonResult) -> MigrationStatus {
    match result {
        ComparisonResult::Equivalent => MigrationStatus::Equivalent,
        ComparisonResult::Known(KnownDifference::CommandCasePreservation) => {
            MigrationStatus::SafeImprovement
        }
        ComparisonResult::Known(KnownDifference::EnvironmentAssignmentCase) => {
            MigrationStatus::SafeImprovement
        }
        ComparisonResult::Known(KnownDifference::QuoteHandling) => MigrationStatus::FeatureGap,
        ComparisonResult::Unexpected(_) => MigrationStatus::Unexpected,
    }
}

#[cfg(test)]
mod tests {
    use super::super::plan::{Environment, IoPlan};
    use super::*;
    use std::ffi::OsString;

    fn plan(argv: &[&str]) -> ExecutionPlan {
        ExecutionPlan {
            argv: argv.iter().map(OsString::from).collect(),
            cwd: None,
            env: Environment::Inherit,
            io: IoPlan::Simple,
        }
    }
    fn plan_cwd(argv: &[&str], cwd: Option<&str>) -> ExecutionPlan {
        ExecutionPlan {
            argv: argv.iter().map(OsString::from).collect(),
            cwd: cwd.map(std::path::PathBuf::from),
            env: Environment::Inherit,
            io: IoPlan::Simple,
        }
    }

    #[test]
    fn identical_plans_are_equivalent() {
        let l = plan(&["git", "add", "-A"]);
        let s = plan(&["git", "add", "-A"]);
        assert_eq!(
            compare_execution_semantics("git add -A", &l, &s),
            ComparisonResult::Equivalent
        );
    }

    #[test]
    fn cwd_none_vs_some_current_is_equivalent() {
        // Semantic: inherit-current == explicit-current in the same process context.
        let legacy = plan_cwd(&["pwd"], Some("/home/christian"));
        let spine = plan_cwd(&["pwd"], None);
        assert_eq!(
            compare_execution_semantics("pwd", &legacy, &spine),
            ComparisonResult::Equivalent
        );
    }

    #[test]
    fn case_only_diff_is_case_preservation() {
        // legacy lowercased argv[0]; spine preserved. Provable from vectors, no source needed.
        let legacy = plan(&["github", "Clone"]);
        let spine = plan(&["GitHub", "Clone"]);
        assert_eq!(
            compare_execution_semantics("GitHub Clone", &legacy, &spine),
            ComparisonResult::Known(KnownDifference::CommandCasePreservation)
        );
    }

    #[test]
    fn quoted_divergence_with_quote_in_source_is_quote_handling() {
        let legacy = plan(&["git", "commit", "-m", "message here"]);
        let spine = plan(&["git", "commit", "-m", "\"message", "here\""]);
        assert_eq!(
            compare_execution_semantics("git commit -m \"message here\"", &legacy, &spine),
            ComparisonResult::Known(KnownDifference::QuoteHandling)
        );
    }

    #[test]
    fn assignment_prefix_case_is_its_own_classification() {
        // `SHELL=/bin/zsh cmd` -- legacy lowercased the assignment prefix, corrupting the env
        // var NAME. Materially different from a plain command-word case difference.
        let legacy = plan(&["shell=/bin/zsh", "/usr/bin/niri-session"]);
        let spine = plan(&["SHELL=/bin/zsh", "/usr/bin/niri-session"]);
        assert_eq!(
            compare_execution_semantics("SHELL=/bin/zsh /usr/bin/niri-session", &legacy, &spine),
            ComparisonResult::Known(KnownDifference::EnvironmentAssignmentCase)
        );
    }

    #[test]
    fn length_diff_without_quotes_is_unexpected() {
        // No quote in source but argv shape differs -> no evidence -> Unexpected (investigate).
        let legacy = plan(&["a", "b"]);
        let spine = plan(&["a", "b", "c"]);
        match compare_execution_semantics("a b", &legacy, &spine) {
            ComparisonResult::Unexpected(d) => assert_eq!(d.field, DifferenceField::Argv),
            other => panic!("expected Unexpected, got {other:?}"),
        }
    }

    #[test]
    fn migration_status_maps_policy() {
        assert_eq!(
            migration_status(&ComparisonResult::Equivalent),
            MigrationStatus::Equivalent
        );
        assert_eq!(
            migration_status(&ComparisonResult::Known(
                KnownDifference::CommandCasePreservation
            )),
            MigrationStatus::SafeImprovement
        );
        assert_eq!(
            migration_status(&ComparisonResult::Known(KnownDifference::QuoteHandling)),
            MigrationStatus::FeatureGap
        );
    }
}
