//! INT-169 spine: the lexer. See docs/rfc-169-parser-spine.md section 5.
//!
//! logos handles the structurally-regular tokens (a run of non-whitespace = a word).
//! A custom stateful layer for context-sensitive regions (strings, heredocs, $(),
//! ${}, interpolation) is added at their roadmap steps (2, 8) -- and it will feed the
//! SAME SpannedToken stream, so the parser never consumes logos directly. That keeps
//! logos an implementation detail of THIS module and leaves room for the hybrid.
//!
//! Today (roadmap step 1, bare commands): Word + skipped whitespace. Nothing else.

use super::ast::Span;
use logos::Logos;

/// Raw token kinds. Only `Word` exists today; operators, quotes, and $-constructs are
/// added at their roadmap steps. Whitespace is not a token -- it separates words and is
/// skipped by the lexer.
#[derive(Logos, Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// A maximal run of non-whitespace characters = one word (step 1).
    #[regex(r"[^\s]+")]
    Word,

    /// Whitespace separates; it is skipped, never emitted as a token.
    #[regex(r"\s+", logos::skip)]
    Whitespace,
}

/// A token with its owned source text and span -- what the parser consumes.
///
/// The parser reads THESE, not logos's iterator. Text is owned (RFC 4.3 -- owned data,
/// inputs are tiny). Span comes straight from logos's byte offsets, feeding our Span type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedToken {
    pub kind: TokenKind,
    pub text: String,
    pub span: Span,
}

/// A lex error: the unlexable slice and where it was. Today the `Word` regex matches any
/// non-whitespace, so bare commands never error -- but `lex` returns `Result` so real
/// error tokens (added later) have a home, and callers are already shaped for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub text: String,
    pub span: Span,
}

/// Tokenize a source line into spanned tokens. Whitespace is skipped; words carry their
/// exact byte span.
pub fn lex(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut out = Vec::new();
    let mut lexer = TokenKind::lexer(source);
    while let Some(result) = lexer.next() {
        let range = lexer.span(); // std::ops::Range<usize>
        let span = Span::new(range.start, range.end);
        match result {
            Ok(kind) => out.push(SpannedToken {
                kind,
                text: lexer.slice().to_string(),
                span,
            }),
            Err(_) => {
                return Err(LexError {
                    text: lexer.slice().to_string(),
                    span,
                })
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_bare_command() {
        // `ls -la /tmp` -> three Word tokens, whitespace skipped, spans exact.
        let toks = lex("ls -la /tmp").expect("bare command lexes");
        assert_eq!(toks.len(), 3, "three words");

        assert_eq!(toks[0].kind, TokenKind::Word);
        assert_eq!(toks[0].text, "ls");
        assert_eq!(toks[0].span, Span::new(0, 2));

        assert_eq!(toks[1].text, "-la");
        assert_eq!(toks[1].span, Span::new(3, 6));

        assert_eq!(toks[2].text, "/tmp");
        assert_eq!(toks[2].span, Span::new(7, 11));
    }

    #[test]
    fn lex_collapses_extra_whitespace() {
        // Multiple/leading/trailing spaces: still just the words, spans track real offsets.
        let toks = lex("  ls    -la  ").expect("lexes");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].text, "ls");
        assert_eq!(toks[0].span, Span::new(2, 4)); // after the two leading spaces
        assert_eq!(toks[1].text, "-la");
    }

    #[test]
    fn lex_single_word() {
        let toks = lex("pwd").expect("lexes");
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].text, "pwd");
        assert_eq!(toks[0].span, Span::new(0, 3));
    }

    #[test]
    fn lex_empty_is_empty() {
        assert!(lex("").expect("empty lexes").is_empty());
        assert!(lex("   ").expect("whitespace-only lexes").is_empty());
    }
}
