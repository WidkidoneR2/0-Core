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
    AstNode, Command, Pipeline, Redirect, RedirectOp, RedirectTarget, Sequence, SequenceOp, Span,
    Spanned, SpecialParam, VariableSyntax, Word, WordPart,
};
use super::lexer::{
    lex, LexIncomplete, LexResult, OperatorKind, QuoteContext, SpannedToken, TokenKind, WordSegment,
};

/// Parse errors carry a span so they can point at exact source (RFC section 4.2).
/// INT-169 G2: valid shell the spine INTENTIONALLY does not own.
///
/// ⚠️ NOT AN ERROR, AND NOT A ParseError VARIANT. If the router has to inspect an error to discover
/// whether it should fall back, the ambiguity this type exists to remove is still there. A refusal is
/// an OWNERSHIP decision the parser makes and the router consumes.
///
/// Named `Refusal` rather than `Unsupported` deliberately: it describes the ownership contract, and
/// the particular reason may be an unsupported operator today and something else tomorrow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// A shell operator outside the spine's grammar. The parser knows the lexer produces these; it
    /// declines to own them. Carries the identity so a refusal can say WHICH construct it declined.
    UnsupportedOperator { kind: OperatorKind, span: Span },
    /// A redirect against an explicit file descriptor (`2>`, `1>>`), deliberately left to legacy.
    FdRedirect { span: Span },
    /// `where cpu > 0.5` is a comparison, not a redirect. A deliberate divergence from bash, which
    /// would create a file named `0.5`.
    ComparisonNotRedirect { span: Span },
}

