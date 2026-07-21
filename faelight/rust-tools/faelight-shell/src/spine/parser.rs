//! INT-169 spine: handwritten recursive-descent parser. See docs/rfc-169-parser-spine.md.
//!
//! Consumes the lexer's SpannedToken stream, produces the AST. Handwritten = debuggable
//! the way the whole shell is debugged: greppable, steppable, a straight call path. For a
//! daily-driver shell where a parser bug is an unusable terminal, that IS the safety model.
//!
//! Today (roadmap step 1, bare commands): a run of Word tokens -> Command{words, []}.
//! Pipelines / redirects / lists / control-flow are added at their roadmap steps, each a
//! new parse method hanging off this same recursive-descent structure -- parse_command
//! stays the leaf; pipeline/list parsing will sit ABOVE it and recurse down to it.

use super::ast::{Command, Node, NodeKind, Span, Spanned, Word};
use super::lexer::{lex, LexError, SpannedToken, TokenKind};

/// Parse errors carry a span so they can point at exact source (RFC section 4.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The lexer rejected the input.
    Lex(LexError),
    /// Nothing to parse (empty / whitespace-only input).
    Empty,
    // grows as the grammar does: UnexpectedToken(Span), Unterminated(Span), ...
}

/// A cursor over the token stream. A recursive-descent parser is just methods that
/// consume tokens and build nodes; `pos` is the current position.
pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// The current token without consuming it.
    fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.pos)
    }

    /// Consume and return the current token's data we need (kind copied, text/span cloned),
    /// advancing the cursor. Returns None at end of stream.
    fn advance(&mut self) -> Option<SpannedToken> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    /// Parse a bare command: one-or-more Word tokens -> Command. (Roadmap step 1.)
    ///
    /// This is the LEAF of the recursive descent. Later, pipeline/list parsing sits above
    /// this and calls it for each command between `|` / `&&` / `;`. Each Word token becomes
    /// a Word of a single Literal part today; at roadmap step 3 ($VAR) this is exactly where
    /// a Word grows multiple parts.
    fn parse_command(&mut self) -> Result<Node, ParseError> {
        let mut words: Vec<Word> = Vec::new();
        let mut span: Option<Span> = None;

        while let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::Word => {
                    let t = self.advance().expect("peek was Some");
                    let tspan = t.span;
                    words.push(Word::literal(t.text));
                    span = Some(match span {
                        None => tspan,
                        Some(s) => s.merge(tspan),
                    });
                }
                // Whitespace never reaches the parser (lexer skips it). When operators
                // arrive at later roadmap steps, the command loop breaks on them here so
                // the caller (pipeline/list parser) can handle the operator.
                TokenKind::Whitespace => {
                    self.pos += 1; // defensive: skip if one ever leaks through
                }
            }
        }

        match span {
            None => Err(ParseError::Empty),
            Some(sp) => Ok(Spanned::new(
                sp,
                NodeKind::Command(Command {
                    words,
                    redirects: vec![],
                }),
            )),
        }
    }

    /// Top-level parse: the whole line -> one Node. Today a single command; later this
    /// dispatches into pipeline/list parsing that recurses down to parse_command.
    fn parse_line(&mut self) -> Result<Node, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::Empty);
        }
        self.parse_command()
    }
}

/// Parse a source line into an AST node. Public entry: lex, then parse.
pub fn parse(source: &str) -> Result<Node, ParseError> {
    let tokens = lex(source).map_err(ParseError::Lex)?;
    let mut parser = Parser::new(tokens);
    parser.parse_line()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spine::ast::{NodeKind, WordPart};

    #[test]
    fn parse_bare_command_matches_hand_built_shape() {
        // `ls -la /tmp` -> the EXACT shape Increment 1 hand-built. Closes the loop:
        // the parser now PRODUCES what the AST shape test asserted.
        let node = parse("ls -la /tmp").expect("parses");
        assert_eq!(
            node.span,
            Span::new(0, 11),
            "node span covers the whole line"
        );

        match node.value {
            NodeKind::Command(cmd) => {
                assert_eq!(cmd.words.len(), 3);
                assert!(cmd.redirects.is_empty());
                assert_eq!(
                    cmd.words[0].parts,
                    vec![WordPart::Literal("ls".to_string())]
                );
                assert_eq!(
                    cmd.words[1].parts,
                    vec![WordPart::Literal("-la".to_string())]
                );
                assert_eq!(
                    cmd.words[2].parts,
                    vec![WordPart::Literal("/tmp".to_string())]
                );
            }
        }
    }

    #[test]
    fn parse_single_word() {
        let node = parse("pwd").expect("parses");
        match node.value {
            NodeKind::Command(cmd) => {
                assert_eq!(cmd.words.len(), 1);
                assert_eq!(
                    cmd.words[0].parts,
                    vec![WordPart::Literal("pwd".to_string())]
                );
            }
        }
        assert_eq!(node.span, Span::new(0, 3));
    }

    #[test]
    fn parse_empty_is_error() {
        assert_eq!(parse(""), Err(ParseError::Empty));
        assert_eq!(parse("    "), Err(ParseError::Empty));
    }

    #[test]
    fn parse_preserves_case() {
        // RFC section 9 rides-with: the spine does NOT lowercase (from_line does; the new
        // path must not). Prove GitHub != github.
        let node = parse("GitHub Clone").expect("parses");
        match node.value {
            NodeKind::Command(cmd) => {
                assert_eq!(
                    cmd.words[0].parts,
                    vec![WordPart::Literal("GitHub".to_string())]
                );
                assert_eq!(
                    cmd.words[1].parts,
                    vec![WordPart::Literal("Clone".to_string())]
                );
            }
        }
    }
}
