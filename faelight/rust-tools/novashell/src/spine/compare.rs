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
//! argv differs -> QuotedWordParsing). It never reparses; it uses the source the way a human
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
    /// The two argvs are identical once backslashes are removed -- legacy strips a backslash that
    /// bash and the spine both keep. A FACT about how the models differ; the policy below decides
    /// what to do about it.
    BackslashPreservation,
    /// Legacy lowercases the command word; the spine preserves case. Evidence: argv elements
    /// differ ONLY by ASCII case, and argv[0] is not an assignment. (Intended improvement.)
    CommandCasePreservation,
    /// MATERIALLY IMPORTANT: the first word is an environment-assignment prefix (`SHELL=...`)
    /// and legacy lowercased it. An assignment prefix is part of the shell LANGUAGE, and env
    /// var names are case-sensitive on Unix -- so legacy silently CORRUPTS the variable name.
    /// Evidence: case-only argv difference where argv[0] contains '='. (A real bug the spine fixes.)
    EnvironmentAssignmentCase,
    /// QUOTED-WORD BOUNDARY PARSING specifically: single quotes, double quotes, adjacent
    /// quoted/unquoted segments, empty quoted strings. Implemented at roadmap step 2, so a
    /// remaining divergence here means LEGACY is the one getting it wrong (it drops empty
    /// quoted args entirely, and splits `VAR="a b"` on the raw space before tokenizing).
    ///
    /// Deliberately NARROWER than "quote handling", which is a family: escapes inside double
    /// quotes, escaped quotes, escaped backslashes, and line continuations are NOT implemented.
    /// Naming the fact after the implemented subset is what makes promoting it to
    /// SafeImprovement defensible. Evidence: argv differs, the source contains a quote
    /// character, and NO backslash (a backslash means escaping may be involved, which the
    /// spine does not model -- those fall through to Unexpected until there is a precise rule).
    QuotedWordParsing,
    /// WHERE A PREFIX ASSIGNMENT LIVES. Legacy has no environment model at all: it mutates the
    /// environment of the shell itself, runs the command, and restores each variable by hand, so
    /// its plan says Inherit. The spine carries the same intent as an Overlay applied to the
    /// child. Same effect on the process, different representation.
    ///
    /// Evidence, provable from the plans alone: legacy env is Inherit, spine env is Overlay, and
    /// the argv vectors are EQUAL. The argv requirement is what keeps this from becoming a blanket
    /// excuse -- if the command words also differ, that is a real divergence and must not be
    /// swallowed by an environment explanation.
    AssignmentPrefixOwnership,
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
///
/// ⚠️ CARRYING IT WAS NEVER THE PROBLEM. `migration_status` matched `Unexpected(_)` and threw all
/// of this away one line before the audit incremented a counter, so 1025 rows arrived as a bare
/// total that cannot say what to build. That is the SAME defect `declined_by_reason` was created to
/// fix, recorded in its own doc comment: the parser already knew the operator, and `Err(_)`
/// discarded it. See `shape()` below -- the audit groups on it now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Difference {
    pub field: DifferenceField,
    pub legacy: String,
    pub spine: String,
}

