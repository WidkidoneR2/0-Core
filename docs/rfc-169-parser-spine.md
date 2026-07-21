# RFC-169: The fsh Parser Spine

**Status:** Accepted — active build (INT-169, in-progress)
**Date:** 2026-07-21
**Author:** Christian (design), captured with Claude
**Supersedes the scope of:** the original INT-169 "one entry point / logos-chumsky" framing
**Depends on:** INT-171 (one parsing entry point — complete)
**Scoped out (own intents):** INT-186 tree-sitter highlighting, INT-168 reedline, INT-187 gix, INT-188 job control

---

## 1. Problem

fsh has a **tokenizer**, not a **parser**. There is no parse-into-structure stage and no
AST. A command is represented as `ExecContext { cmd: String, args: Vec<String> }`, built by
`from_line()` doing `raw.splitn(2, ' ')` + `to_lowercase()` + `tokenize()`. Everything more
structured than "a command word and some argument strings" — pipelines, redirects, quoting,
substitution, conditionals — is handled by **ad-hoc string re-inspection scattered across the
code**.

That scattering is the root cause of a recurring **bug class**, all the same shape (a string
gets re-scanned in one place with logic that another place doesn't share):

- **INT-172** — the line after `2>` was dropped: the redirect handler didn't understand
  redirect *structure*, it re-split the string and lost the tail.
- **INT-174** — `$(...)` executed inside single quotes: expansion scanned raw text with no
  quote-context, so it couldn't tell a live substitution from a literal one.
- **INT-143 / INT-171** — four divergent tokenizers, six bugs, including one that ran a
  command twice and one that reported success for a command that never ran.

These are not unrelated. They are what happens when there is no single structural
representation that every stage routes through. Each fix patches one scanner; the next
scanner can still be wrong independently.

**The absence of a parse → AST layer is the limitation.** The fix is to *add the missing
layer*, not to rewrite working code. fsh's tokenizer works; what's missing is a parse stage
above it that produces a structure execution walks — so a redirect is a `Redirect` node, a
quoted region is a known `WordPart`, and no stage re-guesses from raw text.

---

## 2. The Decision

Build the spine as distinct phases:

```
source text
     │
     ▼
  lexer            logos (regular tokens) + custom stateful lexer (context-sensitive regions)
     │
     ▼
  tokens
     │
     ▼
  parser           handwritten recursive-descent (Pratt for expressions if/when needed)
     │
     ▼
  AST              PURE SYNTAX — knows nothing about env, builtins, PATH, or spawning
     │
     ▼
  expansion        variable expansion, command substitution, globbing, field splitting
     │               — evaluates each Word into one or more strings
     ▼
  execution plan
     │
     ▼
  executor
```

**Lexer:** `logos` for the mechanical regular tokens (words, numbers, operators, whitespace,
punctuation — compiles to a DFA, fast, low-maintenance). A **custom stateful lexer layer** for
the context-sensitive regions (strings, heredocs, `$(...)`, `${...}`, interpolation) — full
control exactly where the bugs live. The custom layer **disambiguates context into distinct
token types before the parser sees them**, so the parser reads a clean, already-disambiguated
stream.

**Parser:** handwritten recursive-descent. Chosen because it is **debuggable the way the whole
shell is debugged** — greppable, steppable, readable as a straight call path. For a daily-driver
shell where a parser bug means an unusable terminal, easy-to-debug *is* the safety model.

**AST:** an enum tree of `Spanned<NodeKind>`, owned data, pure syntax. Detailed in §4.

---

## 3. Alternatives Considered

**chumsky (parser combinators) — DROPPED.** Two reasons. (1) Combinator composition is a
harder, more distributed kind of debugging than a straight recursive-descent call path — the
wrong risk profile for a daily driver. (2) Shell syntax is context-sensitive (`>` is a redirect
here and a comparison there; a bare word is a command, a filename, or a string depending on
position), which fights parser combinators. The hybrid lexer resolves this by disambiguating
context *before* the parser, but with that done, a handwritten parser is simpler and clearer
than pulling in a combinator framework.

**Fully-handwritten lexer (no logos) — SET ASIDE.** Only needed for complete char-level control
or strict compatibility with an existing shell language. fsh is its own thing (not POSIX-sh
compatible by goal), so the last 5% of lexer control isn't needed. logos removes the repetitive
mechanical token-matching; the custom layer covers the parts logos can't.

**tree-sitter — NOT the execution parser (→ INT-186).** tree-sitter is for editor highlighting
(incremental, error-tolerant parsing of source for display). Using it here would define fsh's
grammar a *second* time. Highlighting is a separate concern with its own intent; whether it uses
tree-sitter or spans emitted from this parser is INT-186's question.

---

## 4. The AST Design ★ (100% depth — the expensive-to-change core)

This is the part that must be right the first time. Retrofitting any of these decisions means
touching every constructor, parser rule, visitor, and transformation. They are made once.

### 4.1 Enum tree, not struct-per-construct

```rust
pub enum NodeKind {
    Command(Command),
    Pipeline(Pipeline),
    Sequence(Sequence),
    Redirect(Redirect),
    If(IfNode),
    While(WhileNode),
    Function(Function),
    // future variants (And/Or, Subshell, Case, ...) added when implemented
}
```

Rust enums make recursive tree structures ergonomic: `match node { Command(c) => ..., Pipeline(p)
=> ... }` instead of dynamic dispatch. Adding a construct (`&&`, `||`, subshells, `case`,
arithmetic) is adding a variant, and the compiler's exhaustiveness checking then tells every
visitor exactly what it hasn't handled yet. That is the extensibility that keeps this from being
redone.

### 4.2 Spans everywhere — non-negotiable

Every node carries its source position. Not optional, not only parser nodes — every node.

```rust
pub struct Span {
    pub start: usize,
    pub end: usize,
}

pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

pub type Node = Spanned<NodeKind>;
```

Spans pay off for parser errors, runtime errors, syntax highlighting, a shell debugger, a
completion engine, a formatter, "jump to token", a language server, and tracing. Retrofitting
spans later is painful precisely because every constructor, visitor, parser rule, and
transformation would need updating. Do it once; forget about it forever.

### 4.3 Owned strings

`String` (or `Arc<str>` where sharing a subtree is cheap), never `&str`.

A shell parses tiny inputs — `git commit -m hello` is ~20 bytes — so the allocation cost is
microscopic. A borrowed AST forces `Node<'a>` → `Parser<'a>` → `Lexer<'a>` → `Visitor<'a>` →
`Executor<'a>` → `Completion<'a>`: lifetime plumbing through the entire stack. Owned data makes
caching ASTs, storing history, serialization, macro expansion, AST rewriting, and background
jobs all straightforward. The trade — a few microscopic allocations — is not worth the
lifetime tax.

### 4.4 Words are structured, not strings ★ (the key insight)

The most important modeling decision. A shell argument is **not** a string — it is a `Word`
composed of parts:

```rust
pub struct Command {
    pub words: Vec<Word>,
    pub redirects: Vec<Redirect>,
}

pub struct Word {
    pub parts: Vec<WordPart>,
}

pub enum WordPart {
    Literal(String),
    // named but dormant until implemented:
    Variable(String),
    CommandSub(Box<Node>),
    Arithmetic(Box<Node>),
}
```

`echo foo$BAR"baz"` is one word of three parts (`Literal("foo")`, `Variable("BAR")`,
`Literal("baz")`), not a string. `echo "$HOME"` is a word with a `Variable` part, not the text
`$HOME`. This structural representation is *what makes the 172/174 bug class impossible by
construction*: a `$(...)` inside single quotes is lexed as a `Literal` part, so no downstream
stage can mistake it for a live substitution — the structure already recorded the context.

**`Word` is best understood as the smallest unit that expansion produces.** The parser builds
`Word`s from *syntax*; it does not know or care what `$HOME` evaluates to. That is the
expansion phase's job (§4.6).

Keep `WordPart` small. Do **not** add variants for parameter-expansion operators
(`${VAR:-default}`), brace expansion, process substitution, or globbing until those features are
actually implemented. The architecture already has room; naming every future variant now is
over-design.

### 4.5 No execution concerns in the AST

The AST is pure syntax. It knows nothing about environment variables, builtins, PATH, or process
spawning.

```rust
// WRONG — builtin-ness is a runtime concern (depends on dispatch table, aliases, PATH)
pub struct Command { pub builtin: bool, /* ... */ }

// RIGHT — pure syntax
pub struct Command { pub words: Vec<Word>, pub redirects: Vec<Redirect> }
```

Whether a command is a builtin is decided at execution time against the dispatch table. Baking
it into the parse output couples parsing to runtime state and breaks the phase separation.

### 4.6 Phase separation (the discipline that makes it testable)

- The **lexer** recognizes tokens.
- The **parser** builds `Word`s (and the rest of the AST) from syntax — pure structure.
- The **expansion phase** evaluates each `Word` into one or more strings (variable expansion,
  command substitution, field splitting, globbing — the shell-specific semantics).
- The **executor** receives the fully expanded argument list.

Each stage is independently testable: parse-time knows structure, expand-time knows values,
execute-time knows results. This is the through-line — one structure everything routes through,
each stage with a single responsibility.

**Migration note — expand.rs today.** fsh already has an expansion stage (`expand.rs`, where the
INT-174 quote fix lives), but it operates on **raw strings**, re-scanning text. The spine moves
expansion from string-munging to **`Word`-walking**: expansion consumes the structured `Word {
parts }` instead of re-scanning raw text. This is *how* the spine retires the expand.rs bug
class — the context that string-scanning kept re-deriving (and getting wrong) is recorded in the
structure once, at parse time.

---

## 5. The Lexer Architecture ★

```
Source
  ├─ logos:         words | numbers | operators | whitespace | punctuation   (DFA, fast)
  └─ custom state:  strings | heredocs | command sub | variable expansion | interpolation
                          │
                          ▼
                    token stream (context already disambiguated)
```

**logos** handles the structurally regular tokens — a `|` is a `|` regardless of context, so a
DFA-based generated matcher is ideal: fast, declarative, low-maintenance.

**The custom stateful layer** handles the context-sensitive regions — the ones where INT-172 and
INT-174 lived. Whether a `$` begins a substitution or is literal *depends on lexer state* (are we
inside single quotes? inside a heredoc with a quoted delimiter?). A pure token-regex cannot
express that; it needs state. The custom layer tracks that state and emits **distinct token
types** so the parser never has to re-derive context.

**The handoff (mode-switching).** logos tokenizes the regular stream until it hits a
mode-entering trigger (an opening quote, `$(`, `<<HEREDOC`); control passes to the custom lexer
for that region, which consumes until the region closes, emits a structured token
(`StringLiteral`, `CommandSub`, ...), and hands back. logos supports this directly (callbacks /
`Lexer::morph`), so the hybrid rides a real logos feature rather than fighting it.

**Where the bugs will concentrate — and thus the tests.** The transition points (the `$(` that
enters a substitution, the balancing `)`, the quote that opens/closes a region, the heredoc
delimiter) are exactly where 172/174-class bugs occur. The hybrid's value is real; its risk is
concentrated entirely at the handoffs. Tests must be brutal there — the same REPL-driven
fsh-test cases that caught 172/174, extended per construct.

---

## 6. The Execution Interface ★

The AST slots into fsh's existing execution pipeline without breaking it. Today:

```
execute_with_context (exec.rs):
    parse    → ExecContext::from_line(line, db)      // to be replaced by real parse
    preexec  → can block
    dispatch → commands::execute(line, db, core_root)
    postexec → observe result (knowledge engine, history, causality)
    result
```

INT-171 already produced a clean dispatch seam: `commands::execute` (run it) and
`commands::try_builtin` (probe "is this a builtin?" without spawning), both over
`execute_impl`, with `commands::tokenize` as the single shared tokenizer. **This is where the
AST path is introduced beside the old string path.**

**Target:** `ExecContext` holds the AST (a `Node`), not `cmd: String + args: Vec<String>`. But
that is the *destination*, reached one construct at a time (§7), not in one step.

**Coexistence during migration.** The new parse → AST → execute path runs *beside*
`from_line`, gated, for the construct currently being built. The old path stays live for
everything else. A construct is "done" when its AST path passes the **same REPL tests** the old
path did and fsh still boots, logs in, and deploys. Only when all constructs route through the
AST does `from_line`'s string logic get retired.

---

## 7. The Roadmap — constructs in order

Each step extends the language **without changing the fundamental AST shape**. Variants that
already exist in `NodeKind` / `WordPart` are turned on; nothing is restructured.

1. **Bare commands** — `ls -la /tmp`
2. **Quoted literals** — `"hello world"`, `'abc'`
3. **Variable references** — `$HOME`  (turns on `WordPart::Variable`)
4. **Mixed words** — `foo$BAR"baz"`
5. **Redirections** — `>`, `<`, `>>`  (the INT-172 territory, now structural)
6. **Pipelines** — `|`
7. **Lists** — `&&`, `||`, `;`  (new `NodeKind` variants)
8. **Command substitution** — `$(pwd)`  (turns on `WordPart::CommandSub`; the INT-174 territory)
9. **Control flow** — `if`, `while`, `for`, functions

At every step fsh remains a working daily driver. No big-bang.

---

## 8. The First Construct — bare command, real shape

The first construct is a bare command (`ls -la /tmp`) built with the **real, final** data model —
`Command { words: Vec<Word>, redirects: Vec<Redirect> }`, each argument a `Word { parts:
[Literal(...)] }`. Redirects empty for now; only `WordPart::Literal` implemented.

```
ls -la /tmp
  →  Command
       words:
         Word { parts: [Literal("ls")] }
         Word { parts: [Literal("-la")] }
         Word { parts: [Literal("/tmp")] }
       redirects: []
```

This is **not a simplified prototype** — it is the final architecture with only the currently
supported syntax enabled. Nothing here is special-cased for literals; `Literal` is simply the
only `WordPart` variant implemented so far. When `$HOME` arrives (step 3), the parser builds the
same `Word` structure with a `Variable` part — the parser never cares whether a word has one part
or five; it always builds the same shape.

**Proof-of-shape target:** `ls -la /tmp` flows source → logos + custom lexer → tokens →
handwritten parse → `Command` AST (real `Word`s, `Literal` parts) → execute, with the old
`from_line` path live beside it and the same REPL tests green. That proves the entire spine
skeleton end-to-end on the simplest case before quotes, variables, redirects, or pipes — the
constructs where the hard context-sensitivity lives.

---

## 9. Rides-with fixes (small correctness the new structure enables)

- **Stop lowercasing the command name.** `from_line` does `.to_lowercase()` on the command word;
  a real parser preserves case. Case-folding is a dispatch concern if it's wanted at all, not a
  parse concern.
- **Store `SystemTime`, not a raw `u64` unix timestamp.** The current `timestamp: u64` loses type
  information and precision options.
- **Give each execution a unique ID** (UUID or incrementing). This also lights up the dead
  `correlation_id` column (INT-167), finally making cross-layer tracing real.

These ride with the spine because the new construction path is where they're naturally set.

---

## 10. How it fits fsh's philosophy

- **Debuggable line-by-line.** Handwritten recursive-descent + logos + a custom lexer are all
  readable, greppable, steppable. When a parse goes wrong, the fix is found the way every other
  fsh bug was found this year (172, 174, 183, 185): read the exact code, find the exact line.
- **One structure everything routes through.** The spine's whole point is that redirects,
  pipelines, quoting, and substitution stop being re-derived from raw strings in scattered
  places and become nodes in a single AST every stage walks.
- **Built one construct at a time, old path live.** fsh is the daily driver *and* the demo; it
  works at every step. The design is thorough where thoroughness prevents rework (the AST shape,
  the lexer boundary, spans, phase separation) and deliberately un-elaborated where
  over-specification would *cause* rework (future `WordPart` variants, far-future constructs).

> "fsh already had a correct tokenizer — twice — and still broke, because nothing routed through
> one structure. The spine is not a better parser. It is a single AST every path must go through,
> built one construct at a time, in code Christian can read every line of." 🌲

---

## Appendix A — The core types, collected

```rust
// spans — §4.2
pub struct Span { pub start: usize, pub end: usize }
pub struct Spanned<T> { pub span: Span, pub value: T }
pub type Node = Spanned<NodeKind>;

// the tree — §4.1
pub enum NodeKind {
    Command(Command),
    Pipeline(Pipeline),
    Sequence(Sequence),
    Redirect(Redirect),
    If(IfNode),
    While(WhileNode),
    Function(Function),
    // + And/Or, Subshell, Case, ... as implemented
}

// words — §4.4
pub struct Command {
    pub words: Vec<Word>,
    pub redirects: Vec<Redirect>,
}
pub struct Word { pub parts: Vec<WordPart> }
pub enum WordPart {
    Literal(String),
    Variable(String),      // dormant until step 3
    CommandSub(Box<Node>), // dormant until step 8
    Arithmetic(Box<Node>), // dormant until implemented
}
```

*Node types beyond Command (Pipeline, Redirect, If, ...) are named here and sketched as the
roadmap reaches them; their fields are filled in when the construct is built, because building
the earlier constructs informs the later shapes. The AST's spine — Spanned enum tree, owned
data, structured Words, no execution concerns — is fixed now.*
