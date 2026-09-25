---
id: 215
date: 2026-08-09
type: future
title: "fpatch's safe abort and internal error are typographically identical, so a correct refusal still reads as a crash at a glance"
status: complete
tags: [fpatch, tooling, errors]
---

## Vision
A refusal is recognisable as a refusal without reading it, and a patch shows what it will write, not
only what it will delete.

## The Problem
TWO PROBLEMS IN ONE TOOL, and recon corrected the first one.

⚠️ THE TITLE IS HALF WRONG, and the half that survives is the interesting half. `_refuse` and
`_internal` are already well distinguished in TEXT: different headers, different Status lines,
different exit codes (1 and 2), and the internal presenter is honest that the file state is unknown
where the refusal can promise nothing was written. INT-199 did that work.

What is still identical is the SHAPE: the same 66-character bar, the same section layout, the same
all-caps header in the same position. You have to READ to tell them apart, which is exactly the
at-a-glance failure the title names. Two correct safe-aborts were read as crashes on 2026-08-13
during INT-203, by the author of this intent.

SECOND, AND HE ASKED FOR IT DIRECTLY: the diff shows removal only. Both patch functions print the
lines being replaced with a doubled-angle marker and print NOTHING about what replaces them. Every
patch is accepted on trust that the replacement text is what was intended -- and INT-203 proved that
replacement text can be corrupted in transit without either side noticing.

## The Solution
COLOUR IS THE DIFFERENTIATOR, which folds both problems into one change. Red for a refusal, yellow
for an internal error, and red/green for removed and added lines the way a git diff reads.

The added lines cost nothing to obtain: `new_lines` and `new` are already in scope at both print
sites. No signature changes.

⚠️ OUTPUT LANDS IN THREE PLACES: a terminal, a paste buffer, and an assistant context window. Colour
helps the first and clutters the other two. So it is gated per-stream on isatty -- the diff goes to
stdout and the errors to stderr, and those are separately redirectable -- with an environment
override for the cases isatty gets wrong.

## Evidence (measured 2026-08-13, before any change)
- fpatch.py is 293 lines. `_refuse` at 49 exits 1; `_internal` at 80 exits 2; `_guard` at 119 routes
  an unexpected exception to the internal presenter so it cannot escape as a bare traceback.
- The two presenters ALREADY differ in text and exit code. What they share is the 66-char bar, the
  section layout, and the header position -- the shape, not the words.
- The diff prints at 195-198 (patch_between) and 262-265 (patch). Both print `lines[i]` only.
  `new_lines` and `new` are in scope at both sites, so added lines need no signature change.
- No colour anywhere in the file today.

## Non-goals
- Rewriting the presenters. INT-199 established what they SAY and that part is right.
- A colour theme or a palette module. Four codes and a reset, defined once in this file.
- Colouring anything else in the tree. This intent owns fpatch output only.

## Success Criteria
- [x] G1: a refusal and an internal error are distinguishable AT A GLANCE, without reading the text.
      Demonstrated side by side, both forced, in a terminal
<!-- evidence: forced side by side in a terminal. The refusal bar and header are RED, the
     internal-error bar and header are YELLOW, exit 1 and exit 2. Distinguishable without reading
     the words, which is what the title asked for.
     THE TITLE WAS HALF WRONG and recon corrected it: the two already differed in TEXT and in exit
     code, which INT-199 did. What they shared was the SHAPE -- same bar, same layout, same header
     position. -->
- [x] G2: the diff shows REMOVED and ADDED lines, red and green, the way a git diff reads. Added
      lines come from the replacement text already in scope -- no signature change
<!-- evidence: both print sites show removed lines with a red doubled-angle marker and added lines
     with a green double-plus. No signature change; new_lines and new were already in scope.
     Demonstrated on a three-line span replaced by two, and on a single-line patch.
     RECORDED AT THE SITE: the patch display is the FIRST match only. Pre-existing, because first
     comes from s.index(old), so with count>1 the other sites are replaced but not shown. -->
- [x] G3: COLOUR IS GATED PER-STREAM on isatty, because the diff goes to stdout and errors go to
      stderr and those are separately redirectable
<!-- evidence: captured through a pipe the output carries NO escape bytes, verified by comparing raw
     bytes rather than rendered text. In a terminal the same commands are coloured. -->
- [x] G4: an environment override forces colour on or off for the cases isatty gets wrong, and its
      name and values are stated here rather than discovered in the source
<!-- evidence: FPATCH_COLOR. Unset means isatty decides. Set to 0, no, off, false or empty forces
     colour OFF; any other value forces it ON. Proven both directions through a pipe. -->
- [x] G5: EVERY EXISTING CALLER STILL WORKS -- the refusal paths, both patch functions, and the
      version that raises under FPATCH_DEBUG
<!-- evidence: patch_between and patch exercised in one run, a three-line span replaced by two then
     a single-line patch on the result, both writing correctly. FPATCH_DEBUG=1 still RAISES
     PatchRefused rather than exiting. Refusal paths still exit 1, internal errors exit 2. -->
- [x] G6 CORRECTED BEFORE TICKING: the gate as written conflicts with G2. Byte-identical output and
      "show added lines" cannot both hold, because added lines ARE new output. The honest scope is
      that the two ERROR PRESENTERS are byte-identical with colour off, while the DIFF gains added
      lines deliberately
<!-- evidence: a baseline was captured BEFORE any change -- 168 bytes stdout at exit 0 for a
     successful patch, 836 bytes stderr at exit 1 for a refusal. After the presenter changes both
     compared SAME byte for byte with colour off. The diff differs by design, which is G2. -->
- [x] G7: each gate carries evidence per INT-158
<!-- evidence: every gate above carries a comment naming what was measured. Two things beyond the
     gates are worth recording.
     THE TITLE WAS HALF WRONG, found by recon rather than assumed. The presenters already differed
     in text and exit code; only the SHAPE was shared. The fix is therefore colour rather than more
     words, which is also what makes his diff request the same change rather than a second one.
     AND THE GATES WERE MANGLED WHILE BEING TICKED. A helper anchored on the fragment "- [ ] G1:"
     rather than the whole gate line, so each replacement landed mid-line and stranded the rest of
     the text after the comment close. Five gates, all the same way. fpatch did exactly what it was
     asked -- one match, one replacement -- and the anchor was mine. The block was rewritten whole
     by index rather than repaired with more anchors on damaged text.
     THE RULE: an anchor must be a COMPLETE line, never a prefix of one. -->

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
