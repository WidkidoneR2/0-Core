---
id: 186
date: 2026-07-21
type: arch
title: "tree-sitter syntax highlighting for fsh -- the editor-side of the parse arc, separate from INT-169's execution parser"
status: planned
tags: [fsh, tree-sitter, highlighting, syntax, 169, intelligence-arc]
---

## Vision
Give fsh real syntax highlighting via tree-sitter: as you type, commands/pipes/redirects/strings/
substitutions are colored by their actual grammatical role, not by a flat regex. The editor-side of
the intelligence arc (real parsing -> structured understanding -> the shell shows you what it sees).

## Relationship to INT-169 (the spine) -- READ THIS FIRST
169 builds fsh's EXECUTION parser: logos + handwritten recursive-descent -> AST, the thing commands
route through to RUN. This intent is DIFFERENT: tree-sitter is for HIGHLIGHTING (display), not execution.
tree-sitter is built for incremental, error-tolerant parsing of source in editors -- exactly right for
"color this line as I type it," exactly wrong for "execute this command" (169 owns that).

## THE REAL COST -- the honest gate-zero question
Using tree-sitter means fsh's grammar is defined TWICE: once in 169's handwritten parser (to execute),
once in a tree-sitter grammar (to highlight). Two grammars that must stay in sync = a real maintenance
cost and a drift risk (the same class of problem INT-171 fixed: multiple representations diverging).
GATE ZERO: is that duplication worth it, versus extending 169's handwritten parser to ALSO emit highlight
spans (one grammar, two outputs: AST for execution + spans for color)? That single-grammar alternative may
be the better fit for fsh's "one structure everything routes through" philosophy. ANSWER THIS BEFORE
adopting tree-sitter -- it may resolve to "no, emit spans from the handwritten parser instead."

## Verify-first
fsh ALREADY has some highlighting via rustyline's ForestHelper (and INT-168 moves to reedline, which has
its own highlighting hook). So this is NOT "fsh has no highlighting" -- it is "can tree-sitter (or the
169 parser emitting spans) do BETTER than ForestHelper, and is the gain worth the cost?" Name what the
current highlighting CANNOT do that this would fix.

## Sequencing
AFTER INT-169 has an AST (you cannot decide "emit spans from the parser vs a second tree-sitter grammar"
until the parser exists). AFTER INT-168 (reedline owns the highlight hook the spans feed into). This is a
late intent in the arc, not a now.

## Success Criteria
- [ ] Verify-first: document what fsh's CURRENT highlighting (ForestHelper / reedline) does and cannot do.
- [ ] Gate zero answered: tree-sitter (two grammars) vs 169-parser-emits-spans (one grammar) -- decide with
      reasoning, or CANCEL if the handwritten parser can emit spans well enough (a legitimate outcome).
- [ ] If tree-sitter proceeds: what grammar, how it stays in sync with 169's parser, drift guard.
- [ ] Highlighting demonstrably better on a real case the old approach got wrong.
- [ ] fsh still boots, logs in, deploys. No regression to the line editor.
- [ ] Each gate carries evidence per INT-158.

## The Rule
"Highlighting is display, execution is 169. The honest first question is not 'which tree-sitter grammar'
-- it is 'do we need a second grammar at all, or can the parser we're already building show its work?'" 🌲

---

## VERIFY-FIRST AND GATE ZERO, ANSWERED 2026-09-14 -- INTENT STAYS OPEN

### What the current highlighting actually does

`ForestHelper::highlight` (completion.rs:1193). Read, not remembered:

    let first_word = trimmed.split_whitespace().next().unwrap_or("");

That is the entire parse. One word, taken by whitespace, coloured by a four-way lookup --
dangerous / forest-native / known / unknown -- plus an amber wash over the REST of the line when
the command is dangerous. Natural-language lines get one purple sweep.

**There is no shadow grammar here. There is no grammar at all.** Pipes, redirects, strings,
substitutions, operators, assignments -- none of them are seen. Everything this intent imagined
adding is ABSENT rather than done badly, so the gain it describes is real.

⚠️ AND IT IS NOT QUOTE-AWARE, which is the INT-171 gate-2 defect in a place nobody looked:
`"my file" ls` takes `"my` as the command word and colours it unknown-red. `command_word()` --
the quote-aware derivation INT-171 built and INT-195 made canonical -- is right there and unused.

### Gate zero: tree-sitter, or the parser we already have?

⭐ THE SPINE ALREADY CARRIES SPANS. Measured in spine/lexer.rs:

    Token { text, span, segments }
    WordPart::Literal { text, span }
    WordPart::CommandSub { source, span }

Every token and every lexical segment knows where it started and ended, in the parser this shell
ALREADY runs on every line. The structure a role-based highlighter needs is not missing -- it is
produced and then discarded at the prompt.

So the comparison is not "two grammars vs one". It is:

    tree-sitter    a SECOND grammar for a language this shell already parses, kept in sync by
                   hand, plus a dependency. Its advantages -- incremental, error-tolerant --
                   are real for an EDITOR parsing files it did not write. This shell owns its
                   grammar and reparses one line at a time.

    spans          ONE grammar, two outputs. The AST for execution, spans for colour. Drift is
                   impossible because there is nothing to drift from.

**RULED for the HIGHLIGHTING problem: spans, not tree-sitter.** The duplication this intent
named as the real cost is the whole cost, and the alternative it named as maybe-better is
measurably available.

### ⚠️ NOT CANCELLED, AND THAT IS DELIBERATE

The ruling above is about ONE use -- colouring an fsh line at the prompt. tree-sitter's actual
strength is parsing languages this project does NOT own, which is a different problem that has
not been scoped:

    a code-aware fsearch that finds a FUNCTION rather than a line matching a regex
    structural navigation of Rust, markdown, TOML
    anything where the shell must understand a file it did not write

None of that is this intent's stated vision, and none of it is decided here. The intent stays
open because the tool may be right for a problem not yet written down -- and closing it would
mean re-deriving this recon when that problem arrives.

### What proceeds now, neither of which needs this intent

    NOW, one line    `ForestHelper::highlight` calls `command_word()` instead of
                     split_whitespace. Fixes a real quote bug, needs no spans.

    LATER            `highlight_spans(line) -> Vec<(Span, Role)>` on the spine, consumed by
                     whichever line editor is in place. Belongs beside INT-168, which owns the
                     hook it feeds.

### Success criteria

- [x] Verify-first: document what the CURRENT highlighting does and cannot do.
      <!-- evidence: completion.rs:1193-1242 read 2026-09-14. One word by split_whitespace, four-way lookup, amber rest for dangerous commands. Not quote-aware. -->
- [x] Gate zero answered: tree-sitter vs parser-emits-spans, FOR HIGHLIGHTING.
      <!-- evidence: spine/lexer.rs carries Span on Token, WordPart::Literal and WordPart::CommandSub. One grammar already produces what a highlighter needs; a second would only add drift. Highlighting proceeds via spans. -->
