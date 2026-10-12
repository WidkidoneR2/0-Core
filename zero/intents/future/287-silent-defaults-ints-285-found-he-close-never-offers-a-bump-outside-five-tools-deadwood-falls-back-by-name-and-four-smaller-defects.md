---
id: 287
date: 2026-10-12
type: future
title: "Silent defaults (INT-285) found: the close never offers a bump outside five tools, deadwood falls back by name, and four smaller defects"
status: planned
tags: [285, deadwood, cicomplete, ship, nsh]
---

## Vision
Every tool that changes is offered a version at the close, every binary a suite
spawns is resolved by one owner and refused when missing, and what a tool prints
says what it did.

## The Problem (measured 2026-10-12 during INT-285)
1. cicomplete offers bumps only for TRACKED_CRATES (engine intent/mod.rs:1205:
   novashell, engine, zero-git, friday-chat, db-browse), and bump-versions knows
   the same five (novashell commands/mod.rs:16925). nsh-test changed in e1a19f83,
   and cicomplete 285 closed with no question and no word that it skipped one.
   The 2.0.2 patch was written by hand after the close.
2. write_versions bumps Cargo.toml and Cargo.lock but never the registry row.
   Every bump needs a follow-up "registry:" commit (1bec9193, 454f5ba8, e6a47c75),
   and nsh-test 2.0.1 was missed until core integrity proposal #22 caught it.
3. nsh-test main.rs:86 run_deadwood reads DEADWOOD_BIN with a bare
   "zero-deadwood" default, the shape F-0020 had: PATH decides, and a missing
   target is not refused.
4. The deployed nsh 5.3.1 and a fresh release build of the same source differ
   in bytes. The deployed one predates the unshipped e8679d19 (test-only code).
   Cause not established.
5. ship --dry-run ends "1 shipped" when nothing was shipped.
6. nsh-test main.rs:183-187, repo_present's doc comment, sits above
   fixture_dir's, so rustdoc gives it to fixture_dir.
7. F-0020's text names main.rs:2646, which was :2764 by the time it was fixed.

## Success Criteria
- [ ] G0 census, read-only. Each item re-measured at its current file:line,
      with the shape of each fix and what each change reaches. Recorded here.
- [ ] G1 findings. Each of 1-6 is filed as a finding before any fix, found_by
      this intent.
- [ ] G2 one list of versioned crates. Ruling first: the source of truth
      (registry deployable rows, workspace members, or one shared list
      cicomplete and bump-versions both read). Red: a test where a touched
      crate outside today's five is not offered, shown failing. Fix: both
      read the one owner. Green. A crate that is touched but undecided still
      refuses the close.
- [ ] G3 the close writes the registry. Red: a bump test whose registry row
      stays old. Fix: write_versions updates the row in the same plan, and
      refuses before any write if the row is missing. Green, plus the drift
      check clean on a real close.
      Also any copy of a version hardcoded in source (277's fourth place): found at G0,
      and moved by the same plan or removed.
- [ ] G4 deadwood has one reader. The INT-285 pattern: red case, the reader
      refuses a missing target, green. nsh-test's count goes up by one.
- [ ] G5 why the nsh bytes differ. Two release builds of one commit compared,
      and the deployed binary against a build of its own commit. Cause
      recorded. A fix only if it is a defect; ship nsh is Christian's call.
- [ ] G6 ship --dry-run says what it would do. Red, fix, green: "would ship"
      in the summary, and the real ship's wording unchanged.
- [ ] G7 repo_present's doc back on repo_present. cargo doc clean, nsh-test
      unchanged.
- [ ] G8 ruling on F-0020: findings are history, so leave it with a note,
      or amend it.
- [ ] G9 doors: ship what changed, nsh-test all passing, d 0 failed, core
      intent validate clean.
- [ ] G10 commits: one per fix, each with Intent and Fixes and its plan Seal
      and note; pushed; trace reads every seal intact.
- [ ] G11 each gate carries evidence per INT-158.

## Non-goals
- No change to how nsh behaves. No new crates. No sudo. Omarchy untouched.
- No retroactive bumps for tools closed before this intent.
