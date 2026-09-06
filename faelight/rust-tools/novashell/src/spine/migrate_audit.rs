//! INT-169 spine: the migration audit engine. See docs/rfc-169-parser-spine.md. Increment 10
//! Phase 2. Answers "how migration-ready is the spine against real usage?" -- how much of what
//! the shell executes today the spine would execute IDENTICALLY, or differ from only in ways
//! we've explicitly accepted.
//!
//! ★ THE ENGINE DOES NOT KNOW HOW PLANS ARE PRODUCED (Christian). It only OBSERVES plans and
//! classifies them via compare_execution_semantics + migration_status. Production of the legacy
//! plan (from_line + plan_from_legacy, which needs ForestDb) and the spine plan (parse + lower)
//! happens in the builtin, near the shell where ForestDb naturally lives. Coupling the analysis
//! layer to the legacy execution implementation (a ForestDb handle) would be backwards. So the
//! caller hands over an AuditObservation; the engine judges. Same engine can later observe
//! fixtures, fuzz-generated plans, or another parser -- unchanged.
//!
//! ★ THE ENGINE OWNS THE FAILURE CASE (Christian): AuditObservation.spine is a Result -- a spine
//! lowering that returns UnsupportedConstruct IS migration data (a feature gap), so the engine
//! classifies it rather than the caller inventing a special case.
//!
//! ★★ COMPARISON SCOPE (part of the audit's SPECIFICATION, not a disclaimer): this compares the
//! LEGACY PARSER and the SPINE PARSER given the same input string. The live shell performs
//! variable, command-substitution, glob, and alias expansion BEFORE invoking the legacy parser
//! (main.rs: expand_vars -> expand_subshells -> expand_globs -> alias -> execute -> from_line);
//! those phases are NOT modelled here. So the supported claims are exactly:
//!   parser equivalence     -- YES, measured.
//!   execution equivalence  -- NOT yet demonstrated.
//!   expansion equivalence  -- future work.
//! Note the two models run their phases in OPPOSITE ORDER: legacy is raw -> expand (string
//! munging) -> parse; the spine is raw -> parse -> expand (word walking). That inversion is the
//! RFC's thesis -- legacy expands before it knows what is quoted -- and comparing the two orders
//! is what the variable-expansion milestone will test.
//!
//! ★ MIGRATION-SAFE != CORRECT: "safe to flip a command" means the spine would NOT CHANGE its
//! behavior except where intended -- not that the behavior is correct. A command both models
//! handle wrong-but-identically is flip-safe (no regression); correctness comes later as
//! constructs land. Flip-readiness = zero feature gaps AND zero unexpected differences (every
//! command is Equivalent or an approved SafeImprovement).

use super::compare::{compare_execution_semantics, migration_status, MigrationStatus};
use super::plan::{ExecutionPlan, LowerError};

/// One unit of work handed to the engine. The caller produces both plans (however it likes) and
/// the engine judges. `spine` is a Result so the engine owns the "spine could not lower" case.
pub struct AuditObservation<'a> {
    pub source: &'a str,
    pub legacy: ExecutionPlan,
    pub spine: Result<ExecutionPlan, LowerError>,
}

/// The migration readiness report. Talks about MIGRATION (spine vs legacy), not parsing.
/// Why a history entry was not applicable to comparison. NOT failures -- these are OUTSIDE the
/// current audit model, which compares SINGLE-COMMAND inputs. A future mode could understand
/// pasted blocks and scripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// Empty or whitespace-only entry.
    Empty,
    /// A multi-line entry (pasted heredoc, script block). Not a shell LINE -- the audit's unit
    /// of comparison is one command, so comparing it would measure a newline-tokenization
    /// difference rather than anything about the language.
    Multiline,
    /// A stderr redirect. Legacy delegates the WHOLE line to `sh` (INT-172), so it produces no
    /// plan of its own -- there is nothing to compare, and counting it as a divergence measured
    /// the audit's own model rather than the shell.
    StderrDelegated,
}

