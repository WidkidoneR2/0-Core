---
id: 169
date: 2026-07-16
type: arch
title: "fsh parser: ONE entry point first -- fsh already has two correct tokenizers nobody calls. logos/chumsky only if the grammar demands it"
status: planned
tags: [fsh, parser, ast, logos, chumsky, lexer, phase3, 171]
---

## Vision
Decide -- with evidence -- whether fshs parser is FUNDAMENTALLY LIMITING. If it is, evaluate `logos`
for the lexer first. Consider `chumsky` ONLY if committed to redesigning the grammar and the AST.

## THE FIRST GATE IS A QUESTION, NOT A TASK
This intent may correctly end in "no". That is a real outcome, not a failure. INT-159 is the
precedent: filed as "faelight-vm owns the launch: a real Rust qemu launcher", and the premise
COLLAPSED under testing -- the rewrite was never needed. INT-027s organic rule exists for this:
no big-bang rewrite of working code.

## Do INT-171 FIRST. This intent is not startable before it.
MEASURED 2026-07-16 (INT-143): fsh has FOUR parsers.
  1. commands/mod.rs tokenize_args   -- CORRECT, quote-aware
  2. exec.rs tokenize                -- CORRECT, quote-aware, a byte-for-byte DUPLICATE of #1
  3. main.rs:2411 redirect branch    -- WRONG (split_whitespace)
  4. main.rs:1956 inline-VAR loop    -- WRONG (split_whitespace)
Six bugs came out of that, including one that ran commands twice and one that reported success for
commands that never ran.

THE TRAP THIS INTENT MUST NOT FALL INTO: those bugs are NOT an argument for chumsky. fsh ALREADY HAD
A CORRECT TOKENIZER -- twice. The bugs were code that did not CALL it. You can fail to call chumsky
exactly as easily as you can fail to call tokenize(). Adding a fifth parser to a shell with four
parsers gives you five parsers.
So: 171 consolidates to ONE entry point. THEN this intent can ask its question honestly, because for
the first time there will be a single parser to judge. 171s OUTCOME IS 169s EVIDENCE.

## The AST argument -- the strongest thing on the table (advisory, 2026-07-16)
"Many hobby shells stop at token lists. Commit to an explicit AST."
    Instead of  Vec<String>
    think       Command / Pipeline / Redirect / VariableAssignment / If / While / Function
Once there is a real AST: syntax highlighting becomes easier, formatting becomes possible, scripting
grows naturally, plugins can INSPECT commands, and future language features fit cleanly.
"The AST becomes the language, rather than the parser."
That is a genuinely bigger idea than any crate on the list, and it is the thing this intent is really
about. logos and chumsky are implementation details of it.

## The order the advisory gave, and it is right
1. Is the current parser fundamentally limiting? (ANSWER THIS FIRST, with 171 done)
2. If so -- `logos` first. A lexer generator is a contained change: same grammar, better tokenizing.
3. `chumsky` ONLY if committed to redesigning grammar AND AST. Note the caution from the same call:
   parser = "custom initially; consider chumsky only if it clearly simplifies the grammar".
   Shell syntax is context-sensitive in ways that fight parser combinators -- `>` is a redirect here
   and a comparison there; a word is a command, a filename, or a bare string depending on position.
   That is not chumskys sweet spot, and pretending otherwise is how you get a rewrite that stalls.

## Scope guardrails
NOT PRE-OCTOBER. October is ~10 weeks out. An AST plus an executor redesign is a year of nights,
honestly stated. fsh is the daily driver AND the demo. This intent is filed because it is the right
long-term shape, not because it is next.
NO BIG-BANG. If it starts, it starts behind the 171 entry point, one construct at a time, with the
old path live until the new one passes the same tests.
DO NOT SMUGGLE THIS INTO 171. If a change needs an AST, it belongs here.

## Success Criteria
- [ ] INT-171 is COMPLETE before this intent starts. Not "in progress" -- complete
- [ ] Gate zero, answered with evidence: name THREE things fsh cannot do because of its current
      parser, each a real thing Christian wanted and could not have. If they cannot be named,
      CANCEL THIS INTENT -- that is a legitimate and honest outcome
- [ ] If it proceeds: an RFC first (per the advisory) -- what problem, why this approach, what
      alternatives, what trade-offs, how it fits fshs philosophy. Written before code
- [ ] logos evaluated BEFORE chumsky, and the evaluation says what it measured
- [ ] chumsky only with an explicit AST commitment, written down
- [ ] fsh still boots, still logs in, still deploys, at every step
- [ ] Each gate carries evidence per INT-158

## The Rule
"An intent is a hypothesis. `Is this parser limiting?` is a question with a real answer -- and `no`
is one of them." 🌲
