---
id: 202
date: 2026-08-05
type: future
title: "Fsh-Test 2.0"
status: in-progress
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
- [x] The capture is a bounded window, not a tail: text between the paste-off that follows the
      submitted line and the `133;A` prompt-start marker after it. The three `conform_*` cases pass
      WITHOUT their assertions being changed -- if an assertion has to move, the capture fix is wrong.
<!-- evidence: 7b1fab15. THE ORIGINAL WORDING SAID "the NEXT paste-on" AND THAT BOUNDARY DOES NOT
     WORK: the prompt TEXT is printed before `?2004h`, so ending there still swallows it. `133;A` is
     where the prompt begins, so the prompt is excluded structurally rather than by timing.
     ⚠️ THE OBVIOUS BOUNDARY WAS TRIED FIRST AND BROKE 16 TESTS. OSC 133;C means "output starts
     here", so a C..D window looks correct. For any path that spawns a child -- pipeline, `;`
     sequence, sh, bash, python3 -- the child inherits the pty and writes BEFORE fsh emits anything,
     so B, C and D arrive afterwards in a cluster and C..D is empty. Measured: `echo ZZBUILTIN` puts
     its output between C and D; `echo hi | grep h` puts `hi` before B. All 16 failures ran an
     external child. The probe that justified the first attempt used only builtins -- a probe that
     exercises the shape that works proves nothing about the shape that does not.
     ★ That fsh emits `133;C` after the command has run is a REAL shell-integration defect, and it is
     deliberately NOT fixed here: a harness that only works against a shell we are simultaneously
     changing is worth less than one that works against the shell as it is. -->
- [x] No fixed sleep remains in the per-line path. The harness waits for a marker with a deadline
      and reports a timeout as a FAILURE, not as empty output.
<!-- evidence: b6f57e41. `wait_for(rx, &mut acc, needle, from, limit)` waits on `\x1b[?2004h` -- the
     line editor announcing it is ready -- which is exactly what every sleep approximated. The old
     budget was 2500ms to settle, 1200ms a line, 500 to drain. The first prompt was measured four
     times at 0.594/0.624/0.610/0.607s: a 30ms spread, so the wait is deterministic where a sleep
     cannot be. The determinism is the point and the speed is a side effect. `drain()` is deleted.
     A timeout returns Err naming the command it gave up on. -->
- [x] THE SUITE COMPLETES IN UNDER 60 SECONDS, from 298s, with runtime recorded before and after.
      Isolation is not traded away to get there -- one pty per case, as today.
<!-- evidence: b6f57e41. 299s -> 167 -> 83 -> 62.7 -> 50.6s, 132/132 throughout, measured against the
     DEPLOYED release shell because that is the binary the suite runs by default.
     ★ THE PREDICTION WAS WRONG BY FOUR AND THE MEASUREMENT CORRECTED IT. Removing the sleeps gave
     167s, not the 40 predicted. Phase timing found why: the prompt redraw costs 0.000s, and the
     dominant phase is write-to-submit at 38-42ms PER CHARACTER, because the highlighter repaints the
     whole line on every byte. So the line is PASTED, not typed: the same 37-character line takes
     1.392s to submit written raw and 0.040s inside a bracketed paste, flat with length. Then the
     300ms teardown became a try_wait poll. No test cared that input arrived pasted.
     ⚠️ SHARED PTY SESSIONS WERE AVAILABLE AND REFUSED. They would have won the seconds instantly and
     cost isolation -- `cd` and variables leaking between cases. Still one pty per case. -->
- [x] A case can declare the environment its shell runs under, and legacy-routed twins exist for the
      background cases.
