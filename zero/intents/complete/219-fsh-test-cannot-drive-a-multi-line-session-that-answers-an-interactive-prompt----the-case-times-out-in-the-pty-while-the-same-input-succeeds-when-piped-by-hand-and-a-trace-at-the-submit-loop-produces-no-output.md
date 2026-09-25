---
id: 219
date: 2026-08-12
type: future
title: "fsh-test cannot drive a multi-line session that answers an interactive prompt -- the case times out in the pty while the same input succeeds when piped by hand, and a trace at the submit loop produces no output"
status: complete
tags: [fsh-test, harness, pty, int-196, int-197]
---

## CORRECTED 2026-08-12, SAME DAY IT WAS FILED. THE PREMISE WAS WRONG.

THE HARNESS WORKS. run_repl_answered_after drives a two-line answered session in 999ms, and the
INT-197 gate-6 case it was built for is green and ghost-checked. Every symptom this intent was filed
about came from ONE cause, and it was mine.

PIPING A LONG-RUNNING SUITE THROUGH head OR grep KILLS IT PARTWAY. Closing the pipe gives fsh-test a
broken-pipe panic, so the run stops early -- and a killed run is INDISTINGUISHABLE from a completed
one that found nothing.

That single mistake produced FOUR wrong conclusions in one session: a case that appeared to time
out, a trace that appeared to print nothing, log files that appeared not to exist, and a ghost-check
that appeared not to discriminate. Redirecting to a file resolved all four at once.

THE INTENT IS REWRITTEN RATHER THAN DELETED because the finding underneath is real and cost an hour.
The original text is kept below per INT-027.

## Vision
A measurement of the suite cannot silently truncate. Either the harness survives a closed pipe, or
it says loudly that it was cut short -- because the failure mode is not a wrong answer, it is a
CONFIDENT answer from an incomplete run.

## The Problem
TWO GATES IN TWO INTENTS ARE BLOCKED ON THIS, and in both the shell is correct and the harness
cannot say so.

INT-196 M8 needs several lines and THEN an answer: a dangerous command with an unterminated quote,
completed on a second line, must still reach the guard. It does -- proven by hand, the assembled
buffer produces a challenge and blocks. It was formally deferred because the harness could not
drive it.

INT-197 gate 6 needs an alias defined on one line and invoked on the next. It gates correctly when
piped to the shell by hand. Through the pty it times out at the wait_for boundary, 20.9 seconds.

THE UNDERLYING LIMIT WAS ALREADY FIXED, and the case still fails. run_session applied the supplied
answer to EVERY submitted line, which is why the answered door took exactly one command and said so.
It now applies to the LAST line only, and run_repl_answered_after exists to use that -- carrying an
allow-dead-code suppression and a provenance comment, because its only case had to be removed.

⚠️ THE CAUSE IS NOT LOCATED, and that is the honest state. A trace at the submit loop produced NO
output at all, despite the site being present in source and the binary rebuilt. Three investigative
cycles were spent before stopping deliberately rather than drilling further.

## The Solution
Find out why the trace does not print, because that is a smaller and better-defined question than
why the case times out, and answering it makes the second answerable.

⚠️ DO NOT OPEN BY ADJUSTING THE CASE. Two versions of the INT-197 case were already changed to suit
the harness -- one used a semicolon and was testing the documented compound-line limitation instead
of the alias fix, and the corrected two-line version still timed out. A third adjustment would be
fitting the test to the tool rather than fixing the tool.

## Evidence (measured 2026-08-12)
- INT-197 gate 6: `alias zzq197=mkfs.zzz` then `zzq197` gates correctly when piped by hand --
  alias confirmation, CHALLENGE, blocked. Through the pty the same two lines TIME OUT at 20.9s.
- A trace at the submit loop printed NOTHING under FSH_TEST_TRACE, with the site present in source
  (repl.rs, the per-line loop) and the binary rebuilt. Verified by grep and by an unfiltered run.
- INT-196 M8 is formally deferred for the same reason and its property is likewise proven by hand.
- run_repl_answered_after exists, unused, with allow-dead-code and a provenance comment.
- run_session now applies the answer to the LAST line rather than every line -- that fix is already
  in and is NOT the blocker.

## Non-goals
- Rewriting the harness. It drives 151 cases correctly and its session protocol has one owner.
- Adjusting the blocked cases to suit the tool. Two versions of the INT-197 case were already
  changed once each; a third would be fitting the test to the harness.
- Native heredoc execution, or anything about the shell. The shell is correct in both blocked cases.

