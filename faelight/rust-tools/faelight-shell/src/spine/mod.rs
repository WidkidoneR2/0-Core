//! INT-169: the fsh parser spine -- lex -> parse -> AST.
//!
//! Design: docs/rfc-169-parser-spine.md. Phases: source -> lexer (logos + custom state)
//! -> tokens -> handwritten recursive-descent parser -> AST (pure syntax) -> expansion
//! -> execution. Built one construct at a time (RFC section 7), OLD `from_line` PATH LIVE
//! beside it until each construct passes the same REPL tests.

// INT-169: the dormant WordPart/AstNode variants and some Span/Word helpers are ahead of
// their first live caller (they are exercised by tests and the `spine parse` builtin, and
// filled in as the roadmap turns constructs on). This is the RFC's "real data model,
// unsupported syntax dormant" principle, not dead code. The allow comes off as the parser
// drives more of the live path.
#![allow(dead_code)]

pub mod ast;
pub mod audit;
pub mod compare;
pub mod lexer;
pub mod migrate;
pub mod migrate_audit;
pub mod parser;
pub mod plan;
pub mod render;

#[cfg(test)]
mod proptests;

#[cfg(test)]
mod golden;

#[cfg(test)]
mod tests {
    use super::ast::*;

    /// The first construct's target shape (RFC section 8): `ls -la /tmp` as a `Command`
    /// of three spanned literal `Word`s, redirects empty. Built here BY HAND to assert the
    /// frozen data model is correctly shaped. The parser's own test proves its output
    /// matches this.
    #[test]
    fn ast_shape_bare_command() {
        let ls = Spanned::new(Span::new(0, 2), Word::literal("ls"));
        let la = Spanned::new(Span::new(3, 6), Word::literal("-la"));
        let tmp = Spanned::new(Span::new(7, 11), Word::literal("/tmp"));

        let full_span = ls.span.merge(tmp.span);
        assert_eq!(full_span, Span::new(0, 11), "node span covers ls..tmp");

        let cmd = Command {
            words: vec![ls, la, tmp],
            redirects: vec![],
        };
        assert_eq!(cmd.words.len(), 3, "three words");
        assert!(cmd.redirects.is_empty(), "no redirects yet");
        assert_eq!(
            cmd.words[0].node.parts,
            vec![WordPart::Literal("ls".to_string())],
            "first word is the single literal 'ls'"
        );
        assert!(
            cmd.words.iter().all(|w| w.node.is_all_literal()),
            "all literal today"
        );

        let node: Spanned<AstNode> = Spanned::new(full_span, AstNode::Command(cmd));
        match node.node {
            AstNode::Command(c) => assert_eq!(c.words.len(), 3),
        }
        assert_eq!(node.span.len(), 11, "span length in bytes");
    }

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

    #[test]
    fn word_literal_helper() {
        let w = Word::literal("hello");
        assert_eq!(w.parts, vec![WordPart::Literal("hello".to_string())]);
        assert!(w.is_all_literal());
    }
}
