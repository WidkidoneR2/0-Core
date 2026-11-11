---
id: 312
date: 2026-11-10
type: future
title: "doors is quiet unless a door fails"
status: in-progress
tags: [doors, nsh, intent]
---

## Vision
doors stays the four-door run it is. A passing run shows the header, one
timed line per door, and the summary. The cargo and nsh-test transcripts stay
off that screen unless a door fails. No code is written before G1 is ruled.

## The Problem
Re-read 2026-11-10 against 117b0a27. zero/scripts/doors runs each door with
its stdout and stderr inherited (run_door at lines 39-59). A green laptop run
the same night was core 0.2s (277 passed), shell 4.4s (258 + 6 + 1 passed),
nsh-test 85.6s (304/304 passed), and validate 348 valid. The summary was
4 passed, 0 failed, under about a minute and a half of transcript.

Lines 6-8 refuse every argument and exit 2. There is no way to ask for the
full stream, and no way to keep it for a door that failed.

Christian, 2026-11-10: "We should make the doors quieter that is a good idea."
and "lets make the doors quieter now."

## The Solution
1. Default screen: the header, one timed pass or fail line per door, and the
   summary, including the existing one-line reminder to run d. Colors stay
   on for a terminal when NO_COLOR is unset, and off otherwise.
2. Each door's stdout and stderr are captured to target/doors/<name>.log.
   target/ is already gitignored. A passing door does not print the file.
   A failing door prints the file in full and prints its path. The other
   doors still run. Exit 0 when all pass, exit 1 when any fails.
3. There is no verbose flag. Any argument still exits 2, so the script
   still cannot be handed a command to run.

## Success Criteria
- [x] G0 The recon in The Problem is read against the tree and either stands or
      is corrected here before any other gate. Re-read: run_door is
      zero/scripts/doors lines 39-59, and lines 6-8 refuse every argument.
      Proof: a dated line in this file saying the recon stands, or the
      correction.
<!-- evidence: 2026-11-10. Re-read on 7d5f93ad, the filing of this intent on 117b0a27. The recon stands. run_door is still zero/scripts/doors lines 39-59, and line 45 runs the command with stdout inherited. Lines 6-8 still refuse every argument and exit 2. -->
- [x] G1 The verbose flag. Christian rules yes, no, or a different spelling.
      The shape to accept, change, or reject: only -v and --verbose stream
      the doors as they do today. Any other argument still exits 2. Proof:
      the RULED line.
RULED 2026-11-10 (Christian): "No verbose flag; doors is always quiet and shows the log only on failure."
- [x] G2 Red, in zero/scripts/dev/test_doors.py, only after G1 is ruled. A
      passing run prints the header, the timed lines, and the summary, and
      does not print the door's own output. A failing door prints that output
      and the log path. Shown failing first, under its own plan Seal. Proof:
      the red output.
<!-- evidence: 2026-11-10. Red commit 94618f30, Seal 173417cb500c380c. python3 -m unittest test_doors: 9 passed, 2 failed, exit 1. The failures are test_a_passing_run_hides_the_door_transcript (DOORTRANSCRIPT is on stdout) and test_a_failing_door_prints_its_log_and_the_others_still_run (no target/doors/core.log path). Any argument already exits 2. -->
- [x] G3 Green. The same tests pass. Quiet by default, full output on failure,
      the ruled flag, colors, keep-going, and the exit codes. Proof: the green
      output under its plan Seal.
<!-- evidence: 2026-11-10. This commit. python3 -m unittest test_doors: 11 passed. A passing door's transcript is in target/doors/<name>.log and not on stdout. A failing door prints that file and its path. Any argument still exits 2. There is no verbose flag. Proof is this commit's Seal trailer, the plan seal, and the red seal 173417cb500c380c. -->
- [ ] G4 Christian runs doors on the laptop. A passing run is the quiet
      screen. Proof: the door output.

## Out of scope
- Changing which four doors run, or the build inside the nsh-test door.
- Calling ship or reload. Replacing d.
- A fifth door, or a copy of d's count.

