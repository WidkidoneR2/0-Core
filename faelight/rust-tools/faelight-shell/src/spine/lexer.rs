//! INT-169 spine: the lexer. See docs/rfc-169-parser-spine.md section 5.
//!
//! ROADMAP STEP 2 (quoted literals) lands the RFC's HYBRID: a custom STATEFUL SCANNER walks
//! the source and finds shell-word boundaries, because quoting is context-sensitive and a
//! regex alternation cannot express it. `foo"bar baz"` is ONE word: a non-whitespace-run
//! regex grabs `foo"bar` and is already wrong. logos remains the right tool for the
//! structurally-regular tokens (operators at steps 5-7) and stays a dependency for them; the
//! word path is now scanned by hand. Either way the parser consumes only OUR SpannedToken.
//!
//! ★ SEGMENTS (Christian): the scanner PRESERVES the quote context of each region of a word.
//! "The lexer should preserve information that is cheap to obtain and expensive to
//! reconstruct." The scanner must already know the context to find the boundary correctly --
//! collapsing `foo"$HOME"'bar'` into one flat string destroys the only place the quoting
//! semantics existed, forcing a SECOND scanner at step 3. A WordSegment is "a region with
//! uniform lexical rules".
//!
//! ★ THE LEXER DOES NOT INTERPRET (Christian): it never emits Variable/Literal segments -- it
//! does not know what `$HOME` means. It knows only "this text occurred inside double quotes."
//! The parser maps segments to WordParts (today: every segment becomes Literal). At step 3
//! that mapping gains variable handling with NO lexer change: unquoted and double-quoted
//! regions may contain variables, single-quoted regions are literal only.
//!
//! NOT in step 2: escapes (\" and \ ) -- a separable sub-language, its own step. Note that
//! segments carry OWNED text as well as a span precisely because escapes will break the
//! "segment text is a contiguous source slice" invariant when they land.

use super::ast::Span;

/// Re-exported: `QuoteContext` lives in `ast` because the AST RECORDS it as a fact about how a
/// word was written, and each expansion phase derives its own rule from that fact. The lexer
/// still discovers it. This keeps every existing `lexer::QuoteContext` import valid.
pub use super::ast::QuoteContext;

/// A contiguous region of a word with uniform lexical rules. `foo"bar"` is one word of two
/// segments. Text is the CONTENT (enclosing quote characters consumed); span covers that
/// content in the source.
///
/// INT-169 blocker 4, step 1b: a word can also contain a NESTED SYNTACTIC REGION. `$(...)` is not
/// text with uniform lexical rules -- its contents are a shell program -- so it is a VARIANT rather
/// than a `Text` segment wearing a special context. The scanner still does not interpret it: it
/// records that this range was one region and hands the inner source on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WordSegment {
    /// Ordinary text. `text` is the CONTENT (enclosing quote characters consumed); `span` covers
    /// that content in the source.
    Text {
        text: String,
        span: Span,
        context: QuoteContext,
    },
    /// A command substitution region. `source` is the INNER text WITHOUT the `$(` and `)`
    /// delimiters, because the parser should not re-strip syntax the scanner already recognised.
    /// `span` covers the inner source, not the delimiters.
    CommandSub { source: String, span: Span },
}

/// A shell operator, recorded as WHAT WAS SEEN rather than as a judgement about it.
///
/// ★ Deliberately NOT `Unsupported(char)`. The lexer has no authority over ownership: it reports
/// that a pipe occurred, and the PARSER decides whether a pipe is part of its grammar. Same
/// separation as `QuoteContext` -- facts in the scanner, policy above it. It also means no consumer
/// ever has to know that `>>` is two characters; that belongs here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorKind {
    Pipe,
    Or,
    And,
    /// A lone `&`. ⚠️ INCLUDED DELIBERATELY, though it stretches the "smallest useful set" rule:
    /// omitting it is not neutral. `sleep 5 &` would then lex as three WORDS, parse cleanly, and
    /// execute `sleep` with `&` as a literal argument. An incomplete refusal is a mis-execution.
    Background,
    /// `>&` -- DESCRIPTOR DUPLICATION, and a token in its own right. INT-200: without this the
    /// scanner reads `2>&1` as RedirectOut followed by Background, which is not a redirect
    /// followed by a background job -- it is one operator meaning "send this stream where that
    /// one goes". The fd guard hides that today by refusing first; the moment the guard becomes
    /// a binding, the parser would look for a target word and find an operator.
    ///
    /// ⚠️ Two-character match, so a lone `&` is untouched and `sleep 5 &` still lexes as
    /// Background exactly as before.
    RedirectDup,
    RedirectOut,
    RedirectAppend,
    RedirectIn,
    Sequence,
}

