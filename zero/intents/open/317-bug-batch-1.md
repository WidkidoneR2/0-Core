---
id: 317
date: 2026-11-15
type: future
title: "Bug batch 1: integrity proportion, validate exit, quiet busctl"
status: planned
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
- [ ] G1 Integrity proportion (F-0012). A weight-1 remaining issue and a
      weight-5 remaining issue give different percentages, and the weight-5
      one is lower. Neither is 67.
      ask:
      ruling:
      evidence:
- [ ] G2 Validate exit (F-0054). validate returns an error when the ledger
      has any issue, and Ok when the ledger is clean.
      ask:
      ruling:
      evidence:
- [ ] G3 Quiet busctl (F-0013). desktop() prints nothing from busctl.
      ask:
      ruling:
      evidence:

## Out of scope
- The fixes for F-0012, F-0054, and F-0013. This intent's first commit is
  the red tests only.

## Versions
- none. The red tests do not change a crate version.
