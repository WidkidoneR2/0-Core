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
            assignments: vec![],
            words: vec![ls, la, tmp],
            redirects: vec![],
        };
        assert_eq!(cmd.words.len(), 3, "three words");
        assert!(cmd.redirects.is_empty(), "no redirects yet");
        assert_eq!(
            cmd.words[0].node.parts,
            vec![WordPart::Literal {
                text: "ls".to_string(),
                quoted: crate::spine::ast::QuoteContext::Unquoted,
            }],
            "first word is the single literal 'ls'"
        );
        assert!(
            cmd.words.iter().all(|w| w.node.is_all_literal()),
            "all literal today"
        );

        let node: Spanned<AstNode> = Spanned::new(full_span, AstNode::Command(cmd));
        match node.node {
            // Hand-built as a Command above, so neither composite is reachable.
            AstNode::Sequence(_) => unreachable!("constructed as a Command directly above"),
            AstNode::Command(c) => assert_eq!(c.words.len(), 3),
            // The node is hand-built two lines above, so a pipeline is impossible here.
            // Named rather than wildcarded: a `_` arm would silently absorb the next variant
            // too, and the compiler pointing at this line is the value of an exhaustive match.
            AstNode::Pipeline(_) => unreachable!("constructed as a Command directly above"),
            AstNode::Background(_) => unreachable!("constructed as a Command directly above"),
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
        assert_eq!(
            w.parts,
            vec![WordPart::Literal {
                text: "hello".to_string(),
                quoted: crate::spine::ast::QuoteContext::Unquoted,
            }]
        );
        assert!(w.is_all_literal());
    }
}

/// INT-208 GATE 2: a ParseError becomes a Diagnostic that CARRIES ITS SPAN.
///
/// ⚠️ WHAT THIS REPLACES, and it is the whole point of the intent: two sites in exec.rs read
/// `format!("spine: {e:?}")`. The parser knew the exact byte range of the problem and the identity
/// of the operator it declined -- five of six ParseError variants carry a Span, two carry an
/// OperatorKind as well -- and all of it was flattened into a Debug string a human had to read.
///
/// ★ THE HELP TEXT IS NOT NEW EITHER. Each variant's rustdoc already explains what it means and how
/// it differs from its neighbours; that reasoning lived where no user could reach it. This moves it
/// to where a user is standing.
pub fn diagnose(e: &crate::exec::SpineAttemptError) -> crate::diagnostic::Diagnostic {
    use crate::exec::SpineAttemptError as S;
    match e {
        S::Parse(p) => diagnose_parse(p),
        // ⭐ THE SCANNER KNOWS WHICH, so the reader is told which. "Unexpected end of input" threw
        // away four distinct facts the lexer had already established -- and the heredoc case even
        // carries the delimiter that would end it.
        S::Incomplete(i) => {
            use crate::spine::lexer::LexIncompleteKind as K;
            let (what, how) = match &i.kind {
                K::UnterminatedQuote => ("unterminated quote", "close the quote"),
                K::UnterminatedCommandSub => ("unterminated $(", "close the command substitution"),
                K::TrailingEscape => (
                    "line ends in a backslash",
                    "continue the line, or remove it",
                ),
                _ => ("unfinished input", "the line is not finished"),
            };
            crate::diagnostic::Diagnostic::error(what)
                .with_help(how)
                .with_code("fsh::spine::incomplete")
        }
        // A refusal is VALID SHELL the spine does not own -- not a failure. Legacy handles it,
        // and saying otherwise would be the conflation ParseError's own docs warn against.
        S::Refused(r) => crate::diagnostic::Diagnostic::error(format!("{:?}", r))
            .with_help("valid shell the spine does not implement -- legacy owns it")
            .with_code("fsh::spine::refused"),
        S::Lower(l) => {
            crate::diagnostic::Diagnostic::error(format!("{:?}", l)).with_code("fsh::spine::lower")
        }
    }
}

/// The parse half, where the spans live.
fn diagnose_parse(e: &parser::ParseError) -> crate::diagnostic::Diagnostic {
    use crate::diagnostic::Diagnostic;
    use parser::ParseError;
    match e {
        ParseError::Incomplete(_) => Diagnostic::error("unexpected end of input")
            .with_help("the line is unfinished, not wrong -- close the quote or the $( ")
            .with_code("fsh::spine::incomplete"),
        ParseError::NoCommand => Diagnostic::error("expected a command")
            .with_help("shell structure here requires a command, and there is none")
            .with_code("fsh::spine::no_command"),
        ParseError::UnsupportedOperator { kind, span } => {
            Diagnostic::error(format!("the spine does not implement {:?}", kind))
                .with_label(span.start, span.end, "this operator")
                .with_help("valid shell -- the legacy executor owns this construct")
                .with_code("fsh::spine::unsupported_operator")
        }
        ParseError::FdRedirect { span } => {
            Diagnostic::error("redirect against an explicit file descriptor")
                .with_label(span.start, span.end, "this redirect")
                .with_help("deliberately left to the legacy executor")
                .with_code("fsh::spine::fd_redirect")
        }
        ParseError::ComparisonNotRedirect { span } => Diagnostic::error(
            "read as a comparison, not a redirect",
        )
        .with_label(span.start, span.end, "this `>`")
        .with_help(
            "a target starting with a digit or `=` is a comparison, so `where cpu > 0.5` works",
        )
        .with_code("fsh::spine::comparison_not_redirect"),
        ParseError::MissingRedirectTarget { kind, span } => {
            Diagnostic::error(format!("no target after {:?}", kind))
                .with_label(
                    span.start,
                    span.end,
                    "this redirect has nothing to write to",
                )
                .with_code("fsh::spine::missing_redirect_target")
        }
    }
}
