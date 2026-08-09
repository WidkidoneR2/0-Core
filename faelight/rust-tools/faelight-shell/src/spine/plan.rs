//! INT-169 spine: the ExecutionPlan -- the executor's boundary. See docs/rfc-169-parser-spine.md.
//!
//! ★ EXECUTIONPLAN CHECKPOINT (2026-07-21, Increment 7): the contract the executor consumes.
//! FROZEN: ExecutionPlan is the executor boundary; argv uses OsString (a shell hits non-UTF8
//! paths -- /tmp/\xfffile -- and Rust's process APIs speak OsString); cwd is explicit;
//! environment is explicit (Inherit vs Replace -- `FOO=bar cmd` and `env -i cmd` are DIFFERENT
//! intents a map would erase); the AST does NOT execute directly. RESERVED: the IO graph
//! (redirects, pipelines, fd wiring) -- `io: IoPlan` acknowledges the concern WITHOUT inventing
//! its shape (naming stdin/stdout/stderr as Option<File> would bake in the wrong abstraction
//! level -- a pipeline's IO is a graph, not three handles).
//!
//! PHASE MODEL: source -> lexer -> AST -> EXPANSION -> ExecutionPlan -> executor. Expansion is
//! a LANGUAGE feature, not an execution feature (the AST knows Variable("HOME"); expansion
//! decides "/home/user"; lowering decides argv -- three responsibilities). Today lower()
//! performs literal expansion INLINE via expand_word(), behind a boundary that stays clean
//! when variables arrive -- no ExpandedAst type needed yet.
//!
//! WHY Result: a valid parse must not panic and must not fabricate a lying plan for a construct
//! it can't execute yet -- so lower() honestly reports the CAPABILITY BOUNDARY. Deeper: a
//! pipeline (`cat f | grep x`) is NOT one ExecutionPlan -- it's a graph (Plan A -pipe- Plan B),
//! so "one AST node -> one plan" is false in general. Result leaves room to grow into an
//! ExecutionGraph later without a fallible->infallible ripple or a lie. UnsupportedConstruct is
//! a capability boundary, not a fault: it lets the PARSER run ahead of the EXECUTOR during
//! development (parse a pipeline at step 6 before anything can execute it).

use std::ffi::OsString;
use std::path::PathBuf;

use super::ast::{
    AstNode, Command, QuoteContext, Span, Spanned, SpecialParam, VariableSyntax, Word, WordPart,
};

/// The executor's input: an intent-to-run-in-context, not a bare argv. Even when cwd/env/io
/// are empty today, the contract acknowledges that execution IS contextual (what directory,
/// what environment, what IO wiring).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    /// The resolved command + arguments, in order -- what actually gets spawned. Post-expansion
    /// (OsString, not Word): by plan-time, expansion is done.
    pub argv: Vec<OsString>,
    /// Working directory. None = inherit the current one.
    pub cwd: Option<PathBuf>,
    /// Environment intent (see Environment). Inherit today.
    pub env: Environment,
    /// IO graph -- RESERVED. Simple (no redirects/pipes) today.
    pub io: IoPlan,
}

/// Environment intent. A shell distinguishes augmenting the inherited env (`FOO=bar cmd`) from
/// replacing it (`env -i cmd`); a flat map erases that. Reserved-shape today: always Inherit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    /// Inherit the parent environment (optionally with overrides, once assignments land).
    Inherit,
    /// Replace the environment entirely with exactly these pairs (`env -i` semantics).
    Replace(Vec<(OsString, OsString)>),
}

/// The IO graph -- RESERVED. Redirects, pipelines, and fd wiring get designed at roadmap steps
/// 5/6 (they depend on AST Redirect + Pipeline, which are themselves reserved/unbuilt).
///
/// Today there are two EXECUTION INTENTS, and Capture is deliberately not a redirect: it does not
/// consume the reserved design space above, because nothing about it describes an fd graph.
/// Where a command's stderr goes once the redirects have been read in order.
///
/// ★ RESOLVED, NOT RECORDED. `2>&1` means "wherever stdout points AT THIS POINT IN THE LINE", so
/// lowering walks left to right and emits `Stdout` only when stdout was ALREADY redirected. That is
/// what makes `cmd > out 2>&1` (both to the file) differ from `cmd 2>&1 > out` (stderr to the
/// terminal) without the plan carrying an ordered list -- the ordering is consumed by the phase
/// that reads the syntax, which is where facts about what was written belong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StderrTarget {
    /// Untouched -- the shell's own stderr.
    Inherit,
    /// Its own file. `(path, append)`, same shape as stdout.
    File(std::path::PathBuf, bool),
    /// Share stdout's destination. The executor must DUP, never re-open.
    Stdout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IoPlan {
    /// No redirects, no pipe wiring -- stdin/stdout/stderr inherited from the shell.
    Simple,
    /// INT-169 blocker 4: stdout becomes a VALUE for whoever asked for the execution, rather than
    /// going to the terminal. Command substitution is the caller.
    ///
    /// ⚠️ stderr stays INHERITED, which is what a shell does: `$(cmd)` captures stdout while
    /// errors still reach the terminal. Piping it would swallow them.
    ///
    /// ⚠️ The trailing newline is NOT stripped here. `$(pwd)` yielding a path without its newline
    /// is command-substitution SEMANTICS and belongs to the phase that asked; this variant means
    /// only "stdout is a value".
    Capture,
    /// INT-200: stdin and/or stdout attached to FILES. Paths are already EXPANDED -- `> $LOG`
    /// and `> ~/out.txt` are resolved during lowering, because a redirect target is a word like any
    /// other and the executor must never see an unexpanded one.
    ///
    /// ⚠️ Both optional and both in one variant, because `cmd < in > out` is one command with two
    /// redirects. Repeats are LAST-ONE-WINS, matching bash: `> a > b` truncates both and writes b.
    ///
    /// ⚠️ stderr is deliberately absent. `2>` needs a file descriptor, the lexer does not tokenise
    /// it as one unit, and the parser refuses that shape -- so there is no way to reach here with a
    /// stderr redirect, and inventing a field for it would be room for a phase that does not exist.
    Files {
        stdin: Option<std::path::PathBuf>,
        /// (path, append) -- `>>` is the same wiring with a different open mode.
        stdout: Option<(std::path::PathBuf, bool)>,
        /// INT-200: where stderr goes, RESOLVED. Three states rather than an Option because
        /// `cmd > f 2>&1` must open `f` ONCE and DUP the descriptor -- opening it twice gives
        /// two independent file offsets and the streams overwrite each other.
        stderr: StderrTarget,
    },
}

