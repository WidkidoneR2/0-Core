---
id: 212
date: 2026-08-09
type: future
title: "cicomplete blocks open gates but never requires gates to exist, and three consumers still count a gate format the ledger stopped using"
status: planned
tags: [ledger, intent, gates, enforcement]
---

## Vision
"Complete" means the gates were demonstrated. An intent with no gates cannot reach it, and a
checker that cannot find the file refuses rather than shrugs. One definition of a gate, one owner
of the count, so the ledger cannot disagree with itself about its own central discipline.

## The Problem
The enforcement exists and is half-blind. INT-332 blocks cicomplete when open gates are present --
it reads the markdown format INT-130 established, lists each offender, prints the deferral syntax,
and it works. But it never asks whether a gate exists at all, so a file with zero gate lines walks
straight through. That is how 123 intents came to be marked done without their work being done, and
the empty stubs that remain (145, 010, 013, 014) are the same door still standing open.

Two smaller failures sit beside it. The check is wrapped in `if let Some(ref path)`, so a file the
search cannot locate produces no check rather than a refusal -- and the search uses a hand-written
folder list naming `planned` and `deferred`, neither of which exists in this tree. That is a FOURTH
divergent folder list in the ledger tooling, which is precisely the disease INT-135 gate 7 was
filed to end.

And "gate" has two meanings inside this one file. complete_intent counts markdown; health,
predict_completion and story count the `⬜`/`✅` emoji that INT-130 superseded. The cost is not
cosmetic: a correctly gated intent yields total_gates = 0, so health scores it at zero percent.
The intents following the current convention look like the worst ones in the ledger.

## The Solution
Extract the rule before changing it. The gate check is an inline closure inside complete_intent
today, which is why it has never been tested; lifting it into one pure function over file content
makes G1's red-first demonstration cheap and every later change checkable. Then the three additions
are small: require that gates exist, treat a missing file as a refusal, and derive the folder set
from the same source the rest of the ledger uses rather than a fourth list.

The counting consumers migrate to one owner afterwards, not before -- the enforcer is the site with
teeth, and fixing the reporting first would leave the hole open while making the dashboard prettier.

## Evidence (measured 2026-08-09, not asserted)
- 123 intents were marked done that were not -- the audit that motivates this.
- The check EXISTS: complete_intent, intent/mod.rs:999, INT-332. It reads markdown `- [ ]` / `- [~]`
  and legacy `⬜`, refuses, lists each open gate, prints the deferral syntax. It works.
- THE HOLE: it never requires a gate to EXIST. An intent with zero gate lines passes trivially.
  INT-145 is an empty 11-line template; 010/013/014 are 0-gate stubs; 179's gate was a bare
  placeholder rewritten at completion.
- SECOND HOLE: the whole check sits inside `if let Some(ref path)`. No file found means no check --
  completion proceeds silently.
- THE FOLDER LIST IS PHANTOM: line 1002 searches ["future", "planned", "deferred", "in-progress"].
  `planned` and `deferred` do not exist in this tree. A FOURTH divergent folder list, which is the
  disease INT-135 gate 7 was filed to end.
- TWO DEFINITIONS OF "GATE" IN ONE FILE: the enforcer counts markdown; health (2064),
  predict_completion (2188 and 2197) and story (2445) count `⬜`/`✅` emoji. INT-130 established
  markdown is the real format.
- CONSEQUENCE, not just drift: a fully markdown-gated intent yields total_gates = 0, so health
  computes gate_pct = 0. Correctly gated intents score as the worst ones.

## Non-goals
- Rewriting the deferral mechanism. `⏸ ... approved by: christian <date>` works and stays.
- Removing `override_intent`. A deliberate, recorded escape hatch is not the defect.
- Retroactively gating record dirs (decisions/incidents/philosophy/experiments). They are not
  lifecycle intents and do not carry gates.

## Success Criteria
- [ ] G1 RED FIRST: a zero-gate intent completes TODAY, demonstrated before any change. Prove it
      against the extracted checker rather than by running cicomplete on a real intent -- completion
      checkpoints, bumps a version and emits events, so it is not a cheap probe
- [ ] G2: the gate rule is extracted into ONE pure function over file content, unit-testable with no
      filesystem. Today it is an inline closure inside complete_intent and cannot be tested at all
- [ ] G3: cicomplete REFUSES an intent with no gates, naming that reason distinctly from "open gates"
- [ ] G4: the phantom folder list is resolved -- the folders searched are the folders that exist,
      derived from one source rather than a fourth hand-written list
- [ ] G5: a missing file is a REFUSAL, not a skip. Completion must never proceed because the check
      could not find what it was checking
- [ ] G6: ONE gate-counting owner. health, predict_completion and story consume it instead of
      counting emoji; the four sites are named with line numbers as evidence
- [ ] G7: health reports a REAL gate count for a markdown-gated intent -- shown before and after on
      the same intent, since it currently reads 0/0
- [ ] G8: the grandfathering policy is STATED in this intent, not discovered later: what happens to
      the existing zero-gate intents when the rule lands
- [ ] G9: predict_completion's `take(10)` is recorded or fixed -- it takes ten files in readdir
      order, not the ten most recent, so "from N recent intents" is false today
- [ ] G10: each gate carries evidence per INT-158

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
