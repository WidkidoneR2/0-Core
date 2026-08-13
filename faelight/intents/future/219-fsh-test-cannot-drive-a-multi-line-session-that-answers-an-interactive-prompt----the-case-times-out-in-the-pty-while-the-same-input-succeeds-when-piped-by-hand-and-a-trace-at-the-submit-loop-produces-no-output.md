---
id: 219
date: 2026-08-12
type: future
title: "fsh-test cannot drive a multi-line session that answers an interactive prompt -- the case times out in the pty while the same input succeeds when piped by hand, and a trace at the submit loop produces no output"
status: planned
tags: [fsh-test, harness, pty, int-196, int-197]
---

## Vision
A behaviour proven by hand can be asserted by the suite. Interactive behaviour that needs setup
before the line under test is expressible, rather than being the one shape the harness cannot reach.

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

## Evidence (measured 2026-08-13)
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
- [ ] G1: AN INSTRUMENT FIRES INSIDE THE SUBMIT LOOP. Until a trace at that site produces output,
      nothing else here is measurable -- this is the smaller, better-defined question and answering
      it makes the timeout answerable
- [ ] G2: WHY the earlier trace produced nothing is stated, not worked around. A site that is
      present in source, compiled, and silent is itself a finding
- [ ] G3: the timeout is LOCATED -- which wait_for call, on which line, waiting for what. Named with
      the marker it never saw rather than described as hanging
- [ ] G4: the cause is stated and distinguished: harness protocol, pty behaviour, or a genuine
      difference between piped stdin and a pty that the shell itself exhibits
- [ ] G5: PROVEN by the INT-197 gate-6 case going GREEN, written back as it was before removal --
      alias on one line, invocation on the next, answered
- [ ] G6: INT-196 M8 is revisited with the same door, and either closes or its deferral is renewed
      with a stated reason that is no longer this one
- [ ] G7: run_repl_answered_after loses its allow-dead-code, because it has a caller
- [ ] G8: the 151 existing cases stay green, and the timing is compared before and after -- a change
      to the shared session protocol touches every case in the suite
- [ ] G9: each gate carries evidence per INT-158

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
