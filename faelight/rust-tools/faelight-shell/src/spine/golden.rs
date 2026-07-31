//! INT-169 spine: golden tests from REAL shell history. See docs/rfc-169-parser-spine.md.
//!
//! "Your real usage is the language specification." These are actual commands from
//! Christian's shell_history, each pinned to the parser's CURRENT rendered AST. Two tiers:
//!
//!   BARE_GOLDENS  -- commands the parser handles CORRECTLY today. True regressions:
//!                    these must NOT change unless we break bare-command parsing.
//!   FUTURE_GOLDENS -- real commands whose syntax the spine is expected to OWN eventually.
//!                    They parse imperfectly today and the pinned render is a baseline; a
//!                    golden here flipping is the signal a roadmap step works against real
//!                    input.
//!
//!   FUTURE_REFUSALS -- real shell the spine RECOGNISES and intentionally does not own yet.
//!                    ⚠️ Do not fold these back into FUTURE_GOLDENS. They are not the same
//!                    kind of future state, and merging them would undo INT-169's boundary.
//!
//! ★ AN UNSUPPORTED CONSTRUCT HAS THREE STATES, and the middle one is what makes an
//! incremental shell replacement possible at all:
//!
//!   1. INVISIBLE BUG        the scanner treats the syntax as data, so `echo a | grep a`
//!                           becomes five words and LOWERS -- the shell claiming a command
//!                           it cannot run.
//!   2. EXPLICIT REFUSAL     the lexer identifies it and the parser declines ownership.
//!                           This is where operators sit today.
//!   3. OWNED IMPLEMENTATION the parser builds the node and lowering decides execution.
//!
//! A hybrid shell cannot jump from "legacy owns everything" to "spine owns everything".
//! It goes through "the spine can say which commands are not its own", and these two
//! corpora record which fragments are in which state.
//!
//! Expected output is the exact `render()` of the parsed AST, captured as ground-truth
//! from the real parser (not hand-written). If render's format changes intentionally,
//! regenerate these.

#![cfg(test)]

use super::lexer::OperatorKind;
use super::parser::{parse, ParseError};
use super::render::render;

/// (input, expected rendered AST). Parser handles these correctly today.
const BARE_GOLDENS: &[(&str, &str)] = &[
    (
        "cistart 133",
        "Command @[0,11)\n  Word @[0,7)  Literal \"cistart\"\n  Word @[8,11)  Literal \"133\"\n  redirects: []\n",
    ),
    (
        "git add -A",
        "Command @[0,10)\n  Word @[0,3)  Literal \"git\"\n  Word @[4,7)  Literal \"add\"\n  Word @[8,10)  Literal \"-A\"\n  redirects: []\n",
    ),
    (
        "cat /tmp/test_cat2.txt",
        "Command @[0,22)\n  Word @[0,3)  Literal \"cat\"\n  Word @[4,22)  Literal \"/tmp/test_cat2.txt\"\n  redirects: []\n",
    ),
    (
        "cheat --help",
        "Command @[0,12)\n  Word @[0,5)  Literal \"cheat\"\n  Word @[6,12)  Literal \"--help\"\n  redirects: []\n",
    ),
    (
        "core intent show 173",
        "Command @[0,20)\n  Word @[0,4)  Literal \"core\"\n  Word @[5,11)  Literal \"intent\"\n  Word @[12,16)  Literal \"show\"\n  Word @[17,20)  Literal \"173\"\n  redirects: []\n",
    ),
    (
        "echo xterm-256color",
        "Command @[0,19)\n  Word @[0,4)  Literal \"echo\"\n  Word @[5,19)  Literal \"xterm-256color\"\n  redirects: []\n",
    ),
    (
        "/tmp/testscript.sh",
        "Command @[0,18)\n  Word @[0,18)  Literal \"/tmp/testscript.sh\"\n  redirects: []\n",
    ),
    (
        "rm -rf /tmp/stubtest",
        "Command @[0,20)\n  Word @[0,2)  Literal \"rm\"\n  Word @[3,6)  Literal \"-rf\"\n  Word @[7,20)  Literal \"/tmp/stubtest\"\n  redirects: []\n",
    ),
    (
        "intent show 1994",
        "Command @[0,16)\n  Word @[0,6)  Literal \"intent\"\n  Word @[7,11)  Literal \"show\"\n  Word @[12,16)  Literal \"1994\"\n  redirects: []\n",
    ),
];

/// (input, expected rendered AST). Real commands with quotes/pipes/redirects/vars that
/// parse IMPERFECTLY today -- captured by calling parse() DIRECTLY (NOT via `spine parse`,
/// whose args fsh's dispatch pre-mangles: it expands vars, strips quotes, and splits on
/// pipes/redirects/&& before the builtin ever sees them). These are EXPECTED to change as
/// roadmap steps 2-8 land; a flip here is the signal a construct works on real input.
const FUTURE_GOLDENS: &[(&str, &str)] = &[
    (
        "git commit -m \"message here\"",
        "Command @[0,28)\n  Word @[0,3)  Literal \"git\"\n  Word @[4,10)  Literal \"commit\"\n  Word @[11,13)  Literal \"-m\"\n  Word @[14,28)  Literal \"message here\"\n  redirects: []\n",
    ),
    (
        "echo foo$BAR",
        "Command @[0,12)\n  Word @[0,4)  Literal \"echo\"\n  Word @[5,12)  Literal \"foo\" Variable \"BAR\"\n  redirects: []\n",
    ),
];

