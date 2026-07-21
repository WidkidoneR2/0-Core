---
id: 169
date: 2026-07-16
type: arch
title: "fsh spine: logos lexer + handwritten recursive-descent parser -> AST. The parse/plan/execute rebuild"
status: in-progress
tags: [fsh, parser, ast, logos, lexer, recursive-descent, spine, phase3, 171]
---

## Vision
Build the parse -> AST layer fsh never had. Source -> logos (regular tokens) + custom lexer state
(context-sensitive regions) -> handwritten recursive-descent parser -> AST. ExecContext holds the
AST, not cmd + args. This is the structural reframe of how fsh represents commands -- the thing that
ends the bug class the string-based approach keeps generating.

## GATE ZERO -- ANSWERED WITH EVIDENCE (2026-07-21, 171 complete)
The original gate: "name THREE things fsh cannot do because of its current parser, each a real thing
Christian wanted and could not have -- or CANCEL." Answered, and the evidence is stronger than three
feature-wishes: it is a RECURRING BUG CLASS, all the same shape (commands are cmd+args strings
re-inspected in scattered places, each place able to be wrong independently):
  - INT-172: redirect dropped the line after `2>` (a parser that did not understand redirect structure)
  - INT-174: `$(...)` executed inside single quotes (no quote-context in the "parse")
  - INT-143/171: FOUR divergent tokenizers, six bugs, incl. running a command twice and reporting
    success for a command that never ran
RECON 2026-07-21 (post-171): fsh has a TOKENIZER (commands::tokenize, works), NOT a parser/AST layer.
ExecContext = { cmd: String, args: Vec<String> } (exec.rs:23). from_line() is `raw.splitn(2, ' ')` +
lowercase + tokenize -- there is no parse-into-structure stage at all. Pipelines/redirects/conditionals
are handled by ad-hoc string re-inspection elsewhere. The ABSENCE of a parse->AST layer is the
limitation. Gate resolves YES -- but "add the missing layer", NOT "rewrite working code" (see no-big-bang).

## THE TOOL DECISION (2026-07-21, Christian's call after evaluating the options)
DECIDED: `logos` (lexer) + HANDWRITTEN recursive-descent parser (Pratt for expressions if/when needed).
chumsky is DROPPED. Reasoning:
  - A handwritten recursive-descent parser is DEBUGGABLE THE WAY THE WHOLE SHELL IS DEBUGGED -- greppable,
    steppable, readable as a straight call path. For a daily-driver shell where a parser bug = an unusable
    terminal, "easy to debug" is not a nice-to-have, it is the safety model. chumsky's combinator
    composition is a harder, more distributed kind of debugging -- wrong for this risk profile.
  - logos removes the repetitive MECHANICAL lexer boilerplate (regular tokens: words, numbers, operators,
    whitespace, punctuation) -- fast (compiles to a DFA), declarative, low-maintenance.
  - The CONTEXT-SENSITIVE regions (strings, heredocs, `$(...)` command sub, `${...}` var expansion,
    interpolation) get a CUSTOM STATEFUL LEXER layer -- full control exactly where the bugs live (172/174).
  - This directly RESOLVES the caution below (lines about `>` being redirect-vs-comparison, context fighting
    parser combinators): the custom lexer DISAMBIGUATES context into distinct token types BEFORE the parser
    sees them, so the parser reads a clean, already-disambiguated token stream. No combinator fights context.
FULLY-HANDWRITTEN LEXER was considered and set aside: only needed for complete char-level control or strict
existing-shell compatibility. fsh is its OWN thing (not POSIX-sh compatible by goal), so logos earns its
place -- the last 5% of lexer control is not needed.

## The hybrid lexer shape (Christian's design)
    Source
      -> logos:        words | numbers | operators | whitespace | punctuation
      -> custom state: strings | heredocs | command substitution | variable expansion | interpolation
      -> token stream
      -> recursive-descent parser (Pratt for expressions if needed)
      -> AST: Command / Pipeline / Redirect / Assignment / (If / While / Function as the grammar grows)
      -> ExecContext holds the AST

