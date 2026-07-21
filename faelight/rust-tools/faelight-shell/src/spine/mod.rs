//! INT-169: the fsh parser spine -- lex -> parse -> AST.
//!
//! Design: docs/rfc-169-parser-spine.md. Phases: source -> lexer (logos + custom state)
//! -> tokens -> handwritten recursive-descent parser -> AST (pure syntax) -> expansion
//! -> execution. Built one construct at a time (RFC section 7), OLD `from_line` PATH LIVE
//! beside it until each construct passes the same REPL tests.
//!
//! Today: the AST types only (the first construct's data model). Lexer + parser land next.

// INT-169: the AST types are intentionally AHEAD of their first caller. Nothing
// CONSTRUCTS them yet except the shape tests below -- the lexer (Increment 2) and the
// handwritten parser (Increment 3) are what will build them. This is the RFC's "real
// data model, unsupported syntax dormant" principle (docs/rfc-169-parser-spine.md sec 8),
// not dead code. The allow comes off once the parser constructs these on the live path.
#![allow(dead_code)]

pub mod ast;
pub mod lexer;

#[cfg(test)]
mod tests {
    use super::ast::*;

    /// The first construct's target shape (RFC section 8): `ls -la /tmp` as a `Command`
    /// of three literal `Word`s, redirects empty. Built here BY HAND -- this asserts the
    /// data model is real and correctly shaped BEFORE any lexer/parser exists to produce
    /// it. When the parser lands, its output is compared against exactly this shape.
    #[test]
    fn ast_shape_bare_command() {
        // `ls -la /tmp`
        //  0123456789...   byte offsets:  ls=[0,2) -la=[3,6) /tmp=[7,11)
        let ls = Spanned::new(Span::new(0, 2), Word::literal("ls"));
        let la = Spanned::new(Span::new(3, 6), Word::literal("-la"));
        let tmp = Spanned::new(Span::new(7, 11), Word::literal("/tmp"));

        // Command owns bare Words; we keep their spans alongside to prove spans are usable
        // end-to-end. (When the parser lands it will decide whether Word itself is Spanned;
        // for now we exercise Span at the node level, which is what matters for section 4.2.)
        let cmd = Command {
            words: vec![ls.value.clone(), la.value.clone(), tmp.value.clone()],
            redirects: vec![],
        };

        // Shape assertions
        assert_eq!(cmd.words.len(), 3, "three words");
        assert!(cmd.redirects.is_empty(), "no redirects yet");
        assert_eq!(
            cmd.words[0].parts,
            vec![WordPart::Literal("ls".to_string())],
            "first word is the single literal 'ls'"
        );
        assert!(
            cmd.words.iter().all(|w| w.is_all_literal()),
            "all literal today"
        );

        // Wrap in a real spanned Node -- exercises Node/NodeKind/Spanned/Span together.
        let full_span = ls.span.merge(tmp.span);
        assert_eq!(full_span, Span::new(0, 11), "node span covers ls..tmp");
        let node: Node = Spanned::new(full_span, NodeKind::Command(cmd));

        // Exhaustive match -- compiler enforces we handle every NodeKind variant.
        match node.value {
            NodeKind::Command(c) => assert_eq!(c.words.len(), 3),
        }
        assert_eq!(node.span.len(), 11, "span length in bytes");
    }

    /// Span helpers behave: merge grows to outer bounds, len/is_empty are correct.
    #[test]
    fn span_helpers() {
        let a = Span::new(0, 2);
        let b = Span::new(7, 11);
        assert_eq!(a.merge(b), Span::new(0, 11));
        assert_eq!(b.merge(a), Span::new(0, 11), "merge is order-independent");
        assert_eq!(a.len(), 2);
        assert!(Span::new(5, 5).is_empty());
        assert!(!a.is_empty());
    }

    /// Word::literal builds a single-Literal word; is_all_literal reflects it.
    #[test]
    fn word_literal_helper() {
        let w = Word::literal("hello");
        assert_eq!(w.parts, vec![WordPart::Literal("hello".to_string())]);
        assert!(w.is_all_literal());
    }
}
