//! INT-169: the fsh parser spine -- AST (pure syntax).
//!
//! Design: docs/rfc-169-parser-spine.md. The AST is a Spanned enum tree of owned
//! data with structured Words and NO execution concerns. It knows nothing about
//! environment variables, builtins, PATH, or process spawning -- that is the
//! expansion + execution phases' job. The parser builds this; nothing else.
//!
//! ★ AST STABILITY CHECKPOINT (2026-07-21, after Increment 5 parser torture):
//! CORE-FROZEN with Redirect internals RESERVED. Frozen: Span, Spanned<T>, the AstNode
//! sum type, the Command boundary (owns argv-like words + IO-transformations), the Word
//! model, the WordPart ownership model (the ENUM grows -- freeze the concept, not the
//! variant list), source-preserving semantics. Reserved: Redirect internals (fd is
//! Option, target waits on expansion, dup/append categories, pipeline ownership all TBD
//! at roadmap step 5). Constraint: Command.redirects exists permanently; Redirect may
//! evolve before ExecutionPlan lowering.

/// Byte offsets into the source line. Half-open: `[start, end)`.
///
/// RFC section 4.2 -- spans everywhere, non-negotiable. FROZEN. Every node carries one so
/// parser errors, runtime errors, highlighting, a debugger, completion, a formatter,
/// and tracing can all point at exact source.
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

/// Any value carried with its source span. RFC section 4.2. FROZEN.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub span: Span,
    pub node: T,
}

impl<T> Spanned<T> {
    pub fn new(span: Span, node: T) -> Self {
        Spanned { span, node }
    }
}

/// The AST root: a semantic sum type of executable language constructs. FROZEN.
///
/// NOT a generic tree-node system -- a Word is not an AstNode, a WordPart is not an
/// AstNode; those are structured components UNDER a Command. AstNode is the top-level
/// construct level. An enum (not struct-per-construct) so matching is exhaustive and
/// adding a construct is adding a variant; the compiler then tells every visitor exactly
/// what it has not handled yet.
///
/// Only `Command` exists today. Pipeline / If / While / etc. are added as the roadmap
/// reaches them -- their fields are filled in when the construct is built, because
/// building the earlier constructs informs the later shapes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstNode {
    Command(Command),
    // Pipeline(Pipeline)   -- roadmap step 6 (composition, not a "statement")
    // Sequence(Sequence)   -- roadmap step 7
    // If(IfNode)           -- roadmap step 9
    // While(WhileNode)     -- roadmap step 9
    // Function(Function)   -- roadmap step 9
}

/// A command: structured words plus redirects. RFC sections 4.4 / 4.5. Boundary FROZEN.
///
/// The command owns argv-like things (words) AND IO transformations (redirects) -- that
/// separation is correct and permanent. NO execution concerns: there is deliberately no
/// `builtin: bool`. Whether a command resolves to a builtin is decided at execution time
/// against the dispatch table; baking it into the parse output would couple parsing to
/// runtime state.
///
/// Words are individually spanned (`Spanned<Word>`) -- the "spans everywhere" principle
/// applied fully, so completion, jump-to-token, and per-word error highlighting have
/// exact per-word source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub words: Vec<Spanned<Word>>,
    pub redirects: Vec<Spanned<Redirect>>,
}

/// A word = the smallest unit that EXPANSION produces. RFC section 4.4. FROZEN.
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

/// The pieces a word is composed of. RFC section 4.4. Ownership model FROZEN; the ENUM
/// GROWS (freeze the concept, not the variant list).
///
/// `Literal` is implemented today. The others are named so the concept is fixed, and
/// turned on at their roadmap step. Do NOT add variants for `${VAR:-default}`, brace
/// expansion, process substitution, or globbing until those features are actually
/// implemented -- the architecture already has room; naming every future variant now is
/// over-design.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WordPart {
    Literal(String),
    /// A variable reference SITE, recognised but not evaluated. The AST records that `$HOME`
    /// occurred; what it evaluates to is the expansion phase's business. Produced only from
    /// Unquoted and Double segments -- inside single quotes a `$` is literal text, which is
    /// what makes the INT-172/174 bug class impossible by construction.
    Variable(String),
    /// A special shell parameter -- `$?`, `$$`. NOT a Variable: these are not NAME LOOKUPS,
    /// which is exactly why VarResolver has `last_exit()` and `pid()` as distinct methods.
    /// Modelling them as `Variable("?")` would force every resolver to special-case a name that
    /// can never be set, and would invent a fake variable namespace. Room here for `$#`, `$@`,
    /// `$*` and the positional parameters when they land.
    SpecialParam(SpecialParam),
    /// A command substitution: an EMBEDDED SHELL PROGRAM, parsed rather than stored as text.
    ///
    /// ⚠️ The span inside this node is relative to the SUBSTITUTION'S OWN SOURCE, not the outer
    /// line -- a nested parse is its own coordinate system. Rendering a diagnostic against the
    /// original line would need offsetting by the substitution's start, and the information to do
    /// that is preserved on the lexer's `CommandSub` segment. Recorded, not yet needed.
    CommandSub(Box<Spanned<AstNode>>),
    // Arithmetic(Box<Spanned<AstNode>>) -- later
}

/// The shell's special parameters -- values the shell itself owns, addressed with `$` but never
/// settable as variables. Each maps to its own VarResolver method rather than a name lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialParam {
    /// `$?` -- the previous command's exit status.
    LastExit,
    /// `$$` -- the shell's process id.
    Pid,
}

/// A redirection. RESERVED -- internals designed at roadmap step 5 (fd:Option<u32>,
/// target:Word-after-expansion, Write/Append/dup categories, pipeline ownership). Today
/// an empty placeholder so `Command.redirects` is a real (always-empty) field from day
/// one, not something bolted on later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect;
