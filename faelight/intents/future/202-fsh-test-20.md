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
      and reports a timeout as a failure, not as empty output. Suite runtime recorded before and
      after.
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

## A DECISION THIS INTENT MUST MAKE RATHER THAN INHERIT
fsh-test carries `conform_*` cases and `spine conform` is a separate conformance mechanism with a
different capture path. Two conformance tools with two capture paths is the two-doors shape again.
Decide which one owns bash comparison, and say so here, before either grows further.

## Relationship
- INT-173 is COMPLETE and is what made this possible: it gave fsh-test a REPL door instead of only
  `-c`. This intent fixes how that door LISTENS. Not a reopen.
- INT-201 supplied the motivating case: `d0c04825` fixed a defect the suite could not see.
- INT-157 (VM-based testing) owns the hidden-dependency and minimal-environment job. Gate 4 is a
  hook or a flake check, and must not quietly become 157.

## The Rule
"A suite whose number moves on identical code is not evidence. It is weather."
