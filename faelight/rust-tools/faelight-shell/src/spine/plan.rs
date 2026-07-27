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

use super::ast::{AstNode, Span, Spanned, SpecialParam, Word, WordPart};

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
/// 5/6 (they depend on AST Redirect + Pipeline, which are themselves reserved/unbuilt). Today a
/// single placeholder variant: Simple = no IO transformations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IoPlan {
    /// No redirects, no pipe wiring -- stdin/stdout/stderr inherited from the shell.
    Simple,
}

/// Why an AST node could not be lowered to a single ExecutionPlan. Carries a span -- the AST's
/// promise is that errors point back to source, and lowering keeps that promise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowerError {
    /// A validly-parsed construct the executor cannot run YET (a capability boundary, not a
    /// fault -- the parser is allowed to run ahead of the executor).
    UnsupportedConstruct { kind: &'static str, span: Span },
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
}

/// Expand one word to its final OsString. TODAY: a word is all-Literal, so this concatenates
/// the literal parts. This is the EXPANSION seam -- when WordPart::Variable lands (step 3),
/// `$HOME` resolves HERE, and the lowering boundary above does not change. The match is
/// exhaustive so the compiler flags this function the moment a new WordPart variant exists.
fn expand_word(word: &Word, ctx: &LowerContext) -> Result<OsString, LowerError> {
    let mut out = String::new();
    for part in &word.parts {
        match part {
            WordPart::Literal(s) => out.push_str(s),
            // INT-169 blocker 4: a CAPABILITY BOUNDARY, which is why this function became
            // fallible. The parser understands `$(...)` and the AST preserves it; the executor
            // cannot run one yet. Rendering it back to text would be the one thing that must not
            // happen -- it would make the parser's guarantee false and let a later stage run
            // something other than what was written. The `-> OsString` signature encoded an
            // assumption that held only while every WordPart was expandable.
            WordPart::CommandSub(node) => {
                return Err(LowerError::UnsupportedConstruct {
                    kind: "command substitution",
                    span: node.span,
                })
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
            WordPart::Variable(name) => match ctx.vars {
                // Resolve, matching legacy expand_vars: session vars then process env, and an
                // UNSET name expands to the empty string.
                Some(r) => out.push_str(&r.resolve(name).unwrap_or_default()),
                // No environment: render the SOURCE FORM back out, so argv is byte-identical to
                // the pre-expansion behaviour and the migration audit stays comparable.
                None => {
                    out.push('$');
                    out.push_str(name);
                }
            },
        }
    }
    Ok(OsString::from(out))
}

/// Lower a parsed AST node to the executor's ExecutionPlan. Spine side (the legacy adapter
/// comes later; comparison is only meaningful once BOTH sides speak plans).
///
/// Command -> Ok(plan). Future constructs (Pipeline, Sequence) -> Err(UnsupportedConstruct)
/// until the ExecutionGraph / IO model exists -- honest, not a panic and not a fake plan.
pub fn lower(ast: &Spanned<AstNode>, ctx: &LowerContext) -> Result<ExecutionPlan, LowerError> {
    match &ast.node {
        AstNode::Command(cmd) => {
            let argv: Vec<OsString> = cmd
                .words
                .iter()
                .map(|w| expand_word(&w.node, ctx))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ExecutionPlan {
                argv,
                cwd: None,
                env: Environment::Inherit,
                io: IoPlan::Simple,
            })
        }
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
        let node = parse("git add -A").expect("parses");
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
        let node = parse("pwd").expect("parses");
        let plan = lower(&node, &LowerContext::default()).expect("lowers");
        assert_eq!(plan.argv, vec![OsString::from("pwd")]);
    }

    #[test]
    fn case_preserved_through_lowering() {
        // The rides-with fix survives all the way to argv: GitHub stays GitHub.
        let node = parse("GitHub Clone").expect("parses");
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

    #[test]
    fn no_environment_renders_variables_in_source_form() {
        // The audit's situation. argv must be byte-identical to the pre-resolver behaviour.
        let node = parse("echo $HOME").expect("parses");
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
        let ctx = LowerContext { vars: Some(&vars) };

        let node = parse("echo $MY_VAR").expect("parses");
        let plan = lower(&node, &ctx).expect("lowers");
        assert_eq!(plan.argv[1], OsString::from("hello"));

        // Legacy uses unwrap_or_default: an UNSET variable is the empty string, NOT the source
        // form. That distinction is the whole reason LowerContext.vars is an Option.
        let node = parse("echo $NOPE").expect("parses");
        let plan = lower(&node, &ctx).expect("lowers");
        assert_eq!(plan.argv[1], OsString::from(""));
    }

    #[test]
    fn braces_are_syntax_not_semantics() {
        // ${HOME} and $HOME must produce the SAME AST -- braces are parser syntax.
        let mut m = std::collections::HashMap::new();
        m.insert("HOME", "/home/christian");
        let vars = FakeVars(m);
        let ctx = LowerContext { vars: Some(&vars) };

        let a = lower(&parse("echo $HOME").unwrap(), &ctx).unwrap();
        let b = lower(&parse("echo ${HOME}").unwrap(), &ctx).unwrap();
        assert_eq!(a.argv, b.argv);
        assert_eq!(a.argv[1], OsString::from("/home/christian"));
    }

    #[test]
    fn special_params_use_their_own_resolver_methods() {
        // $? and $$ are NOT name lookups -- FakeVars answers 0 and 1234 from last_exit()/pid(),
        // and its variable map is empty, so a Variable("?") model would have produced "".
        let vars = FakeVars(std::collections::HashMap::new());
        let ctx = LowerContext { vars: Some(&vars) };

        let plan = lower(&parse("echo $?").unwrap(), &ctx).unwrap();
        assert_eq!(plan.argv[1], OsString::from("0"));

        let plan = lower(&parse("echo $$").unwrap(), &ctx).unwrap();
        assert_eq!(plan.argv[1], OsString::from("1234"));

        // Mixed into a word, concatenated with literals.
        let plan = lower(&parse("echo exit=$?").unwrap(), &ctx).unwrap();
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
        let ctx = LowerContext { vars: Some(&vars) };

        for src in ["echo ${VAR:-default}", "echo ${#VAR}", "echo ${VAR/a/b}"] {
            let plan = lower(&parse(src).unwrap(), &ctx).unwrap();
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
        let ctx = LowerContext { vars: Some(&vars) };
        for (src, want) in [
            ("echo $", "$"),
            ("echo ${VAR", "${VAR"),
            ("echo $1", "$1"), // positional params not modelled yet
            ("echo 100$", "100$"),
        ] {
            let plan = lower(&parse(src).unwrap(), &ctx).unwrap();
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
        let ctx = LowerContext { vars: Some(&vars) };

        let node = parse("echo '$MY_VAR'").expect("parses");
        let plan = lower(&node, &ctx).expect("lowers");
        assert_eq!(
            plan.argv[1],
            OsString::from("$MY_VAR"),
            "single quotes suppress expansion"
        );

        // Double quotes DO expand, matching POSIX and legacy's INT-245 behaviour.
        let node = parse("echo \"$MY_VAR\"").expect("parses");
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
        let ctx = LowerContext { vars: Some(&vars) };
        let node = parse("echo foo$BAR/baz").expect("parses");
        let plan = lower(&node, &ctx).expect("lowers");
        assert_eq!(
            plan.argv[1],
            OsString::from("fooxyz/baz"),
            "one word, three parts"
        );
    }

    #[test]
    fn argv_as_utf8_round_trips_and_reports_bad_bytes() {
        let node = parse("git add -A").expect("parses");
        let plan = lower(&node, &LowerContext::default()).expect("lowers");
        assert_eq!(plan.argv_as_utf8().unwrap(), vec!["git", "add", "-A"]);

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
    /// syntax and the AST carries it -- while LOWERING REFUSES, because the executor cannot run a
    /// substitution yet. The failure is a capability report, not a fault.
    #[test]
    fn command_substitution_parses_but_does_not_lower() {
        let ast = parse("echo $(pwd)").expect("valid syntax parses");
        match lower(&ast, &LowerContext::default()) {
            Err(LowerError::UnsupportedConstruct { kind, .. }) => {
                assert_eq!(kind, "command substitution");
            }
            other => panic!("expected UnsupportedConstruct, got {other:?}"),
        }
    }

    #[test]
    fn expand_word_concatenates_literals() {
        // A word is the smallest unit expansion produces; today all-literal -> its bytes.
        let w = Word::literal("/tmp/testscript.sh");
        assert_eq!(
            expand_word(&w, &LowerContext::default()).unwrap(),
            OsString::from("/tmp/testscript.sh")
        );
    }
}