<!-- evidence: 8d491b5a. `run_repl_lines_env(cmds, env)`; `run_repl` and `run_repl_lines` delegate to
     it, so ~40 call sites did not churn to express something almost none of them need. The spawn
     gained `.envs(...)`; before it, EVERY case inherited the runner's environment and therefore ran
     spine-routed, which is why the legacy path had no coverage at all.
     ★ THE RED WAS RUN, NOT DESCRIBED. Gen 464's binary predates d0c04825 and survived GC:
       FSH_BIN=/nix/store/86m8mhwx52s1ris35jp0v4b7kmffzyv7-faelight-forest-9.2.0/bin/faelight-shell
     Against it: 134 cases, ONE failure -- repl_background_job_keeps_quoted_arguments_legacy -- while
     its spine-routed sibling PASSED. Against gen 465: 134/134 in 55.2s.
     ★★ THE SIBLING PASSING ON BOTH BINARIES IS THE FINDING. It carries the name of the bug and could
     never have detected it; the bug was found by hand instead. One screen, two doors, a shell
     disagreeing with itself.
     ⚠️ The second twin (repl_background_redirect_refused_on_legacy) asserts a REFUSAL on purpose: the
     legacy background path applies no redirect at all, and teaching it to would mean a second copy
     of configure_file_io. a33d6cd7 made it refuse; this case stops that becoming a file named with a
     trailing ampersand again. -->
- [ ] The suite runs without being typed -- a pre-commit hook or a `nix flake check` target -- and a
      red run blocks.
- [ ] ONE conformance corpus, ONE verdict model, TWO doors. fsh-test stops carrying a private
      `conform_*` list and drives the shared corpus; the three-verdict model (Agrees,
      DivergesAsDeclared, Unexplained) is the only one; and a case that agrees on one door while
      diverging on the other is reported as a FINDING rather than a pass or a failure.

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
- MORE TESTS IS NOT THE GOAL. The suite stays under the speed gate, and EVERY case must be able to
  FAIL. A case that cannot distinguish fixed from broken is worse than absent -- there are two
  precedents already: the REPL ghost test that was deleted, and repl_background_job_keeps_quoted_
  arguments passing for months while the bug it is named for was live.
- If added coverage puts the suite over sixty seconds, the lever is PARALLELISM, not shared sessions.
  Every case already owns its own pty and its own process, so threads preserve isolation completely.
  And if the target becomes genuinely unreachable, AMEND the gate with the measurement -- a gate that
  is quietly relaxed is the failure this intent exists to prevent.

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

## The coverage to add, and the bug each one would have caught
Six classes, ranked by evidence rather than by category. The first, third and fifth together ARE the
sixth gate; the rest are coverage the harness cannot express today.

1. EXIT STATUS. `133;D;<n>` already carries the code and the capture already parses that region, yet
   every case is stdout-only -- there is no exit-code helper at all. Would have caught 76305252: a
   background command that could not start printed nothing and left the PREVIOUS exit code standing.
2. FILESYSTEM EFFECTS, ESPECIALLY ABSENCE. Snapshot a directory, fail on unexpected entries. The only
   bug class here that has RECURRED: the quoted-`>` split that created a file named `b"`, and the
   ampersand absorbed into a redirect target that created `out.txt &`. Seven of the latter
   accumulated before anyone noticed. No existing case can say "and nothing else was created".
3. BOTH DOORS AGREE. The same input under default routing and FSH_SPINE=0, comparing stdout and
   status; divergence is a finding. This is the execution-level instrument INT-201's fourth gate
   needs, because the migration audit compares PARSERS and cannot license deleting an executor.
4. TELEMETRY. Run a command, then read shell_history and command_execution and assert the stored exit
   code and argv match what happened. Would have caught "Friday: true failed 3 times in a row".
   ★ Friday reasons over exactly these rows, so a wrong row is a wrong belief, and nothing tests them.
5. THE `-c` DOOR IN THE CORPUS. `spine conform` already proved `-c` creates `0.5` and `=` where the
   REPL refuses them. One corpus through both doors makes INT-201's "the digit guard applies to both
   doors" a finding by construction instead of something someone remembers to check.
6. IDEMPOTENCE. Run a redirecting command twice, assert one copy. INT-143's double-exec is the
   precedent.

## Relationship
- INT-173 is COMPLETE and is what made this possible: it gave fsh-test a REPL door instead of only
  `-c`. This intent fixes how that door LISTENS. Not a reopen.
- INT-201 supplied the motivating case: `d0c04825` fixed a defect the suite could not see.
- INT-157 (VM-based testing) owns the hidden-dependency and minimal-environment job. Gate 4 is a
  hook or a flake check, and must not quietly become 157.

## The Rule
"A suite whose number moves on identical code is not evidence. It is weather."