/// ⭐ INT-169 G2: THE ROUTER CONTRACT, EXPRESSED AS A TYPE.
///
/// The invariant: LEGACY FALLBACK IS AN EXPLICIT PARSER REFUSAL, NOT THE GENERIC RECOVERY MECHANISM
/// FOR PARSER FAILURE.
///
/// The router already made this four-way distinction -- by pattern-matching on error variants, with
/// `Err(SpineAttemptError::Parse(_)) => Declined` as a catch-all. That catch-all silently routed
/// incompleteness, emptiness, real refusals AND any future parse error to legacy alike. Making the
/// distinction a type means the router reads a decision instead of re-deriving one, and a new parser
/// outcome becomes a COMPILE ERROR at the router rather than a silent decline.
///
///   Complete   -- the spine owns execution of this input.
///   Incomplete -- may become spine-owned when more input arrives.
///   Refused    -- valid shell, intentionally outside the grammar. Route to legacy.
///   Invalid    -- malformed. SURFACE the defect; never silently route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseResult {
    Complete(Spanned<AstNode>),
    Incomplete(LexIncomplete),
    Refused(Refusal),
    Invalid(ParseError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// INT-169 G1: the scanner reported the line is not FINISHED, not that it is wrong.
    ///
    /// This was `Lex(LexError)`, and it carried the same lie the lexer did: the only two things the
    /// scanner could ever report -- an unterminated quote and an unterminated `$(` -- are
    /// incompleteness. Naming it a lex error made every caller treat unfinished input as failure,
    /// which is why the validator had to translate it back and why the migration audit counts
    /// unterminated quotes as parse errors against the shell.
    ///
    /// ⚠️ It stays a ParseError variant only until G2, where ParseResult gains a real Incomplete arm
    /// and this stops needing to travel inside a failure type at all.
    Incomplete(LexIncomplete),
    /// ⭐ INT-169 G2: THE PARSER FOUND SHELL STRUCTURE REQUIRING A COMMAND, AND NO COMMAND EXISTS
    /// AT THAT POSITION. Renamed from `Empty`, and the old name was actively dangerous: it invited
    /// the invariant "only a caller passing an empty string can produce this", which a proptest
    /// disproved with a one-character counter-example -- `&`. A bare `&` is not empty by trim(); the
    /// lexer recognises an operator, the grammar requires a command where the operator leaves none,
    /// and so there is no valid shell construct here for the spine to decline.
    ///
    /// That makes it INVALID rather than Refused: a refusal means valid shell the spine intentionally
    /// does not own, and this is not valid shell.
    NoCommand,
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
                        // ⚠️ AND SCOPED TO QUERY-SHAPED LINES ONLY, which is the second half of the
                        // scoping and was missing until 2026-08-07. Asking only "does the target
                        // start with a digit" made `echo test > 0.5` a comparison too, which is a
                        // divergence from bash with nothing to gain: bash writes a file called 0.5
                        // and so should fsh. The reason the guard exists is the QUERY language, so
                        // it should only fire for a line written in that language.
                        //
                        // The test is the vocabulary itself, asked rather than re-invented: has any
                        // word so far named a value verb or source? `where cpu` has. `select * from
                        // ps where cpu` has. `echo test` has not, so its `>` is an ordinary redirect.
                        //
                        // ⚠️ A PIPELINE ALONE IS NOT ENOUGH TO DECIDE THIS. `select * from ps where
                        // cpu > 1` has no pipe and begins with a verb rather than a source, so the
                        // is_forest_pipeline check cannot rescue it -- without this guard the spine
                        // would claim it and write a file called `1` instead of running the query.
                        let query_shaped = words.iter().any(|w| {
                            matches!(
                                w.node.parts.as_slice(),
                                [WordPart::Literal { text, .. }]
                                    if crate::value::is_value_verb(text)
                                        || crate::value::is_value_source(text)
                            )
                        });
                        if query_shaped {
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
                // ★ A CONNECTOR ENDS THIS COMMAND, it does not end the parse -- and it is left
                // UNCONSUMED so the layer that owns the boundary decides what follows. A pipe
                // belongs to `parse_pipeline`; `&&`, `||` and `;` belong to `parse_line`.
                //
                // ⚠️ WITHOUT THE THREE CONNECTORS HERE, `parse_command` refuses them through the
                // catch-all below and the sequence layer never runs at all -- which is exactly
                // what the refusal test caught: it stayed green because nothing reached the new
                // code, not because the new code was right.
                TokenKind::Operator(
                    OperatorKind::Pipe
                    | OperatorKind::And
                    | OperatorKind::Or
                    | OperatorKind::Sequence
                    // INT-200: `&` binds LOOSEST of all, so it is left for `parse_line`
                    // exactly as the connectors are. Without this it fell to the catch-all
                    // below and `parse_line`'s Background branch was unreachable -- the same
                    // failure the comment above records for the three connectors, repeated.
                    | OperatorKind::Background,
                ) => break,
                // ⚠️ NO CATCH-ALL, AND THAT IS THE GUARD. A `TokenKind::Operator(kind) => Err(…)`
                // arm lived here until every operator became owned -- four redirect kinds above,
                // five connector kinds just now -- at which point it went unreachable. Deleting it
                // silently would have been the easy move and the wrong one: a TENTH OperatorKind
                // added to the lexer would then fall through to `TokenKind::Word` below and be
                // treated as ordinary text, which is exactly the mis-execution the lexer's own
                // comment warns about for `&` ("an incomplete refusal is a mis-execution").
                //
                // ★ Instead the two arms above name every variant EXPLICITLY, so adding one to
                // OperatorKind makes THIS MATCH non-exhaustive and the compiler demands a decision:
                // does the new operator end a command, or belong inside one? Construction over
                // vigilance -- the same reason `CommandResult::Error` was widened rather than
                // joined by a variant that 18 wildcard arms would have swallowed.
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
            None => Err(ParseError::NoCommand),
            Some(sp) => Ok(Spanned::new(sp, Command { words, redirects })),
        }
    }

    /// Parse ONE PIPELINE. Owns the STAGE boundary -- `parse_command` builds a single stage and
    /// stops at a pipe without consuming it, so the decision about what follows a command lives
    /// here rather than inside the thing being built.
    ///
    /// ★ ONE STAGE YIELDS `Command`, NOT A ONE-STAGE PIPELINE. Wrapping every command would give
    /// consumers two spellings of the same thing and make the audit report a construct change where
    /// the user typed none.
    ///
    /// ⚠️ A trailing or doubled pipe (`ls |`) leaves a stage with no words. `parse_command` already
    /// answers `Empty` for that, and it propagates -- the parser declines rather than inventing an
    /// empty stage the executor would have to interpret.
    fn parse_pipeline(&mut self) -> Result<Spanned<AstNode>, ParseError> {
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

    /// Top-level parse: the whole line -> one node. Owns the SEQUENCE boundary, one level above the
    /// stage boundary, so each layer decides exactly one thing.
    ///
    /// ★ LEFT-ASSOCIATIVE AND FLAT, as bash is: `a && b && c` is ONE sequence of three items, not a
    /// tree of two. Nesting would give the same line two spellings and force every consumer to walk
    /// a structure the user never wrote.
    ///
    /// ★ A LONE PIPELINE STAYS A PIPELINE. Only a real connector produces a `Sequence`, for the same
    /// reason a single command never becomes a one-stage pipeline.
    ///
    /// ⚠️ A TRAILING CONNECTOR (`ls &&`) leaves nothing to parse after it, and `parse_command`
    /// answers `Empty` -- which propagates as a refusal rather than an empty item. Legacy reports
    /// that case perfectly well; inventing a second diagnostic would be the mistake INT-245 warns
    /// about.
    fn parse_line(&mut self) -> Result<Spanned<AstNode>, ParseError> {
        let node = self.parse_sequence()?;
        // INT-200: `&` binds LOOSEST -- it applies to everything parsed so far, which is why it
        // is read here and not inside parse_command. bash treats it as a connector rather than a
        // suffix, and OSH wraps the command in a Sentence; both make it a NODE, not a flag.
        if matches!(
            self.peek().map(|t| t.kind),
            Some(TokenKind::Operator(OperatorKind::Background))
        ) {
            let amp = self.advance().expect("peek was Some");
            // ⚠️ A trailing `&` after a `&&`/`||` list is REFUSED, not wrapped. Background applies
            // to an entire subtree; if the splitter in main.rs has already separated the list,
            // the correct operand is unavailable here and wrapping what DID arrive would
            // background only the tail -- plausible output, wrong semantics. Declining keeps the
            // limitation structural rather than semantic: the AST never claims a scope it did
            // not observe. If the splitter ever learns that `&` binds looser than `&&`, this
            // check simply stops firing and the parser is unchanged.
            if matches!(node.node, AstNode::Sequence(_)) {
                return Err(ParseError::UnsupportedOperator {
                    kind: OperatorKind::Background,
                    span: amp.span,
                });
            }
            if !self.is_at_end() {
                return Err(ParseError::UnsupportedOperator {
                    kind: OperatorKind::Background,
                    span: amp.span,
                });
            }
            let span = node.span.merge(amp.span);
            return Ok(Spanned::new(span, AstNode::Background(Box::new(node))));
        }
        Ok(node)
    }

    /// The `&&` / `||` / `;` layer, split out of parse_line so `&` can wrap its RESULT.
    fn parse_sequence(&mut self) -> Result<Spanned<AstNode>, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::NoCommand);
        }
        let first = self.parse_pipeline()?;
        let connector = |k: Option<TokenKind>| match k {
            Some(TokenKind::Operator(OperatorKind::And)) => Some(SequenceOp::AndThen),
            Some(TokenKind::Operator(OperatorKind::Or)) => Some(SequenceOp::OrElse),
            Some(TokenKind::Operator(OperatorKind::Sequence)) => Some(SequenceOp::Always),
            _ => None,
        };
        if connector(self.peek().map(|t| t.kind)).is_none() {
            return Ok(first);
        }
        let start = first.span;
        let mut rest: Vec<(SequenceOp, Spanned<AstNode>)> = Vec::new();
        while let Some(op) = connector(self.peek().map(|t| t.kind)) {
            self.advance().expect("peek was Some");
            rest.push((op, self.parse_pipeline()?));
        }
        let span = start.merge(rest.last().expect("at least one item").1.span);
        Ok(Spanned::new(
            span,
            AstNode::Sequence(Sequence {
                first: Box::new(first),
                rest,
            }),
        ))
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
            // INT-169 G2: the INNER parser is building an AST fragment, not deciding routing, so it
            // calls parse_inner and keeps ParseError as its vocabulary. The four-way distinction is
            // preserved by the top-level mapping rather than collapsed here: a refusal inside `$( )`
            // becomes a refusal of the whole construct, an invalid inner becomes Invalid, and an
            // incomplete inner becomes Incomplete -- because the spine does not own the outer command
            // either way.
            let inner = parse_inner(source)?;
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
pub fn parse(source: &str) -> ParseResult {
    match parse_inner(source) {
        Ok(node) => ParseResult::Complete(node),
        Err(ParseError::Incomplete(i)) => ParseResult::Incomplete(i),
        // ⭐ THE OWNERSHIP DECISION IS MADE HERE, ONCE, rather than re-derived by whoever routes.
        Err(ParseError::UnsupportedOperator { kind, span }) => {
            ParseResult::Refused(Refusal::UnsupportedOperator { kind, span })
        }
        Err(ParseError::FdRedirect { span }) => ParseResult::Refused(Refusal::FdRedirect { span }),
        Err(ParseError::ComparisonNotRedirect { span }) => {
            ParseResult::Refused(Refusal::ComparisonNotRedirect { span })
        }
        // ⚠️ NoCommand IS NOT "the caller passed an empty string" -- a proptest disproved that with
        // a one-character counter-example, `&`. See the variant's own doc. It is malformed shell:
        // the grammar wanted a command and the input has none, so it surfaces rather than routing.
        // Structurally malformed: the grammar wanted a command and the input has none. Surfaced,
        // never routed -- see the variant's own doc for why `&` is Invalid rather than Refused.
        Err(e @ ParseError::NoCommand) => ParseResult::Invalid(e),
        // Genuinely malformed: the user's mistake, already located. Surfaced, never routed.
        Err(e @ ParseError::MissingRedirectTarget { .. }) => ParseResult::Invalid(e),
    }
}