/// Is this history entry applicable to single-command comparison? None = comparable.
pub fn applicability(source: &str) -> Option<SkipReason> {
    if source.trim().is_empty() {
        return Some(SkipReason::Empty);
    }
    if source.contains('\n') {
        return Some(SkipReason::Multiline);
    }
    // INT-200: A STDERR REDIRECT HAS NO LEGACY PLAN TO COMPARE AGAINST. INT-172's repair was to
    // hand every `2>` line to `sh` WHOLE, so legacy never builds an argv for one -- and once the
    // spine learned fd redirects, every such row read as a divergence. 384 of them.
    //
    // ★ ASKED, NOT COPIED: `detect_redirect` owns the rule, and re-deriving `2>` here would be a
    // second implementation that drifts the first time either changes.
    if matches!(
        crate::expand::detect_redirect(source).1,
        Some((ref t, _)) if t == "__stderr__"
    ) {
        return Some(SkipReason::StderrDelegated);
    }
    None
}

#[derive(Debug, Default, Clone)]
pub struct MigrationReport {
    /// Every history entry the caller offered (the denominator).
    pub seen: usize,
    pub skipped_empty: usize,
    pub skipped_multiline: usize,
    pub skipped_stderr: usize,
    /// Pipelines the spine LOWERS AND RUNS -- a positive category, see `pipeline_owned`.
    pub pipeline_owned: usize,
    pub pipeline_owned_examples: Vec<String>,
    pub compared: usize,
    pub equivalent: usize,
    pub safe_improvement: usize,
    pub feature_gap: usize,
    /// The spine IMPLEMENTS the construct; this audit withheld the capability it needs.
    /// INT-169: `spine migrate` lowers with no runner ON PURPOSE, because measuring parse
    /// capability must not execute twenty-five thousand historical substitutions. Counting
    /// those as feature gaps made the migration read as less complete than it is.
    pub missing_capability: usize,
    pub unexpected: usize,
    /// Spine parse/lower produced no plan at all (couldn't even attempt comparison) -- distinct
    /// from a feature gap (which is a classified UnsupportedConstruct). Bounded examples.
    pub spine_unlowerable: usize,
    /// The spine could not PARSE what legacy accepted (since step 2: an unterminated quote --
    /// legacy's tokenizer silently consumes to end of line, the spine reports a lex error).
    /// A real behavioral difference, and counted so no entry vanishes from the denominator.
    pub spine_parse_error: usize,
    pub spine_parse_error_examples: Vec<String>,
    /// INT-169: WHY each line was declined, counted by construct.
    ///
    /// ★ The bare total was a dead end. 6,223 declines is the remaining migration work, but a
    /// single number cannot say WHAT TO BUILD -- and the parser already knew, because
    /// ParseError::UnsupportedOperator carries the operator. The call site discarded it with
    /// `Err(_)` one line before incrementing the counter.
    ///
    /// Rendered, this sorts six months of real history into a BUILD ORDER: pipes, redirects,
    /// forest pipelines, boolean chains. Biggest bucket first, measured rather than guessed.
    pub declined_by_reason: std::collections::BTreeMap<String, usize>,

    /// ⭐ WHAT THE 1025 ACTUALLY ARE, grouped by mechanical shape rather than counted.
    ///
    /// Added for the reason `declined_by_reason` above records verbatim: the evidence already
    /// existed and the call site threw it away. `compare.rs` builds a `Difference` carrying the
    /// field and BOTH sides, then `migration_status` matched `Unexpected(_)` -- the underscore
    /// discarded all of it one line before this counter incremented.
    pub unexpected_by_shape: std::collections::BTreeMap<String, usize>,

    pub safe_improvement_examples: Vec<String>,
    pub feature_gap_examples: Vec<String>,
    pub missing_capability_examples: Vec<String>,
    pub unexpected_examples: Vec<String>,

    /// Examples keyed BY SHAPE, because a count without instances is half an answer. The
    /// shape split turns a total into a build order; this is what lets you read the group you
    /// are about to work on. Measured 2026-09-02: 43 of 59 unexpected were one shape, and the
    /// flat example list mixed all 59 -- so the largest group could only be sampled by chance.
    pub unexpected_examples_by_shape: std::collections::BTreeMap<String, Vec<String>>,
}

