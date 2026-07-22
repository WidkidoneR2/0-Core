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

use super::ast::{AstNode, Command, Span, Spanned, Word, WordPart};
use super::lexer::{lex, LexError, QuoteContext, SpannedToken, TokenKind, WordSegment};

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

    /// Consume and return the current token (cloned -- borrow-checker + tiny data),
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
    /// a `Spanned<Word>` -- the word carries its own token span (spans everywhere), and its
    /// single Literal part today; at roadmap step 3 ($VAR) this is where a Word grows
    /// multiple parts.
    fn parse_command(&mut self) -> Result<Spanned<AstNode>, ParseError> {
        let mut words: Vec<Spanned<Word>> = Vec::new();
        let mut cmd_span: Option<Span> = None;

        while let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::Word => {
                    let t = self.advance().expect("peek was Some");
                    let wspan = t.span;
                    // Each lexical segment becomes a WordPart. Today every segment maps to a
                    // Literal regardless of its quote context. At step 3 an unquoted or
                    // double-quoted segment may yield SEVERAL parts (Literal + Variable) while
                    // a single-quoted segment stays one Literal -- and that is a change HERE,
                    // in the mapping, not in the lexer.
                    let parts: Vec<WordPart> =
                        t.segments.iter().flat_map(parts_from_segment).collect();
                    words.push(Spanned::new(wspan, Word { parts }));
                    cmd_span = Some(match cmd_span {
                        None => wspan,
                        Some(s) => s.merge(wspan),
                    });
                }
            }
        }

        match cmd_span {
            None => Err(ParseError::Empty),
            Some(sp) => Ok(Spanned::new(
                sp,
                AstNode::Command(Command {
                    words,
                    redirects: vec![],
                }),
            )),
        }
    }

    /// Top-level parse: the whole line -> one node. Today a single command; later this
    /// dispatches into pipeline/list parsing that recurses down to parse_command.
    fn parse_line(&mut self) -> Result<Spanned<AstNode>, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::Empty);
        }
        self.parse_command()
    }
}

/// Map one lexical segment to word parts. THE MAPPING IS WHERE QUOTE CONTEXT IS INTERPRETED --
/// the lexer only recorded which delimiter enclosed the text. A single-quoted segment is one
/// Literal, always: a `$` inside it is ordinary text, and no later stage can mistake it for a
/// live substitution. Unquoted and double-quoted segments are scanned for variable SITES.
fn parts_from_segment(seg: &WordSegment) -> Vec<WordPart> {
    match seg.context {
        QuoteContext::Single => vec![WordPart::Literal(seg.text.clone())],
        QuoteContext::Unquoted | QuoteContext::Double => recognise_variables(&seg.text),
    }
}

/// Split text into alternating Literal / Variable parts. RECOGNITION ONLY -- nothing is
/// evaluated here, and the resulting plan still renders `$NAME` back out, so behaviour is
/// unchanged (see plan::expand_word). Evaluation is the separate variable-expansion milestone.
///
/// Recognises the plain `$NAME` form with a POSIX-shaped name. Deliberately NOT yet: `${NAME}`,
/// `${NAME:-default}`, `$?`, `$$`, positional `$1`. A `$` not starting a valid name is literal.
fn recognise_variables(text: &str) -> Vec<WordPart> {
    let mut parts: Vec<WordPart> = Vec::new();
    let mut literal = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;

    while i < chars.len() {
        if chars[i] == '$' {
            let start = i + 1;
            if start < chars.len() && (chars[start].is_ascii_alphabetic() || chars[start] == '_') {
                let mut j = start;
                while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                    j += 1;
                }
                if !literal.is_empty() {
                    parts.push(WordPart::Literal(std::mem::take(&mut literal)));
                }
                parts.push(WordPart::Variable(chars[start..j].iter().collect()));
                i = j;
                continue;
            }
        }
        literal.push(chars[i]);
        i += 1;
    }

    if !literal.is_empty() {
        parts.push(WordPart::Literal(literal));
    }
    // An all-empty segment (`echo ""`) must still yield one part -- it is a real empty argument.
    if parts.is_empty() {
        parts.push(WordPart::Literal(String::new()));
    }
    parts
}

/// Parse a source line into a spanned AST node. Public entry: lex, then parse.
pub fn parse(source: &str) -> Result<Spanned<AstNode>, ParseError> {
    let tokens = lex(source).map_err(ParseError::Lex)?;
    let mut parser = Parser::new(tokens);
    parser.parse_line()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spine::ast::{AstNode, WordPart};

    #[test]
    fn parse_bare_command_matches_hand_built_shape() {
        // `ls -la /tmp` -> a Command of three spanned words, each a single Literal part.
        let node = parse("ls -la /tmp").expect("parses");
        assert_eq!(
            node.span,
            Span::new(0, 11),
            "node span covers the whole line"
        );

        match node.node {
            AstNode::Command(cmd) => {
                assert_eq!(cmd.words.len(), 3);
                assert!(cmd.redirects.is_empty());
                // per-word spans now present
                assert_eq!(cmd.words[0].span, Span::new(0, 2));
                assert_eq!(cmd.words[1].span, Span::new(3, 6));
                assert_eq!(cmd.words[2].span, Span::new(7, 11));
                assert_eq!(
                    cmd.words[0].node.parts,
                    vec![WordPart::Literal("ls".to_string())]
                );
                assert_eq!(
                    cmd.words[1].node.parts,
                    vec![WordPart::Literal("-la".to_string())]
                );
                assert_eq!(
                    cmd.words[2].node.parts,
                    vec![WordPart::Literal("/tmp".to_string())]
                );
            }
        }
    }

    #[test]
    fn parse_single_word() {
        let node = parse("pwd").expect("parses");
        match node.node {
            AstNode::Command(cmd) => {
                assert_eq!(cmd.words.len(), 1);
                assert_eq!(
                    cmd.words[0].node.parts,
                    vec![WordPart::Literal("pwd".to_string())]
                );
                assert_eq!(cmd.words[0].span, Span::new(0, 3));
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
        // RFC section 9 rides-with: the spine does NOT lowercase. GitHub stays GitHub.
        let node = parse("GitHub Clone").expect("parses");
        match node.node {
            AstNode::Command(cmd) => {
                assert_eq!(
                    cmd.words[0].node.parts,
                    vec![WordPart::Literal("GitHub".to_string())]
                );
                assert_eq!(
                    cmd.words[1].node.parts,
                    vec![WordPart::Literal("Clone".to_string())]
                );
            }
        }
    }
}