impl Difference {
    /// A stable, LOW-CARDINALITY name for what KIND of divergence this is, for grouping.
    ///
    /// ⭐ THE POINT IS TO TURN A TOTAL INTO A BUILD ORDER, exactly as `declined_by_reason` did for
    /// declines. "1025 unexpected" says investigate everything; "900 of one shape" says fix one
    /// thing. So this must NOT include the differing text itself -- that would give 1025 groups of
    /// one and reinvent the problem.
    ///
    /// ⚠️ AND IT MUST NOT GUESS. Each arm below names a MECHANICAL, checkable property of the two
    /// strings. Anything not recognised is `argv: other` rather than a plausible-sounding label,
    /// because a wrong group is worse than an unlabelled one -- it sends the reader to fix
    /// something that was never broken.
    pub fn shape(&self) -> String {
        match self.field {
            DifferenceField::Cwd => "cwd".to_string(),
            DifferenceField::Env => "env".to_string(),
            DifferenceField::Io => self.io_shape(),
            DifferenceField::Argv => self.argv_shape(),
        }
    }
    /// ⭐ THE SAME SPLIT argv GOT, FOR THE SAME REASON, one bucket later. io was the largest
    /// unexpected shape at 49 of 59, and 49 of one label says investigate everything -- which is
    /// exactly what the note above argv_shape says a total must not stay.
    ///
    /// The ten sampled examples were four different problems wearing one name: a stdin redirect,
    /// an append to a quoted path, an unquoted space in a target, and process substitution. Those
    /// are four decisions, not one.
    ///
    /// ⚠️ MECHANICAL ONLY, per the rule above: every arm tests a checkable property of the two
    /// debug-rendered IoPlans. Anything unrecognised stays io-colon-other rather than earning a
    /// plausible name, because a wrong group sends the reader to fix something that was never
    /// broken. IoPlan has three variants -- Simple, Capture, Files -- so the presence tests are
    /// substring checks on names the enum actually has.
    fn io_shape(&self) -> String {
        let (l, s) = (&self.legacy, &self.spine);
        let files = |t: &str| t.starts_with("Files");
        let simple = |t: &str| t.starts_with("Simple");

        // One side sees a redirect where the other sees none. The largest disagreement there
        // can be: not a detail of the redirect but whether one exists.
        if simple(l) != simple(s) {
            return if simple(l) {
                "io: spine redirects, legacy does not".to_string()
            } else {
                "io: legacy redirects, spine does not".to_string()
            };
        }
        if files(l) && files(s) {
            // stdin present on one side only: a stdin redirect is a separate phase from stdout.
            let has_stdin = |t: &str| t.contains("stdin: Some");
            if has_stdin(l) != has_stdin(s) {
                return "io: stdin disagreement".to_string();
            }
            // Same wiring, different open mode: truncate against append.
            let appends = |t: &str| t.contains("true");
            if appends(l) != appends(s) {
                return "io: append vs truncate".to_string();
            }
            return "io: target path differs".to_string();
        }
        if l.starts_with("Capture") != s.starts_with("Capture") {
            return "io: capture disagreement".to_string();
        }
        "io: other".to_string()
    }

