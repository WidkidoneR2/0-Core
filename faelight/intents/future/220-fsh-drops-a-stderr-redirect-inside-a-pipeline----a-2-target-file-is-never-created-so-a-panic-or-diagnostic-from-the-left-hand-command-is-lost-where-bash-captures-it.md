---
id: 220
date: 2026-08-13
type: future
title: "fsh drops a stderr redirect inside a pipeline -- a 2> target file is never created, so a panic or diagnostic from the left-hand command is lost, where bash captures it"
status: planned
tags: [fsh, pipeline, redirect, stderr, int-219]
---

## Vision
A diagnostic from the left-hand side of a pipeline reaches the file it was redirected to. What a
user writes as a redirect is honoured wherever it appears.

## The Problem
MEASURED 2026-08-13, side by side, same command, same binary:

    fsh-test 2>/tmp/x | head -5

Under bash: /tmp/x exists, 250 bytes, holding the panic from the left-hand command.
Under fsh:  /tmp/x IS NEVER CREATED. The panic appears on the terminal instead.

So the redirect is dropped and the stream falls through to the tty. Interactively that looks
harmless -- the text is still on screen -- which is exactly why it went unnoticed. In a script, a
captured log, or any non-interactive use, the diagnostic is simply gone.

⚠️ THE PIPELINE STATUS IS NOT THE DEFECT. Both shells report 0 there, because a pipeline reports its
LAST command and that is POSIX behaviour. Only the redirect differs.

## FOUND WHILE DOING SOMETHING ELSE
INT-219 needs a truncated test run to announce itself on stderr, and the first question was whether
stderr survives. It does under bash and under a plain redirect; it does not through fsh in a
pipeline. That makes this a PREREQUISITE for trusting stderr as a channel in fsh, and it was
separated into its own intent rather than folded into that work.

## The Solution
Recon first, and do not assume where the loss happens. Candidates, in the order they can be ruled
out by measurement rather than by reading:
  - the spine lowers the pipeline and drops the fd plan for the left-hand command
  - the pipeline is delegated whole to sh with the redirect already consumed
  - detect_redirect claims the redirect for the WHOLE line rather than the segment it belongs to

⚠️ RELATED SHAPE, NOT THE SAME BUG: INT-172 was a dropped redirect too -- a line sliced at the
operator, discarding everything to its right including a pipe. That one is fixed and its fixture is
in the deadwood checker. This is the mirror image: the pipe survives and the redirect is lost.

## Evidence (measured 2026-08-13)
- Same command, same binary, both shells, run from python so nothing interpolates:
  `fsh-test 2>/tmp/x | head -5`. Bash created /tmp/x at 250 bytes holding the panic. fsh created
  nothing and the panic went to the terminal.
- Both reported pipeline status 0, which is correct in both and is NOT part of this defect.
- The left-hand command was a real one that genuinely writes to stderr, so the absence is a dropped
  redirect rather than an absent stream.

## Success Criteria
- [ ] G1 RED FIRST: the loss is reproduced by a case, not only by hand -- a command that writes to
      stderr, redirected, inside a pipeline, asserting the target file exists and holds the text
- [ ] G2: WHERE the redirect is lost is LOCATED by measurement, not by reading. Named as a file and
      line, with what was probed and what it reported
- [ ] G3: the fix keeps the pipeline working. A redirect that vanishes and a pipeline that breaks are
      not a trade -- both must hold, with a case for each
- [ ] G4: the same shape is checked for stdout, since a dropped stderr redirect suggests the question
      was never asked for either stream. Whatever is found is recorded even if it is already correct
- [ ] G5: the fsh-test suite stays green, and a case for this shape joins it so the behaviour cannot
      regress silently
- [ ] G6: each gate carries evidence per INT-158

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