## Success Criteria
- [x] G1: A TRUNCATED RUN IS DISTINGUISHABLE FROM A COMPLETE ONE
<!-- evidence: reproduced FIRST, before any change. Piped to head, the suite printed a banner and
     two passing cases, never printed its Results line, and the shell reported 0 -- a confident
     success from a run that stopped after two of 154 cases. Two further facts fell out: the EPIPE
     panic reached the terminal but no redirect target, which became INT-220, and the pipeline
     status belongs to the FILTER, which is correct POSIX and is why the exit code alone could
     never have carried this. -->
- [x] G2: BOTH MECHANISMS, WITH A DIVISION OF RESPONSIBILITY
<!-- evidence: one panic hook in main, chained rather than replacing the prior hook so ordinary
     panics still print normally. It matches TWO substrings rather than the whole payload, because
     the exact wording is a std detail that moves between Rust versions. 200 print sites untouched.
     The stderr marker is the observability half and it is what survives a pipeline, since the
     status there belongs to the filter. -->
- [x] G2b: the exit status is chosen from this tool own vocabulary and DOCUMENTED
<!-- evidence: fsh-test uses 0 for pass and 1 for fail; the 3, 143 and 2 elsewhere in the file are
     expectations ABOUT THE SHELL under test, not this tool own. So 2 was unclaimed and is now
     truncation. NOT 141, which would claim death by SIGPIPE -- Rust ignores that signal, so what
     actually happens is EPIPE caught and a deliberate stop. Distinct and testable beats
     conventional. -->
- [x] G3: PROVEN across FOUR shapes
<!-- evidence: measured through bash, since INT-220 means fsh cannot route stderr in a pipeline, and
     read with PIPESTATUS so the suite own status is visible rather than the filter.
     file redirect -> exit 0, Results 154/154, stderr empty.
     piped to grep -> status 0, Results line still captured, stderr empty.
     piped to head -> status 2, stderr carries the TRUNCATED marker, no Results line.
     The fourth shape, a genuinely failing complete run, is covered by every red case this session:
     it prints Results and exits 1, which is neither 0 nor 2.
     ⚠️ AND A CLAIM WAS CORRECTED BY MEASUREMENT. This was first recorded as protecting fsh only
     partially, inferred from INT-220 rather than tested. Measured through an fsh pipeline with no
     stderr redirect, the TRUNCATED marker ARRIVES. The gap is only 2>file INSIDE a pipeline, which
     is INT-220 and narrower than the note first claimed. -->
- [x] G4: the cases stay green and the timing is unchanged
<!-- evidence: 154/154 in the file-redirect shape. Three full runs at 447s total, 149s each, which
     is the same per-run time measured before the hook. A hook that only fires on a broken pipe
     costs nothing on the normal path. -->
- [x] G5: each gate carries evidence per INT-158
<!-- evidence: every gate above. Worth recording that this intent was REWRITTEN the day it was
     filed, because its original premise -- that the harness could not drive a multi-line answered
     session -- was disproven within the hour. The harness was never broken. The real defect is the
     one measured here, and the original text and its nine gates are kept below per INT-027. -->


## SUPERSEDED GATES, kept per INT-027. These asked about a harness fault that does not exist.
<!--
  (superseded) G1: AN INSTRUMENT FIRES INSIDE THE SUBMIT LOOP. Until a trace at that site produces output,
      nothing else here is measurable -- this is the smaller, better-defined question and answering
      it makes the timeout answerable
  (superseded) G2: WHY the earlier trace produced nothing is stated, not worked around. A site that is
      present in source, compiled, and silent is itself a finding
  (superseded) G3: the timeout is LOCATED -- which wait_for call, on which line, waiting for what. Named with
      the marker it never saw rather than described as hanging
  (superseded) G4: the cause is stated and distinguished: harness protocol, pty behaviour, or a genuine
      difference between piped stdin and a pty that the shell itself exhibits
  (superseded) G5: PROVEN by the INT-197 gate-6 case going GREEN, written back as it was before removal --
      alias on one line, invocation on the next, answered
  (superseded) G6: INT-196 M8 is revisited with the same door, and either closes or its deferral is renewed
      with a stated reason that is no longer this one
  (superseded) G7: run_repl_answered_after loses its allow-dead-code, because it has a caller
  (superseded) G8: the 151 existing cases stay green, and the timing is compared before and after -- a change
      to the shared session protocol touches every case in the suite
  (superseded) G9: each gate carries evidence per INT-158
-->

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
