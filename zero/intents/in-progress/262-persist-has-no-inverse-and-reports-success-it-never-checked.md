---
id: 262
date: 2026-09-24
type: future
title: "persist has no inverse and reports success it never checked"
status: in-progress
tags: [novashell, builtins, state]
---

## Vision

`persist` is a promise that a variable will be there in the next shell. A promise needs three
things this one does not have: a way to take it back, a way to see what was promised, and an
honest answer when it could not be written. After this intent the persisted store is visible,
reversible, and never reports a write it did not make.

## The Problem

Found by INT-247's census on 2026-09-24: FOREST_TEST=hello was in every new shell. It came from a
test on 2026-04-09 -- shell_history row 6602, `export FOREST_TEST=hello` -- that was persisted and
then re-exported by every shell for five and a half months. It was in no startup file and in no
parent process; nsh set it itself, from shell_persist rowid 1. The row was removed by hand the same
day. This intent is why it could live that long.

```text
    WHERE                          WHAT
    engine.rs:474  try_unset       clears the session and the environment, never shell_persist --
                                   so unset hides a persisted variable only until the next shell
    engine.rs:487  try_persist     the only writer, and it has no inverse: nothing in the tree
                                   runs DELETE FROM shell_persist
    engine.rs:496  let _ = ...     the INSERT result is discarded, and :501 prints "persisted
                                   across sessions" whether or not anything was written
    engine.rs:492  indirection     if NAME is not a shell variable but its ENVIRONMENT value names
                                   one, the OTHER variable's value is stored under NAME.
                                   Read in the code, not yet run
    main.rs:2566   the restore     filter_map(r.ok()), unwrap_or_default(), Err(_) => Vec::new() --
                                   an unreadable table restores as an empty one, silently
    main.rs:1493   dispatch        the only call site, after try_export (:1475) and try_unset (:1484)
    discovery      nowhere         no help, completion, registry, cheatsheet or teach entry names
                                   persist, and no nsh-test case covers it. The feature had no
                                   way to show what it held
```

The builtin-name lists that DO carry `unset` are commands/mod.rs:11392 and :11536. What each list
is FOR has not been read yet; it is read before either one is touched.

The same habit INT-247 keeps finding: when the answer is not known, supply the happy one. There,
unwrap_or(100) read an absent health cache as perfect health. Here a discarded Result reads a failed
write as success, and an unreadable table reads as nothing stored.

## The Solution

```text
    unpersist NAME   removes NAME from shell_persist. The session value is left alone -- the
                     mirror of persist, which stores without changing the session. If NAME is
                     still set it says so and points at unset. unset keeps its bash meaning:
                     this session only
    persist          bare, lists what is stored, names and values, the way export -p does.
                     With nothing stored it says so instead of printing nothing
    persist NAME     a failed write is reported as a failure, never as "persisted"
    the restore      an unreadable shell_persist is reported at startup, not restored as empty.
                     A readable empty table stays silent
```

### The order -- tests first, each seen RED on the current binary before its fix

```text
    1  ISOLATION     the cases run against their own state directory, and the real
                     shell_persist is unchanged by the suite. Nothing else is tested until
                     this holds -- a test of persist that writes to the real store is the
                     defect this intent exists to remove
    2  RED CASES     unpersist is unknown; bare persist is not a command; a forced INSERT
                     failure still prints "persisted"; an unreadable table restores silently;
                     the :492 indirection pinned as it behaves today
    3  FIXES         one concern per commit
    4  REGISTRATION  both names wherever unset is registered, once each list's purpose is read
```

Two test tools, both deterministic and neither relying on file permissions:

```text
    a failing INSERT       in the case's own state.db:
                           CREATE TRIGGER ... BEFORE INSERT ON shell_persist
                           BEGIN SELECT RAISE(ABORT, 'forced'); END
    an unreadable table    the case's own state.db holds a shell_persist WITHOUT key and value
                           columns. CREATE TABLE IF NOT EXISTS leaves it alone, and the SELECT
                           cannot prepare
```

### Not in scope

```text
    export parsing         try_export trims quotes with trim_matches -- its own question
    the table name         shell_persist carries no retired name; it stays
    the spine              these builtins stay on the text path; routing is INT-169's decision
```

## Success Criteria

- [ ] ISOLATION: this intent's nsh-test cases run against their own state directory, and the real
      ~/.local/state/zero/state.db shell_persist is unchanged by the suite -- proven by reading it
      before and after a full run
- [ ] Every case below is seen RED on the current binary before its fix lands
- [ ] `unpersist NAME` removes the stored entry: a case persists NAME, unpersists it, and a FRESH
      shell on the same state directory does not have it
- [ ] `unpersist NAME` for a name that is not stored says so, instead of reporting a removal
- [ ] Bare `persist` lists the stored names and values; with nothing stored it says so
- [ ] `persist NAME` whose INSERT fails (the trigger above) reports the failure and does NOT print
      "persisted"
- [ ] A failed persist or unpersist leaves a non-zero exit status. RECON FIRST: these builtins
      return SegmentOutcome::Next, which carries no status
- [ ] The startup restore reports an unreadable shell_persist; a readable empty one stays silent
- [ ] The :492 indirection is pinned by a case showing what it does today, Christian rules keep or
      remove, and the ruling is written here
- [ ] `persist` and `unpersist` appear wherever `unset` is registered (commands/mod.rs:11392 and
      :11536), after each list's purpose is read and written here
- [ ] nsh-test green after ship, and `d` 0 failed

## Relationship

- Found by INT-247's census, 2026-09-24. The stale row was removed by hand that day; this intent
  removes the reason it could survive five months
- INT-192: an unreadable source answering as an empty one -- the restore at main.rs:2566
- INT-251: unknown read as success -- the discarded INSERT result at engine.rs:496
- INT-257 law 1: a write inside the clean room never reaches the host. The ISOLATION gate is that
  law applied to this suite