## Rides with the spine (small correctness fixes the new structure enables)
- Stop lowercasing the command name (from_line's `.to_lowercase()` -- wrong; a real parser preserves case).
- Store SystemTime, not a raw u64 unix timestamp.
- Give each execution a unique ID (UUID or incrementing) -- this also lights up the DEAD correlation_id
  column (INT-167), finally making cross-layer tracing real.

## Scoped OUT of 169 (separate sequenced intents -- do NOT smuggle in)
- tree-sitter = for SYNTAX HIGHLIGHTING (editor side), NOT the parser. chumsky/handwritten owns execution
  parsing; tree-sitter would define fsh's grammar a SECOND time for highlighting -- a real cost to weigh in
  its own intent, not a free add. Its own future intent.
- reedline (INT-168), gix (git migration off libgit2), nix (job control), tokio (async), nu-ansi-term
  (check vs existing `colored` first) -- each its own sequenced sibling intent. 169 is the SPINE ONLY.

## PRESERVED HISTORY (do not lose -- this is why the discipline exists)
### The four-parser measurement (INT-143, 2026-07-16)
  1. commands/mod.rs tokenize_args -- CORRECT, quote-aware
  2. exec.rs tokenize -- CORRECT, quote-aware, a byte-for-byte DUPLICATE of #1
  3. main.rs:2411 redirect branch -- WRONG (split_whitespace)
  4. main.rs:1956 inline-VAR loop -- WRONG (split_whitespace)
### THE TRAP THIS INTENT MUST NOT FALL INTO
Those bugs were NOT an argument for a new parser crate. fsh ALREADY HAD a correct tokenizer -- twice. The
bugs were code that did not CALL it. You can fail to call a handwritten parser exactly as easily as you can
fail to call tokenize(). 171 consolidated to ONE entry point FIRST so this could be judged honestly. The
spine REPLACES the string-reinspection with a single AST every path routes through -- that is the point,
not "a better parser."
### The chumsky context-sensitivity caution (2026-07-16) -- now RESOLVED by the hybrid
"Shell syntax is context-sensitive: `>` is a redirect here and a comparison there; a word is a command,
a filename, or a bare string depending on position. That is not chumsky's sweet spot." -> RESOLVED: the
custom stateful LEXER disambiguates context into distinct tokens before parsing, and the parser is
handwritten (no combinator fighting context at all).

## Scope guardrails (STILL BINDING)
NO BIG-BANG. It starts behind the 171 entry point, ONE CONSTRUCT AT A TIME, old path live until the new one
passes the SAME tests (fsh-test REPL-driven). fsh must boot, log in, and deploy at every step -- it is the
daily driver AND the demo. Honestly stated: the full spine is a long build (many nights), not a weekend.
Christian committed to it 2026-07-21 as active work, incrementally -- "rolling the dice" on the right shape,
built behind the discipline that makes it not-a-gamble.

## Success Criteria
- [x] INT-171 COMPLETE before this starts.
      <!-- DONE -- 171 complete, deployed gen 402, one tokenizer entry point exists. -->
- [x] Gate zero answered with evidence, or cancel.
      <!-- DONE 2026-07-21 -- answered YES. Evidence: the bug class (172 redirect, 174 quote-context,
      143/171 four-tokenizer divergence) + recon finding (tokenizer exists, no parse/AST layer;
      ExecContext = cmd+args, from_line = splitn(2,' ')). The absence of the layer IS the limitation. -->
- [ ] RFC first: what problem, why logos+handwritten, alternatives (chumsky/fully-handwritten) & why not,
      trade-offs, how it fits fsh's philosophy. Written before code. (Largely drafted in the 2026-07-21
      discussion -- capture it as the RFC.)
- [ ] logos added + the hybrid lexer stands up: regular tokens via logos, custom stateful layer for
      strings/heredocs/$()/${}/interpolation. Demonstrated tokenizing a real line into typed tokens.
- [ ] ONE CONSTRUCT end-to-end as proof-of-shape: a simple command -> logos+lexer tokens -> handwritten
      parse -> AST node -> execute, with the OLD path live beside it and the SAME REPL tests passing.
- [ ] AST types defined (Command / Pipeline / Redirect / Assignment to start) + ExecContext holds the AST.
- [ ] The rides-with fixes: stop lowercasing cmd; SystemTime not u64; unique execution ID (lights up 167's
      correlation_id).
- [ ] fsh still boots, logs in, deploys at EVERY step. No big-bang.
- [ ] Each gate carries evidence per INT-158.

## The Rule
"fsh already had a correct tokenizer -- twice -- and still broke, because nothing routed through one
structure. The spine is not a better parser. It is a single AST every path must go through, built one
construct at a time, in code Christian can read every line of." 🌲
