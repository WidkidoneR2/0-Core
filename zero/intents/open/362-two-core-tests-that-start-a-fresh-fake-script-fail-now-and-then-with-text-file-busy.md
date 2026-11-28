---
id: 362
date: 2026-11-28
type: future
title: "two core tests that start a fresh fake script fail now and then with Text file busy"
status: planned
tags: [core, tests, flake]
depends_on: []
format: 3
---

## Vision
The pre-push hook runs the whole core suite. After this,
it stops failing on two tests that write a small fake
program, mark it executable and start it a moment later.
A push is held back only when something is really wrong.
No production code changes: the fix is in the two tests.

## The Problem
On 2026-11-28 the INT-361 push (commit d5a211cf) was
stopped by the pre-push hook: `cargo test -p core` showed
1 failing, domains::intent::panel::tests::
quickshell_present_starts_detached_with_the_panels_path.
INT-361 touched only nsh and nsh-test. Run alone, the
test passed in 0.02s, and the same push went through
once that test passed again.

Measured 2026-11-28 on a box checkout of origin/main
d5a211cf (core 5.5.1), cargo +1.88.0, 8 cores, with
busy-loop processes added to load the machine. The
unmodified test binary was run as built from d5a211cf.

1. The cause is not the 1s wait. The test
   (panel.rs:129) writes a fake `quickshell` script
   (panel.rs:133-139), then calls launch (panel.rs:141),
   which starts it with Command::spawn
   (spawn_detached, panel.rs:50). It then polls argv.txt
   50 times at 20ms (panel.rs:144-152). Running that one
   test alone 700 times, up to 100 at once, under up to
   400 busy loops: 0 failures. Running the whole core
   suite (397 tests, one process, many threads) 190
   times under 32 busy loops: the panel test failed 3
   times. All three were the same message:
   `quickshell did not start: Text file busy (os error
   26)`, panicking at panel.rs:141, the launch call, not
   at the poll.
2. Why. Linux refuses to run a file while any process
   still has it open for writing (ETXTBSY). The suite
   runs many tests at once in one process. If another
   test thread forks (a test that starts git or bash,
   for example) between our fs::write and our exec, the
   forked child carries a copy of our write handle until
   it runs its own exec. Our spawn lands in that gap and
   fails. This is a known hazard for any test that
   writes an executable and runs it at once. It is
   timing, not load alone, but load widens the gap.
3. A second test has the same shape. f0013_desktop_
   notify_prints_nothing (zero/engine/src/domains/
   notify/mod.rs:181) writes a fake `busctl` script
   (notify/mod.rs:206-215) and runs a child that starts
   it. Its call to busctl ignores a failed start
   (notify/mod.rs:170, `.status()` result dropped), so
   a failed start shows only as a marker file that never
   appears, and the test says `fake busctl did not run,
   so this result must not be trusted`. In 550 full-suite
   runs of the build above with the panel test fixed (150
   under 32 busy loops, 400 as four at once under 8 busy
   loops each), the suite was not clean twice: the second
   was captured and is this test; the first run's output
   was not kept, so its test is not known. The failure
   text matches the ETXTBSY shape, but the cause was
   not proven for this one: the start error is dropped,
   and a 400-run instrumented rerun (0 failures, no retry
   needed) did not fail again. It is far rarer than the
   panel test.
4. Other tests also write a script and run it
   (trace.rs:2050 writes a fake `core` that bash runs;
   there are more in zero-doctor and novashell). None
   was identified failing in 950 suite runs. They are not
   touched here; see Limits.
5. Two smaller weaknesses in the panel test, neither
   seen to fail: the 1s wait (a slow start on a loaded
   laptop could miss it), and the fake script writes
   argv.txt with a shell `>` redirect, which creates an
   empty file before printf writes, so a reader could in
   theory see a half-written file. The test already
   skips an empty read; a non-empty partial read would
   fail the assert on the first line.

## The Solution
Test changes only, in two files, with the asserts left
as they are.

G1 panel.rs, test
quickshell_present_starts_detached_with_the_panels_path:
- Call launch in a loop. On Err(CoreError::Reported(why))
  where why contains "Text file busy" and the deadline
  has not passed, sleep 10ms and call launch again. Any
  other error panics with the same message as today
  (`quickshell is on PATH: {err:?}`). Deadline: 5s from
  an Instant.