/// Why an AST node could not be lowered to a single ExecutionPlan. Carries a span -- the AST's
/// promise is that errors point back to source, and lowering keeps that promise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowerError {
    /// A validly-parsed construct the executor cannot run YET (a capability boundary, not a
    /// fault -- the parser is allowed to run ahead of the executor).
    UnsupportedConstruct { kind: &'static str, span: Span },
    /// A construct the spine DOES implement, refused because this lowering environment did
    /// not supply the capability it needs. INT-169: distinct from the above, and the
    /// distinction is the whole point -- `spine migrate` lowers with no runner on purpose,
    /// since measuring parse capability must not execute twenty-five thousand historical
    /// substitutions. Reporting those as unsupported constructs made the migration look less
    /// complete than it is: eighty command substitutions counted as language the spine lacks,
    /// when the spine has had them since blocker 4 and the audit simply withheld the runner.
    ///
    /// ⚠️ Only the RUNNER errors. An absent VarResolver renders the source form and an absent
    /// GlobResolver keeps the pattern as text, because both can degrade honestly -- a
    /// substitution's VALUE cannot be invented, so there is nothing to fall back to.
    MissingCapability { kind: &'static str, span: Span },
    /// A construct that lowered to something the executor would reject. May never be
    /// constructed; the span is kept because the type should embody the source-pointing promise.
    InvalidPlan { message: String, span: Span },
}

/// What the expansion phase needs from the world, and NOTHING else.
///
/// ★ plan.rs stays PURE: it knows the CAPABILITY it needs, not where the values live. It never
/// sees a HashMap, a ForestDb, or the REPL loop. The caller implements this over whatever it
/// has -- fsh's session `shell_vars` plus `std::env` plus the loop's exit code. Shell variables
/// are process/session state, NOT persistent forest knowledge, which is exactly why they must
/// not be pushed into ForestDb to solve reachability.
///
/// Semantics must MATCH legacy expand_vars exactly (main.rs): session vars first, then process
/// env, and an UNSET variable expands to the EMPTY STRING (`unwrap_or_default`), which is not
/// the same as "no environment available" -- see LowerContext.
pub trait VarResolver {
    /// `None` means UNSET -- the caller renders that as an empty string, per shell semantics.
    fn resolve(&self, name: &str) -> Option<String>;
    /// For `$?`. Recognition of `$?` is not implemented yet; this is here so the contract is
    /// complete when the scanner learns it.
    fn last_exit(&self) -> Option<i32>;
    /// For `$$`. Same note as last_exit.
    fn pid(&self) -> u32;
}

/// The capability to RUN a nested command and receive its output as a value.
///
/// ★ Same inversion as VarResolver, for the same reason: plan.rs knows the capability it needs,
/// not where the values live. Running a command needs a process, a database and the REPL's world,
/// and this file must see none of them. The caller implements this over whatever it has.
///
/// ⚠️ Takes an `ExecutionPlan`, never source text. A runner accepting a string would rebuild the
/// string-reinspection architecture the spine exists to remove -- the caller has already decided
/// what to run, and the plan IS that decision.
///
/// ⚠️ Returns plan.rs-owned types only. A `CommandResult` here would drag a commands-layer type
/// into the pure phase and defeat the boundary this trait exists to protect.
pub trait CommandRunner {
    /// `Err` carries a message the caller may render. It means the execution FAILED, which is a
    /// different fact from the capability being absent -- see `LowerContext.runner`.
    fn run_capture(&self, plan: &ExecutionPlan) -> Result<String, String>;
}

/// One position in a pathname pattern.
///
/// ★ STRUCTURED, NOT AN ESCAPED STRING, and the reason is the matcher: it understands `*` and `?`
/// and NOTHING else -- no escape sequences at all. Building a pattern string with quoted
/// metacharacters backslash-escaped would invent an escape language it never had, and `\*` would
/// read as a backslash followed by a wildcard. Here quoting is expressed by WHICH VARIANT a
/// character becomes, which is the same principle as the rest of this blocker: represent the
/// distinction rather than encode it in text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobPart {
    /// Matches exactly this character. A `*` that arrived from quoted text lands here.
    Literal(char),
    /// `?` -- exactly one character.
    Any,
    /// `*` -- zero or more characters.
    Many,
}

/// The capability to match a pathname pattern against the filesystem.
///
/// ★ Third of the same shape, after VarResolver and CommandRunner, which is what makes the seam a
/// pattern rather than something invented once. plan.rs knows the capability, never `read_dir`.
pub trait GlobResolver {
    /// Sorted matches, or empty when nothing matched. The CALLER decides what an empty result
    /// means -- this reports what the filesystem said, not what the shell should do about it.
    fn expand(&self, pattern: &[GlobPart]) -> Vec<OsString>;
}

/// What `lower` is allowed to consult. Explicit rather than ambient.
#[derive(Default, Clone, Copy)]
pub struct LowerContext<'a> {
    /// `None` = NO ENVIRONMENT AVAILABLE, so variables render in SOURCE FORM (`$NAME`).
    ///
    /// ⚠️ This is NOT the same as "unset". An unset variable expands to an empty string; an
    /// absent environment means expansion cannot be performed at all. `spine migrate` replays
    /// historical commands with no shell state, and its comparison is explicitly PARSER
    /// equivalence -- expanding there would make every variable-using command diverge from the
    /// legacy plan purely as an artifact of the audit's own fidelity gap.
    pub vars: Option<&'a dyn VarResolver>,
    /// `None` = NO EXECUTION CAPABILITY, so a command substitution cannot be performed at all.
    ///
    /// ⚠️ Exactly the distinction `vars` draws: this is not "the command failed", it is "this
    /// lowering environment cannot run commands". `spine migrate` replays historical commands with
    /// no shell and must never execute one as a side effect of an audit; the parser-fidelity
    /// comparison would also be meaningless if replaying history ran processes.
    pub runner: Option<&'a dyn CommandRunner>,
    /// `None` = NO FILESYSTEM EXPANSION AVAILABLE.
    ///
    /// ⚠️ The audit is why this is a capability at all. `spine migrate` replays ~25,000 historical
    /// commands; a line containing `*.rs` must not read the current directory, or the result would
    /// depend on WHEN AND WHERE the audit ran rather than on what the command was -- and an audit
    /// would perform thousands of directory reads as a side effect of measuring.
    pub glob: Option<&'a dyn GlobResolver>,
}

