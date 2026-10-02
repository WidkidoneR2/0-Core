---
id: 263
date: 2026-09-24
type: future
title: "the forest_ tables and old event rows still carry the retired names -- rename them under a rehearsed rollback"
status: complete
tags: [schema, migration, state, data, zero]
---

## Vision

state.db speaks Project 0. No live table, column, stored key or stored value names forest or
faelight, and nothing that reads the ledger can mistake a renamed table for an empty one.

## The Problem

INT-247 renamed what the machine SHOWS. What it STORES still carries the retired names, and a
stored name is a data migration, never a text edit (INT-247, Layer 3).

Measured 2026-09-24, read-only:

```text
    forest_events        94,558 rows      forest_memory        4   no code reads it
    forest_predictions   21,614           forest_goals         1
    forest_insights         226           forest_mandates      1
    forest_events_v2          6           forest_plans         1
    forest_tradeoffs          1           forest_strategies    0
```

Stored VALUES carry them too:

```text
    events.source_tool = 'faelight-shell'   every row NovaShell wrote before 408a731f. Writers say
                                            nsh since then; the old rows are history in a live table
    domain = 'forest'                        stored keys
    forest_goals.title                       matched BY TITLE (goals/mod.rs), so renaming a title
                                             without the row re-proposes the goal
    intelligence name                        core version prints a stored "Forest Mind"
```

And the code names tables that do not exist: forest_lesson and forest_operations.

The risk is INT-192's collapse: a query against a renamed table errors, unwrap_or_default()
turns the error into zero rows, and the ledger reads as EMPTY.

## The Solution

```text
    1  CENSUS      every table, column, stored key and stored value with a retired name, with
                   row counts; every SQL literal in live code that names one. Read-only
    2  NAMES       Christian rules each new name before anything changes. Plain words; zero_*
                   only where a plain name collides with a table that already exists
    3  BACKUP      sqlite .backup of state.db; the restore rehearsed on a copy
    4  REHEARSE    the whole migration on a copy: ALTER TABLE RENAME, UPDATE of stored values,
                   row counts identical before and after, only names changed
    5  ONE COMMIT  the migration and every SQL literal that names the tables, nothing else
    6  DOORS       nsh -c, a PTY session, d, history. If any looks empty, restore the backup
```

Dead tables are decided at step 2: dropped or kept, never renamed out of habit.

## Success Criteria

- [x] The census is regenerated at the start, on the live state.db, read-only: every table,
      column, stored key and value with a retired name, with row counts, and every live SQL
      literal that names one
<!-- evidence: 2026-10-02 at 64abb9d2, read-only (sqlite mode=ro), two runs in this session. No table, and no index, view or column, names a retired word (every table listed with its row count). Stored text values counted per column (events.source_tool 12,317 rows say faelight-shell; history columns carry old paths and commands). fsearch forest_ --type rs --live and fsearch faelight-shell --type rs --live: no results. -->
- [x] Every new name is ruled by Christian and written into this file BEFORE any change
<!-- evidence: the names were ruled in INT-247 before the change: 247:2904, plain words, zero_* only on a collision. Applied by 18e1f3d8 (INT-247 pass 6A): forest_events -> zero_events because events existed; forest_events_v2 -> events_v2; goals, insights, mandates, plans, predictions, strategies, tradeoffs. forest_memory was absorbed into friday_knowledge (c025b12c), not renamed. -->
- [x] A backup of state.db exists and a restore from it was rehearsed on a copy, row counts
      compared table by table
<!-- evidence: 18e1f3d8 body: backup state.db.pre-pass6-20260930T045915, quick_check ok; the rename and its reverse rehearsed on a backup copy, schema and counts equal. -->
- [x] The migration was rehearsed end to end on a copy: every table and row count identical
      before and after, only names and stored values changed
<!-- evidence: 18e1f3d8 body: rename and reverse rehearsed on a backup copy, schema and counts equal; row counts equal across the live rename. Stored values were not changed: ruled history (R1, Close below). -->
- [x] ONE commit carries the migration and every SQL literal that names the renamed tables.
      Nothing else is in it