/// Accumulator. Feed it observations; it classifies each and tallies.
#[derive(Debug, Default)]
pub struct MigrationAudit {
    report: MigrationReport,
}

impl MigrationAudit {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an entry that is outside the comparison domain. The caller checks applicability()
    /// first so it need not produce plans for these.
    pub fn skip(&mut self, reason: SkipReason) {
        self.report.seen += 1;
        match reason {
            SkipReason::Empty => self.report.skipped_empty += 1,
            SkipReason::Multiline => self.report.skipped_multiline += 1,
            SkipReason::StderrDelegated => self.report.skipped_stderr += 1,
        }
    }

    /// Record a PIPELINE the spine lowers and executes. INT-200.
    ///
    /// ★ NOT A GAP AND NOT A SKIP -- a WIN with nothing to compare it against. Legacy builds no
    /// single plan for a pipeline (its live path routes one to the native pipeline or to sh,
    /// never through ExecContext), so a comparison would measure the audit's model rather than
    /// the shell. Counted on its own so the report says `owned` where it used to say `gap`.
    pub fn pipeline_owned(&mut self, source: &str) {
        self.report.seen += 1;
        self.report.pipeline_owned += 1;
        push_example(&mut self.report.pipeline_owned_examples, source);
    }

    /// Record a source the spine could not parse at all. Counted, never silently dropped.
    pub fn spine_parse_error(&mut self, source: &str, why: &crate::spine::parser::ParseResult) {
        use crate::spine::lexer::LexIncompleteKind as K;
        use crate::spine::parser::{ParseError as E, ParseResult as R, Refusal as Ref};
        self.report.seen += 1;
        self.report.spine_parse_error += 1;
        push_example(&mut self.report.spine_parse_error_examples, source);
        // INT-169 G2: the audit reads the parser DECISION rather than reaching into an error to
        // recover one. The four outcomes say different things about the gap, and lumping them lost
        // that: an unterminated quote is INCOMPLETE INPUT, not a spine parse failure, and 39 of them
        // were being counted against the shell as though the spine could not parse valid syntax.
        // INT-196 follow-on: THE STAGE IS PART OF THE KEY, not something the renderer infers from
        // the text. One bucket was answering two questions -- parse-stage outcomes recorded here and
        // lowering declines recorded in observe() -- and the shared header called all of them
        // "Declined by construct". For 62 of the 116 that was flatly wrong: an unterminated quote is
        // UNFINISHED INPUT, which the correction above this function already states and the display
        // then contradicted.
        let reason = match why {
            R::Refused(Ref::ComparisonNotRedirect { .. }) => {
                "refused: comparison, not a redirect (> 0.5) -- deliberate divergence".to_string()
            }
            R::Refused(Ref::FdRedirect { .. }) => {
                "refused: redirect with explicit fd (2>, 1>>) -- left to legacy".to_string()
            }
            R::Refused(Ref::UnsupportedOperator { kind, .. }) => {
                format!("refused: operator {kind:?}")
            }
            R::Invalid(E::MissingRedirectTarget { kind, .. }) => {
                format!("invalid: redirect with no target ({kind:?})")
            }
            R::Invalid(E::NoCommand) => "invalid: empty".to_string(),
            R::Invalid(e) => format!("invalid: {e:?}"),
            R::Incomplete(i) => match i.kind {
                K::UnterminatedQuote => "incomplete: quote not closed".to_string(),
                K::UnterminatedCommandSub => "incomplete: $( ) not closed".to_string(),
                K::TrailingEscape => "incomplete: line ends in a backslash".to_string(),
                K::HeredocBody { .. } => "incomplete: heredoc body not arrived".to_string(),
            },
            // The caller only reaches here when the spine did NOT produce a plan.
            R::Complete(_) => "complete (unreachable here)".to_string(),
        };
        *self.report.declined_by_reason.entry(reason).or_insert(0) += 1;
    }

