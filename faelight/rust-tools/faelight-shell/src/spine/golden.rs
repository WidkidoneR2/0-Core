//! INT-169 spine: golden tests from REAL shell history. See docs/rfc-169-parser-spine.md.
//!
//! "Your real usage is the language specification." These are actual commands from
//! Christian's shell_history, each pinned to the parser's CURRENT rendered AST. Two tiers:
//!
//!   BARE_GOLDENS  -- commands the parser handles CORRECTLY today. True regressions:
//!                    these must NOT change unless we break bare-command parsing.
//!   FUTURE_GOLDENS -- real commands with quotes / pipes / redirects / vars that parse
//!                    IMPERFECTLY today (the lexer splits on whitespace, so `"a b"` is not
//!                    yet one word, `|` is just a literal, etc). These capture CURRENT
//!                    behavior as a baseline that is EXPECTED to change as roadmap steps
//!                    2-8 land -- a golden here flipping is the signal a construct works
//!                    against real input.
//!
//! Expected output is the exact `render()` of the parsed AST, captured as ground-truth
//! from the real parser (not hand-written). If render's format changes intentionally,
//! regenerate these.

#![cfg(test)]

use super::parser::parse;
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
    // step 2 LANDED (2026-07-22): "message here" is now ONE word, quotes consumed, and the
    // word span covers the quote characters. This golden FLIPPED -- the proof step 2 works
    // against a real command from history. It is now CORRECT behavior, so it is a true
    // regression and belongs in BARE_GOLDENS; left here until the tiers are re-sorted.
    (
        "git commit -m \"message here\"",
        "Command @[0,28)\n  Word @[0,3)  Literal \"git\"\n  Word @[4,10)  Literal \"commit\"\n  Word @[11,13)  Literal \"-m\"\n  Word @[14,28)  Literal \"message here\"\n  redirects: []\n",
    ),
    // step 6 (pipelines): `|` becomes a Pipeline node above two Commands.
    (
        "ls faelight | grep vm",
        "Command @[0,21)\n  Word @[0,2)  Literal \"ls\"\n  Word @[3,11)  Literal \"faelight\"\n  Word @[12,13)  Literal \"|\"\n  Word @[14,18)  Literal \"grep\"\n  Word @[19,21)  Literal \"vm\"\n  redirects: []\n",
    ),
    // step 5 (redirections, the 172 territory): `2>` becomes a Redirect, not a word.
    (
        "cat log 2> /dev/null",
        "Command @[0,20)\n  Word @[0,3)  Literal \"cat\"\n  Word @[4,7)  Literal \"log\"\n  Word @[8,10)  Literal \"2>\"\n  Word @[11,20)  Literal \"/dev/null\"\n  redirects: []\n",
    ),
    // step 7 (lists): `&&` becomes a Sequence/And node above two Commands.
    (
        "a && b",
        "Command @[0,6)\n  Word @[0,1)  Literal \"a\"\n  Word @[2,4)  Literal \"&&\"\n  Word @[5,6)  Literal \"b\"\n  redirects: []\n",
    ),
    // steps 3-4 (variables + mixed words): `foo$BAR` splits into Literal(foo)+Variable(BAR).
    (
        "echo foo$BAR",
        "Command @[0,12)\n  Word @[0,4)  Literal \"echo\"\n  Word @[5,12)  Literal \"foo$BAR\"\n  redirects: []\n",
    ),
];

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

    #[test]
    fn bare_goldens_all_parse() {
        assert!(BARE_GOLDENS.len() >= 8, "corpus should have a real spread");
        for (input, _) in BARE_GOLDENS {
            assert!(parse(input).is_ok(), "{input:?} should parse");
        }
    }
}