/// Inputs the spine now DECLINES, with the operator it declined on.
///
/// ★ A SECOND KIND OF FUTURE STATE, and the reason these left FUTURE_GOLDENS. That corpus
/// records `current AST -> future AST`: a construct lands and the pinned render changes.
/// Operators do not move that way. They go `accidental AST -> explicit refusal -> owned AST`,
/// and the middle state is not noise -- it is the whole point of INT-169's operator tokens.
/// Before them `|` was ordinary word content, so `ls faelight | grep vm` produced a five-word
/// Command and LOWERED, which is a shell claiming a command it cannot run.
///
/// These flip BACK when pipelines, redirects and boolean chains actually land.
///
/// ⚠️ `2>` is deliberately not one token: it lexes as Word("2") then RedirectOut, so the
/// refusal is correct while whether redirects own numeric fd prefixes stays a later grammar
/// decision.
const FUTURE_REFUSALS: &[(&str, OperatorKind)] = &[
    // `ls faelight | grep vm` left this table in INT-200 -- the parser now BUILDS a Pipeline
    // for it, so it is no longer a refusal. Lowering declines it instead, one layer down.
    // `a && b` left this table in INT-200 -- the parser builds a Sequence for it now, and
    // lowering declines it one layer down instead.
];

/// INT-200: refusals that carry NO OperatorKind, so they cannot live in the table above. A redirect
/// against an explicit file descriptor is recognised and deliberately left to legacy -- INT-172
/// routes it to sh, where it works -- and it is counted separately in the audit so that "stderr
/// redirects we decline on purpose" never reads as "redirects we cannot do".
const FUTURE_FD_REFUSALS: &[&str] = &["cat log 3>&2", "make 4> build.log"];

#[cfg(test)]
mod tests {
    use super::*;

    /// Every BARE golden parses to exactly its pinned rendered AST. True regression tests --
    /// a failure here means bare-command parsing changed (almost certainly a bug).
    #[test]
    fn bare_goldens_match() {
        for (input, expected) in BARE_GOLDENS {
            let node = parse(input)
                .unwrap_or_else(|e| panic!("golden input {input:?} failed to parse: {e:?}"));
            let got = render(&node);
            assert_eq!(
                &got, expected,
                "\ngolden mismatch for input {input:?}\n--- expected ---\n{expected}\n--- got ---\n{got}\n"
            );
        }
    }

    /// Sanity: the corpus is non-trivial and every entry round-trips (parses without error).
    /// Every FUTURE golden parses to its pinned CURRENT rendered AST. These are baselines,
    /// not correctness -- when one FLIPS (a construct landed), update it and note which step.
    #[test]
    fn future_goldens_match() {
        for (input, expected) in FUTURE_GOLDENS {
            let node = parse(input)
                .unwrap_or_else(|e| panic!("future golden {input:?} failed to parse: {e:?}"));
            let got = render(&node);
            assert_eq!(
                &got, expected,
                "\nFUTURE golden changed for input {input:?} -- if a construct landed, update the golden.\n--- expected ---\n{expected}\n--- got ---\n{got}\n"
            );
        }
    }

    /// ★ THE MIGRATION CONTRACT, and a different question from either golden corpus. The goldens
    /// ask "did the supported language change?"; this asks "can the spine prove a command is
    /// outside its ownership WITHOUT lowering or executing it?"
    ///
    /// That distinction is what makes the legacy fallback safe. A router may only fall back on a
    /// refusal that happened before anything ran -- `lower()` cannot serve, because it executes
    /// command substitutions before it can discover what it does not support, and legacy would
    /// then run them a second time.
    #[test]
    fn refused_constructs_never_reach_lowering() {
        for (input, want) in FUTURE_REFUSALS {
            match parse(input) {
                Err(ParseError::UnsupportedOperator { kind, .. }) => {
                    assert_eq!(kind, *want, "{input:?} was declined as the wrong construct")
                }
                other => panic!("{input:?} crossed the ownership boundary: {other:?}"),
            }
        }
    }

    /// ⚠️ THE CORPUS MOVED (INT-200): `2>` and `2>&1` are OWNED now, so what is left here is the
    /// descriptors nobody in this history has ever typed -- zero `3>`, zero `1>&2`, zero anything
    /// above 2. Keeping the old entries would have made a passing test out of behaviour that
    /// changed.
    ///
    /// ★ AND THE BOUNDARY MOVED WITH THEM: these now PARSE and are declined at LOWERING, because
    /// the parser can build the shape while only the executor knows it has no wiring for it.
    /// Asserting the parse alone would prove nothing, so the plan is what is checked.
    #[test]
    fn unused_descriptors_are_declined_at_lowering() {
        for input in FUTURE_FD_REFUSALS {
            let node = parse(input).unwrap_or_else(|e| panic!("{input:?} should parse: {e:?}"));
            match crate::spine::plan::lower(&node, &crate::spine::plan::LowerContext::default()) {
                Err(crate::spine::plan::LowerError::UnsupportedConstruct { .. }) => {}
                other => panic!("{input:?} must be declined at lowering: {other:?}"),
            }
        }
    }

    #[test]
    fn bare_goldens_all_parse() {
        assert!(BARE_GOLDENS.len() >= 8, "corpus should have a real spread");
        for (input, _) in BARE_GOLDENS {
            assert!(parse(input).is_ok(), "{input:?} should parse");
        }
    }
}
