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

    #[test]
    fn unicode_content_survives() {
        let toks = lex("echo \"héllo 中\"").expect("lexes");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[1].text, "héllo 中");
    }
}
