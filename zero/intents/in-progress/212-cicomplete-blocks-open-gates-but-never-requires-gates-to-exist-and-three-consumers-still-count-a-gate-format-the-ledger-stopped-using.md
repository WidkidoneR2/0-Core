---
id: 212
date: 2026-08-09
type: future
title: "cicomplete blocks open gates but never requires gates to exist, and three consumers still count a gate format the ledger stopped using"
status: in-progress
tags: [ledger, intent, gates, enforcement]
depends_on: [294]
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

## Re-measured when INT-212 started (2026-10-22)

The line numbers in the Evidence above date from 2026-08-09 and are stale. Measured at commit 25fc081d in zero/engine/src/domains/intent/mod.rs:

- Enforcer: complete_intent at 2521; `if let Some(ref path)` at 2542; deferral check 2569-2593; open-gate scan 2594-2606. The file finder is written out at 2544/2553 and again at 2650/2664 in that one function.
- Folder lists, hand-written: complete_intent 2544 and 2650 (future, planned, deferred, in-progress); cancel_intent 4437 (in-progress, future, planned, deferred); defer_intent 4530 and override_intent 4577 (in-progress, future, complete); validate_issues LIFECYCLE 448 and FOLDERS 458; next_id LIFECYCLE 2808. Neither LIFECYCLE copy names planned/, which holds one intent (249) that next_id never scans.
- Finders by id: `starts_with(id)` at 1024, 2553, 2664, 3556, 3666, 3785; 3709 and 3860 test the prefix in another shape. The loose test has no `-` boundary: 22 of the 99 short ids (1 to 99) also match another intent's file by prefix, measured against future, planned, in-progress, complete and cancelled. A measured collision, not yet a demonstrated wrong completion; G5 demonstrates it.
- Two more holes, read from the code and not yet run. The open-gate scan exempts any line that contains `⏸` (2604) while the signature check only inspects lines that START with it (2573), so an open gate with a bare `⏸` appended is exempt unsigned. Both refusal paths, bad deferral at 2592 and open gates at 2628, `return Ok(())`, while the version refusal at 2638 returns an error: a refused cicomplete can read as success by exit status.
- Emoji counting is at three sites, 3464, 3584 and 3837, each `content.matches("⬜") + content.matches("✅")`, not the four named above.
- The intents base path has three owners: intents_dir(ctx) at mod.rs:45, zero_core::paths::intents_dir() at paths.rs:431, and ctx.fpath("intents").

## Gate format (ruled by INT-294 R1, 2026-10-22)

The section is `## Success Criteria`. Each item is one gate: `- [ ] G<n> text`, followed by an INT-158 evidence comment. `[x]` is done, and that includes a gate closed by declining it with numbered reasons. `[~]` is retired for new intents and still reads as open. A deferral keeps its existing form. The emoji form is legacy and is read only until its consumers migrate. Why this name: the scaffold at mod.rs:3020 already writes it, decision 142 names it as file content, and the tooling needs each item to be one independently checkable condition with evidence.

## Success Criteria

- [ ] G1 RED FIRST: a zero-gate intent completes TODAY, and a stray `- [ ]` outside this section blocks one, both demonstrated against the extracted checker before any change. Not by running cicomplete on a real intent
- [ ] G2 ONE pure function over file content owns the gate rule: it reads only the `## Success Criteria` section and returns each gate with its id, its state (open, done, deferred) and whether evidence follows. A gate is deferred only by the full signed form on its own line; a bare `⏸` on an open gate does not exempt it, shown red first against 2604. Unit-tested with no filesystem
- [ ] G3 cicomplete REFUSES an intent with no gates, naming that reason distinctly from open gates, and every refusal exits non-zero: the two refusal paths return Ok(()) today (2592, 2628), shown red first against the exit status
- [ ] G4 ONE folder source: one module-level LIFECYCLE set (future, planned, in-progress, complete, cancelled) with named subsets for active work and for searching, replacing the eight hand-written lists named above, so next_id scans planned/. A test reads the real intents/ tree and fails red when a directory is in neither the lifecycle set nor the record set, so a new folder cannot be missed silently again
- [ ] G5 ONE finder, find_intent_file, locates an intent by id in the lifecycle folders and replaces the copies named above. It matches the id exactly with a `-` boundary and accepts padded or unpadded input, shown red first against the 22 of 99 short ids that collide today. A missing file is a REFUSAL, not a skip: completion never proceeds because the check could not find what it checks
- [ ] G6 ONE gate-counting owner. health, predict_completion and story consume the G2 function instead of counting emoji; the three sites are 3464, 3584 and 3837 (measured at 25fc081d)
- [ ] G7 health reports a REAL gate count for a markdown-gated intent, shown before and after on the same intent (it reads 0/0 today)
- [ ] G8 grandfathering is STATED: 28 complete intents have no checkbox gate today (measured 2026-10-22, an upper bound because emoji gates are not excluded). The rule applies at cicomplete only, forward-only per INT-158; the 28 are listed, not rewritten
- [ ] G9 a tool answers does this intent have unmet gates, for any intent, and an intent in complete/ with an unticked gate is reportable (196 is one today). Absorbed from INT-232 G3 and G4
- [ ] G10 predict_completion take(10) is recorded or fixed: it takes ten files in readdir order, not the ten most recent
- [ ] G11 each gate carries evidence per INT-158

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