    /// Observe one (source, legacy plan, spine result). The engine classifies and records.
    pub fn observe(&mut self, obs: AuditObservation) {
        self.report.seen += 1;
        self.report.compared += 1;

        let spine_plan = match obs.spine {
            Ok(p) => p,
            Err(LowerError::UnsupportedConstruct { kind, .. }) => {
                // Spine parsed but cannot lower this construct yet -- a migration feature gap.
                self.report.feature_gap += 1;
                push_example(&mut self.report.feature_gap_examples, obs.source);
                // INT-200: SAY WHICH CONSTRUCT. `kind` was bound and dropped, so a forest
                // value pipeline and a shell pipeline the spine cannot execute yet counted
                // identically -- and those need opposite responses. A forest pipeline is
                // declined FOREVER (legacy's `apply_pipeline` is the only implementation of
                // those verbs); a shell pipeline is declined only until execution lands.
                // Third time this session an observation layer knew the reason and would
                // not say it.
                *self
                    .report
                    .declined_by_reason
                    .entry(format!("unlowerable: {kind}"))
                    .or_insert(0) += 1;
                return;
            }
            Err(LowerError::MissingCapability { kind: _, span: _ }) => {
                // NOT a feature gap. Beside spine_unlowerable and spine_parse_error, which
                // exist for the same reason: keep it visible, keep it out of the denominator's
                // wrong bucket.
                self.report.missing_capability += 1;
                push_example(&mut self.report.missing_capability_examples, obs.source);
                return;
            }
            Err(LowerError::InvalidPlan {
                message: _,
                span: _,
            }) => {
                // Spine produced no usable plan -- can't compare. Its own bucket.
                self.report.spine_unlowerable += 1;
                return;
            }
        };

        let result = compare_execution_semantics(obs.source, &obs.legacy, &spine_plan);
        match migration_status(&result) {
            MigrationStatus::Equivalent => self.report.equivalent += 1,
            MigrationStatus::SafeImprovement => {
                self.report.safe_improvement += 1;
                push_example(&mut self.report.safe_improvement_examples, obs.source);
            }
            MigrationStatus::FeatureGap => {
                self.report.feature_gap += 1;
                push_example(&mut self.report.feature_gap_examples, obs.source);
            }
            MigrationStatus::Unexpected => {
                self.report.unexpected += 1;
                // ⚠️ READ THE EVIDENCE BEFORE COUNTING IT. The comparison already knows what
                // diverged; this is the line that used to lose it.
                if let super::compare::ComparisonResult::Unexpected(d) = &result {
                    let shape = d.shape();
                    *self
                        .report
                        .unexpected_by_shape
                        .entry(shape.clone())
                        .or_insert(0) += 1;
                    // The example goes in the SAME arm as the count, because they answer one
                    // question together. The flat list below is kept -- it is what the report has
                    // always printed -- but a reader working one shape needs that shape instances.
                    let bucket = self
                        .report
                        .unexpected_examples_by_shape
                        .entry(shape)
                        .or_default();
                    push_example(bucket, obs.source);
                } else {
                    push_example(&mut self.report.unexpected_examples, obs.source);
                }
            }
        }
    }

    pub fn finish(self) -> MigrationReport {
        self.report
    }
}

impl MigrationReport {
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("Migration Audit (spine vs legacy)\n");