/// Expand one AST word to the argv entries it produces -- ZERO, ONE, or MANY.
///
/// ★ The cardinality is the shell's actual algorithm, not a concession to globbing. `*.rs`
/// becomes several words, a glob matching nothing becomes none, and brace expansion and
/// sequences will want the same shape. Returning one OsString encoded an assumption that held
/// only while every expansion was one-to-one.
///
/// TODAY every word still yields exactly one entry, so argv is byte-identical -- the
/// representation changes here and the behaviour changes in the pathname-expansion step.
///
/// Historical note: a word was all-Literal, so this concatenates
/// the literal parts. This is the EXPANSION seam -- when WordPart::Variable lands (step 3),
/// `$HOME` resolves HERE, and the lowering boundary above does not change. The match is
/// exhaustive so the compiler flags this function the moment a new WordPart variant exists.
fn expand_word(word: &Word, ctx: &LowerContext) -> Result<Vec<OsString>, LowerError> {
    let mut out = String::new();
    // Char indices in `out` where an UNQUOTED metacharacter landed. Recording positions rather
    // than building the pattern inline is what lets every other arm stay untouched: a variable,
    // special parameter or substitution appends to `out` as before, and its characters reach the
    // pattern automatically as literals. Building the pattern in this arm alone would have made
    // `$HOME/*` produce a pattern of just `/*`, matching against the wrong thing entirely.
    let mut active: Vec<usize> = Vec::new();
    for part in &word.parts {
        match part {
            // Two accumulators, walked together: `out` is the literal text, `pattern` is the
            // same characters with metacharacters ACTIVE only where they were unquoted. A `*`
            // from `'*'` or `"*"` becomes GlobPart::Literal, so it can never match a filename.
            WordPart::Literal { text, quoted } => {
                // INT-169 blocker 6: TILDE, and it belongs HERE rather than in an exclusion. The
                // router claimed `echo ~` and printed a literal tilde, because the spine had no
                // tilde phase while legacy expands one below the routing point. The command
                // succeeded against the wrong path -- a semantic regression, not a crash.
                //
                // The rule is POSITION-SENSITIVE, which is why this cannot be a character scan
                // over the joined text: `~` expands at WORD START only (`a~b` is literal), when
                // UNQUOTED, and only before `/` or end of word. `out.is_empty()` is what word
                // start means here, since `out` accumulates across every part.
                //
                // HOME comes from the SAME resolver as `$HOME`. Never `std::env::var` -- with no
                // resolver the tilde stays literal and the audit stays deterministic, exactly as
                // variables and globs already behave.
                let mut rest: &str = text;
                if out.is_empty()
                    && *quoted == QuoteContext::Unquoted
                    && (text == "~" || text.starts_with("~/"))
                {
                    if let Some(home) = ctx.vars.and_then(|r| r.resolve("HOME")) {
                        // Pushed WITHOUT recording glob-active positions: an expanded home path
                        // is a value, not a pattern.
                        out.push_str(&home);
                        rest = &text[1..];
                    }
                }
                for ch in rest.chars() {
                    if *quoted == QuoteContext::Unquoted && (ch == '*' || ch == '?') {
                        active.push(out.chars().count());
                    }
                    out.push(ch);
                }
            }
            // INT-169 blocker 4: a CAPABILITY BOUNDARY, which is why this function became
            // fallible. The parser understands `$(...)` and the AST preserves it; the executor
            // cannot run one yet. Rendering it back to text would be the one thing that must not
            // happen -- it would make the parser's guarantee false and let a later stage run
            // something other than what was written. The `-> OsString` signature encoded an
            // assumption that held only while every WordPart was expandable.
            WordPart::CommandSub(node) => {
                // The capability, not the outcome. `None` means this lowering environment cannot
                // run commands at all -- an audit replaying history, or the raw diagnostic door.
                // That is a different fact from a command that ran and failed, which arrives below
                // as InvalidPlan.
                let runner = ctx.runner.ok_or(LowerError::MissingCapability {
                    kind: "command substitution",
                    span: node.span,
                })?;
                // Lowered through the substitution entry point, so the plan ASKS for capture
                // rather than being lowered normally and retrofitted. The IO intent originates
                // from the syntax that needs it.
                let nested = lower_substitution(node, ctx)?;
                let captured =
                    runner
                        .run_capture(&nested)
                        .map_err(|message| LowerError::InvalidPlan {
                            message,
                            span: node.span,
                        })?;
                // Trailing newlines are stripped HERE, not by IoPlan::Capture. The plan says only
                // that stdout is a value; that a shell drops the newline from `$(pwd)` is command
                // substitution SEMANTICS, and this is the phase that owns them.
                out.push_str(captured.trim_end_matches('\n'));
            }
            // RECOGNITION MILESTONE: a variable SITE is recognised but NOT evaluated, so it
            // renders back to its source form and argv is byte-identical to before. This keeps
            // the migration audit stable while the AST becomes structurally richer. The
            // variable-expansion milestone replaces this with real resolution -- and THAT is
            // when the audit is expected to move.
            // Special parameters are the shell's own values, not name lookups -- each has its
            // own resolver method. With no environment they render back to source form, exactly
            // like Variable, so the audit stays comparable.
            WordPart::SpecialParam(sp) => match ctx.vars {
                Some(r) => match sp {
                    SpecialParam::LastExit => out.push_str(&r.last_exit().unwrap_or(0).to_string()),
                    SpecialParam::Pid => out.push_str(&r.pid().to_string()),
                },
                None => out.push_str(match sp {
                    SpecialParam::LastExit => "$?",
                    SpecialParam::Pid => "$$",
                }),
            },
            WordPart::Variable { name, syntax } => match ctx.vars {
                // Resolve, matching legacy expand_vars: session vars then process env, and an
                // UNSET name expands to the empty string.
                Some(r) => out.push_str(&r.resolve(name).unwrap_or_default()),
                // No environment: render the SOURCE FORM back out, so argv is byte-identical to
                // the pre-expansion behaviour and the migration audit stays comparable.
                // ⚠️ Reproduce what was WRITTEN. This said "the source form" while emitting
                // `$NAME` for BOTH spellings, so every `${NAME}` in real history counted as a
                // divergence against a legacy tokenizer that had simply kept the text.
                None => match syntax {
                    VariableSyntax::Bare => {
                        out.push('$');
                        out.push_str(name);
                    }
                    VariableSyntax::Braced => {
                        out.push_str("${");
                        out.push_str(name);
                        out.push('}');
                    }
                },
            },
        }
    }
    // ⚠️ Only LITERAL parts can contribute an active metacharacter. A variable, special
    // parameter or substitution whose VALUE contains `*` does not glob here, though a real shell
    // would expand `x='*'; echo $x`. Conservative on purpose: the spine does not pretend to
    // understand what it has not built, and the divergence is stated rather than discovered.
    if !active.is_empty() {
        if let Some(resolver) = ctx.glob {
            let pattern: Vec<GlobPart> = out
                .chars()
                .enumerate()
                .map(|(i, ch)| match (active.contains(&i), ch) {
                    (true, '*') => GlobPart::Many,
                    (true, '?') => GlobPart::Any,
                    _ => GlobPart::Literal(ch),
                })
                .collect();
            let matches = resolver.expand(&pattern);
            // ⚠️ A pattern that matched NOTHING keeps the word as written. Returning the empty
            // result directly would DELETE the argument, which is what `spine-exec echo
            // nomatch*.xyz` did before this line existed -- it printed nothing at all. The trait
            // says the caller decides what empty means, and this is the caller deciding: bash's
            // default is nullglob off, so an unmatched pattern is ordinary text.
            if !matches.is_empty() {
                return Ok(matches);
            }
        }
    }
    Ok(vec![OsString::from(out)])
}

/// Lower a parsed AST node to the executor's ExecutionPlan. Spine side (the legacy adapter
/// comes later; comparison is only meaningful once BOTH sides speak plans).
///
/// Command -> Ok(plan). Future constructs (Pipeline, Sequence) -> Err(UnsupportedConstruct)
/// until the ExecutionGraph / IO model exists -- honest, not a panic and not a fake plan.
pub fn lower(ast: &Spanned<AstNode>, ctx: &LowerContext) -> Result<ExecutionPlan, LowerError> {
    lower_with_io(ast, ctx, IoPlan::Simple)
}