- The fake script writes argv.tmp and then `mv`s it to
  argv.txt, so argv.txt appears whole or not at all.
- The wait for argv.txt uses an Instant deadline of 5s
  instead of 50 x 20ms. It still ends the moment the file
  is read, so the normal case takes the same time.
- The three asserts after the wait are unchanged. The
  sibling test quickshell_missing_is_a_safe_stop_and_
  starts_nothing never runs its script and is unchanged.
Box prototype: 21 lines added, 3 removed in panel.rs.

G2 notify/mod.rs, test f0013_desktop_notify_prints_
nothing: run the child (the Command at notify/mod.rs:220)
inside a loop of at most 5 attempts, with 50ms between
them, and stop at the first attempt after which the
marker file exists. After the loop the code and the three
asserts are as they are now: if the marker never appears
the test still fails with the same message. Box
prototype: 29 lines added, 16 removed in notify/mod.rs
(most of it re-indenting the Command).

G3 is the box proof, with no repo commit. The whole
core suite is run many times under busy loops, with the
d5a211cf binary and with the fixed binary, and the
count of panel-test failures (and any other failure) is
compared.

G4 is the laptop proof.

## Limits
- In scope: the test quickshell_present_starts_detached_
  with_the_panels_path in zero/engine/src/domains/
  intent/panel.rs; the test f0013_desktop_notify_prints_
  nothing in zero/engine/src/domains/notify/mod.rs; a
  core patch bump at close.
- Out of scope: any non-test code (launch, spawn_detached
  and desktop stay as they are); the other tests that
  write and run a script (trace.rs:2050, zero-doctor
  hardening.rs:1062 and 1270, novashell guards.rs:133);
  a shared test helper; the pre-push hook; running the
  suite with fewer threads.
- How deep: four gates, under an hour. No new crate,
  dependency or function outside the two tests.
- Stop and ask: a fix seems to need a change to
  non-test code; an assert in either test has to change;
  the box proof still shows the panel test failing; some
  other test fails in the stress runs in a way these two
  fixes do not explain.
- Done means: G1 to G4 ticked with evidence.
- Must not change: any non-test line, live state.db, the
  pre-push hook, any other test, and any secret or
  confidential value.

## Success Criteria
- [ ] G1 panel.rs test: retries only "Text file busy" on
      launch for up to 5s, the fake script writes
      argv.tmp then moves it to argv.txt, the wait for
      argv.txt is a 5s Instant deadline, and the asserts
      are unchanged. cargo test -p core panel passes
      (3 tests) and cargo test -p core is 397 passed.
      ask: the panel test no longer fails when another thread forks during its start
      ruling:
      evidence:
- [ ] G2 notify/mod.rs test: the child run is repeated
      up to 5 times until the marker exists, and the
      asserts are unchanged. cargo test -p core
      f0013_desktop passes and cargo test -p core is 397
      passed.
      ask: the busctl test does not fail on a start that lost the same race
      ruling:
      evidence:
- [ ] G3 box proof: the same stress (whole core suite
      repeated under busy loops) on d5a211cf and on the
      int-362 tip, with the run counts and failure counts
      for each. On d5a211cf the panel test failed 3 of
      190 runs in the draft measurement (all Text file
      busy); on the int-362 tip it fails in none, and no
      test fails in a way the two fixes do not explain.
      ask: the failure rate drops to zero in the same stress, with the numbers shown
      ruling:
      evidence:
- [ ] G4 laptop: cargo test -p core passes (397), doors
      and d are green, and the push of the close commit
      passes the pre-push hook on its first try.
      ask: the laptop agrees and the pre-push hook is quiet
      ruling:
      evidence:

## Job order
Jobs run on branch int-362 only. Agents never push to
main. Job I files the intent. G1, then G2 (a different
file, independent). G3 is a proof with no commit. G4 is
the laptop. The seal is the first 16 hex of sha256 of
int362-plan.txt: 7cac268579e056ff.

## Versions
- core patch (test-only change, same as INT-352 to
  INT-360 each bumped the tool they touched; 5.5.1 to
  5.5.2). If Christian prefers no bump for a test-only
  change, the close commit says so in its Release trailer
  instead.
