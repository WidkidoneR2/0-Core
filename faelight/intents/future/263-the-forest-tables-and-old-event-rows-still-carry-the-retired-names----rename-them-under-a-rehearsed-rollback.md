---
id: 263
date: 2026-09-24
type: future
title: "the forest_ tables and old event rows still carry the retired names -- rename them under a rehearsed rollback"
status: planned
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

- [ ] The census is regenerated at the start, on the live state.db, read-only: every table,
      column, stored key and value with a retired name, with row counts, and every live SQL
      literal that names one
- [ ] Every new name is ruled by Christian and written into this file BEFORE any change
- [ ] A backup of state.db exists and a restore from it was rehearsed on a copy, row counts
      compared table by table
- [ ] The migration was rehearsed end to end on a copy: every table and row count identical
      before and after, only names and stored values changed
- [ ] ONE commit carries the migration and every SQL literal that names the renamed tables.
      Nothing else is in it
- [ ] A reader whose table is missing reports UNREADABLE, not empty -- proven by a class test
      that renames a table under a reader and watches it refuse
- [ ] Both doors after the migration: nsh -c, a PTY session, d, history. None looks empty
- [ ] The guard holds it: a test fails if a live SQL literal names a retired table
- [ ] forest_goals is migrated with its title matcher in the same commit, and core goals
      generate proposes nothing it already has

## Relationship

- Parent: INT-247 -- the rename. This is its SCHEMA item, filed 2026-09-24
- Sibling: INT-264 -- contracts. D-Bus names and commands are not stored data
- Follows INT-247 Layer 3's rules: backup first, one risk per commit, never fix forward on state
