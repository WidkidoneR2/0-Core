---
id: 216
date: 2026-08-09
type: future
title: "commands::tokenize reports no byte offsets, so any caller needing the remainder of a line must re-split raw text and lose the quoting"
status: planned
tags: [fsh, tokenizer, spans, aliases]
---

## Vision
A caller that needs part of a line asks the tokenizer WHERE that part is, and slices the original
text. Nothing re-splits raw text to find a boundary the tokenizer already crossed.

## The Problem
`commands::tokenize` is INT-171 gate 1, the single quote-aware tokenizer, and it returns
`Vec<String>` with the quotes STRIPPED. That is right for building argv. It is wrong for every
caller that needs the REST of a line, because the only two ways back are both lossy: re-split the
raw text and lose quote awareness, or rejoin the tokens and lose the quoting itself.

Both failure modes are demonstrated, not hypothetical. INT-196 found the alias expansion loop
deriving its name quote-aware and then taking the remainder with a raw split at the first space --
which lands INSIDE the quotes when the name contains one. Proven on gen 488: an alias named with a
space, invoked quoted with an argument, expanded to its body plus a fragment of its own name.

⚠️ AND THE OBVIOUS FIX IS WORSE, which is the real finding. Rebuilding the remainder from the
tokenizer and rejoining with spaces removes the fragment and SPLITS A QUOTED ARGUMENT IN TWO --
exactly what that implementation was written to replace, per its own doc. It was written, measured
and reverted. All seven alias regression cases stayed green through the bad version, so it was
discriminated by ARGUMENT COUNT rather than by eye: the deployed shell passed one argument, the
rejoin passed two.

## The Solution
Report spans. A token knows where it started and ended in the source; the tokenizer simply does not
say. With offsets, a caller takes the remainder as a SLICE of the original line, so the bytes the
user typed survive untouched -- no re-splitting, no rejoining, no quote loss.

⚠️ THIS IS A CHANGE TO A SHARED OWNER, which is why it is its own intent rather than a fix inside
INT-196. The tokenizer has callers beyond the alias loop, and adding a second quote-aware scanner
at any one of them to find an offset would be the duplicate-interpretation disease the whole spine
migration exists to end.

★ THE SPINE ALREADY HAS THE SHAPE. `spine/lexer.rs` spans everything, and `Spanned<T>` is how the
AST carries position. This is the legacy tokenizer learning what the scanner already knows -- not a
new idea, a missing one.

## ⏸ DEFERRED ON PURPOSE (2026-08-09) -- ARCHITECTURAL LIFETIME, NOT TRIAGE
This is a REAL defect and it is deliberately not being built. The reason is not that the shape is
rare, though it is. It is that `commands::tokenize` is LEGACY infrastructure with a planned
replacement, and this change adds capability to the retiring layer. The spine already carries the
span information -- `spine/lexer.rs` and `Spanned<T>` -- so fixing it here writes code that the
migration will delete.

⚠️ THE TEMPTING VERSION WAS ALREADY TRIED AND REVERTED, which is what makes the deferral concrete
rather than a shrug. See the evidence below: the rejoin removes the fragment and splits a quoted
argument in two.

⏭ WHEN ALIAS RESOLUTION MOVES ONTO THE SPINE, this becomes a REGRESSION TEST against the new
implementation -- not a reason to modify the old tokenizer. The reproduction in the evidence below
is the case to run.

⏭ REVISIT TRIGGER: alias expansion consuming parser-owned structure. Not before.

## Evidence (measured 2026-08-09, gen 488)
- `alias "zz sp"="echo SPACEOK"; "zz sp" tail` expands to `SPACEOK sp tail` -- the argument `sp`
  came from inside the alias NAME. Proven on the deployed shell.
- The rejoin fix was written and reverted: `alias zzp="printf %s|"; zzp "a b"` passed ONE argument
  on the deployed shell and TWO with the rejoin. The seven `repl_193` alias cases stayed green
  through it, which is why an argument-count probe was needed rather than reading output.
- `commands::tokenize` (commands/mod.rs) returns `Vec<String>`, quotes stripped, no positions.
- The spine already spans everything: `spine/lexer.rs` plus `Spanned<T>`.

## Non-goals
- Replacing `tokenize` with the spine lexer. That is INT-169 territory and a different question.
- Changing what a token IS. Quote stripping stays correct for argv; this adds WHERE, not what.
- Fixing every caller. The alias loop is the demonstrated one; others migrate on evidence.

## Success Criteria
- [ ] G1 RED-FIRST, and it exists as a reproduction before any signature changes: an alias whose
      NAME contains a space, invoked quoted with an argument, expands to its body plus a fragment
      of that name. Asserted through the REPL door, since alias expansion only runs there
- [ ] G2: the tokenizer reports positions. Shape ruled BEFORE writing -- a parallel `Vec<Span>`, a
      token struct, or a second function -- with the reason stated, because the existing return
      type has callers and churning them is a cost this intent must justify
- [ ] G3: EVERY existing caller of `tokenize` is enumerated with file:line and each one either
      keeps the old shape or moves, with a stated reason. Taken fresh, not inherited
- [ ] G4: the alias loop takes its remainder as a SLICE of the original line using the reported
      offset. No re-split, no rejoin
- [ ] G5: G1 goes green, and the argument-count probe still passes ONE argument for a quoted
      argument -- both halves, because the rejoin fix satisfied one and broke the other
- [ ] G6: the seven `repl_193` alias cases stay green, and the count probe is added as a case so
      the next attempt cannot pass them while splitting a quoted argument
- [ ] G7: GHOST-CHECKED -- G1 red under a revert of the span change, green on restore
- [ ] G8: INT-196 site 4 is revisited and its exception comment either removed or updated, since
      this intent is the reason it was recorded rather than fixed
- [ ] G9: each gate carries evidence per INT-158

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
