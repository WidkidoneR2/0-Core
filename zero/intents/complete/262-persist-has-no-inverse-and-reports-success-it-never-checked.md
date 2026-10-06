---
id: 262
date: 2026-09-24
type: future
title: "persist has no inverse and reports success it never checked"
status: complete
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

- [x] ISOLATION: this intent's nsh-test cases run against their own state directory, and the real
      ~/.local/state/zero/state.db shell_persist is unchanged by the suite -- proven by reading it
      before and after a full run
      <!-- evidence: every INT-262 case passes ZERO_STATE_DB = repl::case_db_path(), its own
      /tmp/nsh-test-<pid>/caseN.db. The live ~/.local/state/zero/state.db shell_persist, read
      in SQLite read-only mode, held key, value and 0 rows after the 215/215 baseline run
      (2026-10-05), 0 rows after five full runs exercising every persist case (after fix 4),
      and 0 rows again on 2026-10-06 after the last debug runs. -->
- [x] Every case below is seen RED on the current binary before its fix lands
      <!-- evidence: each case was seen red on the binary built before its fix. a47e3711 (seal
      deec732e2a2ad0a2): 216/223, the seven original cases red for their named reasons and the
      :492 pin green. c955d854 (seal 87f9590b95c60237): persist_262_stores_its_own_value red
      with stored Some(hidden), 218/223. 75968536 (seal 6b24d6f713a95a8f):
      restore_262_unreadable_store_is_reported red, the banner held no report, 224/225.
      d4a62e06 (seal 2ba8df44f7fb1b93): discover_262_where_and_explain_know_persist red, where
      persist said not found, 225/226. One exception, by design:
      restore_262_empty_store_is_silent (4aa97d7d) guards a silence that already held, so it
      passed from the start. -->
- [x] `unpersist NAME` removes the stored entry: a case persists NAME, unpersists it, and a FRESH
      shell on the same state directory does not have it
      <!-- evidence: unpersist_262_removes_for_a_fresh_shell is green at 0f2a2b21 (seal
      8fbc3ffbdbb48fb3): the row is gone and a fresh shell on the same case database prints
      <>. Live on the shipped nsh 5.0.3: persist T262, then unpersist T262, then persist
      printed nothing is persisted. -->
- [x] `unpersist NAME` for a name that is not stored says so, instead of reporting a removal
      <!-- evidence: unpersist_262_not_stored_says_so is green at 0f2a2b21 (seal 8fbc3ffbdbb48fb3).
      Live on the shipped nsh 5.0.3: a second unpersist T262 printed T262 is not persisted. -->
- [x] Bare `persist` lists the stored names and values; with nothing stored it says so
      <!-- evidence: persist_262_bare_empty_says_so and persist_262_bare_lists_the_store are green
      at 0f2a2b21 (seal 8fbc3ffbdbb48fb3). Live on the shipped nsh 5.0.3: nothing is
      persisted, then T262=hello after persist T262. -->
- [x] `persist NAME` whose INSERT fails (the trigger above) reports the failure and does NOT print
      "persisted"
      <!-- evidence: persist_262_failed_insert_is_reported is green at c6b05edf (seal
      b7eb6d44d7b905d5): with the BEFORE INSERT trigger planted, the output holds no persisted
      line and the status is non-zero. -->
- [x] A failed persist or unpersist leaves a non-zero exit status. RECON FIRST: these builtins
      return SegmentOutcome::Next, which carries no status
      <!-- evidence: persist: the same case asserts a non-zero status (c6b05edf). unpersist: the
      DELETE failure path from 0f2a2b21, demonstrated 2026-10-06 on the debug build with a
      BEFORE DELETE trigger: x unpersist U9: not removed (reason forced), and echo $? printed
      status=1. RECON FIRST is answered in ## Recon: builtins report status through
      set_last_exit, not SegmentOutcome. -->
- [x] The startup restore reports an unreadable shell_persist; a readable empty one stays silent
      <!-- evidence: restore_262_unreadable_store_is_reported went red then green at 75968536 (seal
      6b24d6f713a95a8f); restore_262_empty_store_is_silent holds at 4aa97d7d (seal
      52d894a55e2f7fe3). Both read the pre-prompt output through run_repl_lines_banner, added
      at 4aa97d7d because the harness dropped it. -->
- [x] The :492 indirection is pinned by a case showing what it does today, Christian rules keep or
      remove, and the ruling is written here
      <!-- evidence: pinned by persist_262_indirection_pinned at a47e3711 (P262 stored hidden).
      Ruled remove on 2026-10-05, written in ## Rulings. Removed at c955d854 (seal
      87f9590b95c60237); the case became persist_262_stores_its_own_value, expecting Q262. -->
- [x] `persist` and `unpersist` appear wherever `unset` is registered (commands/mod.rs:11392 and
      :11536), after each list's purpose is read and written here
      <!-- evidence: both lists were read and their purpose written in ## Recon: explain_cmd and
      where_cmd, discovery only. persist and unpersist joined both at d4a62e06 (seal
      2ba8df44f7fb1b93). Live on the shipped nsh 5.0.3: where unpersist printed builtin native
      nsh. -->
- [x] A SUCCESSFUL persist or unpersist leaves exit status 0. Today neither builtin calls
      set_last_exit, so `$?` after them is the previous command's (`false; persist X; echo $?`
      is predicted to print 1). Seen RED first, like every case here
      <!-- evidence: persist_262_success_leaves_status_zero was red with Some(1) at a47e3711 and is
      green at c6b05edf (seal b7eb6d44d7b905d5); unpersist sets 0 on success at 0f2a2b21. Live
      on the shipped nsh 5.0.3: echo $? printed 0 after persist T262 and after unpersist T262. -->
- [x] nsh-test green after ship, and `d` 0 failed
      <!-- evidence: shipped nsh 5.0.3 on 2026-10-06, a release build from d4a62e06. nsh-test
      226/226 on the shipped binary; d 28/28, 100 percent, 0 failed, working tree clean and
      all commits pushed. -->

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

## Rulings (2026-10-05)

- The :492 indirection: REMOVE. `persist NAME` stores NAME's own value -- the shell variable, else
  the environment value -- never another variable's. Removed at c955d854
- shell_persist's schema: owned by StateDb::open's shell-table batch (db.rs), which runtime_init
  runs for both doors; the REPL restore only reads it. Landed at cf650f88
- The startup-restore case: a harness helper that keeps the pre-prompt output
  (run_repl_lines_banner), not a test through bare `persist`. Landed at 4aa97d7d
- `unpersist` of a name that is not stored leaves status 0, as `unset` does. Taken as recommended
  at 0f2a2b21, not ruled explicitly; open to revision

## START HERE

Written 2026-10-06; supersedes the 2026-10-05 entry. Every gate is ticked with its evidence. The
work is eight sealed commits, all pushed and shipped as nsh 5.0.3: a47e3711 (the RED cases),
cf650f88 fix 1, c6b05edf fix 2, c955d854 fix 3, 0f2a2b21 fix 4, 4aa97d7d fix 5a, 75968536 fix 5b,
d4a62e06 fix 6. Next: `cicomplete 262`, then INT-277 -- cistart, tick G0 from its ## Recon, and
take its three G2 rulings before any work.

Carried forward: a two-step payload recognises its first step by a marker that survives rustfmt
(the case name), never by its bytes. 5b's step 2 was refused for exactly that and re-planned
engine-only; fix 6 used the name guard and went through a reflow untouched.

## Versions
- novashell 5.0.3 -> 5.1.0 (minor)
