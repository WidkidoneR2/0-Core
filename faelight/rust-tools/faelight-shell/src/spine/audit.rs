//! INT-169 spine: `spine audit` -- measure how much of the REAL shell language exists in the
//! new model. See docs/rfc-169-parser-spine.md. Introspection, not a test: history is not a
//! fixture, it is the shell's accumulated language corpus. On-demand only (never startup/CI).
//!
//! ★ THE SEAM (Christian): the engine takes an ITERATOR of command strings, NOT a database.
//! `state.db -> history iterator -> audit engine`. So the same engine later audits a file, a
//! fixture, stdin, or exported history without a rewrite. The builtin is a thin db->iterator
//! adapter over this pure engine.
//!
//! Pipeline per command:  raw -> parse() -> lower() -> validate_plan() -> record.
//! Each command runs inside catch_unwind so ONE bad historical command cannot abort the audit.
//! A panic is a SERIOUS bug category (real history is where integration bugs surface that
//! proptest's generators miss), reported separately from a normal parse/lower failure.
//!
//! NOT in this increment: legacy comparison (needs the legacy->plan adapter, Increment 9); any
//! timing/benchmarking (this measures CAPABILITY, not performance -- that comes after the
//! architecture stabilizes); input normalization (dedup is raw-string only -- audit must not
//! quietly become a normalization engine).

use std::collections::BTreeMap;
use std::collections::HashSet;
use std::panic::{catch_unwind, AssertUnwindSafe};

use super::lexer::OperatorKind;
use super::parser::{parse, ParseResult};
use super::plan::{lower, ExecutionPlan, LowerError};

/// The result of auditing a history corpus. Two metrics kept distinct: VOLUME (every entry --
/// "what % of my actual activity is covered?") and SHAPE (unique inputs -- "how many language
/// shapes do I use?"). Collapsing them hides a perspective.
#[derive(Debug, Default, Clone)]
pub struct AuditReport {
    /// Total history entries seen (volume metric).
    pub total_entries: usize,
    /// Distinct command strings analyzed (shape metric). Raw-string dedup only.
    pub unique_inputs: usize,
    /// Entries not analyzable as a command (empty/whitespace-only). Accounted for so the
    /// percentages are never mysterious.
    pub skipped: usize,

    /// Unique inputs that parsed to an AST.
    pub parse_success: usize,
    /// Unique inputs the parser rejected (with a sample).
    pub parse_failure: usize,
    pub parse_failure_examples: Vec<String>,

    /// Unique inputs that lowered to a valid ExecutionPlan.
    pub lower_success: usize,
    /// Unique inputs that parsed but hit a capability boundary (by construct kind).
    pub unsupported: BTreeMap<String, usize>,
    /// A few real commands per unsupported kind -- counts size the problem, examples say what
    /// to build next.
    pub unsupported_examples: Vec<String>,

    /// Constructs the spine implements but this audit could not exercise, because it lowers
    /// with NO capabilities on purpose -- supplying a runner would execute every historical
    /// command substitution just to measure it. Counted apart from `unsupported`, which would
    /// otherwise read as language the spine lacks.
    pub missing_capability: usize,
    pub missing_capability_examples: Vec<String>,

    /// Plans that lowered but failed a well-formedness check (a real bug -- lowering produced
    /// something the executor would reject).
    pub malformed_plans: usize,
    pub malformed_examples: Vec<String>,

    /// Commands that PANICKED during parse/lower. A SERIOUS bug category, NOT a normal failure.
    pub panics: usize,
    pub panic_examples: Vec<String>,

    /// AUTHORITATIVE: inputs the parser identified as valid shell it declines to own.
    ///
    /// ⚠️ THIS DOC ONCE SAID THE OPPOSITE -- "heuristic", "not modelled by the AST", "parsed as
    /// literal words", "may include false positives" -- all true when `|` was ordinary word
    /// content and operator-shape had to be INFERRED from a lowered argv. Every clause became
    /// false when the lexer gained operator tokens, and the render block below already recorded
    /// that while this doc kept claiming the old behaviour. Two comments in one file disagreeing
    /// is how instrumentation goes quietly wrong.
    ///
    /// ★ AND ITS MEANING HAS NARROWED TO ONE CASE: every operator now PARSES, so the only
    /// remaining source is a `&` whose operand the parser cannot see (`a && b &`, where main.rs
    /// split the list upstream). If that number is ever large, the splitter changed -- not the
    /// grammar.
    pub operator_shaped: usize,
    pub operator_shaped_examples: Vec<String>,
}

