---
id: 294
date: 2026-10-17
type: future
title: "eight open intents name something declared that nothing checks"
status: planned
tags: [ledger, intent, enforcement, design]
---

## Vision
The system checks what it declares. The ledger has one design for declared against checked, one
canonical gate format, and the eight intents that circle that gap either collapse into the design
or each keep one gate against it. Design first; no code in this intent.

## The Problem
Measured 2026-10-17 from the open-intent listing. By title, eight future intents each name
something declared that nothing checks:

- 211 gate has four spellings and date means four things (the body is still the empty template)
- 212 cicomplete blocks open gates but never requires gates to exist (10 gates open, none done)
- 226 work done without an intent leaves no evidentiary record
- 231 code cites intents that do not exist and nothing notices
- 232 the ledger cannot mechanically answer whether an intent is done
- 233 canonical ownership is not machine-checkable
- 234 edits bypass the mutation primitive and nothing verifies the write landed
- 235 a declared capability is never checked to resolve

Only 211 and 212 were read. The other six are taken from their titles, so G0 verifies that they
share one defect before anything is merged. INT-158 says of the evidence convention that nothing
enforces it. During INT-290 two checks were written by hand because the ledger has none: a hash
check of the saved plan texts against their Seals, and a per-gate evidence check run before the
last tick. Decision 142 (store facts, derive conclusions) and the eight Zero laws are the inputs.

## The Solution
Read all eight bodies, decision 142 and the Zero laws. Tabulate, per intent, what is declared, where it is
declared, and what checks it today (nothing, or what). Rule the canonical gate format first, from 211,
because 212 and 232 depend on it. Then keep, merge or cancel each intent with a reason written in the
ledger, through the tools and not by hand. Any code a ruling needs becomes its own intent.

## Success Criteria
- [ ] G0 census, read-only: the eight bodies, decision 142 and the Zero laws read; a table of declared,
      where declared, and what checks it; overlaps marked; a verdict on whether the eight share one
      defect. Evidence is the table, kept in ~/.cache/zero.
- [ ] G1 rulings, each approved by Christian: the canonical gate format, and for each of the eight
      whether it stays, merges or is cancelled, with the reason.
- [ ] G2 the plan: for each survivor, one line stating what would fail red, and its depends_on edges
      checked against the rule what would break if I started anyway.
- [ ] G3 the ledger changed through the tools per the rulings; core intent validate clean; no file
      edited by hand to cancel or merge.
- [ ] G4 live. `core intent next` and `core intent blocked` give true answers over the new graph.
- [ ] G5 doors: no code is shipped;
      nsh-test all passing; d 0 failed; core intent validate clean.
- [ ] G6 commits: Intent INT-294 and the plan Seal with its note under refs/notes/seals; pushed
      with the pre-push hook passing; main and refs/notes/seals reach GitHub; core intent trace
      reads every seal intact.
- [ ] G7 each gate carries evidence per INT-158.

## Non-goals
- No change to the ledger format before G1 rules the gate format.
- Open intents outside the eight are not touched.
- No rebuild of the intent tool and no code in this intent.
- No new crates. No sudo. Omarchy untouched.

## Dependencies
depends_on: none. It orders 211 and 212 and may absorb them; they are inputs, not blockers.