/// Raw token kinds. `Word` and now `Operator`; `$`-constructs arrive at their roadmap steps.
///
/// ⚠️ Operators exist so the spine can REFUSE, not so it can execute them. Nothing here implies a
/// pipeline or redirect will ever be run by this path -- see the parser's UnsupportedConstruct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Word,
    Operator(OperatorKind),
}

/// The operator starting at `i`, and how many CHARS it spans. `None` for anything else.
///
/// ⚠️ Called ONLY in unquoted context. `echo "|"` is data, and the caller's state machine is the
/// only thing that knows the difference -- which is precisely why a stateless pattern matcher
/// cannot own this and logos stays deferred.
fn operator_at(chars: &[(usize, char)], i: usize) -> Option<(OperatorKind, usize)> {
    let c = chars.get(i)?.1;
    let next = chars.get(i + 1).map(|x| x.1);
    Some(match (c, next) {
        ('>', Some('>')) => (OperatorKind::RedirectAppend, 2),
        ('>', Some('&')) => (OperatorKind::RedirectDup, 2),
        ('&', Some('&')) => (OperatorKind::And, 2),
        ('|', Some('|')) => (OperatorKind::Or, 2),
        ('|', _) => (OperatorKind::Pipe, 1),
        ('&', _) => (OperatorKind::Background, 1),
        ('>', _) => (OperatorKind::RedirectOut, 1),
        ('<', _) => (OperatorKind::RedirectIn, 1),
        (';', _) => (OperatorKind::Sequence, 1),
        _ => return None,
    })
}

/// A token with its owned content, span, and lexical segments -- what the parser consumes.
///
/// `text` is the concatenated content of all segments (quotes consumed). `span` covers the
/// word's full source extent INCLUDING its quote characters. `segments` preserves the
/// per-region quote context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedToken {
    pub kind: TokenKind,
    pub text: String,
    pub span: Span,
    pub segments: Vec<WordSegment>,
}

/// INT-169 G1: WHY a line is not finished. NOT an error -- see LexResult below.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexIncompleteKind {
    /// A quote opened and the line ended before it closed.
    UnterminatedQuote,
    /// A `$(` opened and its closing paren never arrived.
    UnterminatedCommandSub,
    /// The line ended in a single backslash -- an explicit request to continue. Previously the
    /// validator owned this, because a line ending in one is lexically FINE and the lexer had no
    /// way to say "finished scanning, but not finished".
    TrailingEscape,
    /// A heredoc was introduced and its body has not arrived. Carries the delimiter so a caller
    /// knows what ends it, and whether it was quoted, which decides whether the body expands.
    ///
    /// ⚠️ AWARENESS, NOT EXECUTION. The scanner knowing it is inside a heredoc continuation does
    /// not claim fsh can execute one -- that capability is explicitly out of 169's scope.
    HeredocBody { delimiter: String, quoted: bool },
}

/// A line that scanned correctly and simply has not finished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexIncomplete {
    pub kind: LexIncompleteKind,
    pub text: String,
    pub span: Span,
}

/// ⭐ INT-169 G1: THE SCANNER IS THE SOLE OWNER OF LEXICAL CONTINUATION STATE.
///
/// `lex` used to return `Result<Vec<SpannedToken>, LexError>`, and both of the only two error kinds
/// it could produce -- an unterminated quote and an unterminated `$(` -- were INCOMPLETENESS rather
/// than invalidity. The name and the Result shape were the lie; the behaviour was already right.
///
/// That lie cost real things. completion.rs had to translate the "error" back into Incomplete to
/// hold the prompt open, and then invent THREE more continuation rules the lexer could not express:
/// a trailing backslash, a heredoc introduction (`input.contains("<<")`), and comment stripping --
/// each added after an apostrophe in ordinary English prose hung the prompt. And migrate_audit
/// counts unterminated quotes as spine PARSE ERRORS against the shell, which they are not.
///
/// ⚠️ TWO ARMS, NOT THREE, DELIBERATELY. There is no genuinely-invalid lexical condition in this
/// language today, and inventing an `Invalid` variant to fill a third slot would be vocabulary with
/// no emitter. The three-way distinction arrives at the PARSER, where invalid syntax genuinely
/// exists -- G2's ParseResult.
///
/// THE INVARIANT: callers consume an explicit `Incomplete` state; they never infer one from an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexResult {
    Complete(Vec<SpannedToken>),
    Incomplete(LexIncomplete),
}