/// Inspect a lowered plan for well-formedness. SEPARATE from lowering: lowering produces, this
/// inspects. Returns the reason a plan is malformed, or None if it is well-formed.
fn validate_plan(plan: &ExecutionPlan) -> Option<String> {
    if plan.argv.is_empty() {
        return Some("argv is empty".to_string());
    }
    if plan.argv[0].is_empty() {
        return Some("argv[0] is empty".to_string());
    }
    // env / cwd / io have no invariants to violate yet (Inherit / None / Simple). Checks grow
    // as those fields gain real shapes.
    None
}

/// Audit a corpus of raw command strings. Pure: no I/O, no database -- feed it any iterator.
pub fn audit_history(commands: impl Iterator<Item = String>) -> AuditReport {
    let mut report = AuditReport::default();
    let mut seen: HashSet<String> = HashSet::new();

    for raw in commands {
        report.total_entries += 1;

        let trimmed = raw.trim();
        if trimmed.is_empty() {
            report.skipped += 1;
            continue;
        }
        // Raw-string dedup only. NOT normalization: `ls -la` and `ls   -la` stay distinct.
        if !seen.insert(raw.clone()) {
            continue;
        }
        report.unique_inputs += 1;

        // One bad command must not abort the whole run; a panic is a serious, separate category.
        let outcome = catch_unwind(AssertUnwindSafe(|| audit_one(&raw)));
        match outcome {
            Err(_) => {
                report.panics += 1;
                push_example(&mut report.panic_examples, &raw);
            }
            Ok(Outcome::ParseFailure) => {
                report.parse_failure += 1;
                push_example(&mut report.parse_failure_examples, &raw);
            }
            Ok(Outcome::Unsupported(kind)) => {
                report.parse_success += 1;
                *report.unsupported.entry(kind).or_insert(0) += 1;
                push_example(&mut report.unsupported_examples, &raw);
            }
            Ok(Outcome::MissingCapability(_kind)) => {
                report.parse_success += 1;
                report.missing_capability += 1;
                push_example(&mut report.missing_capability_examples, &raw);
            }
            Ok(Outcome::Malformed(_reason)) => {
                report.parse_success += 1;
                report.malformed_plans += 1;
                push_example(&mut report.malformed_examples, &raw);
            }
            // INT-169: a REFUSAL is not a parse failure in the ordinary sense. It is valid shell
            // the spine declines to own, and the flip decision needs that counted separately from
            // malformed input -- "how much of real history can we not take" is a different
            // question from "how much of it is broken".
            // ⚠️ NOT counted as a parse failure. A refusal is valid shell the spine declines; a
            // failure is input nothing can parse. Folding them together made the headline read
            // "12,128 failures" when ~11,385 of those were the boundary working correctly, and
            // that is precisely the distinction the router runs on.
            Ok(Outcome::RefusedOperator(_)) => {
                report.operator_shaped += 1;
                push_example(&mut report.operator_shaped_examples, &raw);
            }
            Ok(Outcome::Ok) => {
                report.parse_success += 1;
                report.lower_success += 1;
            }
        }
    }

    report
}

enum Outcome {
    ParseFailure,
    /// Valid shell the parser positively identified and declined -- not malformed input.
    RefusedOperator(OperatorKind),
    Unsupported(String),
    /// The spine CAN lower this; this audit supplied no capability for it. Distinct from
    /// Unsupported, which means the executor has no such construct at all.
    MissingCapability(String),
    Malformed(String),
    Ok,
}

