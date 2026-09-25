---
id: 244
date: 2026-09-05
type: fix
title: "shell_snapshots has four ALTER-only columns so a database built from source cannot run the snapshot list query"
status: planned
tags: [fix, bugfix]
---

## Vision
`timeline` shows the snapshots that exist. A database built from source has the
same columns as this machine's.

## The Problem
Found 2026-09-05. **Two separate defects in one table, and both report as an
empty result.**

### ① The table has four columns nothing creates
`PRAGMA table_info(shell_snapshots)` on the live database returns **thirteen**
columns. The `CREATE TABLE` at `commands/mod.rs:13585` defines **nine**.

```
missing from source:  command, git_hash, cwd, intent_id
```

They were added by `ALTER TABLE` and exist only on machines that ran the version
that added them. **A database built from source cannot run the query at
`commands/mod.rs:14953`**, which selects all four. `Err(_) => "No snapshots
yet."` reports that failure as an empty result.

⭐ Same shape as INT-214 (`events.source_tool` / `correlation_id` never created
by any commit), in a second table.

### ② `timeline` is broken on THIS machine, right now
```
nsh -c "timeline 3"   ->  "○ No snapshots yet. Run: snapshot"
sqlite3 ... "SELECT COUNT(*) FROM shell_snapshots"   ->  574
```

574 rows, and the command says there are none.

**The cause, measured:** `db::capture_snapshot` writes name, timestamp, health,
command, git_hash, cwd and intent_id. It NEVER writes `commits`, `processes` or
`load_avg`. So every automatic snapshot (`auto-git`, `auto-rm`) has NULL in
those three columns:

```
574 | auto-git | 1783041981 | 100 | NULL | NULL | NULL
```

`timeline`'s reader binds them as non-null (`r.get::<_, i64>(4)`, `(5)`,
`(6)`), so the row conversion fails and the whole command returns the empty
message.

⚠️ **TWO WRITERS, ONE TABLE, DISAGREEING ABOUT WHICH COLUMNS EXIST.**
`snapshot_cmd` fills all nine; `capture_snapshot` fills seven different ones.

📍 **PROVEN NOT TO BE TODAY'S WORK.** `git stash` back to `f8d44e6d`, rebuild,
`timeline 3` -> the identical "No snapshots yet". The defect predates the
INT-230 G4 changes entirely.

### Why it stayed invisible
`Err(_) => return CommandResult::Output("No snapshots yet")` discards the
reason. A failed query and an empty table produce the same output -- INT-192's
subject exactly, and this is a live instance of it.

## The Solution
Three parts, and they are separable:

1. **The canonical schema gains the four ALTER-only columns**, so a fresh
   database is born able to run every query against this table. (INT-214's fix,
   applied here.)
2. **The two writers agree.** Either `capture_snapshot` fills the operational
   columns, or the readers accept that automatic snapshots do not have them --
   a ruling, not a default.
3. **The reader stops swallowing the reason.** A query that fails says so
   instead of reporting an empty table.

## Success Criteria
- [ ] G1 RED FIRST, ALREADY CAPTURED: `timeline 3` prints "No snapshots yet"
      against 574 rows, and a git-stashed build does the same -- so the defect
      is pre-existing rather than introduced
- [ ] G2 THE COLUMN CENSUS: every column the live table has, every column the
      source creates, and every column each writer fills. As a table in this
      intent
- [ ] G3 THE CANONICAL SCHEMA CREATES ALL THIRTEEN COLUMNS. Proven against a
      FRESH database, not this one
- [ ] G4 THE RULING on the operational columns is recorded: does
      `capture_snapshot` fill them, or do readers treat them as optional?
- [ ] G5 `timeline` LISTS THE 574 EXISTING ROWS, including the NULL-bearing
      automatic ones
- [ ] G6 A FAILED QUERY IS DISTINGUISHABLE FROM AN EMPTY TABLE. "No snapshots
      yet" is only printed when there are genuinely no snapshots
- [ ] G7 Proven on a database built FROM SOURCE with a clean HOME and
      `FAELIGHT_STATE_DB`, since that is the case the ALTER-only columns break
- [ ] G8 each gate carries evidence per INT-158

## Non-goals
- Migrating the 574 existing rows. They are the historical record; the columns
  are nullable and reading them is what must work.
- The snapshot feature's design.
- INT-192's tri-state contract. This intent is one instance; 192 owns the class.