    /// Argv is where the volume is, so it is the one split further.
    fn argv_shape(&self) -> String {
        let (l, s) = (&self.legacy, &self.spine);

        // ⭐ THE BACKSLASH CASE, and it is named first because it is the one this was written to
        // measure. `grep -r "a\|b"` parses to `a\|b` in the spine; if legacy differs ONLY by
        // backslashes, the two agree about every other character and the divergence is one rule.
        if l.replace('\\', "") == s.replace('\\', "") && l != s {
            return "argv: backslash handling".to_string();
        }
        // Same content, different quoting characters left in place.
        let strip = |t: &str| t.replace(['"', '\''], "");
        if strip(l) == strip(s) && l != s {
            return "argv: quote characters retained".to_string();
        }
        // Case-only: legacy lowercased the command word for years (INT-195).
        if l.to_lowercase() == s.to_lowercase() && l != s {
            return "argv: case only".to_string();
        }
        // One side produced more or fewer words -- a splitting disagreement, not a text one.
        let (lw, sw) = (l.split_whitespace().count(), s.split_whitespace().count());
        if lw != sw {
            return format!("argv: word count ({lw} vs {sw})");
        }
        "argv: other".to_string()
    }
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
    // ENV IS A FIRST-CLASS PART OF EQUIVALENCE. The old comment here said only one shape existed,
    // which was true while every plan said Inherit and stopped being true when Overlay landed.
    //
    // A prefix assignment is the one shape where the two models genuinely cannot agree on
    // representation: legacy has no env field to fill, because it achieves the same effect by
    // mutating the shell and restoring afterwards. Classified rather than excused, and only when
    // argv already matches.
    if legacy.env != spine.env {
        if matches!(legacy.env, super::plan::Environment::Inherit)
            && matches!(spine.env, super::plan::Environment::Overlay(_))
            && legacy.argv == spine.argv
        {
            return ComparisonResult::Known(KnownDifference::AssignmentPrefixOwnership);
        }
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
    let has_quote = source.contains('"') || source.contains('\'');
    let has_backslash = source.contains('\\');
    if has_quote && !has_backslash {
        return ComparisonResult::Known(KnownDifference::QuotedWordParsing);
    }

    // ⭐ BACKSLASH PRESERVATION, and it is a NEW FACT rather than a widening of the rule above.
    //
    // The guard at has_backslash was correct when written: the comment beside it said that without
    // a PRECISE rule we must not claim an explanation. There is one now, and it is mechanical
    // rather than inferred from the source text -- strip backslashes from BOTH argvs and ask
    // whether what remains is identical. If it is, the two models agree about every other
    // character and the entire divergence is one escaping rule.
    //
    // ⚠️ AND THE SPINE IS THE CORRECT ONE, measured against bash rather than reasoned about:
    //     bash -c 'printf "%s\n" "a\|b"'   ->   a\|b
    // Inside double quotes a backslash before an ordinary character is LITERAL, so `a\|b` is what
    // reaches the program. The spine keeps it; legacy strips it. Same shape as QuotedWordParsing
    // above, where the policy moved once the spine became the right one.
    let strip_bs = |v: &Vec<std::ffi::OsString>| -> Vec<String> {
        v.iter()
            .map(|a| a.to_string_lossy().replace('\\', ""))
            .collect()
    };
    if strip_bs(&legacy.argv) == strip_bs(&spine.argv) {
        return ComparisonResult::Known(KnownDifference::BackslashPreservation);
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
        // Step 2 implemented quoted-word parsing, so a divergence in THIS narrow fact now
        // means legacy is wrong, not that the spine is behind. The FACT never changed meaning
        // -- only the policy moved, which is exactly what the fact/policy split is for.
        ComparisonResult::Known(KnownDifference::QuotedWordParsing) => {
            MigrationStatus::SafeImprovement
        }
        // ⚠️ SAFE IMPROVEMENT BECAUSE BASH WAS ASKED, not because the spine is newer. Inside double
        // quotes a backslash before an ordinary character is literal, so legacy has been silently
        // deleting a character the program was meant to receive. 972 rows of six months of real
        // history, and every one of them was legacy getting it wrong.
        ComparisonResult::Known(KnownDifference::BackslashPreservation) => {
            MigrationStatus::SafeImprovement
        }
        // The spine is the correct one here. Legacy reaches POSIX scoping by mutating the shell
        // and restoring by hand -- and that restore is the only thing between `FOO=1 echo hi` and
        // a permanent leak, which is the INT-143 bug B disaster. An overlay cannot leak.
        ComparisonResult::Known(KnownDifference::AssignmentPrefixOwnership) => {
            MigrationStatus::SafeImprovement
        }
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
    fn quoted_divergence_without_backslash_is_quoted_word_parsing() {
        let legacy = plan(&["git", "commit", "-m", "message here"]);
        let spine = plan(&["git", "commit", "-m", "\"message", "here\""]);
        assert_eq!(
            compare_execution_semantics("git commit -m \"message here\"", &legacy, &spine),
            ComparisonResult::Known(KnownDifference::QuotedWordParsing)
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

    /// A prefix assignment lives in argv for legacy and in env for the spine. Same effect on the
    /// process, different representation, and legacy has no env field to fill because it mutates
    /// the shell and restores by hand instead.
    #[test]
    fn a_prefix_assignment_in_env_rather_than_argv_is_classified() {
        let legacy = plan(&["echo", "hi"]);
        let mut spine = plan(&["echo", "hi"]);
        spine.env = super::super::plan::Environment::Overlay(vec![(
            std::ffi::OsString::from("FOO"),
            std::ffi::OsString::from("1"),
        )]);
        assert_eq!(
            compare_execution_semantics("FOO=1 echo hi", &legacy, &spine),
            ComparisonResult::Known(KnownDifference::AssignmentPrefixOwnership)
        );
    }

    /// THE GUARD ON THAT RULE, and it is the load-bearing half. An env difference explains an env
    /// difference -- it must never excuse a command that also differs. If argv diverges too, that
    /// is real and stays Unexpected.
    #[test]
    fn an_env_difference_does_not_excuse_a_diverging_argv() {
        let legacy = plan(&["echo", "hi"]);
        let mut spine = plan(&["echo", "there"]);
        spine.env = super::super::plan::Environment::Overlay(vec![(
            std::ffi::OsString::from("FOO"),
            std::ffi::OsString::from("1"),
        )]);
        match compare_execution_semantics("FOO=1 echo hi", &legacy, &spine) {
            ComparisonResult::Unexpected(d) => assert_eq!(d.field, DifferenceField::Env),
            other => panic!("a diverging argv must not be classified: {other:?}"),
        }
    }

    #[test]
    fn quote_plus_backslash_is_unexpected_not_classified() {
        // The evidence rule excludes backslash: escaping is NOT implemented, and a backslash
        // means the divergence may involve it. Without a precise rule we must not claim
        // QuotedWordParsing (which now asserts the spine is CORRECT). Honest fallback:
        // Unexpected, so the data can name the next fact.
        let legacy = plan(&["echo", "a\\\"b"]);
        let spine = plan(&["echo", "a\\", "b"]);
        match compare_execution_semantics("echo \"a\\\"b\"", &legacy, &spine) {
            ComparisonResult::Unexpected(d) => assert_eq!(d.field, DifferenceField::Argv),
            other => panic!("backslash must not be classified as quoting: {other:?}"),
        }
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
            migration_status(&ComparisonResult::Known(KnownDifference::QuotedWordParsing)),
            MigrationStatus::SafeImprovement
        );
    }
}