/// The parse -> lower -> validate pipeline for a single command.
fn audit_one(raw: &str) -> Outcome {
    let node = match parse(raw) {
        ParseResult::Complete(n) => n,
        // ★ RE-SOURCED FROM A FACT, and now from the TYPE. This already distinguished an operator
        // refusal from a parse failure, which was the four-way instinct arriving early -- but it
        // had to reach into the error to do it. INT-169 G2 makes the parser say which, so the
        // audit reads a decision instead of inferring one.
        ParseResult::Refused(crate::spine::parser::Refusal::UnsupportedOperator {
            kind, ..
        }) => return Outcome::RefusedOperator(kind),
        ParseResult::Refused(_) => return Outcome::ParseFailure,
        ParseResult::Incomplete(_) => return Outcome::ParseFailure,
        ParseResult::Invalid(_) => return Outcome::ParseFailure,
    };
    // No environment, deliberately: `spine audit` measures parse+lower CAPABILITY over the
    // history corpus, not runtime values. Variables render in source form.
    match lower(&node, &super::plan::LowerContext::default()) {
        Err(LowerError::UnsupportedConstruct { kind, .. }) => {
            Outcome::Unsupported(kind.to_string())
        }
        // Same distinction, second report: the spine HAS command substitution -- this audit
        // supplies no capabilities, so it cannot exercise it. Calling that unsupported was the
        // same mislabel migrate_audit carried.
        Err(LowerError::MissingCapability { kind, .. }) => {
            Outcome::MissingCapability(kind.to_string())
        }
        Err(LowerError::InvalidPlan { message, .. }) => Outcome::Malformed(message),
        Ok(plan) => match validate_plan(&plan) {
            Some(reason) => Outcome::Malformed(reason),
            None => Outcome::Ok,
        },
    }
}

/// Keep a small bounded sample -- examples are for direction, not exhaustiveness.
fn push_example(bucket: &mut Vec<String>, cmd: &str) {
    const MAX: usize = 10;
    if bucket.len() < MAX {
        bucket.push(cmd.to_string());
    }
}

impl AuditReport {
    /// Render the report as human-readable text for the `spine audit` builtin.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("Spine Audit Report\n\n");
        out.push_str(&format!("History entries: {}\n", self.total_entries));
        out.push_str(&format!("Unique commands: {}\n", self.unique_inputs));
        if self.skipped > 0 {
            out.push_str(&format!("Skipped (empty): {}\n", self.skipped));
        }
        out.push('\n');

        out.push_str("Parse:\n");
        out.push_str(&format!("  success: {}\n", self.parse_success));
        out.push_str(&format!("  failure: {}\n", self.parse_failure));
        out.push('\n');

        out.push_str("Lower:\n");
        out.push_str(&format!("  success:     {}\n", self.lower_success));
        let unsupported_total: usize = self.unsupported.values().sum();
        out.push_str(&format!("  unsupported: {unsupported_total}\n"));
        if self.malformed_plans > 0 {
            out.push_str(&format!("  malformed:   {}\n", self.malformed_plans));
        }
        out.push('\n');

        // AUTHORITATIVE, not a heuristic. This used to be a guess made by inspecting a lowered
        // argv, because while `|` was ordinary word content there was no better evidence. The
        // parser now returns UnsupportedOperator, so this counts a fact and every clause of the
        // old note ("heuristic", "not modeled by the AST", "parsed as literal words", "may include
        // false positives") became false the moment operators became tokens.
        if self.operator_shaped > 0 {
            out.push_str("Declined -- valid shell outside the spine grammar:\n");
            out.push_str(&format!("  {}\n", self.operator_shaped));
            out.push_str(
                "  note: the parser identified a pipe, redirect, boolean, sequence or background\n",
            );
            out.push_str(
                "  operator and declined ownership BEFORE lowering. These are correct commands\n",
            );
            out.push_str("  that the legacy path continues to run.\n");
            if !self.operator_shaped_examples.is_empty() {
                out.push_str("  examples:\n");
                for ex in &self.operator_shaped_examples {
                    out.push_str(&format!("    {ex}\n"));
                }
            }
            out.push('\n');
        }

        if !self.unsupported.is_empty() {
            out.push_str("Unsupported constructs:\n");
            for (kind, count) in &self.unsupported {
                out.push_str(&format!("  {kind}: {count}\n"));
            }
            out.push('\n');
        }