/// Lower a COMMAND SUBSTITUTION: the same lowering, asking for captured output.
///
/// ★ A separate entry point rather than a flag on LowerContext, because the context describes what
/// capabilities are AVAILABLE while this describes what the SYNTAX requires. A context flag would
/// also apply to the outer command, claiming the whole line captures.
/// Does any stage after the first name a forest VALUE verb? ★ Shared by `lower_pipeline` and the
/// refusal arm so the ownership question has one answer, not two.
///
/// Stage 0 is skipped because it PRODUCES the value -- it is a command even in a forest pipeline,
/// which is exactly why `value::parse_pipeline` skips it too.
#[cfg(test)]
mod forest_pipeline_tests {
    use super::super::parser::parse;
    use super::*;

    fn is_forest(src: &str) -> bool {
        match parse(src).expect_complete("parses").node {
            AstNode::Pipeline(pl) => is_forest_pipeline(&pl),
            other => panic!("not a pipeline: {:?}", other),
        }
    }

    #[test]
    fn a_query_verb_after_a_shell_command_is_shell() {
        // INT-201 regression: `sort` is in VALUE_VERBS, so this was called a forest pipeline,
        // refused forever by the spine, and -- once the legacy pipeline executor was deleted --
        // refused outright. Every pipeline ending in sort stopped working.
        assert!(!is_forest("echo a | sort -k1 -rn"));
        assert!(!is_forest("echo b | sort"));
        assert!(!is_forest("git log | head -3"));
    }

    #[test]
    fn a_query_verb_after_a_source_is_a_query() {
        assert!(is_forest("ps | sort cpu desc"));
        assert!(is_forest("tools | first 3"));
        assert!(is_forest("intents | where status = open"));
    }

    #[test]
    fn a_comparison_never_reaches_this_predicate() {
        // `ps | where cpu > 0.5` is refused by the PARSER, not here: the digit guard reports
        // ComparisonNotRedirect because the shell grammar sees `>` as a redirect whose target is a
        // number. Recorded as a test rather than left out, because the two declines look identical
        // from the prompt -- both end at the query executor -- and merging them in one's head is
        // how this predicate gets blamed for a decision it never made.
        assert!(parse("ps | where cpu > 0.5").is_complete() == false);
    }

    #[test]
    fn a_source_without_a_query_verb_is_shell() {
        // Starting at a source is necessary, not sufficient -- `ps | grep brave` is a shell pipe.
        assert!(!is_forest("ps | grep brave"));
    }
}

fn is_forest_pipeline(pl: &super::ast::Pipeline) -> bool {
    // A LANGUAGE IS IDENTIFIED BY WHERE IT STARTS, not by a word appearing in the middle. Asking
    // only "does a later stage name a value verb" made `echo a | sort -k1 -rn` a forest pipeline,
    // because `sort` is in the vocabulary -- so the spine refused it forever, as it should refuse a
    // real query, and the line reached legacy. While legacy still had an inline pipeline executor
    // that was invisible; once INT-201 deleted it the line was refused outright, and every pipeline
    // ending in `sort` stopped working. Found in ordinary use the day after.
    //
    // The source test is what the two-languages design already implies: `ps | sort cpu desc` is a
    // query, `echo a | sort -k1 -rn` is a shell pipeline that happens to contain the same word.
    let starts_at_source = matches!(
        pl.stages
            .first()
            .and_then(|st| st.node.words.first())
            .map(|w| w.node.parts.as_slice()),
        Some([WordPart::Literal { text, .. }]) if crate::value::is_value_source(text)
    );
    if !starts_at_source {
        return false;
    }
    pl.stages.iter().skip(1).any(|st| {
        // A non-literal first word (`$CMD | where x`) is NOT treated as a verb: it cannot be known
        // here, and a computed verb is not something the forest DSL accepts either.
        matches!(
            st.node.words.first().map(|w| w.node.parts.as_slice()),
            Some([WordPart::Literal { text, .. }]) if crate::value::is_value_verb(text)
        )
    })
}

/// Lower a PIPELINE into one plan per stage. INT-200.
///
/// ★ THE VEC'S ORDER IS THE WIRING. `ExecutionPlan` describes exactly one process, and rather than
/// stretch it into a graph, the sequence carries the composition -- stage N's stdout feeds stage
/// N+1's stdin. A single command yields a one-element vector, so the caller has one shape to handle.
///
/// ⚠️ A FOREST PIPELINE IS REFUSED FOREVER, not until some later increment. `where`, `sort` and
/// `first` are query verbs with no programs behind them; claiming one would spawn `where` and fail.
///
/// ⚠️ Each stage lowers with `IoPlan::Simple`, so its OWN redirects still apply -- the executor
/// treats pipe wiring as a DEFAULT that a stage's redirect overrides, which is what bash does.
pub fn lower_pipeline(
    ast: &Spanned<AstNode>,
    ctx: &LowerContext,
) -> Result<Vec<ExecutionPlan>, LowerError> {
    match &ast.node {
        // A backgrounded node is not a pipeline stage. `a | b &` backgrounds the WHOLE
        // pipeline, so the wrapper must be removed by the caller before stages are lowered --
        // never flattened into one, which would background a single stage and leave the rest.
        AstNode::Background(inner) => Err(LowerError::UnsupportedConstruct {
            kind: "background (unwrap it before lowering)",
            span: inner.span,
        }),
        AstNode::Command(cmd) => Ok(vec![lower_command(ast.span, cmd, ctx, IoPlan::Simple)?]),
        // A sequence is not a pipeline and never becomes one: `a && b` is two executions
        // decided by exit status, not two processes sharing a pipe.
        AstNode::Sequence(seq) => Err(LowerError::UnsupportedConstruct {
            kind: "sequence",
            span: seq.first.span,
        }),
        AstNode::Pipeline(pl) => {
            if is_forest_pipeline(pl) {
                return Err(LowerError::UnsupportedConstruct {
                    kind: "forest value pipeline (legacy owns these)",
                    span: pl.stages.first().map(|s| s.span).unwrap_or(ast.span),
                });
            }
            pl.stages
                .iter()
                .map(|st| lower_command(st.span, &st.node, ctx, IoPlan::Simple))
                .collect()
        }
    }
}

fn lower_substitution(
    ast: &Spanned<AstNode>,
    ctx: &LowerContext,
) -> Result<ExecutionPlan, LowerError> {
    lower_with_io(ast, ctx, IoPlan::Capture)
}

