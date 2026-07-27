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

/// The lexical context a region of a word was written in. NOT an interpretation -- just
/// which delimiter (if any) enclosed the text. Expansion rules are applied later, by the
/// phase that owns them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteContext {
    /// Bare text, no enclosing quotes.
    Unquoted,
    /// Inside '...' -- fully literal once expansion exists.
    Single,
    /// Inside "..." -- allows expansion once expansion exists.
    Double,
}

/// A contiguous region of a word with uniform lexical rules. `foo"bar"` is one word of two
/// segments. Text is the CONTENT (enclosing quote characters consumed); span covers that
/// content in the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordSegment {
    pub text: String,
    pub span: Span,
    pub context: QuoteContext,
}

/// Raw token kinds. Only `Word` exists today; operators and $-constructs arrive at their
/// roadmap steps (and those, being structurally regular, are where logos returns).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Word,
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

/// A lex error: the unlexable slice and where it was. Step 2 produces the first real one --
/// an unterminated quote, spanned from its opening delimiter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub text: String,
    pub span: Span,
}

/// Scan a source line into spanned word tokens. Whitespace separates words unless quoted;
/// quote characters are consumed and recorded as segment context.
pub fn lex(source: &str) -> Result<Vec<SpannedToken>, LexError> {
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
                    return Err(LexError {
                        text: source[pos..].to_string(),
                        span: Span::new(pos, source.len()),
                    });
                }
                let close = chars[j].0;
                let after_close = close + chars[j].1.len_utf8();
                for k in idx..=j {
                    seg_text.push(chars[k].1);
                    text.push(chars[k].1);
                }
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
                        segments.push(WordSegment {
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
                    segments.push(WordSegment {
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
            return Err(LexError {
                text: source[at..].to_string(),
                span: Span::new(at, source.len()),
            });
        }

        if !seg_text.is_empty() {
            segments.push(WordSegment {
                text: std::mem::take(&mut seg_text),
                span: Span::new(seg_start, word_end),
                context: QuoteContext::Unquoted,
            });
        }

        out.push(SpannedToken {
            kind: TokenKind::Word,
            text,
            span: Span::new(word_start, word_end),
            segments,
        });
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_bare_command() {
        let toks = lex("ls -la /tmp").expect("bare command lexes");
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[0].kind, TokenKind::Word);
        assert_eq!(toks[0].text, "ls");
        assert_eq!(toks[0].span, Span::new(0, 2));
        assert_eq!(toks[1].text, "-la");
        assert_eq!(toks[1].span, Span::new(3, 6));
        assert_eq!(toks[2].text, "/tmp");
        assert_eq!(toks[2].span, Span::new(7, 11));
        assert_eq!(toks[0].segments.len(), 1);
        assert_eq!(toks[0].segments[0].context, QuoteContext::Unquoted);
    }

    #[test]
    fn lex_collapses_extra_whitespace() {
        let toks = lex("  ls    -la  ").expect("lexes");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].text, "ls");
        assert_eq!(toks[0].span, Span::new(2, 4));
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

    #[test]
    fn quoted_span_is_one_word() {
        // THE step-2 case: a quoted region keeps its spaces and becomes ONE word.
        let toks = lex("git commit -m \"message here\"").expect("lexes");
        assert_eq!(toks.len(), 4, "the quoted value is one word, not two");
        assert_eq!(
            toks[3].text, "message here",
            "quotes consumed, content kept"
        );
        assert_eq!(toks[3].span, Span::new(14, 28), "span covers the quotes");
        assert_eq!(toks[3].segments.len(), 1);
        assert_eq!(toks[3].segments[0].context, QuoteContext::Double);
        assert_eq!(toks[3].segments[0].span, Span::new(15, 27), "content span");
    }

    #[test]
    fn adjacent_segments_are_one_word_with_context_each() {
        // `foo"bar baz"qux` is ONE word of three segments -- the case a regex cannot express.
        let toks = lex("foo\"bar baz\"qux").expect("lexes");
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].text, "foobar bazqux");
        assert_eq!(toks[0].segments.len(), 3);
        assert_eq!(toks[0].segments[0].context, QuoteContext::Unquoted);
        assert_eq!(toks[0].segments[0].text, "foo");
        assert_eq!(toks[0].segments[1].context, QuoteContext::Double);
        assert_eq!(toks[0].segments[1].text, "bar baz");
        assert_eq!(toks[0].segments[2].context, QuoteContext::Unquoted);
        assert_eq!(toks[0].segments[2].text, "qux");
    }

    #[test]
    fn single_quotes_are_their_own_context() {
        let toks = lex("echo 'a b'").expect("lexes");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[1].text, "a b");
        assert_eq!(toks[1].segments[0].context, QuoteContext::Single);
    }

    #[test]
    fn other_delimiter_inside_quotes_is_content() {
        // A ' inside "..." is ordinary text, and vice versa.
        let toks = lex("echo \"it's fine\"").expect("lexes");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[1].text, "it's fine");
    }

    #[test]
    fn empty_quotes_are_a_real_empty_argument() {
        let toks = lex("echo \"\"").expect("lexes");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[1].text, "");
        assert_eq!(toks[1].segments.len(), 1, "the empty quoted region is kept");
        assert_eq!(toks[1].segments[0].context, QuoteContext::Double);
    }

    #[test]
    fn unterminated_quote_is_a_lex_error_spanned_from_the_opener() {
        let err = lex("echo \"unclosed").expect_err("unterminated quote must error");
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
        let toks = lex("echo $(echo a b)").expect("lexes");
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
        let toks = lex(r#"echo $(echo ")")"#).expect("lexes");
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
        let toks = lex("echo $(echo $(pwd))").expect("lexes");
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
        let toks = lex("echo '$(echo a b)'").expect("lexes");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[1].text, "$(echo a b)");
        assert_eq!(toks[1].segments[0].context, QuoteContext::Single);
    }

    /// Unterminated `$(` errors from the OPENER, the same shape as an unclosed quote, rather
    /// than inventing a second convention for the same class of failure.
    #[test]
    fn unterminated_command_substitution_errors_from_the_opener() {
        let err = lex("echo $(echo a").expect_err("unterminated substitution must error");
        assert_eq!(err.span.start, 5, "points at the $ that opened it");
    }

    #[test]
    fn unicode_content_survives() {
        let toks = lex("echo \"héllo 中\"").expect("lexes");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[1].text, "héllo 中");
    }
}
