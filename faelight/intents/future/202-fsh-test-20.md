---
id: 202
date: 2026-08-05
type: future
title: "Fsh-Test 2.0"
status: planned
tags: [faelight, fsh, fsh-test, nix]
---
## Vision
A suite whose number means something. fsh-test decides when the shell is safe to deploy, and in
October it is the evidence behind every claim made about this shell. Today it prints a number that
moves between runs on identical code.

## The Problem
Three faults, all measured 2026-08-05, all in `faelight/rust-tools/fsh-test/src/repl.rs`.

FAULT 1 -- THE CAPTURE TAKES A TAIL, NOT A WINDOW. `find_last(raw, b"\x1b[?2004l")` returns
everything after the LAST bracketed-paste-off marker, which includes the next prompt redraw. Three
conformance cases recorded fsh's "output" as the prompt itself -- powerline glyphs and all -- plus
`x   exited 1 -- general error`, and compared that against bash's empty string. They are not
language divergences. The comparison is eating the shell's own chrome.

FAULT 2 -- IT SLEEPS INSTEAD OF WAITING. The budget is `sleep(2500ms)` to settle, then per line
`sleep(1200ms)`, `drain(500ms)`, `sleep(300ms)`. Every number is a guess about how fast the machine
is. Run the suite while cargo is compiling and the guesses stop holding. ~30 REPL cases at ~4.9-5.3s
each is nearly the whole 298s runtime, and nearly all of it is spent asleep.

FAULT 3 -- NO CASE CAN CHOOSE ITS ROUTING, AND IT IS THE WORST ONE. `Command::new(fsh_bin())` spawns
with no `.env()`, so every REPL case inherits the harness's environment and runs SPINE-ROUTED.
`repl_background_job_keeps_quoted_arguments` therefore exercises the spine, which handles quoting
correctly, and never reaches the legacy path that was mangling it. It passed for months while the
bug was live. The bug was found by hand and fixed at `d0c04825`. The test named for it never saw it.

## THE MEASUREMENT THAT MADE THIS INTENT NECESSARY
Two runs, minutes apart, on binaries differing only in a function unreachable by either:
  run A  target/debug (with d0c04825)   131/132   failed: repl_background_job_keeps_quoted_arguments
  run B  deployed gen 464 (without it)  129/132   failed: three conform_* cases
  run C  target/debug again             132/132   nothing failed
Disjoint failure sets, and the build WITH the change did better than the build without. The stored
history in `fsh_test_results` says the same: `repl_background_job_honours_its_redirect` failed four
times at `a0c60409`, three at `827d76a6`, plus `dd19eb9c` and `a70e696c`, and passed in between on
those same hashes.

## THE PERFORMANCE FLOOR, MEASURED -- 0.60s
Five minutes is too long to run often, and a suite nobody runs protects nothing. So the target is
not aspirational, it is arithmetic on a measured floor.

A pty probe spawned fsh and waited for the bracketed-paste-ON marker -- the line editor announcing
it is ready -- four times: 0.594, 0.624, 0.610, 0.607 seconds, on ~2KB of banner. THE 30ms SPREAD
MATTERS MORE THAN THE NUMBER. Waiting for the marker is not merely faster than sleeping 2500ms, it
is DETERMINISTIC, which is why gate 2 fixes the flake and not just the clock.

Two suspects were ruled out rather than assumed. `nixos-rebuild list-generations --json`, which the
banner runs on every start, takes 120ms. `faelight-shell -c 'true'` is ~0.00s -- it skips the
banner, the config load and the session bookkeeping entirely, which is the two-doors difference and
not a measurement of anything the harness waits for. 0.60s to a prompt is fine for a user; there is
no startup problem here.

  today   2500 settle + per line (1200 + 500 + 300) + ~400 drain  ~= 4.9s per single-line case,
          which matches every observed timing, and ~40 REPL cases is essentially the whole 298s
  after   0.60 settle + ~0.2 per line + ~0.1                       ~= 1.0s per case  ->  ~40s total

AND ISOLATION SURVIVES. Sharing one pty across cases was the old idea for dodging the per-case
settle; it leaks `cd` and variables between tests. At 0.6s it is unnecessary. Do not trade isolation
for speed that marker-waiting already gives.

