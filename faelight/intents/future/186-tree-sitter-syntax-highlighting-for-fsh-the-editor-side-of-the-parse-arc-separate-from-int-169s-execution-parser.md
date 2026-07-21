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
