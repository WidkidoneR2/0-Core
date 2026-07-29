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

use super::ast::{AstNode, Command, Span, Spanned, SpecialParam, VariableSyntax, Word, WordPart};
use super::lexer::{
    lex, LexError, OperatorKind, QuoteContext, SpannedToken, TokenKind, WordSegment,
};

/// Parse errors carry a span so they can point at exact source (RFC section 4.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The lexer rejected the input.
    Lex(LexError),
    /// Nothing to parse (empty / whitespace-only input).
    Empty,
    /// A shell operator the spine's grammar does not cover. INT-169: THE REFUSAL BOUNDARY.
    ///
    /// ★ NOT "unexpected". The parser knows perfectly well the lexer can produce operators -- this
    /// is valid shell that belongs to a grammar this parser does not implement. That is a different
    /// diagnostic from malformed input, and conflating them would make the eventual router unable to
    /// tell "not mine" from "broken".
    ///
    /// ⚠️ Also distinct from `LowerError::UnsupportedConstruct`, despite the family resemblance.
    /// That one means "I understand this AST shape but cannot execute it"; this one means "this
    /// token stream is outside my grammar". `echo a | b` may one day parse into a Pipeline and
    /// STILL be refused at lowering -- different stages, different errors.
    ///
    /// Carries the operator identity, not just a span, because a refusal that cannot say WHICH
    /// construct it declined is a worse diagnostic and a weaker test.
    UnsupportedOperator { kind: OperatorKind, span: Span },
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
                // INT-169: DECLINE OWNERSHIP, and do not advance. The spine parser is not learning
                // pipelines here -- it is learning where its language ENDS. The token is left in
                // place because refusing is not consuming.
                TokenKind::Operator(kind) => {
                    return Err(ParseError::UnsupportedOperator {
                        kind,
                        span: tok.span,
                    })
                }
                TokenKind::Word => {
                    let t = self.advance().expect("peek was Some");
                    let wspan = t.span;
                    // Each lexical segment becomes a WordPart. Today every segment maps to a
                    // Literal regardless of its quote context. At step 3 an unquoted or
                    // double-quoted segment may yield SEVERAL parts (Literal + Variable) while
                    // a single-quoted segment stays one Literal -- and that is a change HERE,
                    // in the mapping, not in the lexer.
                    // INT-169 blocker 4, step 1b-ii: `?` rather than `flat_map`. A command
                    // substitution is an embedded shell PROGRAM, so if it fails to parse the
                    // OUTER parse must fail too -- degrading it to text would erase a syntax
                    // error and let a later stage run something other than what was written.
                    let mut parts: Vec<WordPart> = Vec::new();
                    for seg in &t.segments {
                        parts.extend(parts_from_segment(seg)?);
                    }
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
fn parts_from_segment(seg: &WordSegment) -> Result<Vec<WordPart>, ParseError> {
    match seg {
        WordSegment::Text { text, context, .. } => Ok(match context {
            QuoteContext::Single => vec![WordPart::Literal {
                text: text.clone(),
                quoted: QuoteContext::Single,
            }],
            QuoteContext::Unquoted | QuoteContext::Double => recognise_variables(text, *context),
        }),
        // INT-169 blocker 4, step 1b-i: BEHAVIOUR-PRESERVING PLACEHOLDER. The scanner now hands
        // over the region structurally, but this reconstructs the raw text so the AST is byte for
        // byte what it was before the enum landed -- which is what lets the existing tests prove
        // the refactor changed nothing. Step 1b-ii replaces THIS ARM ONLY, with a recursive parse
        // into `WordPart::CommandSub` and `Result` propagation, because an embedded shell program
        // that fails to parse must not be silently degraded into text.
        WordSegment::CommandSub { source, .. } => {
            // Recursion terminates by construction: the inner source of a `$(...)` is strictly
            // shorter than the text that contained it, so each call operates on a smaller region.
            let inner = parse(source)?;
            Ok(vec![WordPart::CommandSub(Box::new(inner))])
        }
    }
}

/// Split text into alternating Literal / Variable parts. RECOGNITION ONLY -- nothing is
/// evaluated here, and the resulting plan still renders `$NAME` back out, so behaviour is
/// unchanged (see plan::expand_word). Evaluation is the separate variable-expansion milestone.
///
/// Recognises the plain `$NAME` form with a POSIX-shaped name. Deliberately NOT yet: `${NAME}`,
/// `${NAME:-default}`, `$?`, `$$`, positional `$1`. A `$` not starting a valid name is literal.
fn recognise_variables(text: &str, quoted: QuoteContext) -> Vec<WordPart> {
    let mut parts: Vec<WordPart> = Vec::new();
    let mut literal = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;

    while i < chars.len() {
        if chars[i] != '$' {
            literal.push(chars[i]);
            i += 1;
            continue;
        }
        // DISPATCH ON WHAT FOLLOWS `$`. Every expansion form lives in this one match, so adding
        // a form is adding an arm rather than another nested if:
        //     $?         -> SpecialParam(LastExit)
        //     $$         -> SpecialParam(Pid)
        //     ${NAME}    -> Variable(NAME)      (braces are SYNTAX, not semantics)
        //     $NAME      -> Variable(NAME)
        //     anything else -> a literal dollar sign
        match chars.get(i + 1).copied() {
            Some('?') => {
                flush(&mut parts, &mut literal, quoted);
                parts.push(WordPart::SpecialParam(SpecialParam::LastExit));
                i += 2;
            }
            Some('$') => {
                flush(&mut parts, &mut literal, quoted);
                parts.push(WordPart::SpecialParam(SpecialParam::Pid));
                i += 2;
            }
            Some('{') => match brace_identifier(&chars, i) {
                Some((name, next_i)) => {
                    flush(&mut parts, &mut literal, quoted);
                    parts.push(WordPart::Variable {
                        name,
                        syntax: VariableSyntax::Braced,
                    });
                    i = next_i;
                }
                // NOT a plain ${NAME} -- so `${VAR:-default}`, `${#VAR}`, `${VAR/a/b}` and an
                // unterminated brace all stay LITERAL. The spine does not implement those forms,
                // so it does not pretend to understand them.
                //
                // Legacy differs here, and this is deliberate divergence rather than an
                // oversight: expand_vars reads the whole brace body as a NAME, looks up a
                // variable literally called "VAR:-default", finds nothing, and substitutes the
                // EMPTY STRING. That is a legacy bug. Leaving the text intact is the honest
                // "I do not understand this yet", and it makes the difference VISIBLE in the
                // migration audit instead of silently wrong.
                None => {
                    literal.push('$');
                    i += 1;
                }
            },
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                let start = i + 1;
                let mut j = start;
                while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                    j += 1;
                }
                flush(&mut parts, &mut literal, quoted);
                parts.push(WordPart::Variable {
                    name: chars[start..j].iter().collect(),
                    syntax: VariableSyntax::Bare,
                });
                i = j;
            }
            _ => {
                literal.push('$');
                i += 1;
            }
        }
    }

    flush(&mut parts, &mut literal, quoted);
    // An all-empty segment (`echo ""`) must still yield one part -- it is a real empty argument.
    if parts.is_empty() {
        parts.push(WordPart::Literal {
            text: String::new(),
            quoted,
        });
    }
    parts
}

