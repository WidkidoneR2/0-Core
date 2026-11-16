---
id: 318
date: 2026-11-15
type: future
title: "Bug batch 2: name dropped intents, honest gate footer, doctor history table"
status: in-progress
tags: [bugs, ledger]
format: 2
---

## Vision
Three measured bugs get red tests before any fix. A dropped intent file is
named, a failing gate does not claim it could not run, and doctor history
reads the table that stores health.

## The Problem
parse_intent returns None for a file with no title, and read_open drops that
file without naming it. The ledger holds the file; the loader does not say
so (F-0002).

zero-gate prints its could-not-run line for a plain Fail, so a red test
reads as if the gate itself could not run (F-0016).

core doctor history queries horizon_snapshots for score and captured_at.
That table has id, horizon, snapshot, and created_at. The query fails with
no such column: score. Health history lives in health_patterns (F-0110).

## The Solution
The loader names every file it skips, and a valid file beside a dropped one
still loads. The could-not-run footer prints only when a gate is Unknown.
Doctor history reads health_patterns.

## Success Criteria
- [x] G1 Dropped intent files are named (F-0002). A file with id and status
      and no title is named by the loader, and the valid file beside it
      still loads.
      ask: open/901-notitle.md is named, and the valid row still loads
      ruling: Christian 2026-11-15: approved plan seal a747226fa8ce2c62
      evidence: f0002_read_open_names_dropped_file on d8e9e7e1: open/901-notitle.md
      is named, and 900-kept.md still loads.
- [x] G2 Honest gate footer (F-0016). A Fail-only run has no could-not-run
      footer. Unknown still prints it, and so does Fail together with
      Unknown. Pass and Skip do not.
      ask: Fail alone has no footer; Unknown has the footer
      ruling: Christian 2026-11-15: approved plan seal a747226fa8ce2c62
      evidence: f0016_fail_only_has_no_footer,
      f0016_unknown_only_has_footer, f0016_fail_and_unknown_has_footer, and
      f0016_pass_and_skip_have_no_footer on 3f8cf075.
- [x] G3 Doctor history table (F-0110). run_history returns Ok when health
      is a health_patterns row and horizon_snapshots has no score column.
      ask: run_history returns Ok against health_patterns
      ruling: Christian 2026-11-15: approved plan seal a747226fa8ce2c62
      evidence: f0110_history_reads_health_patterns on 1150dffc.

## Out of scope
- F-0012, F-0054, F-0013, and the other batches are out of scope.