<!-- evidence: 18e1f3d8: 26 files, the SQL, table-name and comment sites plus three printed table labels, nothing else (git show --stat 18e1f3d8, read 2026-10-02). The migration itself ran as a fingerprinted payload, Migrate-Fingerprint 465e7c5b7c48, not as code in the tree. -->
- [x] A reader whose table is missing reports UNREADABLE, not empty -- proven by a class test
      that renames a table under a reader and watches it refuse
<!-- evidence: DECLINED, R2, ruled by Christian 2026-10-02. 1) no renamed table is left for a reader to miss: no live .rs file names forest_ (fsearch, 2026-10-02) and the guard below keeps it so. 2) the missing-table-reads-empty class that remains lives in Friday: reasoning.rs:201 Err(_) => return None, planning.rs:899 .ok(). 3) it is Friday code and goes to the Friday intent, not to a schema rename. -->
- [x] Both doors after the migration: nsh -c, a PTY session, d, history. None looks empty
<!-- evidence: 18e1f3d8: ship 3 shipped 0 failed, d 0 failed, nsh-test 202/202. Again 2026-10-02 at 64abb9d2: d 0 failed, 27 of 28 passed, Fingerprint recorded a56683c54812378b; nsh-test (nsh -c and PTY sessions) 213/213 against the deployed nsh of 2026-10-01 23:30, which predates 7a90e018 (23:47), the last novashell/zero-core commit; history: shell_history holds 221,741 rows, read through the renamed schema by the census. None looks empty. -->
- [x] The guard holds it: a test fails if a live SQL literal names a retired table
<!-- evidence: 12772479: nsh-test no_live_retired_name_in_any_tracked_file (nsh-test main.rs:1507) reads every tracked file of every type, any case, for both retired words, so a live SQL literal naming a forest_ table fails it. Seen RED at 24 lines, then GREEN (247:3546). Green 2026-10-02, 213/213. -->
- [x] forest_goals is migrated with its title matcher in the same commit, and core goals
      generate proposes nothing it already has
<!-- evidence: 18e1f3d8 moved the table and its matcher together (goals/mod.rs, 18 lines). The generate run is DECLINED: 1) the one goals row, GOAL-001, has a title with no retired word (read 2026-10-02); 2) no title was renamed, so the hazard the gate guards, a title renamed apart from its row, cannot occur. -->

## Close 2026-10-02

The work of this intent landed under INT-247: the tables in 18e1f3d8 (pass 6A), Friday's knowledge
in 97bddede (6B2a), forest_memory absorbed into friday_knowledge (c025b12c). Re-measured read-only
2026-10-02 at 64abb9d2.

```text
    RULINGS -- Christian, 2026-10-02
    R1  stored history stays: shell history, commands, cwd, payloads, the 12,317 events rows
        with source_tool faelight-shell, session_state last_dir, events rowid 6606 (domain
        '"forest'). This amends the Vision: no live table, column or stored key names a
        retired word; stored history keeps the names it was written with. Reasons:
        1  nothing matches on them: source_tool is read inside a 10-minute window (friday
           planning.rs:893), a one-hour window (reasoning.rs:196) and by correlation_id
           (novashell commands/mod.rs:6140)
        2  last_dir is saved by nothing and restored by nothing (session.rs:113-115); nsh-test
           main.rs:2393 seeds it and proves it is not restored
        3  history keeps its names (AGENTS.md, Layer 0)
    R2  the missing-table-reads-empty class left in Friday (reasoning.rs:201, planning.rs:899)
        goes to the Friday intent
```

Found, not fixed: the deployed nsh (2026-10-01 23:30) predates 7a90e018 (23:47), the last
novashell/zero-core commit. Ship before the next code change is judged on the deployed binary.

## Relationship

- Parent: INT-247 -- the rename. This is its SCHEMA item, filed 2026-09-24
- Sibling: INT-264 -- contracts. D-Bus names and commands are not stored data
- Follows INT-247 Layer 3's rules: backup first, one risk per commit, never fix forward on state
