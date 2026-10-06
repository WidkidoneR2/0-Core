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
- [ ] A SUCCESSFUL persist or unpersist leaves exit status 0. Today neither builtin calls
      set_last_exit, so `$?` after them is the previous command's (`false; persist X; echo $?`
      is predicted to print 1). Seen RED first, like every case here
- [ ] nsh-test green after ship, and `d` 0 failed

## Relationship

- Found by INT-247's census, 2026-09-24. The stale row was removed by hand that day; this intent
  removes the reason it could survive five months
- INT-192: an unreadable source answering as an empty one -- the restore at main.rs:2566
- INT-251: unknown read as success -- the discarded INSERT result at engine.rs:496
- INT-257 law 1: a write inside the clean room never reaches the host. The ISOLATION gate is that
  law applied to this suite

## Recon (2026-10-05 -- re-located after the 2026-09-25 tree move)

The Problem table above is the 2026-09-24 record and stays as written. Where each piece is
today, in zero/shell/novashell/src/:

```text
    WHAT                    2026-09-24          TODAY
    try_unset               engine.rs:474       engine.rs:474 -- session var and env only
    try_persist             engine.rs:487       engine.rs:487
    the indirection         engine.rs:492       engine.rs:492
    the discarded INSERT    engine.rs:496       engine.rs:496-499 `let _ =`, then 500-504
                                                print "persisted" unconditionally
    dispatch                main.rs:1493        main.rs:1487, after export :1469, unset :1478
    the restore             main.rs:2566        main.rs:2526-2551: CREATE result discarded
                                                :2529, filter_map(r.ok()) :2542,
                                                unwrap_or_default :2543, Err(_) => Vec::new()
                                                :2544
    unset lists             commands/mod.rs     commands/mod.rs:11309 in explain_cmd (:11198)
                            :11392, :11536      and :11452 in where_cmd (:11405)
```

What the reading settled:

- try_persist matches only the prefix "persist " (engine.rs:488). Bare `persist` and
  `unpersist NAME` both fall through to the external-command path. Predicted RED: command not
  found.
- Status: SegmentOutcome (engine.rs:93) is Next | ExitShell and carries no status, and does not
  need to. Builtins report through engine.set_last_exit(Some(code)) (engine.rs:1775, dozens of
  callers). try_persist and try_unset never call it; main.rs:1386's set_last_exit(Some(0))
  belongs to `flow` only. So after persist, $? is the previous command's -- the class INT-169's
  comment at main.rs:1383 records. The fix is a set_last_exit call, not a new variant.
- The two unset lists are both "what is this word" answers: explain_cmd describes a builtin,
  where_cmd prints "builtin  native nsh". Neither dispatches. Registering persist and unpersist
  there is discovery, not routing.
- Isolation already exists: nsh-test gives each REPL case its own database through
  ZERO_STATE_DB = repl::case_db_path() (nsh-test repl.rs:224; /tmp/nsh-test-<pid>/caseN.db, the
  directory removed at the end of the run). zero_core::paths::state_db() honours the override
  (zero-core paths.rs:287) and nsh's database follows it (db.rs:69).
- The two-session pattern exists: nsh-test main.rs:2609-2626 runs session 1 on a case database,
  writes into that database directly with rusqlite, then runs session 2 against it. The same
  shape gives the FRESH-shell proof, plants the forced-INSERT trigger and builds the broken
  table.
- ISOLATION baseline, read 2026-10-05 in SQLite read-only mode after a full 215/215 nsh-test
  run: ~/.local/state/zero/state.db shell_persist has columns key, value and 0 rows.

## START HERE

Written 2026-10-05. No gate is ticked. Next: the ISOLATION proof as the first case group (the
live shell_persist read before and after a full run), then the RED cases on the current binary
-- bare persist, unpersist, forced INSERT failure, unreadable table, stale status after persist,
the :492 indirection pinned as it behaves -- each seen red before any fix. Plan, review, apply
for every edit; one concern per commit.