        // ⚠️ WHICH SHELL PRODUCED THIS REPORT. Typed at a prompt, `spine migrate` runs the DEPLOYED
        // binary, so a comparator change sitting unbuilt in the working tree is simply not in these
        // numbers. That cost real time on 2026-08-24: the same command was run three times against
        // a shell that predated the change being measured, and nothing in the output said so.
        //
        // ⭐ THE SUITE LEARNED THE SAME LESSON THE SAME DAY and its wording is deliberate here too:
        // state the identity, then the CONSEQUENCE. A version printed alone is a fact a reader
        // skims; a version printed beside what it means is one they act on.
        //
        // Unlike fsh-test there is no NSH_BIN to point elsewhere -- this code IS the shell -- so
        // the honest move is to name the running build and the command that rebuilds it.
        let version = env!("CARGO_PKG_VERSION");
        let exe = std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "(unknown path)".to_string());
        out.push_str(&format!("  produced by: fsh {version}  {exe}\n"));
        if !exe.contains("/target/") {
            out.push_str(
                "  ⚠️  this is the DEPLOYED shell -- comparator changes you have not deployed are \
                 NOT in these numbers\n",
            );
            out.push_str(
                "     for the working tree: cargo build -p novashell && \
                 ./target/debug/faelight-shell -c 'spine migrate'\n",
            );
        }
        out.push('\n');
        out.push_str(&format!("History entries seen:     {}\n", self.seen));
        out.push_str(&format!("Applicable to comparison: {}\n", self.compared));
        let skipped = self.skipped_empty + self.skipped_multiline + self.skipped_stderr;
        out.push_str(&format!("Skipped:                  {skipped}\n"));
        if self.skipped_empty > 0 {
            out.push_str(&format!("  empty:     {}\n", self.skipped_empty));
        }
        if self.skipped_multiline > 0 {
            out.push_str(&format!("  multiline: {}\n", self.skipped_multiline));
        }
        if self.skipped_stderr > 0 {
            out.push_str(&format!(
                "  stderr-delegated: {} (legacy hands these to sh whole -- no plan to compare)\n",
                self.skipped_stderr
            ));
        }
        out.push_str("  (excluded: Increment 10 compares single-command inputs)\n\n");
        let pct = |n: usize| -> String {
            if self.compared == 0 {
                "  0.0%".to_string()
            } else {
                format!("{:5.1}%", (n as f64) * 100.0 / (self.compared as f64))
            }
        };
        out.push_str(&format!(
            "Equivalent:        {:>7}  {}\n",
            self.equivalent,
            pct(self.equivalent)
        ));
        out.push_str(&format!(
            "Safe improvements: {:>7}  {}\n",
            self.safe_improvement,
            pct(self.safe_improvement)
        ));
        out.push_str(&format!(
            "Feature gaps:      {:>7}  {}\n",
            self.feature_gap,
            pct(self.feature_gap)
        ));
        out.push_str(&format!(
            "Unexpected:        {:>7}  {}\n",
            self.unexpected,
            pct(self.unexpected)
        ));
        if self.spine_unlowerable > 0 {
            out.push_str(&format!("Spine unlowerable: {}\n", self.spine_unlowerable));
        }
        if self.spine_parse_error > 0 {
            out.push_str(&format!(
                "No spine plan: {}  (see the stage breakdown below -- most are unfinished input)\n",
                self.spine_parse_error
            ));
        }
        if self.pipeline_owned > 0 {
            out.push_str(&format!(
                "Pipelines owned:   {:>7}  (spine runs these -- legacy has no single plan to compare)\n",
                self.pipeline_owned
            ));
        }
        // ★ THE BUILD ORDER, from six months of real history. A bare decline total says how
        // much work remains; this says WHICH work, sorted by what actually gets typed.
        if !self.declined_by_reason.is_empty() {
            // THREE STAGES, THREE HEADERS. One block headed "Declined by construct" covered all of
            // them, and for the incomplete rows that was flatly wrong: an unterminated quote is
            // UNFINISHED INPUT, not a decline, which the comment on spine_parse_error already says
            // and this display then contradicted. The stage now lives in the key, recorded where the
            // fact is known, so this only groups what it was told.
            let mut rows: Vec<_> = self.declined_by_reason.iter().collect();
            rows.sort_by(|a, b| b.1.cmp(a.1));
            let group = |title: &str, prefix: &str, out: &mut String| {
                let hits: Vec<_> = rows.iter().filter(|(r, _)| r.starts_with(prefix)).collect();
                if hits.is_empty() {
                    return;
                }
                let total: usize = hits.iter().map(|(_, c)| **c).sum();
                out.push_str(&format!("{title} ({total}):\n"));
                for (reason, count) in hits {
                    let shown = reason.strip_prefix(prefix).unwrap_or(reason);
                    out.push_str(&format!("  {count:>7}  {shown}\n"));
                }
                out.push('\n');
            };
            group(
                "Unfinished input -- the line was never completed, so nothing was refused",
                "incomplete: ",
                &mut out,
            );
            group(
                "Refused at parse -- valid shell the spine does not own; legacy runs it",
                "refused: ",
                &mut out,
            );
            group(
                "Malformed -- the parser located a real syntax error",
                "invalid: ",
                &mut out,
            );
            group(
                "Declined at lowering -- parsed cleanly, not lowerable",
                "unlowerable: ",
                &mut out,
            );
        }
        out.push('\n');

        out.push_str(&format!(
            "Not exercised:     {:>7}  {}\n",
            self.missing_capability,
            pct(self.missing_capability)
        ));
        let mut section = |title: &str, examples: &[String]| {
            if !examples.is_empty() {
                out.push_str(title);
                out.push('\n');
                for ex in examples {
                    out.push_str(&format!("  {ex}\n"));
                }
                out.push('\n');
            }
        };
        section(
            "Safe-improvement examples:",
            &self.safe_improvement_examples,
        );
        section("Feature-gap examples:", &self.feature_gap_examples);
        section(
            "Not-exercised examples (spine supports these; audit withholds the runner):",
            &self.missing_capability_examples,
        );
        section(
            "Unexpected examples (INVESTIGATE):",
            &self.unexpected_examples,
        );
        section("No-spine-plan examples:", &self.spine_parse_error_examples);

        // A single boolean throws away the context this audit exists to produce. Report the
        // evidence and a recommendation; leave the decision to informed judgement.
        out.push_str("Migration assessment\n\n");
        out.push_str(
            "Scope: compares the legacy and spine PARSERS on the same input string. The live\n               shell expands variables, command substitutions, globs, and aliases BEFORE the\n               legacy parser runs; those phases are not modelled here.\n\n",
        );
        out.push_str(&format!(
            "Language feature gaps:   {}\n",
            if self.feature_gap == 0 {
                "none".to_string()
            } else {
                self.feature_gap.to_string()
            }
        ));
        out.push_str(&format!("Unexpected differences:  {}\n", self.unexpected));
        // ⭐ THE SHAPE BREAKDOWN, and it is the whole point: a total says investigate everything,
        // a grouping says fix one thing. Sorted biggest-first so the build order reads off the top.
        if !self.unexpected_by_shape.is_empty() {
            let mut rows: Vec<_> = self.unexpected_by_shape.iter().collect();
            rows.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            out.push_str("  by shape:\n");
            for (shape, count) in rows {
                out.push_str(&format!("    {count:>6}  {shape}\n"));
                // The instances belong UNDER their count, not in a separate pile. A flat example
                // list mixed every shape together, so the largest group could only be sampled by
                // chance -- 43 of 59 rows were one shape and the ten printed examples spanned all
                // of them. Three per shape is enough to tell debris from a real gap by reading.
                if let Some(examples) = self.unexpected_examples_by_shape.get(shape) {
                    for ex in examples.iter().take(3) {
                        out.push_str(&format!("            {ex}\n"));
                    }
                }
            }
        }
        if self.spine_parse_error > 0 {
            out.push_str(&format!(
                "Rows with no spine plan: {} (outside the comparison domain -- not a language\n  gap. The stage breakdown above splits them: unfinished input, deliberate refusals,\n  and genuinely malformed lines are three different facts)\n",
                self.spine_parse_error
            ));
        }
        out.push('\n');
        // ⚠️ THIS ASKED A QUESTION THAT WAS ANSWERED MONTHS AGO. The line read manual review
        // before ENABLING spine execution -- but the spine executes by default and NSH_SPINE=0
        // is the escape hatch, not the switch. It was reporting readiness for a flip that had
        // already happened.
        //
        // Worse, it could never say otherwise: flip_ready wanted feature_gap == 0, and 62 of
        // the 158 gaps are forest value pipelines that LEGACY IS MEANT TO OWN. A condition
        // that cannot be satisfied is not a gate, it is a constant.
        //
        // ⭐ THE LIVE QUESTION IS WHETHER LEGACY CAN BE DELETED, and it has a different shape:
        // not does the spine handle everything, but does anything still NEED legacy. The
        // feature gaps answer it directly -- each one is a line legacy runs and the spine
        // cannot -- so they are reported as what they are rather than as a blocker to a
        // decision already made.
        // ⚠️ THIS TOTAL USED TO COUNT KINDS THIS FILE HAD ALREADY RULED DELIBERATE.
        // The sentence offers two exits -- reach zero, OR rule each remaining kind -- and then
        // added the ruled kinds to the number anyway, so the second exit could never register.
        // That is the same defect the comment above diagnoses one screen up: a condition that
        // cannot be satisfied is not a gate, it is a constant.
        //
        // The knowledge was already here. The dispatch at the top of this file says it outright:
        // "A forest pipeline is declined FOREVER (legacy's apply_pipeline is the only
        // implementation of those verbs); a shell pipeline is declined only until execution
        // lands." The total did not use its own distinction.
        //
        // RULED KINDS, each tied to where the ruling is written:
        //   forest value pipeline   -- migrate_audit.rs, dispatch comment above: legacy owns the
        //                              forest verbs; apply_pipeline is their only implementation.
        //   bare assignment         -- plan.rs, lower_command: an ExecutionPlan describes ONE
        //                              PROCESS and a bare assignment is a session statement, so
        //                              it "cannot be represented rather than merely being
        //                              unimplemented". This kind can never reach zero.
        //
        // Matched on the kind strings because those strings are already load-bearing -- they are
        // the keys of declined_by_reason, so a rename already changes the report. A marker on
        // LowerError would survive renaming and is the better fix when someone touches that enum.
        const RULED_DELIBERATE: [&str; 2] = [
            "unlowerable: forest value pipeline (legacy owns these)",
            "unlowerable: bare assignment with no command (a shell statement, not a process)",
        ];
        let ruled: usize = RULED_DELIBERATE
            .iter()
            .filter_map(|k| self.declined_by_reason.get(*k))
            .sum();
        let total = self.feature_gap + self.spine_unlowerable;
        let blocking = total.saturating_sub(ruled);

        out.push_str(&format!(
            "Legacy still owns {total} lines in this corpus: {ruled} of ruled kinds and
  {blocking} not yet ruled. THE {blocking} ARE THE BLOCKER -- the ruled kinds have taken the
  second exit this sentence offers and are not waiting on anything.
"
        ));
        if blocking == 0 {
            out.push_str(
                "  Every remaining kind is ruled deliberate. Nothing in this corpus needs legacy.
",
            );
        }
        out
    }
}