/// The shared lowering. The two entry points above differ by exactly the thing that differs.
/// Lower ONE command. Split out of `lower_with_io` so a pipeline can lower each stage:
/// stages are `Spanned<Command>` while the entry points take `Spanned<AstNode>`, so they
/// cannot call each other directly.
fn lower_command(
    span: Span,
    cmd: &Command,
    ctx: &LowerContext,
    io: IoPlan,
) -> Result<ExecutionPlan, LowerError> {
    // INT-200, STEP 1 OF 2: the parser now BUILDS redirects; lowering cannot yet
    // execute them. Refusing here is not a formality -- without it a parsed redirect would
    // lower to IoPlan::Simple and the command would run with the redirect SILENTLY
    // DROPPED: output to the terminal, no file, no error. That is the exact
    // silent-corruption shape two of today's bugs had.
    //
    // ★ The refusal is also the correct BEHAVIOUR meanwhile: the router declines, legacy
    // runs the redirect as it always has, and `spine migrate` moves these lines from
    // "operator RedirectOut" into the feature-gap bucket -- which is honest, because they
    // now parse and merely cannot execute.
    // INT-200, STEP 2: redirects become an IO intent. Targets are expanded HERE, using
    // the same `expand_word` every argv word goes through, so `> $LOG` and `> ~/out.txt`
    // work by construction rather than by a second implementation.
    //
    // ★ LEFT TO RIGHT, AND THE ORDER IS THE SEMANTICS. `2>&1` copies wherever stdout points
    // AT THAT MOMENT, so this loop resolves it against the stdout seen so far rather than
    // the final one.
    let mut stdin_path: Option<std::path::PathBuf> = None;
    let mut stdout_path: Option<(std::path::PathBuf, bool)> = None;
    let mut stderr = StderrTarget::Inherit;
    for r in &cmd.redirects {
        let append = matches!(r.node.op, crate::spine::ast::RedirectOp::Append);
        // Which stream is being redirected. `None` means the operator's default: stdin for
        // a read, stdout for a write.
        let fd = r.node.fd.unwrap_or(
            if matches!(r.node.op, crate::spine::ast::RedirectOp::Read) {
                0
            } else {
                1
            },
        );
        match &r.node.target {
            crate::spine::ast::RedirectTarget::Stream(n) => {
                // Only `2>&1` exists in six months of history -- zero `1>&2`, zero anything
                // else. Refusing the rest is honest: legacy already handles them via sh, and
                // building a descriptor graph for no users is the opposite of measured work.
                if fd == 2 && *n == 1 {
                    stderr = if stdout_path.is_some() {
                        StderrTarget::Stdout
                    } else {
                        // stdout is still the terminal HERE, so stderr stays there too --
                        // even if a later `> out` moves stdout. This is the case bash users
                        // trip over, and getting it wrong is silent.
                        StderrTarget::Inherit
                    };
                } else {
                    return Err(LowerError::UnsupportedConstruct {
                        kind: "descriptor duplication other than 2>&1",
                        span: r.span,
                    });
                }
            }
            crate::spine::ast::RedirectTarget::File(w) => {
                let expanded = expand_word(w, ctx)?;
                // ⚠️ AMBIGUOUS REDIRECT. `> *.txt` matching two files has no correct answer,
                // and bash refuses it by name. This is InvalidPlan rather than
                // UnsupportedConstruct ON PURPOSE: a refusal falls back to legacy, and
                // legacy would guess. A DEFECT surfaces. Picking a file to overwrite is
                // worse than stopping.
                let [one] = expanded.as_slice() else {
                    return Err(LowerError::InvalidPlan {
                        message: format!(
                            "ambiguous redirect: target expanded to {} words",
                            expanded.len()
                        ),
                        span: r.span,
                    });
                };
                let path = std::path::PathBuf::from(one);
                match fd {
                    0 => stdin_path = Some(path),
                    1 => stdout_path = Some((path, append)),
                    2 => stderr = StderrTarget::File(path, append),
                    _ => {
                        return Err(LowerError::UnsupportedConstruct {
                            kind: "redirect of a descriptor other than 0, 1 or 2",
                            span: r.span,
                        })
                    }
                }
            }
        }
    }
    // ⚠️ `$(cmd > f)` asks for capture AND a file. Both claim stdout, and the enum cannot
    // express both -- correctly, since the answer is genuinely undecided. Refuse rather
    // than silently letting one win.
    let io = if stdin_path.is_some() || stdout_path.is_some() || stderr != StderrTarget::Inherit {
        if io != IoPlan::Simple {
            return Err(LowerError::UnsupportedConstruct {
                kind: "redirect inside a capture",
                span: span,
            });
        }
        IoPlan::Files {
            stdin: stdin_path,
            stdout: stdout_path,
            stderr,
        }
    } else {
        io
    };
    // Collected, then FLATTENED: one AST word may produce several argv entries. Kept
    // explicit rather than flat_map because Result inside flat_map hides the types at
    // exactly the point a reader needs to see them.
    let argv: Vec<OsString> = cmd
        .words
        .iter()
        .map(|w| expand_word(&w.node, ctx))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();
    Ok(ExecutionPlan {
        argv,
        cwd: None,
        env: Environment::Inherit,
        io,
    })
}

fn lower_with_io(
    ast: &Spanned<AstNode>,
    ctx: &LowerContext,
    io: IoPlan,
) -> Result<ExecutionPlan, LowerError> {
    match &ast.node {
        // ⚠️ INT-200: BACKGROUND IS AN EXECUTION MODE, NOT A PLAN SHAPE. An ExecutionPlan
        // describes one FOREGROUND process the caller waits on; a background job is spawned,
        // registered with the shell's JobTable and deliberately NOT waited on. Stretching this
        // type to carry "do not wait" would put a scheduling decision inside a description of
        // what to run -- so the router unwraps Background above lowering and lowers the OPERAND.
        AstNode::Background(inner) => Err(LowerError::UnsupportedConstruct {
            kind: "background (unwrap it before lowering)",
            span: inner.span,
        }),
        // INT-200: a pipeline is not ONE process, and ExecutionPlan describes exactly one.
        // Rather than stretching the type, lowering refuses here; `lower_pipeline` will return
        // a Vec whose ORDER is the wiring -- the shape `lower_substitution` already uses for
        // `$()`. Until it exists, refusing keeps the router honest: it declines, legacy runs
        // the pipe, and no half-executed pipeline can reach a process.
        //
        // ⚠️⚠️ TWO REFUSALS THAT LOOK ALIKE AND ARE NOT. A FOREST pipeline (`ps | where cpu > 0`)
        // must be declined FOREVER -- `where`, `sort` and `first` are Christian's query verbs, not
        // programs, and `value::apply_pipeline` in legacy is their only implementation. A SHELL
        // pipeline is declined only until execution exists. Reporting both as "pipeline" would hide
        // the first inside the second and, once pipes run, the spine would try to SPAWN `where`.
        //
        // ★ The vocabulary is asked for, never copied -- `value::is_value_verb` is the single owner
        // and a drift test proves it agrees with the parser that builds the ops.
        AstNode::Pipeline(pl) => {
            // Skipped: stage 0 PRODUCES the value, so it is a command even in a forest pipeline.
            // `parse_pipeline` skips it for exactly this reason.
            let forest = pl.stages.iter().skip(1).any(|st| {
                // A non-literal first word (`$CMD | where x`) is NOT treated as a verb: it cannot
                // be known here, and a computed verb is not something the forest DSL accepts
                // either. Erring toward "shell" is safe -- the spine still refuses below.
                matches!(
                    st.node.words.first().map(|w| w.node.parts.as_slice()),
                    Some([crate::spine::ast::WordPart::Literal { text, .. }])
                        if crate::value::is_value_verb(text)
                )
            });
            Err(LowerError::UnsupportedConstruct {
                kind: if forest {
                    "forest value pipeline (legacy owns these)"
                } else {
                    "pipeline"
                },
                span: pl.stages.first().map(|s| s.span).unwrap_or(ast.span),
            })
        }
        AstNode::Command(cmd) => lower_command(ast.span, cmd, ctx, io),
        // ⚠️ A SEQUENCE CANNOT BE LOWERED HERE AND MUST NOT BE. `touch x && ls *.txt` expands
        // the second item's glob, and doing that before `touch` runs looks at a world that does
        // not exist yet. Pipeline stages are safe because they spawn together; a sequence is
        // ordered, so each item is lowered immediately BEFORE it runs -- which no function
        // returning a single plan can express. Refusing keeps the router honest meanwhile.
        AstNode::Sequence(seq) => Err(LowerError::UnsupportedConstruct {
            kind: "sequence",
            span: seq.first.span,
        }),
    }
}

