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
- [x] G1 RED FIRST: the loss is reproduced by a case, not only by hand
<!-- evidence: repl_220_a_stderr_redirect_survives_a_pipeline, 155/155. It writes to stderr inside a
     pipeline with a redirect, then cats the target back. Before the fix the file did not exist.
     A FAST reproduction came first: a one-line sh writing both streams, measured in 251ms rather
     than the 150s the original fsh-test reproduction took. That is what made every later probe
     cheap enough to iterate on. -->
- [x] G2: WHERE the redirect is lost is LOCATED by measurement, not by reading
<!-- evidence: commands/mod.rs:8861, spawn_pipeline: `cmd.stderr(Stdio::inherit())`, unconditional.
     Narrowed by measurement at each step rather than by reading ahead. Four shapes showed 1> in a
     pipeline works and 2> does not, on EITHER side, so it was not about position. Spine on and off
     both lost it, so it was not the router. The trace showed the spine CLAIMS both the single and
     the piped form, so it was not a refusal. Lowering was ruled out by reading only after the
     measurement pointed there: plan.rs:725-739 converts a redirect into IoPlan::Files regardless of
     the IoPlan handed in, so the Simple passed at line 565 is correct and each stage plan carries
     its own Files. That left the executor, which reads stdin at 8796 and stdout at 8826 from the
     plan and never reads stderr at all. -->
- [x] G3: the fix keeps the pipeline working
<!-- evidence: six shapes, all correct after: single 2>, single 1> 2>&1, pipe 2> on the left, pipe
     1> on the left, pipe 1> 2>&1, and pipe 2> on the right. Pipe output still flows in every case.
     THE ONE OWNER RULE HELD: the stderr logic was EXTRACTED from configure_file_io into
     open_stderr_sink as its own commit, verified pure by the four shapes being byte-identical
     before and after, and the pipeline then CALLS it. No second interpretation of StderrTarget.
     The dup needed the stdout handle KEPT rather than dropped into the child, because sharing means
     a clone -- two opens give two write offsets and the streams overwrite each other silently.
     RECORDED: pipe 1> 2>&1 interleaves as ZZERR then ZZOUT while the single-command form gives
     ZZOUT then ZZERR. Both correct, both in one file; the order differs because a pipeline stage
     writes stdout through the cloned handle. -->
- [x] G4: the same shape is checked for stdout
<!-- evidence: stdout was ALREADY correct and is recorded as measured rather than fixed. spawn_pipeline
     read stdout from the plan at 8826 all along, which is exactly why 1> in a pipeline worked
     throughout and made the asymmetry the first real clue. -->
- [x] G5: the suite stays green and a case joins it
<!-- evidence: 155/155 with the new case. 207 unit tests, no warnings. -->
- [x] G6: each gate carries evidence per INT-158
<!-- evidence: every gate above. Worth recording the method rather than only the result: SIX
     measurements narrowed this before a line was changed, and two of my predictions were wrong
     along the way -- I expected the pipeline lowering to drop the redirect by passing Simple, and
     the read showed Simple is precisely what ALLOWS the conversion. The measurement was right each
     time and the reading-ahead was not. -->


<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
