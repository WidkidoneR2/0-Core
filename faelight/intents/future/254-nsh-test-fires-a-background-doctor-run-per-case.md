---
id: 254
date: 2026-09-20
type: future
title: "nsh-test fires a background doctor run per case"
status: planned
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

## Success Criteria

- [ ] The background refresh does not run once per test case. **Proven by counting:** a full
      nsh-test produces at most ONE doctor run, and the number is measured before and after.
- [ ] The time it costs is measured first, so the fix has a number to beat rather than a feeling.
- [ ] ⭐ THE NOTIFICATION IS TRACED TO ITS ACTUAL CAUSE, not to a plausible one. Seven theories
      were ruled out by measurement on 2026-09-20; the eighth must be demonstrated, not argued.
- [ ] Whatever the fix, a REAL stale health event still refreshes -- INT-124 exists because a
      stale banner number is a lie, and INT-176 because a 700ms block at the prompt is felt.
      Neither is undone by this.
- [ ] nsh-test still 196/196, and the suite still exercises the shell it is testing.

## Relationship

Found while switching the doctor to faelight-doctor (INT-222). The background run is not new --
main.rs has spawned it since INT-176 -- but it became visible when the notification started
arriving, and the per-case cost was never measured until now.
