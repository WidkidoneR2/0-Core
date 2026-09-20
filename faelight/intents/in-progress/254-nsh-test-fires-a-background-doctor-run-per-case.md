---
id: 254
date: 2026-09-20
type: future
title: "nsh-test fires a background doctor run per case"
status: in-progress
tags: [nsh-test, notification, marko, doctor]
---

## Vision

The test suite runs the shell, not the doctor. A harness that redirects state does not make the
shell think its health is stale.

## The Problem -- MEASURED 2026-09-20

nsh-test gives every case its own empty database:

```text
    repl.rs:229   case_db_path() -> /tmp/nsh-test-<pid>/case<N>.db, N incrementing
    main.rs:120   every spawned shell gets FAELIGHT_STATE_DB pointed at it
```

And nsh at startup (main.rs:60, INT-124) refreshes the doctor health event when the latest one
predates the current boot. AN EMPTY DATABASE HAS NO EVENTS, so last_event_ts is 0, the freshness
check fails, and the shell spawns a detached `core doctor run`.

★ ONE PER CASE. 196 cases, 196 full doctor runs, each measured at roughly 450ms against the real
machine -- reading the real registry, running every probe, shelling out to systemctl, journalctl,
pacman, cargo and git -- and writing the result into a throwaway file nobody reads.

Confirmed by absence: the real state.db has NO doctor events during the test window, because
every one of those runs wrote somewhere else.

### The correlation, narrowed 2026-09-20

Observed across a dozen commands: the notifications appear WHEN AND ONLY WHEN nsh-test runs.

```text
    plain commands, a 3-command chain    none
    git commit --allow-empty             none  -- nothing staged, the hook skips the suite
    git commit with staged changes       YES   -- pre-commit runs zero-gate runs nsh-test
    bare gp, nothing to commit           YES   -- pre-push does the same
    nsh-test run directly                YES
```

One notification per burst of roughly four, against ~196 cases -- so not one per case. Something
about a subset of those background runs reaches Red, and it is that subset that has to be found.

## SOLVED 2026-09-20 -- the cause, measured

```text
    nsh-test case INT-230 runs a shell with HOME and XDG_STATE_HOME pointed at an
    empty directory, deliberately: it asserts that nsh says "needs 0-Core, which is
    not present" instead of pretending.

    nsh startup (main.rs:60) spawns `core doctor run` when the health event is stale.
    An empty HOME has no events, so it always is.

    That doctor resolves registry/doctor/checks.toml under the FAKE HOME, does not
    find it, and from_engine() emits Tier::Critical + Status::Unknown -- the
    load-error path INT-222 built so a missing check set could never render as a
    clean pass.

    verdict() reads a critical-tier Unknown as RED. The notification fires.
```

REPRODUCED EXACTLY, and the notification appeared as it ran:

```text
    HOME=/tmp/probe-home XDG_STATE_HOME=/tmp/probe-home core doctor run
    -> Passed: 0  Warnings: 0  Failed: 0  Unknown: 1 (Check Set)  Health: 0%
```

★ FOUR REPL SESSIONS IN THAT ONE CASE, FOUR SPAWNED SHELLS, FOUR DOCTORS, FOUR
NOTIFICATIONS -- which is the burst size observed all along.

⭐ AND EVERY PART OF THIS IS WORKING AS DESIGNED EXCEPT ONE. The test is right to fake HOME.
The doctor is right to call a missing check set critical. verdict() is right to read that as
Red. THE DEFECT IS THAT A BACKGROUND CACHE-WARM CAN SHOUT: INT-176 silenced that spawn with
stdout and stderr to /dev/null, and notify::desktop goes over busctl, which those redirects
do not touch.

## What is NOT known

⚠️ THE NOTIFICATIONS ARE NOT EXPLAINED. Running the suite produces desktop notifications reading
"a critical-tier check failed or could not run", and the cause was not found.

What was ruled out, each by measurement rather than argument:

```text
    the git hooks           pre-commit and pre-push exec zero-gate; neither mentions doctor
    a systemd timer         systemctl --user list-timers: 0 timers
    a second notify site    only doctor/mod.rs:398 sends that text
    the environment         env -i with HOME and PATH reproduces 92%, no failures
    the working directory   running from /tmp reproduces 92%
    a Red verdict           EVERY recorded doctor event reads health 92, result ok
    the empty database      reproduced exactly with FAELIGHT_STATE_DB=/tmp/probe.db:
                            24 passed, 2 unknown (System Services, Friday), NOT Red
```

The reproduction is exact and it is Amber. Something else fires the notification, and finding it
is part of this intent rather than a guess to be recorded as a cause.

### And the spawn is never reaped

Sampling the process table during a suite run caught `[core] <defunct>` -- a zombie. main.rs
spawns the refresh with `.spawn()` and never waits on it, so each finished child lingers until
its parent exits. Harmless in a login shell that lives for hours; less so when the suite creates
a hundred short-lived parents.

## THE PERFORMANCE PREMISE WAS WRONG -- measured 2026-09-20

This intent is titled "fires a background doctor run per case" and its body estimated 196 runs
at ~450ms. BOTH NUMBERS WERE MINE, NOT MEASURED. The real figures:

```text
    suite wall clock        32.1s
    suite user + sys        8.6s
    one real doctor run     963ms
    one fake-HOME run       5ms   -- the check set fails to load instantly and it
                                     returns a single Unknown without running a probe
    spawn sites in source   133
```

★ 133 REAL DOCTOR RUNS WOULD COST ~128 SECONDS OF CPU. The suite spends 8.6. The runs are not
happening at that rate: the fake-HOME ones are nearly free, and the INT-124 freshness check
throttles the rest -- once one run writes an event, the others in that session see it as fresh
and skip.

⚠️ SO THE COST THIS INTENT WAS FILED TO REMOVE LARGELY DOES NOT EXIST. The notification was
real, was measured, and is fixed. The performance half was an estimate dressed as a finding,
and it is recorded here as wrong rather than quietly dropped.

## Success Criteria

- ⏸ The background refresh does not run once per test case -- deferred: THE PREMISE WAS
      WRONG. Measured 2026-09-20, the suite spends 8.6s of CPU where 133 real doctor runs
      would cost ~128s. The fake-HOME spawns cost 5ms and the freshness check throttles the
      rest. There is no cost here to remove -- approved by: christian 2026-09-20
      <!-- original gate text: **Proven by counting:** a full
      nsh-test produces at most ONE doctor run, and the number is measured before and after.
- [x] The time it costs is measured first. DONE, and the measurement RETIRED the fix: 32.1s
      wall, 8.6s CPU, 963ms per real run, 5ms per fake-HOME run, 133 spawn sites. The number
      to beat turned out not to need beating.
- [x] ⭐ THE NOTIFICATION IS TRACED TO ITS ACTUAL CAUSE. The eighth was demonstrated, not
      argued: nsh-test case INT-230 fakes HOME, the spawned shell refreshes health, that
      doctor cannot find checks.toml, the load error is correctly critical, verdict is Red.
      Reproduced exactly with HOME=/tmp/probe-home, notification appearing on cue.
      <!-- Seven theories
      were ruled out by measurement on 2026-09-20; the eighth must be demonstrated, not argued.
- [x] Whatever the fix, a REAL stale health event still refreshes -- INT-124 exists because a
      stale banner number is a lie, and INT-176 because a 700ms block at the prompt is felt.
      Neither is undone by this.
- [x] nsh-test still 196/196 against the pre-push gate, which builds and tests the code
      being sent rather than the deployed shell.

## Relationship

Found while switching the doctor to faelight-doctor (INT-222). The background run is not new --
main.rs has spawned it since INT-176 -- but it became visible when the notification started
arriving, and the per-case cost was never measured until now.