/// Close off any pending literal run before pushing a non-literal part.
fn flush(parts: &mut Vec<WordPart>, literal: &mut String, quoted: QuoteContext) {
    if !literal.is_empty() {
        parts.push(WordPart::Literal {
            text: std::mem::take(literal),
            quoted,
        });
    }
}

/// `${NAME}` where NAME is a POSIX identifier. Returns the name and the index just past the
/// closing brace, or None if the braces do not contain EXACTLY an identifier -- which is what
/// keeps `${VAR:-default}` and friends as literal text rather than a fabricated lookup.
fn brace_identifier(chars: &[char], dollar: usize) -> Option<(String, usize)> {
    let start = dollar + 2;
    let mut j = start;
    while j < chars.len() && chars[j] != '}' {
        j += 1;
    }
    if j >= chars.len() {
        return None;
    }
    let name: String = chars[start..j].iter().collect();
    let mut cs = name.chars();
    let first = cs.next()?;
    if !(first.is_ascii_alphabetic() || first == '_') {
        return None;
    }
    if !cs.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    Some((name, j + 1))
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
                    vec![WordPart::Literal {
                        text: "ls".to_string(),
                        quoted: QuoteContext::Unquoted,
                    }]
                );
                assert_eq!(
                    cmd.words[1].node.parts,
                    vec![WordPart::Literal {
                        text: "-la".to_string(),
                        quoted: QuoteContext::Unquoted,
                    }]
                );
                assert_eq!(
                    cmd.words[2].node.parts,
                    vec![WordPart::Literal {
                        text: "/tmp".to_string(),
                        quoted: QuoteContext::Unquoted,
                    }]
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
                    vec![WordPart::Literal {
                        text: "pwd".to_string(),
                        quoted: QuoteContext::Unquoted,
                    }]
                );
                assert_eq!(cmd.words[0].span, Span::new(0, 3));
            }
        }
        assert_eq!(node.span, Span::new(0, 3));
    }

    /// INT-169 blocker 5: QUOTING SURVIVES PARSING. A shell expands `*` as a pathname pattern
    /// but treats `'*'` and `"*"` as literal text, so the AST has to record which was written.
    ///
    /// ★ This asserts the FACT, not merely that the three differ. The contract is that the AST
    /// remembers WHY they differ, because pathname expansion, field splitting and quote removal
    /// will each derive their own rule from the same fact. It deliberately does not mention
    /// globbing: the guarantee is about the parser, and it held before any glob code existed.
    #[test]
    fn quoting_survives_into_word_parts() {
        let cases = [
            ("echo *", QuoteContext::Unquoted),
            ("echo '*'", QuoteContext::Single),
            ("echo \"*\"", QuoteContext::Double),
        ];
        for (source, expected) in cases {
            let ast = parse(source).expect("parses");
            let AstNode::Command(cmd) = &ast.node;
            assert_eq!(
                cmd.words[1].node.parts,
                vec![WordPart::Literal {
                    text: "*".to_string(),
                    quoted: expected,
                }],
                "{source} must record its quoting"
            );
        }
    }

    /// ★ THE REFUSAL BOUNDARY, which is the whole point of the operator tokens. The spine does not
    /// implement pipelines, redirects, boolean chains, sequences or background execution -- and
    /// before this it could not SAY so, because those characters were ordinary word content. A
    /// router built on the old parser would have claimed every one of these.
    #[test]
    fn the_parser_declines_shell_it_does_not_implement() {
        for (source, want) in [
            ("echo hi | grep h", OperatorKind::Pipe),
            ("ls > out.txt", OperatorKind::RedirectOut),
            ("ls >> out.txt", OperatorKind::RedirectAppend),
            ("wc -l < in.txt", OperatorKind::RedirectIn),
            ("a && b", OperatorKind::And),
            ("a || b", OperatorKind::Or),
            ("a ; b", OperatorKind::Sequence),
            ("sleep 5 &", OperatorKind::Background),
        ] {
            match parse(source) {
                Err(ParseError::UnsupportedOperator { kind, .. }) => {
                    assert_eq!(kind, want, "{source} declined as the wrong construct")
                }
                other => panic!("{source} was not declined: {other:?}"),
            }
        }
    }

    /// The refusal points at the OPERATOR, not at the whole command. The input is not malformed --
    /// it is valid shell belonging to a grammar this parser does not implement -- so the span marks
    /// the ownership boundary rather than blaming the line.
    #[test]
    fn the_refusal_points_at_the_operator() {
        let source = "echo hi | grep h";
        match parse(source) {
            Err(ParseError::UnsupportedOperator { span, .. }) => {
                assert_eq!(&source[span.start..span.end], "|");
            }
            other => panic!("expected a refusal, got {other:?}"),
        }
    }

    /// ⚠️ THE INVERSE, and the one that keeps the boundary honest: a QUOTED operator is an argument,
    /// so these must still parse. A refusal that fired here would hand perfectly good commands to
    /// the legacy path forever and nobody would notice.
    #[test]
    fn a_quoted_operator_is_still_ours() {
        for source in ["echo \"|\"", "echo \'&&\'", "echo \">\""] {
            assert!(parse(source).is_ok(), "{source} should parse");
        }
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
                    vec![WordPart::Literal {
                        text: "GitHub".to_string(),
                        quoted: QuoteContext::Unquoted,
                    }]
                );
                assert_eq!(
                    cmd.words[1].node.parts,
                    vec![WordPart::Literal {
                        text: "Clone".to_string(),
                        quoted: QuoteContext::Unquoted,
                    }]
                );
            }
        }
    }
}