/// Test-only conveniences, per the precedent LexResult set: the tests are part of this module's
/// contract and evolve with the model, rather than being handed Result-shaped accessors in
/// production. Each names the arm it asserts, so a case landing on another one says which.
#[cfg(test)]
impl ParseResult {
    pub(crate) fn expect_complete(self, msg: &str) -> Spanned<AstNode> {
        match self {
            ParseResult::Complete(n) => n,
            other => panic!("{msg}: got {other:?}"),
        }
    }
    pub(crate) fn is_complete(&self) -> bool {
        matches!(self, ParseResult::Complete(_))
    }
}

/// The parser proper. Internal, so `ParseError` stays the parser's own vocabulary while
/// `ParseResult` is what execution consumes.
fn parse_inner(source: &str) -> Result<Spanned<AstNode>, ParseError> {
    // INT-169 G1: a lex outcome is Complete or Incomplete, never an error. parse still returns a
    // Result until G2 introduces ParseResult, so incompleteness travels as a NAMED ParseError
    // variant rather than being disguised as a lexical failure.
    let tokens = match lex(source) {
        LexResult::Complete(t) => t,
        LexResult::Incomplete(i) => return Err(ParseError::Incomplete(i)),
    };
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
            // A wrapper would mean the case under test grew a trailing `&`. Panic rather than
            // unwrap: these assert a bare Command, and silently looking through a Background
            // would let the test pass while proving something else.
            AstNode::Background(_) => panic!("expected a Command, got a backgrounded node"),
            AstNode::Sequence(s) => {
                panic!("expected a Command, got a sequence of {}", s.rest.len() + 1)
            }
            AstNode::Command(c) => c,
            AstNode::Pipeline(p) => {
                panic!("expected a single Command, got {} stages", p.stages.len())
            }
        }
    }

    #[test]
    fn parse_bare_command_matches_hand_built_shape() {
        // `ls -la /tmp` -> a Command of three spanned words, each a single Literal part.
        let node = parse("ls -la /tmp").expect_complete("parses");
        assert_eq!(
            node.span,
            Span::new(0, 11),
            "node span covers the whole line"
        );

        match node.node {
            // A wrapper would mean the case under test grew a trailing `&`. Panic rather than
            // unwrap: these assert a bare Command, and silently looking through a Background
            // would let the test pass while proving something else.
            AstNode::Background(_) => panic!("expected a Command, got a backgrounded node"),
            AstNode::Sequence(s) => {
                panic!("expected a Command, got a sequence of {}", s.rest.len() + 1)
            }
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
        let node = parse("pwd").expect_complete("parses");
        match node.node {
            // A wrapper would mean the case under test grew a trailing `&`. Panic rather than
            // unwrap: these assert a bare Command, and silently looking through a Background
            // would let the test pass while proving something else.
            AstNode::Background(_) => panic!("expected a Command, got a backgrounded node"),
            AstNode::Sequence(s) => {
                panic!("expected a Command, got a sequence of {}", s.rest.len() + 1)
            }
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
            let ast = parse(source).expect_complete("parses");
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

    /// CONTRACT: every operator the lexer can emit is ACCEPTED by the parser.
    ///
    /// ⚠️ THIS IS THE INVERSE OF THE TEST IT REPLACES, and the inversion is the point. The old one
    /// asserted that the parser DECLINED constructs it did not implement -- true when operators
    /// were new, and progressively less true as each was owned. Its probe moved from a pipe to
    /// `&&` to `&` as the boundary receded, and with background parsed there was nowhere left to
    /// move it: the parse-time refusal boundary no longer exists. Refusal now lives entirely at
    /// LOWERING, which is a different phase with its own tests.
    ///
    /// ★ AND THE INVERSE IS THE STRONGER GUARD: this fails the moment a regression RE-REFUSES an
    /// operator, which the old shape could never detect.
    #[test]
    fn the_parser_accepts_every_operator_the_lexer_emits() {
        for source in [
            "echo hi | grep h",
            "ls > out.txt",
            "ls >> out.txt",
            "wc -l < in.txt",
            "ls 2>&1",
            "a && b",
            "a || b",
            "a ; b",
            "sleep 5 &",
        ] {
            assert!(
                parse(source).is_complete(),
                "{source} must parse -- the parser owns every operator now"
            );
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
            let node = parse(source).expect_complete(&format!("{source} should parse"));
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
        let node = parse("cat log 2> /dev/null").expect_complete("parses");
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
        let node = parse("echo 2 > f").expect_complete("spaced numeral parses");
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
        let node = parse("ls /nope 2>&1").expect_complete("parses");
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

    /// CONTRACT: a refusal points at the exact source that caused it.
    ///
    /// ⚠️ THE PHASE MOVED, NOT THE CONTRACT. This used to check a PARSE refusal, but the parser
    /// now accepts every operator the lexer emits, so the rejection it should point at is a
    /// LOWERING one. The probe is a forest value pipeline because that is refused BY DESIGN --
    /// `sort` and `first` are query verbs with no programs behind them -- rather than as a
    /// staging point toward some later increment. A probe chosen from what is merely unfinished
    /// becomes a moving target: this one has already been re-pointed twice.
    #[test]
    fn a_rejection_identifies_the_construct_that_caused_it() {
        let node = parse("ps | sort cpu desc").expect_complete("a forest pipeline PARSES");
        let ctx = crate::spine::plan::LowerContext::default();
        match crate::spine::plan::lower_pipeline(&node, &ctx) {
            Err(crate::spine::plan::LowerError::UnsupportedConstruct { kind, span }) => {
                assert!(kind.contains("forest"), "named the construct: {kind}");
                assert!(span.start < span.end, "span points somewhere real");
            }
            other => panic!("expected a lowering refusal, got {other:?}"),
        }
    }

    /// ⚠️ THE INVERSE, and the one that keeps the boundary honest: a QUOTED operator is an argument,
    /// so these must still parse. A refusal that fired here would hand perfectly good commands to
    /// the legacy path forever and nobody would notice.
    #[test]
    fn a_quoted_operator_is_still_ours() {
        for source in ["echo \"|\"", "echo \'&&\'", "echo \">\""] {
            assert!(parse(source).is_complete(), "{source} should parse");
        }
    }

    /// INT-169 G2: EMPTY IS NOT AN EXECUTION RESULT, so it is not a ParseResult arm.
    ///
    /// This asserted `parse("") == Err(ParseError::NoCommand)`, which encoded the opposite: that an
    /// input containing no command is a parser outcome execution has to classify. It is not. Empty
    /// is neither malformed nor a refusal, and asking the router what execution ownership means for
    /// it has no useful answer, because there is no execution attempt.
    ///
    /// So the contract this now guards is the GUARD ITSELF: every reachable caller checks before
    /// parsing -- the REPL skips blank lines, `spine parse` and `spine exec` both test trim().
    /// `parse()` carries a debug_assert saying so, which is why this case must not call it with an
    /// empty string: it would trip the assertion it exists to document.
    #[test]
    fn parse_result_classification_matrix() {
        // ⭐ INT-169 G2 / INT-196: THE CLASSIFICATION IS THE CONTRACT. The router reads these arms to
        // decide ownership, so asserting WHICH arm each construct lands in is asserting the routing
        // decision at its source rather than re-deriving it downstream.
        //
        // ⚠️ THIS EXISTS BECAUSE THE DISTINCTION WAS BROKEN ONCE AND 139 GREEN TESTS COULD NOT SEE
        // IT. Widening the type left Refused falling through to the defect arm, so `2>` -- an
        // everyday construct -- would have reported a spine error instead of declining to legacy.
        // A manual probe caught it. This is the permanent version of that probe.
        assert!(
            matches!(parse("echo hi"), ParseResult::Complete(_)),
            "an ordinary command must be Complete -- the spine owns it"
        );
        assert!(
            matches!(parse("echo \"unclosed"), ParseResult::Incomplete(_)),
            "an unterminated quote must be Incomplete -- more input may complete it, and calling it \
             a failure is what made the validator invent its own continuation rules"
        );
        // MEASURED, AND IT CORRECTED A CLAIM I HAD BEEN REPEATING ALL SESSION: `cat log 2>
        // /dev/null` parses COMPLETE, with fd: Some(2) present in the AST. An fd redirect is NOT a
        // parser refusal -- the parser handles it, and the decision to leave it to legacy is made at
        // LOWERING. The FdRedirect refusal covers a narrower case than the everyday `2>`.
        assert!(
            matches!(parse("cat log 2> /dev/null"), ParseResult::Complete(_)),
            "an fd redirect PARSES: the routing decision for it is made at lowering, not here"
        );
        assert!(
            matches!(
                parse("ps | where cpu > 0.5"),
                ParseResult::Refused(Refusal::ComparisonNotRedirect { .. })
            ),
            "`> 0.5` must be REFUSED as a comparison rather than read as a redirect -- bash would \
             create a file named 0.5, and this deliberate divergence keeps the query language usable"
        );
        assert!(
            matches!(
                parse("echo a >"),
                ParseResult::Invalid(ParseError::MissingRedirectTarget { .. })
            ),
            "a redirect with no target must be INVALID: the parser located the mistake, so legacy \
             must not be asked to rediscover it by scanning the text"
        );
    }

    #[test]
    fn bare_operator_is_invalid_not_refused() {
        // ⭐ THE REGRESSION TEST FOR THE DISTINCTION, and proptest found the input. A refusal means
        // valid shell the spine declines to own and routes to legacy; `&` is not valid shell, so it
        // must surface instead. Asserting the CLASSIFICATION, not merely that parsing fails.
        assert!(matches!(
            parse("&"),
            ParseResult::Invalid(ParseError::NoCommand)
        ));
        assert!(matches!(
            parse(""),
            ParseResult::Invalid(ParseError::NoCommand)
        ));
    }

    #[test]
    fn parse_preserves_case() {
        // RFC section 9 rides-with: the spine does NOT lowercase. GitHub stays GitHub.
        let node = parse("GitHub Clone").expect_complete("parses");
        match node.node {
            // A wrapper would mean the case under test grew a trailing `&`. Panic rather than
            // unwrap: these assert a bare Command, and silently looking through a Background
            // would let the test pass while proving something else.
            AstNode::Background(_) => panic!("expected a Command, got a backgrounded node"),
            AstNode::Sequence(s) => {
                panic!("expected a Command, got a sequence of {}", s.rest.len() + 1)
            }
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