        if !self.unsupported_examples.is_empty() {
            out.push_str("Top unsupported examples:\n");
            for ex in &self.unsupported_examples {
                out.push_str(&format!("  {ex}\n"));
            }

            if self.missing_capability > 0 {
                out.push_str("Not exercised -- capability withheld by this audit:\n");
                out.push_str(&format!("  {}\n", self.missing_capability));
                out.push_str(
                    "  note: the spine implements these; the audit lowers with no runner so it\n",
                );
                out.push_str("  never executes historical commands to measure them.\n");
                for ex in &self.missing_capability_examples {
                    out.push_str(&format!("    {ex}\n"));
                }
                out.push('\n');
            }
            out.push('\n');
        }

        if self.malformed_plans > 0 {
            out.push_str("Malformed-plan examples (BUG):\n");
            for ex in &self.malformed_examples {
                out.push_str(&format!("  {ex}\n"));
            }
            out.push('\n');
        }

        if self.panics > 0 {
            out.push_str(&format!("Panics: {} (SERIOUS)\n", self.panics));
            for ex in &self.panic_examples {
                out.push_str(&format!("  {ex}\n"));
            }
            out.push('\n');
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audits_a_known_corpus() {
        let corpus = vec![
            "git add -A".to_string(),  // bare -> Ok
            "cistart 133".to_string(), // bare -> Ok
            "".to_string(),            // skipped
            "   ".to_string(),         // skipped
            "git add -A".to_string(),  // duplicate -> not re-counted as unique
        ];
        let r = audit_history(corpus.into_iter());
        assert_eq!(r.total_entries, 5);
        assert_eq!(r.skipped, 2);
        assert_eq!(r.unique_inputs, 2, "two distinct non-empty commands");
        assert_eq!(r.parse_success, 2);
        assert_eq!(r.lower_success, 2);
        assert_eq!(r.panics, 0);
        assert!(r.unsupported.is_empty());
        assert_eq!(r.operator_shaped, 0, "no operators in this corpus");
    }

    /// CONTRACT: a construct the spine cannot execute never reaches a plan.
    ///
    /// ⚠️ THE PROBE HAS MOVED THREE TIMES -- pipe, then `&&`, then `&` -- because each was chosen
    /// from what the spine had not implemented YET, and each became owned. This one is chosen from
    /// what is refused BY DESIGN: `a && b &` cannot be backgrounded correctly because main.rs
    /// splits the boolean list before the parser sees it, so the true operand is unavailable.
    /// Wrapping what does arrive would background only the tail. That refusal outlives increments.
    #[test]
    fn a_construct_the_spine_cannot_execute_never_reaches_a_plan() {
        let r = audit_history(vec!["a && b &".to_string()].into_iter());
        assert_eq!(
            r.lower_success, 0,
            "must not lower a command it cannot execute"
        );
        assert_eq!(r.operator_shaped, 1, "counted as a refusal");
        // ⚠️ A refusal is NOT a parse failure. Valid shell the spine declines is a different fact
        // from input nobody can read, and folding them together made the audit headline report
        // ~11,385 correct routing decisions as syntax errors.
        assert_eq!(r.parse_failure, 0, "declined, not malformed");
        assert_eq!(r.panics, 0);
    }

    #[test]
    fn empty_corpus_is_clean() {
        let r = audit_history(std::iter::empty());
        assert_eq!(r.total_entries, 0);
        assert_eq!(r.unique_inputs, 0);
        assert_eq!(r.lower_success, 0);
    }

    #[test]
    fn validate_catches_empty_argv() {
        // A hand-built degenerate plan (argv empty) is flagged malformed.
        use super::super::plan::{Environment, ExecutionPlan, IoPlan};
        let bad = ExecutionPlan {
            argv: vec![],
            cwd: None,
            env: Environment::Inherit,
            io: IoPlan::Simple,
        };
        assert!(validate_plan(&bad).is_some());
    }

    #[test]
    fn report_renders_without_panic() {
        let r = audit_history(vec!["pwd".to_string()].into_iter());
        let text = r.render();
        assert!(text.contains("Spine Audit Report"));
        assert!(text.contains("Unique commands: 1"));
    }
}