/// Scan a source line into spanned word tokens. Whitespace separates words unless quoted;
/// quote characters are consumed and recorded as segment context.
pub fn lex(source: &str) -> LexResult {
    // INT-169 G1: A TRAILING BACKSLASH IS AN EXPLICIT REQUEST TO CONTINUE, and it is checked BEFORE
    // the scan because it is a property of the line's END rather than of any word in it. The
    // validator owned this rule and its comment said why the lexer could not: "the one case the
    // lexer cannot answer, because a line ending in one is lexically fine". That was true while the
    // only outcome was Ok-or-error -- a line ending in a backslash scans perfectly. It has somewhere
    // to go now.
    //
    // ⚠️ A DOUBLED BACKSLASH IS A LITERAL AND ENDS THE LINE NORMALLY, which is why this counts the
    // run rather than testing the last character. `echo a\\` is finished; `echo a\` is not.
    let trailing_backslashes = source.chars().rev().take_while(|c| *c == '\\').count();
    if trailing_backslashes % 2 == 1 {
        let at = source.len() - 1;
        return LexResult::Incomplete(LexIncomplete {
            kind: LexIncompleteKind::TrailingEscape,
            text: "\\".to_string(),
            span: Span {
                start: at,
                end: source.len(),
            },
        });
    }

    // INT-169 G1: A HEREDOC INTRODUCTION WITHOUT ITS BODY IS INCOMPLETE, and it is answered BEFORE
    // the scan for a blunt reason: a heredoc body is not shell. Scanning it as shell is what hung the
    // prompt when a pasted block contained an English possessive, a Rust lifetime or a contraction --
    // the scanner reported an unterminated quote and the REPL waited for a closing quote that was
    // never coming. The validator worked around it with `input.contains("<<")`, which its own comment
    // called crude and correct-for-now.
    //
    // ⚠️ THIS DOES NOT CLAIM fsh CAN EXECUTE A HEREDOC. The scanner knows it is inside a continuation
    // and says so; execution stays with sh until an intent takes it. Awareness, not capability.
    //
    // The delimiter and its quoting come from the ONE function that already knows how to find them,
    // rather than a second reader of the same syntax.
    if let Some((delim, quoted)) = crate::expand::find_heredoc_intro(source) {
        let body_closed = source.lines().skip(1).any(|l| l.trim() == delim.as_str());
        if !body_closed {
            let at = source.find("<<").unwrap_or(0);
            return LexResult::Incomplete(LexIncomplete {
                kind: LexIncompleteKind::HeredocBody {
                    delimiter: delim,
                    quoted,
                },
                text: source[at..].to_string(),
                span: Span {
                    start: at,
                    end: source.len(),
                },
            });
        }
    }

    let chars: Vec<(usize, char)> = source.char_indices().collect();
    let n = chars.len();
    let mut out: Vec<SpannedToken> = Vec::new();
    let mut idx = 0usize;

    while idx < n {
        while idx < n && chars[idx].1.is_whitespace() {
            idx += 1;
        }
        if idx >= n {
            break;
        }

        // INT-209: A COMMENT IS A LEXICAL STATE, and this is where it belongs -- the outer loop has
        // just skipped whitespace, so standing here IS being at the start of a word. The word-start
        // rule comes by construction rather than by testing the preceding character.
        //
        // Unquoted by construction too: a `#` inside quotes is consumed by the inner walk as part
        // of a word and never reaches this point, for the same reason operator_at is only called in
        // unquoted context. `echo "# x"` is data.
        //
        // NOT mid-word: `foo#bar` is one word, and this check cannot see it because the outer loop
        // only stands here BETWEEN words.
        //
        // The rest of the line is a comment, so scanning STOPS -- a comment does not end a word, it
        // ends the input. Hence breaking the OUTER loop.
        if chars[idx].1 == '#' {
            break;
        }

        let word_start = chars[idx].0;
        let mut segments: Vec<WordSegment> = Vec::new();
        let mut text = String::new();
        let mut seg_text = String::new();
        let mut seg_start = word_start;
        let mut context = QuoteContext::Unquoted;
        let mut open_quote_at: Option<usize> = None;
        let mut word_end = word_start;

        while idx < n {
            let (pos, ch) = chars[idx];
            let after = pos + ch.len_utf8();

            if context == QuoteContext::Unquoted && ch.is_whitespace() {
                break;
            }

            // INT-169: an operator is a BOUNDARY INTERRUPT, handled exactly like whitespace above --
            // break, let the existing word emission run, then consume the operator after it. That
            // deliberately avoids adding a second way for a word to end; there is one such path and
            // this gives it one more reason to fire.
            //
            // ⚠️ Unquoted only. `echo "|"` and `echo '|'` are data, and `$(...)` never reaches here
            // because its whole region is consumed above.
            if context == QuoteContext::Unquoted && operator_at(&chars, idx).is_some() {
                break;
            }

            // INT-169 blocker 4, step 1a: `$(` opens a NESTED SYNTACTIC REGION. Whitespace
            // inside it must not terminate the word -- the same reason quotes are handled here.
            // This is BOUNDARY DETECTION, not interpretation: the scanner never learns what the
            // substitution means, only that this range of characters is one region. Step 1b will
            // preserve it structurally; today it is consumed as text so the word is correct.
            //
            // Depth-counted, because `$(echo "$(git branch)")` nests. Quote-aware INSIDE the
            // region, because a `)` within quotes does not close it.
            if ch == '$'
                && context != QuoteContext::Single
                && idx + 1 < n
                && chars[idx + 1].1 == '('
            {
                let mut depth = 1usize;
                let mut j = idx + 2;
                let mut inner_quote: Option<char> = None;
                while j < n {
                    let c2 = chars[j].1;
                    match inner_quote {
                        Some(q) if c2 == q => inner_quote = None,
                        Some(_) => {}
                        None if c2 == '"' || c2 == '\'' => inner_quote = Some(c2),
                        None if c2 == '(' => depth += 1,
                        None if c2 == ')' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        None => {}
                    }
                    j += 1;
                }
                if depth != 0 || j >= n {
                    // Unterminated, spanned from the opener -- same shape as an unclosed quote.
                    return LexResult::Incomplete(LexIncomplete {
                        kind: LexIncompleteKind::UnterminatedCommandSub,
                        text: source[pos..].to_string(),
                        span: Span::new(pos, source.len()),
                    });
                }
                let close = chars[j].0;
                let after_close = close + chars[j].1.len_utf8();
                // INT-169 blocker 4, step 1b-i: the region becomes its OWN segment. Pending text
                // is flushed first so segment order matches source order. The token's flat `text`
                // still receives the RAW form including delimiters, so word-level behaviour is
                // unchanged and every step-1a test still holds.
                if !seg_text.is_empty() {
                    segments.push(WordSegment::Text {
                        text: std::mem::take(&mut seg_text),
                        span: Span::new(seg_start, pos),
                        context,
                    });
                }
                segments.push(WordSegment::CommandSub {
                    source: (idx + 2..j).map(|k| chars[k].1).collect(),
                    span: Span::new(chars[idx + 2].0, close),
                });
                for k in idx..=j {
                    text.push(chars[k].1);
                }
                // ⚠️ seg_start MUST advance: the next Text segment begins after the region, and
                // leaving it behind would give that segment a span reaching back over this one.
                seg_start = after_close;
                word_end = after_close;
                idx = j + 1;
                continue;
            }

            if ch == '"' || ch == '\'' {
                let this = if ch == '"' {
                    QuoteContext::Double
                } else {
                    QuoteContext::Single
                };
                if context == QuoteContext::Unquoted {
                    // Opening delimiter: close any pending unquoted run first.
                    if !seg_text.is_empty() {
                        segments.push(WordSegment::Text {
                            text: std::mem::take(&mut seg_text),
                            span: Span::new(seg_start, pos),
                            context: QuoteContext::Unquoted,
                        });
                    }
                    context = this;
                    open_quote_at = Some(pos);
                    seg_start = after;
                    word_end = after;
                    idx += 1;
                    continue;
                }
                if context == this {
                    // Closing delimiter. An EMPTY quoted segment is kept -- `echo ""` is a
                    // real empty argument, not nothing.
                    segments.push(WordSegment::Text {
                        text: std::mem::take(&mut seg_text),
                        span: Span::new(seg_start, pos),
                        context,
                    });
                    context = QuoteContext::Unquoted;
                    open_quote_at = None;
                    seg_start = after;
                    word_end = after;
                    idx += 1;
                    continue;
                }
                // The other delimiter inside a quoted region is ordinary content.
            }

            seg_text.push(ch);
            text.push(ch);
            word_end = after;
            idx += 1;
        }

        if context != QuoteContext::Unquoted {
            let at = open_quote_at.unwrap_or(word_start);
            return LexResult::Incomplete(LexIncomplete {
                kind: LexIncompleteKind::UnterminatedQuote,
                text: source[at..].to_string(),
                span: Span::new(at, source.len()),
            });
        }

        if !seg_text.is_empty() {
            segments.push(WordSegment::Text {
                text: std::mem::take(&mut seg_text),
                span: Span::new(seg_start, word_end),
                context: QuoteContext::Unquoted,
            });
        }

        // ⚠️ GUARDED ON SEGMENTS, NOT TEXT. A line beginning with an operator, or the boundary in
        // `echo hi|grep`, would otherwise emit an empty Word. It cannot be a text check: `echo ""`
        // is a real empty argument and legitimately produces an empty token WITH a segment.
        if !segments.is_empty() {
            out.push(SpannedToken {
                kind: TokenKind::Word,
                text,
                span: Span::new(word_start, word_end),
                segments,
            });
        }

        // The word ended because an operator was found; emit it and resume.
        if let Some((op, len)) = operator_at(&chars, idx) {
            let start = chars[idx].0;
            let last = chars[idx + len - 1];
            let end = last.0 + last.1.len_utf8();
            out.push(SpannedToken {
                kind: TokenKind::Operator(op),
                text: source[start..end].to_string(),
                span: Span::new(start, end),
                segments: Vec::new(),
            });
            idx += len;
        }
    }

    LexResult::Complete(out)
}

