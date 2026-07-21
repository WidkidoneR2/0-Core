//! INT-169: the fsh parser spine -- AST (pure syntax).
//!
//! Design: docs/rfc-169-parser-spine.md. The AST is a Spanned enum tree of owned
//! data with structured Words and NO execution concerns. It knows nothing about
//! environment variables, builtins, PATH, or process spawning -- that is the
//! expansion + execution phases' job. The parser builds this; nothing else.
//!
//! Only `Command` (with `Literal`-only words) is BUILT today -- the first construct
//! (RFC section 8). Other NodeKind variants and WordPart variants are named for shape
//! and turned on as the roadmap (RFC section 7) reaches them. This is the real data
//! model with unsupported syntax not yet enabled -- not a prototype to be replaced.

/// Byte offsets into the source line. Half-open: `[start, end)`.
///
/// RFC section 4.2 -- spans everywhere, non-negotiable. Every node carries one so
/// parser errors, runtime errors, highlighting, a debugger, completion, a formatter,
/// and tracing can all point at exact source. Retrofitting spans later would touch
/// every constructor and visitor; they are built in from the first line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// Construct a span. Debug-asserts `start <= end` -- a reversed span is always a bug.
    pub fn new(start: usize, end: usize) -> Self {
        debug_assert!(start <= end, "Span::new: start ({start}) > end ({end})");
        Span { start, end }
    }

    /// Length of the spanned region in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// True if the span covers no bytes.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// The smallest span covering both `self` and `other` (for building a parent
    /// node's span from its children). Grows to the outer bounds of the two.
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// Any value carried with its source span. RFC section 4.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

impl<T> Spanned<T> {
    pub fn new(span: Span, value: T) -> Self {
        Spanned { span, value }
    }
}

/// A fully-parsed node: one syntactic construct plus its span.
pub type Node = Spanned<NodeKind>;

/// The tree. RFC section 4.1 -- an enum (not struct-per-construct) so matching is
/// exhaustive and adding a construct is adding a variant. The compiler then tells
/// every visitor exactly what it has not handled yet.
///
/// Only `Command` exists today. Pipeline / Sequence / Redirect / If / While / Function
/// (and later And/Or, Subshell, Case, ...) are added as the roadmap reaches them --
/// their fields are filled in when the construct is built, because building the earlier
/// constructs informs the later shapes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Command(Command),
    // Pipeline(Pipeline)   -- roadmap step 6
    // Sequence(Sequence)   -- roadmap step 7
    // Redirect(Redirect)   -- roadmap step 5 (as a Command field first)
    // If(IfNode)           -- roadmap step 9
    // While(WhileNode)     -- roadmap step 9
    // Function(Function)   -- roadmap step 9
}

/// A command: structured words plus redirects. RFC sections 4.4 / 4.5.
///
/// NO execution concerns -- there is deliberately no `builtin: bool`. Whether a command
/// resolves to a builtin is decided at execution time against the dispatch table; baking
/// it into the parse output would couple parsing to runtime state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub words: Vec<Word>,
    pub redirects: Vec<Redirect>,
}

/// A word = the smallest unit that EXPANSION produces. RFC section 4.4.
///
/// A shell argument is not a string -- `echo foo$BAR"baz"` is one word of three parts.
/// The parser builds this from syntax; it does not know or care what `$BAR` evaluates to.
/// Modeling words as structured parts is what makes the INT-172 / INT-174 bug class
/// impossible by construction: a `$(...)` inside single quotes is a `Literal` part, so no
/// later stage can mistake it for a live substitution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    pub parts: Vec<WordPart>,
}

impl Word {
    /// A word made of a single literal part -- the only kind built today (roadmap step 1).
    pub fn literal(s: impl Into<String>) -> Word {
        Word {
            parts: vec![WordPart::Literal(s.into())],
        }
    }

    /// True if every part is a `Literal` (no expansion needed). Convenience for the
    /// early roadmap steps and tests; expansion will use richer inspection later.
    pub fn is_all_literal(&self) -> bool {
        self.parts.iter().all(|p| matches!(p, WordPart::Literal(_)))
    }
}

/// The pieces a word is composed of. RFC section 4.4.
///
/// `Literal` is implemented today. The others are named so the shape is fixed, and turned
/// on at their roadmap step. Do NOT add variants for `${VAR:-default}`, brace expansion,
/// process substitution, or globbing until those features are actually implemented -- the
/// architecture already has room; naming every future variant now is over-design.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WordPart {
    Literal(String),
    // Variable(String)       -- roadmap step 3
    // CommandSub(Box<Node>)  -- roadmap step 8
    // Arithmetic(Box<Node>)  -- later
}

/// A redirection. RFC section 4.4 -- shape is filled in at roadmap step 5; today it is an
/// empty placeholder so `Command.redirects` is a real (always-empty) field from day one,
/// not something bolted on later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect;
