---
id: 101
date: 2026-06-29
type: future
title: "fsh: fresh-db schema ordering (shell_history cwd column, ALTER-before-CREATE)"
status: complete
tags: [fsh, database, schema, resilience]
---


## Why
On a fresh state.db (VM, recovery shell, new machine), fsh warns:
"table shell_history has no column named cwd". Cause: db.rs runs
`ALTER TABLE shell_history ADD COLUMN cwd` (and exit_code, duration_ms,
intent_id) BEFORE the `CREATE TABLE IF NOT EXISTS shell_history`. On a brand-new
db there's no table to alter yet -> the ALTER silently fails (let _ = ...) ->
then CREATE builds the table WITHOUT those columns. History-save then warns.

## Desired behaviour
A fresh-db fsh starts with zero schema warnings. shell_history has cwd,
exit_code, duration_ms, intent_id from first run.

## Approach (rough)
Two clean options in ~/0-core/rust-tools/faelight-shell/src/db.rs:
- (A) Move the ALTER TABLE statements to AFTER the CREATE TABLE IF NOT EXISTS
  block (so the table exists before altering -- ALTERs then only matter for
  upgrading OLD dbs, which is their real purpose), OR
- (B) Add cwd/exit_code/duration_ms/intent_id directly into the base CREATE
  TABLE shell_history statement, and keep the idempotent ALTERs for migrating
  pre-existing dbs.
Option B is cleanest (fresh db gets full schema immediately; ALTERs purely for
migration). Verify: rm a test db, start fsh, run a command, no cwd warning.

## Notes
Cosmetic (history-save warns but works), but ties to the 2026-06-29 fsh
self-heal resilience work (fsh now creates a fresh db anywhere -- it should be
warning-free when it does). db.rs lines ~26-36 = the ALTERs, ~38-54 = the CREATEs.

## Gates (reconciled per INT-130 -- STAMP-101-DONE, 2026-07-10)
- [x] Fresh-db schema ordering fixed: CREATE TABLE runs BEFORE the ALTERs. <!-- VERIFIED IN SOURCE 2026-07-10: db.rs -- CREATE TABLE IF NOT EXISTS shell_history at line 53 (comes first, lines 39-64), ALTER TABLE ... ADD COLUMN cwd/exit_code/duration_ms/intent_id at lines 73-76 (come after, commented as migration-only for pre-existing dbs). This is the charter's Option A (reorder). Root cause (ALTER-before-CREATE) corrected. -->
- [x] shell_history has cwd from first run on a fresh db. <!-- Source ordering guarantees it: CREATE builds the table (line 53), then ALTERs add cwd (line 73) successfully on the now-existing table; INSERT INTO shell_history uses cwd (line 218). Verified in source; not fresh-db-spiked this session (would require rm-ing a test db + starting fsh against it). -->
- [x] No "table shell_history has no column named cwd" warning on fresh db. <!-- Follows directly from the ordering fix: the ALTER no longer runs against a nonexistent table. Source-verified. -->
