---
id: 317
date: 2026-11-15
type: future
title: "Bug batch 1: integrity proportion, validate exit, quiet busctl"
status: in-progress
tags: [bugs, ledger]
format: 2
---

## Vision
Three measured bugs get red tests before any fix. The doctor shows a
proportion, validate's exit code is something a gate can trust, and a
desktop notification does not print the bus reply.

## The Problem
integrity_pct is 67 whenever anything is found and nothing is auto-fixed.
total_weight is three times the found weight, so the Integrity line shows
presence, not proportion (F-0012).

core intent validate exits 0 when it reports invalid intents, so no gate
or script can depend on its exit code (F-0054).

notify desktop runs busctl with .status(), so the busctl reply prints into
the doctor output (F-0013).

## The Solution
A remaining issue's weight changes the percentage, and a heavier remaining
issue scores lower. validate returns an error for every issue, including
ones that are not format 2. desktop notification output stays on the bus
and does not print into the caller.

## Success Criteria
- [x] G1 Integrity proportion (F-0012). A weight-1 remaining issue and a
      weight-5 remaining issue give different percentages, and the weight-5
      one is lower. Neither is 67.
      ask: weight 1 and weight 5 give different percentages, weight 5 lower, neither 67
      ruling: Christian 2026-11-15: approved plan seal e13ecf18d135a511
      evidence: f0012_integrity_pct_is_a_proportion on 4cc55f9b: weight 1 = 75%,
      weight 5 = 38%.
- [x] G2 Validate exit (F-0054). validate returns an error when the ledger
      has any issue, and Ok when the ledger is clean.
      ask: validate errors on any ledger issue, and is Ok when the ledger is clean
      ruling: Christian 2026-11-15: approved plan seal e13ecf18d135a511
      evidence: f0054_validate_errs_on_any_issue and
      f0054_validate_ok_on_clean_ledger on 29b20b73; real ledger validate exits 0.
- [x] G3 Quiet busctl (F-0013). desktop() prints nothing from busctl.
      ask: desktop() prints nothing from busctl
      ruling: Christian 2026-11-15: approved plan seal e13ecf18d135a511
      evidence: f0013_desktop_notify_prints_nothing on 3d86e385.

## Out of scope
- F-0002, F-0036, F-0040, and the other batches are out of scope.

## Versions
- core: patch (fixes three promised behaviours)