/// Test-only conveniences. INT-169 G1: the tests are part of this module's contract and evolve with
/// the model, so they assert against LexResult rather than being handed `expect`-shaped accessors in
/// production. Named for what they assert, so a case that gets the OTHER arm fails loudly.
#[cfg(test)]
impl LexResult {
    fn expect_complete(self, msg: &str) -> Vec<SpannedToken> {
        match self {
            LexResult::Complete(t) => t,
            LexResult::Incomplete(i) => panic!("{msg}: got Incomplete({:?})", i.kind),
        }
    }
    fn expect_incomplete(self, msg: &str) -> LexIncomplete {
        match self {
            LexResult::Incomplete(i) => i,
            LexResult::Complete(t) => panic!("{msg}: got Complete with {} tokens", t.len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment_is_a_lexical_state() {
        // INT-209: the four cases that DEFINE the rule, so it cannot drift into something looser.
        let toks = lex("echo hi # tail").expect_complete("lexes");
        let words: Vec<&str> = toks.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(words, vec!["echo", "hi"], "a comment ends the input");

        let toks = lex("echo \"# x\"").expect_complete("lexes");
        assert_eq!(
            toks.len(),
            2,
            "a # inside quotes is DATA -- the inner walk consumes it and it never reaches the check"
        );

        let toks = lex("echo foo#bar").expect_complete("lexes");
        let words: Vec<&str> = toks.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(
            words,
            vec!["echo", "foo#bar"],
            "mid-word is NOT a comment: the outer loop only stands between words, so it cannot see \
             this one -- which is the rule holding by construction rather than by a guard"
        );

        let toks = lex("# whole line").expect_complete("lexes");
        assert!(toks.is_empty(), "a comment-only line lexes to nothing");
    }

    // INT-169 blocker 4, step 1b-i: test-only accessors. NOT production API -- adding real
    // accessors would let `WordSegment` keep pretending to be a struct, which is the shape the
    // enum exists to end. These PANIC naming the actual variant, so a test that silently began
    // receiving a CommandSub fails loudly instead of quietly asserting nothing.
    fn seg_text(seg: &WordSegment) -> &str {
        match seg {
            WordSegment::Text { text, .. } => text,
            other => panic!("expected a Text segment, got {other:?}"),
        }
    }

    fn seg_context(seg: &WordSegment) -> QuoteContext {
        match seg {
            WordSegment::Text { context, .. } => *context,
            other => panic!("expected a Text segment, got {other:?}"),
        }
    }

    fn seg_span(seg: &WordSegment) -> Span {
        match seg {
            WordSegment::Text { span, .. } => *span,
            other => panic!("expected a Text segment, got {other:?}"),
        }
    }

    /// INT-169: OPERATORS ARE TOKENS. The spine does not execute a pipe -- it must be able to SEE
    /// one in order to decline the command, and until this existed `|` was an ordinary character
    /// inside a word, so `echo hi | grep h` parsed happily into five words and the router would
    /// have claimed a command it cannot run.
    #[test]
    fn operators_become_their_own_tokens() {
        let toks = lex("echo hi | grep h").expect_complete("lexes");
        assert_eq!(toks.len(), 5);
        assert_eq!(toks[2].kind, TokenKind::Operator(OperatorKind::Pipe));
        assert_eq!(toks[2].text, "|");
        assert_eq!(toks[3].text, "grep");
    }

    /// ADJACENCY, which is where a whitespace-based split would quietly fail. Shell users do not
    /// add spaces around pipes, and `hi|grep` must be three tokens rather than one word.
    #[test]
    fn an_operator_splits_a_word_without_whitespace() {
        let toks = lex("echo hi|grep h").expect_complete("lexes");
        assert_eq!(
            toks.iter().map(|t| t.text.clone()).collect::<Vec<_>>(),
            vec!["echo", "hi", "|", "grep", "h"]
        );
        assert_eq!(toks[2].kind, TokenKind::Operator(OperatorKind::Pipe));
    }

    /// ★ QUOTES PROTECT OPERATORS, which is the whole reason recognition lives inside the scanner's
    /// state machine rather than in a stateless matcher: only the scanner knows the current region.
    #[test]
    fn a_quoted_operator_is_data() {
        for source in ["echo \"|\"", "echo \'|\'", "echo \">>\"", "echo \'&&\'"] {
            let toks = lex(source).expect_complete("lexes");
            assert_eq!(toks.len(), 2, "{source} must stay two words");
            assert!(
                toks.iter().all(|t| t.kind == TokenKind::Word),
                "{source} produced an operator token"
            );
        }
    }

    /// LONGEST MATCH. `>>` is one operator, not two, and the parser must never learn that.
    #[test]
    fn multi_character_operators_win_over_their_prefixes() {
        for (source, want) in [
            ("echo a >> b", OperatorKind::RedirectAppend),
            ("echo a > b", OperatorKind::RedirectOut),
            ("a && b", OperatorKind::And),
            ("a || b", OperatorKind::Or),
            ("a | b", OperatorKind::Pipe),
        ] {
            let toks = lex(source).expect_complete("lexes");
            let ops: Vec<_> = toks.iter().filter(|t| t.kind != TokenKind::Word).collect();
            assert_eq!(ops.len(), 1, "{source}");
            assert_eq!(ops[0].kind, TokenKind::Operator(want), "{source}");
        }
    }

    /// ⚠️ THE CASE THAT WOULD HAVE BEEN A FALSE OWNERSHIP CLAIM. A lone `&` was nearly left out as
    /// "not the smallest useful set"; omitting it means `sleep 5 &` lexes as three words, parses
    /// cleanly, and runs `sleep` with `&` as a literal argument. An incomplete refusal executes.
    #[test]
    fn a_lone_ampersand_is_an_operator() {
        let toks = lex("sleep 5 &").expect_complete("lexes");
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[2].kind, TokenKind::Operator(OperatorKind::Background));
    }

    /// ★ `>&` IS ONE OPERATOR, and the reason this test sits beside the lone-ampersand case: they
    /// are the two halves of the same distinction. Without the two-character arm, `2>&1` scans as
    /// RedirectOut followed by Background -- which reads as "redirect, then background the job" and
    /// is not what anyone wrote. It is descriptor duplication: send this stream where that one goes.
    ///
    /// ⚠️ The fd guard in the parser hides this today by refusing first. The moment that guard
    /// becomes a BINDING, the parser consumes `>`, looks for a target word, and finds an operator --
    /// so this must be a token before fd redirects can be built at all.
    #[test]
    fn a_dup_redirect_is_one_operator_not_two() {
        let toks = lex("ls 2>&1").expect_complete("lexes");
        let ops: Vec<_> = toks.iter().filter(|t| t.kind != TokenKind::Word).collect();
        assert_eq!(
            ops.len(),
            1,
            "2>&1 is ONE operator, not RedirectOut plus Background"
        );
        assert_eq!(ops[0].kind, TokenKind::Operator(OperatorKind::RedirectDup));
        // The numeral and the destination stay separate WORDS -- binding them is the parser's job,
        // and the lexer deliberately does not know that `2` names a stream.
        assert_eq!(toks.len(), 4, "ls, 2, >&, 1");
    }

    /// The inverse, and the one that proves the two-character match did not swallow the lone form:
    /// a `&` NOT preceded by `>` is still Background, so `sleep 5 &` is unchanged.
    #[test]
    fn a_dup_arm_does_not_swallow_a_lone_ampersand() {
        let toks = lex("ls > f & sleep 5 &").expect_complete("lexes");
        let ops: Vec<_> = toks
            .iter()
            .filter_map(|t| match t.kind {
                TokenKind::Operator(k) => Some(k),
                TokenKind::Word => None,
            })
            .collect();
        assert_eq!(
            ops,
            vec![
                OperatorKind::RedirectOut,
                OperatorKind::Background,
                OperatorKind::Background
            ],
            "a space between > and & keeps them separate operators"
        );
    }

    /// An operator inside a command substitution belongs to the NESTED program, and the region is
    /// consumed whole before the operator check ever sees those characters.
    #[test]
    fn an_operator_inside_a_substitution_is_not_ours() {
        let toks = lex("echo $(echo a | b)").expect_complete("lexes");
        assert_eq!(
            toks.len(),
            2,
            "{:?}",
            toks.iter().map(|t| &t.text).collect::<Vec<_>>()
        );
        assert!(toks.iter().all(|t| t.kind == TokenKind::Word));
    }

    /// A leading operator must not emit an empty Word first -- the reason the word emission is
    /// guarded on SEGMENTS rather than on text.
    #[test]
    fn a_leading_operator_emits_no_empty_word() {
        let toks = lex("| grep h").expect_complete("lexes");
        assert_eq!(toks[0].kind, TokenKind::Operator(OperatorKind::Pipe));
        assert_eq!(toks.len(), 3);
    }

    #[test]
    fn lex_bare_command() {
        let toks = lex("ls -la /tmp").expect_complete("bare command lexes");
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[0].kind, TokenKind::Word);
        assert_eq!(toks[0].text, "ls");
        assert_eq!(toks[0].span, Span::new(0, 2));
        assert_eq!(toks[1].text, "-la");
        assert_eq!(toks[1].span, Span::new(3, 6));
        assert_eq!(toks[2].text, "/tmp");
        assert_eq!(toks[2].span, Span::new(7, 11));
        assert_eq!(toks[0].segments.len(), 1);
        assert_eq!(seg_context(&toks[0].segments[0]), QuoteContext::Unquoted);
    }

    #[test]
    fn lex_collapses_extra_whitespace() {
        let toks = lex("  ls    -la  ").expect_complete("lexes");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].text, "ls");
        assert_eq!(toks[0].span, Span::new(2, 4));
        assert_eq!(toks[1].text, "-la");
    }

    #[test]
    fn lex_single_word() {
        let toks = lex("pwd").expect_complete("lexes");
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].text, "pwd");
        assert_eq!(toks[0].span, Span::new(0, 3));
    }

    #[test]
    fn lex_empty_is_empty() {
        assert!(lex("").expect_complete("empty lexes").is_empty());
        assert!(lex("   ")
            .expect_complete("whitespace-only lexes")
            .is_empty());
    }

    #[test]
    fn quoted_span_is_one_word() {
        // THE step-2 case: a quoted region keeps its spaces and becomes ONE word.
        let toks = lex("git commit -m \"message here\"").expect_complete("lexes");
        assert_eq!(toks.len(), 4, "the quoted value is one word, not two");
        assert_eq!(
            toks[3].text, "message here",
            "quotes consumed, content kept"
        );
        assert_eq!(toks[3].span, Span::new(14, 28), "span covers the quotes");
        assert_eq!(toks[3].segments.len(), 1);
        assert_eq!(seg_context(&toks[3].segments[0]), QuoteContext::Double);
        assert_eq!(
            seg_span(&toks[3].segments[0]),
            Span::new(15, 27),
            "content span"
        );
    }

    #[test]
    fn adjacent_segments_are_one_word_with_context_each() {
        // `foo"bar baz"qux` is ONE word of three segments -- the case a regex cannot express.
        let toks = lex("foo\"bar baz\"qux").expect_complete("lexes");
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].text, "foobar bazqux");
        assert_eq!(toks[0].segments.len(), 3);
        assert_eq!(seg_context(&toks[0].segments[0]), QuoteContext::Unquoted);
        assert_eq!(seg_text(&toks[0].segments[0]), "foo");
        assert_eq!(seg_context(&toks[0].segments[1]), QuoteContext::Double);
        assert_eq!(seg_text(&toks[0].segments[1]), "bar baz");
        assert_eq!(seg_context(&toks[0].segments[2]), QuoteContext::Unquoted);
        assert_eq!(seg_text(&toks[0].segments[2]), "qux");
    }

    #[test]
    fn single_quotes_are_their_own_context() {
        let toks = lex("echo 'a b'").expect_complete("lexes");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[1].text, "a b");
        assert_eq!(seg_context(&toks[1].segments[0]), QuoteContext::Single);
    }

    #[test]
    fn other_delimiter_inside_quotes_is_content() {
        // A ' inside "..." is ordinary text, and vice versa.
        let toks = lex("echo \"it's fine\"").expect_complete("lexes");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[1].text, "it's fine");
    }

    #[test]
    fn empty_quotes_are_a_real_empty_argument() {
        let toks = lex("echo \"\"").expect_complete("lexes");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[1].text, "");
        assert_eq!(toks[1].segments.len(), 1, "the empty quoted region is kept");
        assert_eq!(seg_context(&toks[1].segments[0]), QuoteContext::Double);
    }

    #[test]
    fn unterminated_quote_is_a_lex_error_spanned_from_the_opener() {
        let err = lex("echo \"unclosed").expect_incomplete("unterminated quote must error");
        assert_eq!(err.span.start, 5, "points at the opening quote");
        assert_eq!(err.span.end, 14);
    }

    /// INT-169 blocker 4, the FAILING CASE that justifies the scanner change.
    ///
    /// `$(...)` is a nested syntactic region: the whitespace inside it must NOT terminate the
    /// word, exactly as inside quotes. Recording this as a test rather than probing through the
    /// shell, because two shell probes gave two different confounds -- quotes reaching the lexer,
    /// then fsh expanding the substitution before the builtin ever saw it. `lex` is a pure
    /// function; ask it directly.
    ///
    /// ⚠️ EXPECTED TO FAIL TODAY. The scanner has no `$(` branch, so it splits on the inner
    /// spaces and produces four words instead of two.
    #[test]
    fn command_substitution_is_one_word() {
        let toks = lex("echo $(echo a b)").expect_complete("lexes");
        assert_eq!(
            toks.len(),
            2,
            "`$(...)` is a nested region: inner whitespace must not split the word, got {:?}",
            toks.iter().map(|t| t.text.clone()).collect::<Vec<_>>()
        );
        assert_eq!(toks[0].text, "echo");
        assert_eq!(toks[1].text, "$(echo a b)");
    }

    /// A `)` inside quotes does not close the region -- the scanner tracks quote state while
    /// counting depth, or `$(echo ")")` would terminate at the wrong paren.
    #[test]
    fn command_substitution_ignores_parens_inside_quotes() {
        let toks = lex(r#"echo $(echo ")")"#).expect_complete("lexes");
        assert_eq!(
            toks.len(),
            2,
            "{:?}",
            toks.iter().map(|t| &t.text).collect::<Vec<_>>()
        );
        assert_eq!(toks[1].text, r#"$(echo ")")"#);
    }

    /// Depth counting, not first-close-wins. `$(echo $(pwd))` is the forcing case for nesting.
    #[test]
    fn command_substitution_can_nest() {
        let toks = lex("echo $(echo $(pwd))").expect_complete("lexes");
        assert_eq!(
            toks.len(),
            2,
            "{:?}",
            toks.iter().map(|t| &t.text).collect::<Vec<_>>()
        );
        assert_eq!(toks[1].text, "$(echo $(pwd))");
    }

    /// Single quotes own the region: `$(` inside them is ordinary text, so the branch must be
    /// gated on quote context or the scanner would claim syntax the shell says is literal.
    #[test]
    fn single_quoted_command_substitution_is_literal() {
        let toks = lex("echo '$(echo a b)'").expect_complete("lexes");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[1].text, "$(echo a b)");
        assert_eq!(seg_context(&toks[1].segments[0]), QuoteContext::Single);
    }

    /// Unterminated `$(` errors from the OPENER, the same shape as an unclosed quote, rather
    /// than inventing a second convention for the same class of failure.
    #[test]
    fn unterminated_command_substitution_errors_from_the_opener() {
        let err = lex("echo $(echo a").expect_incomplete("unterminated substitution must error");
        assert_eq!(err.span.start, 5, "points at the $ that opened it");
    }

    #[test]
    fn unicode_content_survives() {
        let toks = lex("echo \"héllo 中\"").expect_complete("lexes");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[1].text, "héllo 中");
    }
}
