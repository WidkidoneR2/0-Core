---
id: 104
date: 2026-07-01
type: future
title: "Shell SnapShots Schema Intent"
status: complete
tags: [shell, fsh, Schema, Intent]
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

---

## Why
shell_snapshots has TWO conflicting schema definitions -- a latent inconsistency
found during INT-101 (2026-06-30). One table, two different column sets, reconciled
only by ALTER statements. Fragile and confusing; a debugging landmine.

## The inconsistency (exact locations)
- CREATE: commands/mod.rs:10349 creates shell_snapshots with columns:
    (name, timestamp, health, commits, processes, load_avg, top_proc, note)
- INSERT: db.rs:402 inserts into shell_snapshots with DIFFERENT columns:
    (name, timestamp, health, command, git_hash, cwd, intent_id)
- ALTER: db.rs:40-43 adds (command, git_hash, cwd, intent_id) via ALTER TABLE --
  these ALTERs are what let db.rs's INSERT work against mod.rs's table. So the
  table ends up with the UNION of both column sets, only because the ALTERs patch
  the gap at runtime.

## Why it currently "works" (and why that's fragile)
The db.rs ALTERs (INT-250-era) bolt the db.rs columns onto the mod.rs-created
table. So at runtime the table has all columns and both code paths function. But:
- There is NO single source of truth for shell_snapshots' schema.
- Two files must be kept mentally in sync; a change to one won't flag the other.
- A fresh-db ordering issue (cf INT-101) could re-expose it: if db.rs's ALTERs run
  before mod.rs's CREATE on a fresh db, they no-op (table absent) -- exactly the
  INT-101 class, but for shell_snapshots. (INT-101 fixed shell_HISTORY; snapshots
  was explicitly left out of scope and flagged here.)

## Desired outcome
One authoritative schema for shell_snapshots. Options:
- (A) Consolidate the full column set into the mod.rs CREATE (all 11 columns:
  name,timestamp,health,commits,processes,load_avg,top_proc,note,command,git_hash,
  cwd,intent_id) and drop the reconciling ALTERs -- single source of truth.
- (B) Decide which columns are ACTUALLY used. The two INSERTs (mod.rs:11019 uses
  commits/processes/load_avg/top_proc/note; db.rs:402 uses command/git_hash/cwd/
  intent_id) may represent two DIFFERENT snapshot purposes conflated in one table.
  Investigate whether this should be ONE table or TWO.

## Investigation needed first (do before choosing A vs B)
- Are BOTH INSERTs live? Which code paths call each? (mod.rs snapshot vs db.rs
  snapshot -- what triggers each?)
- Is one path dead code? If so, remove it and its columns.
- If both live and serve different purposes -> maybe two tables, not one.
This is "understand before you fix" -- do NOT just union the columns without
knowing whether the two INSERTs are two features or one confused one.

## Gates
- [ ] Both shell_snapshots INSERT paths traced; live-vs-dead determined
- [ ] Decision recorded: one table (unified schema) or two tables (split purpose)
- [ ] Single source of truth for the schema (no CREATE/INSERT column mismatch)
- [ ] Fresh-db proof: shell_snapshots correct on a brand-new db, no ALTER reliance
- [ ] No regression: existing snapshot save/read paths still work

## Notes
Found during INT-101 (shell_history fresh-db fix). Deliberately scoped OUT of 101
to keep that fix surgical. db.rs shell_snapshots ALTERs were left untouched there.
This is the follow-up. Not urgent (works today), but real -- the kind of latent
schema debt that bites during recovery or a fresh install.
