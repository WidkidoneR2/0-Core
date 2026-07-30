//! INT-169 spine: property/fuzz tests. See docs/rfc-169-parser-spine.md.
//!
//! The parser is where the language semantics are born, so it gets abused BEFORE anything
//! is built on top of it (before the AST is frozen, before ExecutionPlan is designed). We
//! generate thousands of shell-shaped lines and assert invariants that must hold for ANY
//! input -- the parser must survive hostile text without panicking, and its spans must tell
//! the truth. proptest shrinks any failure to a minimal reproduction.
//!
//! Test-only: this whole module is #[cfg(test)], never compiled into the shipped binary.

#![cfg(test)]

use super::ast::{AstNode, WordPart};
use super::parser::parse;
use proptest::prelude::*;

// ---- Layered generators (shell syntax is its own ecosystem; not random ASCII) ----

/// Layer 1: boring atoms.
fn boring() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("abc".to_string()),
        Just("123".to_string()),
        Just("_".to_string()),
        Just("-".to_string()),
        Just(".".to_string()),
        Just("/".to_string()),
        Just("ls".to_string()),
        Just("-la".to_string()),
        "[a-zA-Z0-9_./-]{1,8}".prop_map(|s| s),
    ]
}

/// Layer 3: shell-ish punctuation (not yet meaningful to the parser, but must not break it).
fn punct() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("|".to_string()),
        Just(">".to_string()),
        Just("<".to_string()),
        Just("&".to_string()),
        Just(";".to_string()),
        Just("$".to_string()),
        Just("()".to_string()),
        Just("{}".to_string()),
        Just("[]".to_string()),
        Just("||||".to_string()),
        Just("2>".to_string()),
    ]
}

/// Layer 4: quoting (empty, nested, escaped) -- not yet interpreted, must not break lexing.
fn quoting() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("\"\"".to_string()),
        Just("''".to_string()),
        Just("\"a b\"".to_string()),
        Just("\"a\\\"b\"".to_string()),
        Just("'a\"b'".to_string()),
        Just("\"\"\"\"\"".to_string()),
    ]
}

/// Layer 5: unicode -- the char-boundary minefield for byte-offset spans.
fn unicode() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("é".to_string()),
        Just("中".to_string()),
        Just("🙂".to_string()),
        Just("Γειά".to_string()),
        Just("café".to_string()),
        Just("a🙂b".to_string()),
    ]
}

/// A fragment from any layer.
fn fragment() -> impl Strategy<Value = String> {
    prop_oneof![boring(), punct(), quoting(), unicode()]
}

/// Layer 6: combinations -- fragments joined by varied whitespace (where bugs appear).
fn shell_line() -> impl Strategy<Value = String> {
    let ws = prop_oneof![
        Just(" ".to_string()),
        Just("  ".to_string()),
        Just("\t".to_string()),
        Just(" \t ".to_string()),
    ];
    (prop::collection::vec(fragment(), 0..8), ws).prop_map(|(parts, sep)| parts.join(&sep))
}

// ---- The properties: boring and brutal ----

proptest! {
    /// Property 1: the parser NEVER panics, on any input. Sounds trivial; is not.
    #[test]
    fn parser_never_panics(input in shell_line()) {
        let _ = parse(&input);
    }

    /// Also never panics on totally arbitrary text (not just shell-shaped).
    #[test]
    fn parser_never_panics_arbitrary(input in ".*") {
        let _ = parse(&input);
    }

    /// Property 2 + 4: every word's span is valid AND losslessly extracts the source.
    /// span.start <= span.end <= input.len(), AND input[span] slices without panic and
    /// equals the word's literal text. This is the identity of the spine -- source truth
    /// preserved (the GitHub-case guarantee as an invariant) -- and it double-checks that
    /// byte-offset spans land on char boundaries (else the slice would panic on unicode).
    #[test]
    fn spans_valid_and_lossless(input in shell_line()) {
        if let Ok(node) = parse(&input) {
            prop_assert!(node.span.start <= node.span.end, "node span reversed");
            prop_assert!(node.span.end <= input.len(), "node span past input end");
            match &node.node {
                AstNode::Command(cmd) => {
                    for word in &cmd.words {
                        prop_assert!(word.span.start <= word.span.end, "word span reversed");
                        prop_assert!(word.span.end <= input.len(), "word span past input end");
                        // Slicing on the span must not panic -> byte offsets land on char
                        // boundaries. NOTE: since step 2 the word's TEXT is not this slice
                        // (quote characters are consumed), so we check the span, not equality.
                        let _ = &input[word.span.start..word.span.end];
                        prop_assert!(!word.node.parts.is_empty(), "a word has at least one part");
                        for part in &word.node.parts {
                            // An EMPTY literal is legitimate since step 2: `echo ""` is a real
                            // empty argument. Only Literal is produced today; the dormant
                            // variants arrive at their roadmap steps.
                            // Literal and Variable are both produced now; the dormant variants
                            // arrive at their own milestones.
                            // Bound first, then asserted: prop_assert! stringifies its condition into a
                            // format string, so braces inside the PATTERN would be read as placeholders.
                            let is_known_part = matches!(
                                part,
                                WordPart::Literal { .. } | WordPart::Variable { .. }
                                    | WordPart::SpecialParam(_)
                            );
                            prop_assert!(is_known_part);
                        }
                    }
                }
            }
        }
    }

    /// Property 5: determinism -- same input parses to the same result. Catches state leak.
    #[test]
    fn parser_deterministic(input in shell_line()) {
        let a = parse(&input);
        let b = parse(&input);
        prop_assert_eq!(a, b);
    }

    /// Property: quoting can MERGE whitespace-separated chunks into one word, but can never
    /// SPLIT one. So words <= whitespace chunks always. And for quote-free input the parser
    /// is still exactly a whitespace splitter, so equality still holds there -- keeping the
    /// strong step-1 guarantee on the case where it is still true.
    #[test]
    fn word_count_matches_tokens(input in shell_line()) {
        if let Ok(node) = parse(&input) {
            let chunks = input.split_whitespace().count();
            match &node.node {
                AstNode::Command(cmd) => {
                    prop_assert!(cmd.words.len() <= chunks,
                        "more words than whitespace chunks for {:?}", input);
                    if !input.contains('"') && !input.contains('\'') {
                        prop_assert_eq!(cmd.words.len() + cmd.redirects.len() * 2, chunks,
                            "quote-free input: words plus redirect pairs must equal chunks: {:?}", input);
                    }
                }
            }
        }
    }
}