## THE DESIGN QUESTION THIS INTENT EXISTS TO ANSWER
    What has to be true before a green suite is allowed to mean the shell is correct?

## The Solution -- determinism first, coverage second, automation last
The order is not preference, it is dependency. A suite that flakes cannot be trusted, so the capture
and the timing come first. A suite that only ever knocks on one door cannot be complete, so routing
comes second. And a suite nobody runs protects nothing -- but automating a 298-second flaky suite
would make every commit slower and every red meaningless, so automation comes LAST, after the first
two have made it fast and deterministic.

## Success Criteria
- [ ] The capture is a bounded window, not a tail: text between the paste-off that follows the
      submitted line and the NEXT paste-on. The three `conform_*` cases pass WITHOUT their
      assertions being changed -- if an assertion has to move, the capture fix is wrong.
- [ ] No fixed sleep remains in the per-line path. The harness waits for a marker with a deadline
      and reports a timeout as a FAILURE, not as empty output.
- [ ] THE SUITE COMPLETES IN UNDER 60 SECONDS, from 298s, with runtime recorded before and after.
      Isolation is not traded away to get there -- one pty per case, as today.
- [ ] A case can declare the environment its shell runs under, and legacy-routed twins exist for the
      background cases.
- [ ] The suite runs without being typed -- a pre-commit hook or a `nix flake check` target -- and a
      red run blocks.

## The reds, named in advance (INT-158: watch it FAIL first)
- Gate 1 is already red and recorded above: the three conform captures contain prompt text.
- Gate 2's red is reproducible by running the suite under load -- start a `cargo build` alongside it.
- Gate 3 has the best red in the project and it needs no special build. The gen-464 binary predates
  `d0c04825`; a `FSH_SPINE=0` twin of `repl_background_job_keeps_quoted_arguments` must FAIL against
  it and PASS against the current one. That proves the new test detects the bug the old test was
  blind to.
- Gate 4's red is a deliberately broken commit that the hook must refuse.

## Scope guardrails
- This is NOT a rewrite. Three focused changes in `repl.rs` plus new cases. "2.0" names the version,
  not a new harness.
- Do NOT weaken an assertion to make a case pass. A capture fix that requires the assertion to move
  has fixed the wrong thing.
- Do NOT delete a flaky test to make the number green. The flake is the bug.
- Do NOT add a dependency without discussing it first.

## THE CONFORMANCE RULING -- ONE CORPUS, ONE VERDICT MODEL, TWO DOORS
This looked like duplication and is not. `spine conform` compares through `-c`, on stdout and exit
status only, with three verdicts -- Agrees, DivergesAsDeclared, Unexplained -- where a declared
divergence that starts MATCHING bash again counts as a failure. fsh-test's `conform_*` cases compare
through the REPL pty. Different doors.

AND THE DIFFERENCE IS LOAD-BEARING. `spine conform`'s first run caught `-c` creating `0.5` and `=`
while the REPL refuses them -- the digit guard applying at one door and not the other. Deleting
either tool would have hidden that.

So: the three-verdict model becomes the ONLY verdict model. fsh-test stops carrying a private case
list and drives the SHARED corpus through the pty door. A case that agrees on one door and diverges
on the other then becomes a finding BY CONSTRUCTION rather than something someone has to remember to
check -- which is exactly INT-201's open gate, "the digit guard applies to both doors."

⚠️ ORDERING: the shared corpus depends on gate 1. `spine conform` reads stdout from a pipe and never
sees prompt chrome; the pty door does. The bounded capture window is the precondition for trusting
the REPL side of a shared corpus, not a parallel task.

## Relationship
- INT-173 is COMPLETE and is what made this possible: it gave fsh-test a REPL door instead of only
  `-c`. This intent fixes how that door LISTENS. Not a reopen.
- INT-201 supplied the motivating case: `d0c04825` fixed a defect the suite could not see.
- INT-157 (VM-based testing) owns the hidden-dependency and minimal-environment job. Gate 4 is a
  hook or a flake check, and must not quietly become 157.

## The Rule
"A suite whose number moves on identical code is not evidence. It is weather."