fn push_example(bucket: &mut Vec<String>, cmd: &str) {
    const MAX: usize = 10;
    const MAX_LEN: usize = 100;
    if bucket.len() < MAX {
        // Truncate on a char boundary -- an example is for direction, and some history rows are
        // entire flattened scripts that make the report unreadable.
        let shown: String = cmd.chars().take(MAX_LEN).collect();
        if shown.len() < cmd.len() {
            bucket.push(format!("{shown}..."));
        } else {
            bucket.push(shown);
        }
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

    #[test]
    fn equivalent_and_improvement_and_gap_tally() {
        let mut audit = MigrationAudit::new();
        // equivalent
        audit.observe(AuditObservation {
            source: "git add -A",
            legacy: plan(&["git", "add", "-A"]),
            spine: Ok(plan(&["git", "add", "-A"])),
        });
        // safe improvement (legacy lowercased argv[0])
        audit.observe(AuditObservation {
            source: "GitHub clone",
            legacy: plan(&["github", "clone"]),
            spine: Ok(plan(&["GitHub", "clone"])),
        });
        // feature gap (spine couldn't lower)
        audit.observe(AuditObservation {
            source: "cat f | grep x",
            legacy: plan(&["cat", "f", "|", "grep", "x"]),
            spine: Err(LowerError::UnsupportedConstruct {
                kind: "pipeline",
                span: super::super::ast::Span::new(0, 1),
            }),
        });
        let r = audit.finish();
        assert_eq!(r.compared, 3);
        assert_eq!(r.equivalent, 1);
        assert_eq!(r.safe_improvement, 1);
        assert_eq!(r.feature_gap, 1);
        assert_eq!(r.unexpected, 0);
    }
}