impl ExecutionPlan {
    /// argv as UTF-8 strings, for the BUILTIN dispatch path only.
    ///
    /// ★ WHY THIS IS FALLIBLE AND LIVES HERE. argv is OsString because a shell must be able to
    /// SPAWN a non-UTF-8 path (`/tmp/\xfffile`) -- and execute_plan does exactly that, never
    /// touching UTF-8. Builtins are the opposite: they are string logic (comparing literals,
    /// parsing numbers, matching subcommands), and there is nothing correct a builtin can do
    /// with a non-UTF-8 argument. So the conversion is not a compromise at that boundary, it is
    /// the honest statement that argv reaching a builtin must be valid UTF-8.
    ///
    /// The SPINE owns this decision because the spine is the new structured path; the legacy
    /// path already paid the tokenization cost in String. Failure is LOUD and names the
    /// offending argument -- a silent to_string_lossy() here would become exactly the kind of
    /// hidden compatibility layer this rebuild exists to remove.
    pub fn argv_as_utf8(&self) -> Result<Vec<String>, String> {
        self.argv
            .iter()
            .map(|a| {
                a.to_str()
                    .map(str::to_string)
                    .ok_or_else(|| format!("argument is not valid UTF-8: {:?}", a))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spine::parser::parse;

    #[test]
    fn bare_command_lowers_to_argv() {
        let node = parse("git add -A").expect_complete("parses");
        let plan = lower(&node, &LowerContext::default()).expect("lowers");
        assert_eq!(
            plan.argv,
            vec![
                OsString::from("git"),
                OsString::from("add"),
                OsString::from("-A")
            ]
        );
        assert_eq!(plan.cwd, None);
        assert_eq!(plan.env, Environment::Inherit);
        assert_eq!(plan.io, IoPlan::Simple);
    }

    #[test]
    fn single_word_lowers() {
        let node = parse("pwd").expect_complete("parses");
        let plan = lower(&node, &LowerContext::default()).expect("lowers");
        assert_eq!(plan.argv, vec![OsString::from("pwd")]);
    }

    #[test]
    fn case_preserved_through_lowering() {
        // The rides-with fix survives all the way to argv: GitHub stays GitHub.
        let node = parse("GitHub Clone").expect_complete("parses");
        let plan = lower(&node, &LowerContext::default()).expect("lowers");
        assert_eq!(
            plan.argv,
            vec![OsString::from("GitHub"), OsString::from("Clone")]
        );
    }

    /// A resolver over a fixed map, matching legacy expand_vars semantics: a miss is UNSET.
    struct FakeVars(std::collections::HashMap<&'static str, &'static str>);
    impl VarResolver for FakeVars {
        fn resolve(&self, name: &str) -> Option<String> {
            self.0.get(name).map(|s| s.to_string())
        }
        fn last_exit(&self) -> Option<i32> {
            Some(0)
        }
        fn pid(&self) -> u32 {
            1234
        }
    }

    /// A runner over a fixed answer. Parallel to FakeVars: deterministic, no shell, no process.
    /// It ALSO asserts the plan it receives, because the IO intent reaching the runner is the
    /// thing argv cannot show.
    /// Records the patterns it is asked to expand. No filesystem: the thing under test is what
    /// LOWERING SENDS, not what a directory happens to contain, so this must not depend on the
    /// machine running it.
    struct FakeGlob(std::cell::RefCell<Vec<Vec<GlobPart>>>);
    impl GlobResolver for FakeGlob {
        fn expand(&self, pattern: &[GlobPart]) -> Vec<OsString> {
            self.0.borrow_mut().push(pattern.to_vec());
            vec![OsString::from("matched.txt")]
        }
    }

    /// ★ THE TILDE CONTRACT. Not a feature test -- a POSITION-AND-QUOTE test, because the router
    /// claimed `echo ~` and printed a literal tilde while legacy expanded it below the routing
    /// point. The command succeeded against the wrong path: a silent semantic regression, which is
    /// the class of bug this whole migration exists to make impossible.
    #[test]
    fn tilde_expands_only_at_word_start_and_only_unquoted() {
        let mut m = std::collections::HashMap::new();
        m.insert("HOME", "/home/test");
        let vars = FakeVars(m);
        let ctx = LowerContext {
            vars: Some(&vars),
            runner: None,
            glob: None,
        };
        for (source, want) in [
            ("echo ~", "/home/test"),
            ("echo ~/project", "/home/test/project"),
            // NOT word start -- a tilde inside a word is ordinary text.
            ("echo a~b", "a~b"),
            // Quoted -- the AST kept QuoteContext, and this is what it is for.
            ("echo \"~\"", "~"),
            ("echo \'~\'", "~"),
            // Not a home reference: `~user` is another shell's feature the spine has not built.
            ("echo ~nobody", "~nobody"),
        ] {
            let node = parse(source).expect_complete("parses");
            let plan = lower(&node, &ctx).expect("lowers");
            assert_eq!(
                plan.argv,
                vec![OsString::from("echo"), OsString::from(want)],
                "{source}"
            );
        }
    }

    /// ⚠️ NO RESOLVER MEANS NO EXPANSION -- and specifically NOT a fallback to the host
    /// environment. `spine migrate` lowers this way over real history, so reading the process HOME
    /// here would make an audit's result depend on which machine ran it.
    #[test]
    fn tilde_without_a_resolver_stays_literal() {
        let node = parse("echo ~/project").expect_complete("parses");
        let plan = lower(&node, &LowerContext::default()).expect("lowers");
        assert_eq!(
            plan.argv,
            vec![OsString::from("echo"), OsString::from("~/project")]
        );
    }

    /// The interaction that proves the SHAPE rather than the feature: the home path arrives as a
    /// VALUE while the pattern after it stays active. A tilde fix that made the whole word literal
    /// would pass every test above and break this one.
    #[test]
    fn a_tilde_prefix_does_not_disarm_a_later_glob() {
        let mut m = std::collections::HashMap::new();
        m.insert("HOME", "/home/test");
        let vars = FakeVars(m);
        let fake = FakeGlob(std::cell::RefCell::new(Vec::new()));
        let ctx = LowerContext {
            vars: Some(&vars),
            runner: None,
            glob: Some(&fake),
        };
        let node = parse("echo ~/docs/*.md").expect_complete("parses");
        lower(&node, &ctx).expect("lowers");
        let seen = fake.0.borrow();
        assert_eq!(seen.len(), 1, "the word reached the globber");
        let rendered: String = seen[0]
            .iter()
            .map(|p| match p {
                GlobPart::Literal(c) => *c,
                GlobPart::Many => '*',
                GlobPart::Any => '?',
            })
            .collect();
        assert_eq!(rendered, "/home/test/docs/*.md");
        assert!(
            matches!(seen[0][seen[0].len() - 4], GlobPart::Many),
            "the star stayed ACTIVE after tilde expansion: {rendered}"
        );
    }

    /// THE BOUNDARY DECISION, asserted on the pattern rather than the result. The matcher and the
    /// filesystem adapter may both change; what must never change is that lowering sends a
    /// STRUCTURED pattern in which quoting has already decided which characters are active.
    #[test]
    fn quoting_decides_which_characters_reach_the_globber_as_wildcards() {
        let cases = [
            ("echo *", vec![GlobPart::Many]),
            (
                "echo a?c",
                vec![
                    GlobPart::Literal('a'),
                    GlobPart::Any,
                    GlobPart::Literal('c'),
                ],
            ),
        ];
        for (source, expected) in cases {
            let fake = FakeGlob(std::cell::RefCell::new(Vec::new()));
            let ctx = LowerContext {
                vars: None,
                runner: None,
                glob: Some(&fake),
            };
            let node = parse(source).expect_complete("parses");
            let plan = lower(&node, &ctx).expect("lowers");
            assert_eq!(
                fake.0.borrow().len(),
                1,
                "{source} should reach the globber once"
            );
            assert_eq!(fake.0.borrow()[0], expected, "{source} pattern");
            assert_eq!(
                plan.argv,
                vec![OsString::from("echo"), OsString::from("matched.txt")],
                "{source} argv comes from the resolver"
            );
        }
    }

    /// A quoted metacharacter is INERT: it never reaches the globber at all, so the word cannot
    /// match a filename no matter what the directory contains.
    #[test]
    fn quoted_metacharacters_never_reach_the_globber() {
        for source in ["echo \'*\'", "echo \"*\"", "echo \'a?c\'"] {
            let fake = FakeGlob(std::cell::RefCell::new(Vec::new()));
            let ctx = LowerContext {
                vars: None,
                runner: None,
                glob: Some(&fake),
            };
            let node = parse(source).expect_complete("parses");
            let plan = lower(&node, &ctx).expect("lowers");
            assert!(
                fake.0.borrow().is_empty(),
                "{source} must not be treated as a pattern"
            );
            assert_eq!(plan.argv.len(), 2, "{source} stays one argument");
        }
    }

    /// A resolver that found NOTHING must not delete the argument either. Distinct from the test
    /// below: there the capability is absent, here it ran and matched nothing. Both keep the word,
    /// and only this one was missing -- which is why `spine-exec echo nomatch*.xyz` printed an
    /// empty line on real metal while every unit test passed.
    #[test]
    fn a_pattern_that_matches_nothing_survives_as_text() {
        struct EmptyGlob;
        impl GlobResolver for EmptyGlob {
            fn expand(&self, _pattern: &[GlobPart]) -> Vec<OsString> {
                Vec::new()
            }
        }
        let empty = EmptyGlob;
        let ctx = LowerContext {
            vars: None,
            runner: None,
            glob: Some(&empty),
        };
        let node = parse("echo nomatch*.xyz").expect_complete("parses");
        let plan = lower(&node, &ctx).expect("lowers");
        assert_eq!(
            plan.argv,
            vec![OsString::from("echo"), OsString::from("nomatch*.xyz")]
        );
    }

    /// No capability means DO NOT EXPAND -- not "drop the argument". An audit replaying history has
    /// no filesystem, and a word must survive as the text that was written.
    #[test]
    fn without_a_globber_an_active_pattern_stays_literal() {
        let node = parse("echo *.rs").expect_complete("parses");
        let plan = lower(&node, &LowerContext::default()).expect("lowers");
        assert_eq!(
            plan.argv,
            vec![OsString::from("echo"), OsString::from("*.rs")]
        );
    }

    struct FakeRunner(&'static str);
    impl CommandRunner for FakeRunner {
        fn run_capture(&self, plan: &ExecutionPlan) -> Result<String, String> {
            assert_eq!(
                plan.io,
                IoPlan::Capture,
                "a substitution must be lowered asking for capture"
            );
            Ok(self.0.to_string())
        }
    }

    /// The whole contract in one test: the nested program is lowered with capture intent, run
    /// through the capability, and its trailing newline dropped -- `$(pwd)` yields a path, not a
    /// path plus a line break.
    #[test]
    fn command_substitution_expands_through_the_runner() {
        let runner = FakeRunner("/home/christian\n");
        let ctx = LowerContext {
            vars: None,
            runner: Some(&runner),
            glob: None,
        };
        let node = parse("echo $(pwd)").expect_complete("parses");
        let plan = lower(&node, &ctx).expect("lowers");
        assert_eq!(
            plan.argv,
            vec![OsString::from("echo"), OsString::from("/home/christian")]
        );
        assert_eq!(plan.io, IoPlan::Simple, "the OUTER plan is not a capture");
    }

    #[test]
    fn no_environment_renders_variables_in_source_form() {
        // The audit's situation. argv must be byte-identical to the pre-resolver behaviour.
        let node = parse("echo $HOME").expect_complete("parses");
        let plan = lower(&node, &LowerContext::default()).expect("lowers");
        assert_eq!(
            plan.argv,
            vec![OsString::from("echo"), OsString::from("$HOME")]
        );
    }

    #[test]
    fn resolver_expands_and_unset_becomes_empty() {
        let mut m = std::collections::HashMap::new();
        m.insert("MY_VAR", "hello");
        let vars = FakeVars(m);
        let ctx = LowerContext {
            vars: Some(&vars),
            runner: None,
            glob: None,
        };

        let node = parse("echo $MY_VAR").expect_complete("parses");
        let plan = lower(&node, &ctx).expect("lowers");
        assert_eq!(plan.argv[1], OsString::from("hello"));

        // Legacy uses unwrap_or_default: an UNSET variable is the empty string, NOT the source
        // form. That distinction is the whole reason LowerContext.vars is an Option.
        let node = parse("echo $NOPE").expect_complete("parses");
        let plan = lower(&node, &ctx).expect("lowers");
        assert_eq!(plan.argv[1], OsString::from(""));
    }

    #[test]
    fn braces_are_syntax_not_semantics() {
        // ${HOME} and $HOME must produce the SAME AST -- braces are parser syntax.
        let mut m = std::collections::HashMap::new();
        m.insert("HOME", "/home/christian");
        let vars = FakeVars(m);
        let ctx = LowerContext {
            vars: Some(&vars),
            runner: None,
            glob: None,
        };

        let a = lower(&parse("echo $HOME").expect_complete("parses"), &ctx).expect("parses");
        let b = lower(&parse("echo ${HOME}").expect_complete("parses"), &ctx).expect("parses");
        assert_eq!(a.argv, b.argv);
        assert_eq!(a.argv[1], OsString::from("/home/christian"));
    }

    #[test]
    fn special_params_use_their_own_resolver_methods() {
        // $? and $$ are NOT name lookups -- FakeVars answers 0 and 1234 from last_exit()/pid(),
        // and its variable map is empty, so a Variable("?") model would have produced "".
        let vars = FakeVars(std::collections::HashMap::new());
        let ctx = LowerContext {
            vars: Some(&vars),
            runner: None,
            glob: None,
        };

        let plan = lower(&parse("echo $?").expect_complete("parses"), &ctx).expect("parses");
        assert_eq!(plan.argv[1], OsString::from("0"));

        let plan = lower(&parse("echo $$").expect_complete("parses"), &ctx).expect("parses");
        assert_eq!(plan.argv[1], OsString::from("1234"));

        // Mixed into a word, concatenated with literals.
        let plan = lower(&parse("echo exit=$?").expect_complete("parses"), &ctx).expect("parses");
        assert_eq!(plan.argv[1], OsString::from("exit=0"));
    }

    #[test]
    fn unsupported_brace_forms_stay_literal_even_when_the_name_resolves() {
        // ★ THE DELIBERATE DIVERGENCE. Legacy reads the whole brace body as a NAME, looks up a
        // variable literally called "VAR:-default", finds nothing, and substitutes EMPTY. The
        // spine does not implement default-expansion, so it does not pretend to: the text stays
        // intact. Note VAR *is* set here -- so this is not "resolver missed it", it is the
        // parser declining to claim it understood the form.
        let mut m = std::collections::HashMap::new();
        m.insert("VAR", "real");
        let vars = FakeVars(m);
        let ctx = LowerContext {
            vars: Some(&vars),
            runner: None,
            glob: None,
        };

        for src in ["echo ${VAR:-default}", "echo ${#VAR}", "echo ${VAR/a/b}"] {
            let plan = lower(&parse(src).expect_complete("parses"), &ctx).expect("parses");
            let got = plan.argv[1].to_string_lossy().to_string();
            assert!(
                got.starts_with("${"),
                "{src} must stay literal, got {got:?}"
            );
        }
    }

    #[test]
    fn lone_and_unterminated_dollars_stay_literal() {
        let vars = FakeVars(std::collections::HashMap::new());
        let ctx = LowerContext {
            vars: Some(&vars),
            runner: None,
            glob: None,
        };
        for (src, want) in [
            ("echo $", "$"),
            ("echo ${VAR", "${VAR"),
            ("echo $1", "$1"), // positional params not modelled yet
            ("echo 100$", "100$"),
        ] {
            let plan = lower(&parse(src).expect_complete("parses"), &ctx).expect("parses");
            assert_eq!(plan.argv[1], OsString::from(want), "for input {src:?}");
        }
    }

    #[test]
    fn single_quoted_dollar_is_never_expanded() {
        // The INT-172/174 bug class, made impossible by construction: inside single quotes the
        // scanner produced a Literal, so there is no Variable part for any resolver to touch.
        let mut m = std::collections::HashMap::new();
        m.insert("MY_VAR", "hello");
        let vars = FakeVars(m);
        let ctx = LowerContext {
            vars: Some(&vars),
            runner: None,
            glob: None,
        };

        let node = parse("echo '$MY_VAR'").expect_complete("parses");
        let plan = lower(&node, &ctx).expect("lowers");
        assert_eq!(
            plan.argv[1],
            OsString::from("$MY_VAR"),
            "single quotes suppress expansion"
        );

        // Double quotes DO expand, matching POSIX and legacy's INT-245 behaviour.
        let node = parse("echo \"$MY_VAR\"").expect_complete("parses");
        let plan = lower(&node, &ctx).expect("lowers");
        assert_eq!(
            plan.argv[1],
            OsString::from("hello"),
            "double quotes expand"
        );
    }

    #[test]
    fn mixed_word_concatenates_literal_and_variable() {
        let mut m = std::collections::HashMap::new();
        m.insert("BAR", "xyz");
        let vars = FakeVars(m);
        let ctx = LowerContext {
            vars: Some(&vars),
            runner: None,
            glob: None,
        };
        let node = parse("echo foo$BAR/baz").expect_complete("parses");
        let plan = lower(&node, &ctx).expect("lowers");
        assert_eq!(
            plan.argv[1],
            OsString::from("fooxyz/baz"),
            "one word, three parts"
        );
    }

    #[test]
    fn argv_as_utf8_round_trips_and_reports_bad_bytes() {
        let node = parse("git add -A").expect_complete("parses");
        let plan = lower(&node, &LowerContext::default()).expect("lowers");
        assert_eq!(
            plan.argv_as_utf8().expect("parses"),
            vec!["git", "add", "-A"]
        );

        // A non-UTF-8 argument can still be SPAWNED, but must not reach a builtin silently.
        use std::os::unix::ffi::OsStringExt;
        let bad = ExecutionPlan {
            argv: vec![OsString::from("cat"), OsString::from_vec(vec![0xff, 0xfe])],
            cwd: None,
            env: Environment::Inherit,
            io: IoPlan::Simple,
        };
        let err = bad.argv_as_utf8().expect_err("must refuse, not mangle");
        assert!(
            err.contains("not valid UTF-8"),
            "error names the problem: {err}"
        );
    }

    /// INT-169 blocker 4: the boundary in one test. PARSING SUCCEEDS -- `$(...)` is valid shell
    /// syntax and the AST carries it -- while LOWERING REFUSES, because THIS CONTEXT SUPPLIED NO
    /// RUNNER. ⚠️ Not because the executor lacks substitution: it has had one since blocker 4.
    /// The comment here used to say it could not run one yet, and the error said the same, which
    /// is how eighty command substitutions came to be reported as language the spine lacks.
    #[test]
    fn command_substitution_parses_but_does_not_lower() {
        let ast = parse("echo $(pwd)").expect_complete("valid syntax parses");
        match lower(&ast, &LowerContext::default()) {
            Err(LowerError::MissingCapability { kind, .. }) => {
                assert_eq!(kind, "command substitution");
            }
            other => panic!("expected MissingCapability, got {other:?}"),
        }
    }

    #[test]
    fn expand_word_concatenates_literals() {
        // A word is the smallest unit expansion produces; today all-literal -> its bytes.
        let w = Word::literal("/tmp/testscript.sh");
        assert_eq!(
            expand_word(&w, &LowerContext::default()).expect("parses"),
            vec![OsString::from("/tmp/testscript.sh")]
        );
    }
}
