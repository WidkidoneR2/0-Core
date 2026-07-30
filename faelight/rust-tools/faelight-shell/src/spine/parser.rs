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

use super::ast::{
    AstNode, Command, Pipeline, Redirect, RedirectOp, RedirectTarget, Span, Spanned, SpecialParam,
    VariableSyntax, Word, WordPart,
};
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
    /// A redirect written against an explicit file descriptor (`2>`, `1>>`). INT-200: the same
    /// argument the comment above makes for operator identity, one level down. This was reported as
    /// an ordinary RedirectOut refusal, so the audit could not distinguish *stderr redirects the
    /// spine deliberately leaves to legacy* from *redirects it fails to handle* -- and 2,230
    /// declines looked like unfinished work when most of them are the guard behaving correctly.
    FdRedirect { span: Span },
    /// A `>` whose target begins with a digit or `=`, which fsh reads as a COMPARISON rather
    /// than a redirect. Legacy refuses it so `where cpu > 0.5` works; the spine matches that,
    /// and the audit counts it separately so a deliberate divergence never reads as a gap.
    ComparisonNotRedirect { span: Span },
    /// A redirect with no target after it. Legacy already reports this well (INT-245's bare-redirect
    /// guard), so the spine declines rather than inventing a second message for the same mistake.
    MissingRedirectTarget { kind: OperatorKind, span: Span },
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
    fn parse_command(&mut self) -> Result<Spanned<Command>, ParseError> {
        let mut words: Vec<Spanned<Word>> = Vec::new();
        let mut redirects: Vec<Spanned<Redirect>> = Vec::new();
        let mut cmd_span: Option<Span> = None;

        while let Some(tok) = self.peek() {
            match tok.kind {
                // INT-200: REDIRECTS ARE OURS. `>`, `>>`, `<` and now `>&`.
                TokenKind::Operator(
                    op @ (OperatorKind::RedirectOut
                    | OperatorKind::RedirectAppend
                    | OperatorKind::RedirectIn
                    | OperatorKind::RedirectDup),
                ) => {
                    // ⚠️ THE fd TRAP, NOW A BINDING RATHER THAN A REFUSAL -- and the danger it was
                    // guarding against is still here. The lexer does not make `2>` one token; it
                    // yields Word("2") then the operator. Consuming blindly reads
                    // `cat log 2> /dev/null` as `cat log 2` with stdout redirected, silently turning
                    // the descriptor into an ARGUMENT.
                    //
                    // ★ ADJACENCY DECIDES IT, which is what spans-everywhere is for. `2>` leaves no
                    // gap; `echo 2 > f` does, and that one really is echoing a number. So a bare
                    // numeral touching the operator is CONSUMED as the descriptor -- popped from
                    // `words`, because it was never an argument.
                    let fd = if words.last().is_some_and(|w| {
                        w.span.end == tok.span.start
                            && matches!(w.node.parts.as_slice(), [WordPart::Literal { text, .. }]
                                if !text.is_empty() && text.chars().all(|c| c.is_ascii_digit()))
                    }) {
                        let w = words.pop().expect("checked just above");
                        match &w.node.parts[..] {
                            [WordPart::Literal { text, .. }] => text.parse::<u32>().ok(),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let opspan = tok.span;
                    self.advance().expect("peek was Some");
                    // A redirect with no target is a syntax error legacy already reports well
                    // (INT-245's bare-redirect guard). Refuse rather than inventing a second
                    // message for the same mistake.
                    let target_tok = match self.peek() {
                        Some(t) if t.kind == TokenKind::Word => {
                            self.advance().expect("peek was Some")
                        }
                        _ => {
                            return Err(ParseError::MissingRedirectTarget {
                                kind: op,
                                span: opspan,
                            })
                        }
                    };
                    let tspan = target_tok.span;
                    let mut parts: Vec<WordPart> = Vec::new();
                    for seg in &target_tok.segments {
                        parts.extend(parts_from_segment(seg)?);
                    }
                    let target = if op == OperatorKind::RedirectDup {
                        // `2>&1` names a STREAM. A non-numeric target after `>&` is shell fsh does
                        // not model (`2>&-` closes a descriptor), so it is declined rather than
                        // guessed at.
                        match parts.as_slice() {
                            [WordPart::Literal { text, .. }]
                                if text.chars().all(|c| c.is_ascii_digit()) =>
                            {
                                match text.parse::<u32>() {
                                    Ok(n) => RedirectTarget::Stream(n),
                                    Err(_) => {
                                        return Err(ParseError::MissingRedirectTarget {
                                            kind: op,
                                            span: opspan.merge(tspan),
                                        })
                                    }
                                }
                            }
                            _ => {
                                return Err(ParseError::MissingRedirectTarget {
                                    kind: op,
                                    span: opspan.merge(tspan),
                                })
                            }
                        }
                    } else {
                        // ⚠️ A COMPARISON IS NOT A REDIRECT -- a DELIBERATE DIVERGENCE FROM BASH,
                        // which would create a file named `0.5`. Legacy refuses it, prints the line
                        // as text, and that refusal is why `ps | where cpu > 0.5` works at all.
                        //
                        // ★ SCOPED TO FILE TARGETS ONLY, and that scoping is load-bearing: `2>&1`
                        // has the target `1`, which starts with a digit. Applying this guard to a
                        // stream target would refuse the single most common fd form in the corpus.
                        if let Some(WordPart::Literal { text, .. }) = parts.first() {
                            if text
                                .chars()
                                .next()
                                .is_some_and(|c| c.is_ascii_digit() || c == '=')
                            {
                                return Err(ParseError::ComparisonNotRedirect {
                                    span: opspan.merge(tspan),
                                });
                            }
                        }
                        RedirectTarget::File(Word { parts })
                    };
                    // `>&` is an OUTPUT redirection whose destination happens to be a stream, so the
                    // op stays Write and the TARGET carries the difference. A separate Dup op would
                    // encode the same fact twice.
                    let rop = match op {
                        OperatorKind::RedirectAppend => RedirectOp::Append,
                        OperatorKind::RedirectIn => RedirectOp::Read,
                        _ => RedirectOp::Write,
                    };
                    redirects.push(Spanned::new(
                        opspan.merge(tspan),
                        Redirect {
                            fd,
                            op: rop,
                            target,
                        },
                    ));
                    cmd_span = Some(match cmd_span {
                        None => opspan.merge(tspan),
                        Some(s) => s.merge(tspan),
                    });
                }
                // INT-169: DECLINE OWNERSHIP for everything else, and do not advance. The spine
                // parser is not learning pipelines here -- it is learning where its language ENDS.
                // ★ A PIPE ENDS THIS COMMAND, it does not end the parse. Left UNCONSUMED so
                // parse_line owns the stage boundary -- the parser that builds a stage should
                // not also decide what follows it.
                TokenKind::Operator(OperatorKind::Pipe) => break,
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
            Some(sp) => Ok(Spanned::new(sp, Command { words, redirects })),
        }
    }

    /// Top-level parse: the whole line -> one node. Owns the STAGE BOUNDARY -- `parse_command`
    /// builds a single stage and stops at a pipe without consuming it, so the decision about what
    /// follows a command lives here rather than inside the thing being built.
    ///
    /// ★ ONE STAGE YIELDS `Command`, NOT A ONE-STAGE PIPELINE. Wrapping every command would give
    /// consumers two spellings of the same thing and make the audit report a construct change where
    /// the user typed none.
    ///
    /// ⚠️ A trailing or doubled pipe (`ls |`, `a || b` mis-lexed) leaves a stage with no words.
    /// `parse_command` already answers `Empty` for that, and it propagates -- the parser declines
    /// rather than inventing an empty stage the executor would have to interpret.
    fn parse_line(&mut self) -> Result<Spanned<AstNode>, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::Empty);
        }
        let first = self.parse_command()?;
        if !matches!(
            self.peek().map(|t| t.kind),
            Some(TokenKind::Operator(OperatorKind::Pipe))
        ) {
            let span = first.span;
            return Ok(Spanned::new(span, AstNode::Command(first.node)));
        }
        let mut stages = vec![first];
        while matches!(
            self.peek().map(|t| t.kind),
            Some(TokenKind::Operator(OperatorKind::Pipe))
        ) {
            self.advance().expect("peek was Some");
            stages.push(self.parse_command()?);
        }
        let span = stages
            .first()
            .expect("at least one stage")
            .span
            .merge(stages.last().expect("at least one stage").span);
        Ok(Spanned::new(span, AstNode::Pipeline(Pipeline { stages })))
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

    /// INT-200: `AstNode` gained `Pipeline`, so every `let AstNode::Command(c) = …` in these tests
    /// became a refutable binding and every `match` became non-exhaustive. These cases assert
    /// COMMAND shape and are not about pipelines, so they say so once here rather than each growing
    /// an arm that panics.
    ///
    /// ⚠️ Deliberately PANICS on a pipeline instead of returning Option. A test that silently
    /// skipped when the parse changed shape would go green while proving nothing.
    fn as_command(node: Spanned<AstNode>) -> Command {
        match node.node {
            AstNode::Command(c) => c,
            AstNode::Pipeline(p) => {
                panic!("expected a single Command, got {} stages", p.stages.len())
            }
        }
    }

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
            AstNode::Pipeline(p) => panic!("expected Command, got {} stages", p.stages.len()),
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
            AstNode::Pipeline(p) => panic!("expected Command, got {} stages", p.stages.len()),
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
            let AstNode::Command(cmd) = &ast.node else {
                panic!("{source} should parse as a Command")
            };
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
            // `| ` left this list in INT-200: it is OWNED now, not refused.
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

    /// ⚠️ REDIRECTS LEFT THIS LIST DELIBERATELY (INT-200). They are no longer declined at PARSE
    /// time -- the parser builds them into `Command.redirects`. They are still declined at
    /// LOWERING, which is a different boundary and has its own test. The distinction matters: a
    /// construct the parser understands but the executor cannot run is a capability gap, not a
    /// grammar gap, and the audit counts them differently.
    #[test]
    fn redirects_parse_into_structure_rather_than_being_refused() {
        for (source, op, target) in [
            ("ls > out.txt", RedirectOp::Write, "out.txt"),
            ("ls >> out.txt", RedirectOp::Append, "out.txt"),
            ("wc -l < in.txt", RedirectOp::Read, "in.txt"),
        ] {
            let node = parse(source).unwrap_or_else(|e| panic!("{source} should parse: {e:?}"));
            let cmd = as_command(node);
            assert_eq!(cmd.redirects.len(), 1, "{source}");
            let r = &cmd.redirects[0].node;
            assert_eq!(r.op, op, "{source}");
            assert_eq!(r.fd, None, "{source} has no explicit fd");
            assert_eq!(
                match &r.target {
                    RedirectTarget::File(w) => w.parts.clone(),
                    RedirectTarget::Stream(n) =>
                        panic!("{source} is a file redirect, got stream {n}"),
                },
                vec![WordPart::Literal {
                    text: target.to_string(),
                    quoted: QuoteContext::Unquoted
                }],
                "{source} target is kept UNEXPANDED -- a redirect target is a word like any other"
            );
        }
    }

    /// ★ THE fd TRAP, NOW PROVEN AS A BINDING RATHER THAN A REFUSAL -- and this is the assertion
    /// that must never be loosened. The lexer yields Word("2") then the operator, so a parser that
    /// consumed the operator without claiming the numeral would read `cat log 2> /dev/null` as
    /// `cat log 2` with STDOUT redirected: the descriptor silently becomes an argument, the wrong
    /// stream is captured, and nothing errors.
    ///
    /// ⚠️ So the test asserts BOTH halves: the fd landed on the redirect AND the numeral left argv.
    /// Checking only the first would still pass if `2` were duplicated into the arguments.
    #[test]
    fn an_fd_prefixed_redirect_binds_the_descriptor() {
        let node = parse("cat log 2> /dev/null").expect("parses");
        let cmd = as_command(node);
        assert_eq!(
            cmd.words.len(),
            2,
            "the descriptor must NOT survive as an argument"
        );
        assert_eq!(cmd.redirects.len(), 1);
        let r = &cmd.redirects[0].node;
        assert_eq!(r.fd, Some(2), "stderr, not the default stdout");
        assert_eq!(r.op, RedirectOp::Write);

        // ...and a SPACED numeral is an ordinary argument, not a descriptor. Adjacency is the whole
        // distinction, so both sides of it are asserted here rather than in separate tests.
        let node = parse("echo 2 > f").expect("spaced numeral parses");
        let cmd = as_command(node);
        assert_eq!(cmd.words.len(), 2, "echo and 2");
        assert_eq!(cmd.redirects.len(), 1);
        assert_eq!(cmd.redirects[0].node.fd, None, "no descriptor was written");
    }

    /// `2>&1` -- the most common fd form in six months of history, and the one that needs the
    /// lexer's `>&` token. Without it the scanner produces RedirectOut then Background and the
    /// parser looks for a target word where an operator sits.
    #[test]
    fn a_dup_redirect_binds_both_descriptors() {
        let node = parse("ls /nope 2>&1").expect("parses");
        let cmd = as_command(node);
        assert_eq!(cmd.words.len(), 2, "ls and the path -- no stray numerals");
        assert_eq!(cmd.redirects.len(), 1);
        let r = &cmd.redirects[0].node;
        assert_eq!(r.fd, Some(2), "stderr is the stream being redirected");
        assert_eq!(
            r.target,
            RedirectTarget::Stream(1),
            "the destination is a STREAM, not a file named 1"
        );
    }

    /// The refusal points at the OPERATOR, not at the whole command. The input is not malformed --
    /// it is valid shell belonging to a grammar this parser does not implement -- so the span marks
    /// the ownership boundary rather than blaming the line.
    #[test]
    /// ⚠️ The example moved from a pipe to `&&` (INT-200): a pipe now PARSES into a Pipeline,
    /// so it no longer demonstrates a refusal. The property under test is unchanged -- the span
    /// marks the operator, not the line.
    fn the_refusal_points_at_the_operator() {
        let source = "a && b";
        match parse(source) {
            Err(ParseError::UnsupportedOperator { span, .. }) => {
                assert_eq!(&source[span.start..span.end], "&&");
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
            AstNode::Pipeline(p) => panic!("expected Command, got {} stages", p.stages.len()),
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
