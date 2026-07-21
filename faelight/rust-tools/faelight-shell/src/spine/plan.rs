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

use super::ast::{AstNode, Span, Spanned, Word, WordPart};

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

/// Expand one word to its final OsString. TODAY: a word is all-Literal, so this concatenates
/// the literal parts. This is the EXPANSION seam -- when WordPart::Variable lands (step 3),
/// `$HOME` resolves HERE, and the lowering boundary above does not change. The match is
/// exhaustive so the compiler flags this function the moment a new WordPart variant exists.
fn expand_word(word: &Word) -> OsString {
    let mut out = String::new();
    for part in &word.parts {
        match part {
            WordPart::Literal(s) => out.push_str(s),
            // WordPart::Variable(name) => out.push_str(&resolve_var(name)),  // step 3
        }
    }
    OsString::from(out)
}

/// Lower a parsed AST node to the executor's ExecutionPlan. Spine side (the legacy adapter
/// comes later; comparison is only meaningful once BOTH sides speak plans).
///
/// Command -> Ok(plan). Future constructs (Pipeline, Sequence) -> Err(UnsupportedConstruct)
/// until the ExecutionGraph / IO model exists -- honest, not a panic and not a fake plan.
pub fn lower(ast: &Spanned<AstNode>) -> Result<ExecutionPlan, LowerError> {
    match &ast.node {
        AstNode::Command(cmd) => {
            let argv: Vec<OsString> = cmd.words.iter().map(|w| expand_word(&w.node)).collect();
            Ok(ExecutionPlan {
                argv,
                cwd: None,
                env: Environment::Inherit,
                io: IoPlan::Simple,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spine::parser::parse;

    #[test]
    fn bare_command_lowers_to_argv() {
        let node = parse("git add -A").expect("parses");
        let plan = lower(&node).expect("lowers");
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
        let plan = lower(&node).expect("lowers");
        assert_eq!(plan.argv, vec![OsString::from("pwd")]);
    }

    #[test]
    fn case_preserved_through_lowering() {
        // The rides-with fix survives all the way to argv: GitHub stays GitHub.
        let node = parse("GitHub Clone").expect("parses");
        let plan = lower(&node).expect("lowers");
        assert_eq!(
            plan.argv,
            vec![OsString::from("GitHub"), OsString::from("Clone")]
        );
    }

    #[test]
    fn expand_word_concatenates_literals() {
        // A word is the smallest unit expansion produces; today all-literal -> its bytes.
        let w = Word::literal("/tmp/testscript.sh");
        assert_eq!(expand_word(&w), OsString::from("/tmp/testscript.sh"));
    }
}
