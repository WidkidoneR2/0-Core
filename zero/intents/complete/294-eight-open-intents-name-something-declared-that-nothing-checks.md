---
id: 294
date: 2026-10-17
type: future
title: "eight open intents name something declared that nothing checks"
status: complete
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
- [x] G0 census, read-only: the eight bodies, decision 142 and the Zero laws read; a table of declared,
      where declared, and what checks it; overlaps marked; a verdict on whether the eight share one
      defect. Evidence is the table, kept in ~/.cache/zero.
<!-- evidence: 2026-10-22. The census table is kept at ~/.cache/zero/int294-g0-census.md (sha256 prefix 803b2d0471b6): the eight bodies, decision 142 and Zero laws 1 to 3 read (the laws from Christian statement of 2026-08-18; fsearch found no repo file for them). Verdict: the eight share a symptom, declared and nothing checks it, not one defect. Four groups: A 211 232 212, B 231 235, C 233 234, D 226. Ledger counts measured the same day are in the file. -->
- [x] G1 rulings, each approved by Christian: the canonical gate format, and for each of the eight
      whether it stays, merges or is cancelled, with the reason.
<!-- evidence: ruled and approved by Christian 2026-10-22 in session. R1 gate format: the section is Success Criteria, each item is G<n> with an evidence comment, [x] includes a gate declined with numbered reasons, [~] is retired for new intents, the deferral form is unchanged, the emoji form is legacy (recorded in INT-212 under Gate format). R2: date means the date filed (recorded in INT-232 G2, the measurement against git is still open there). R3 to R10: 211 merged into 232 and cancelled; 232 narrowed to the axes gate; 212 keeps the code and its gates are rewritten; 226 231 233 234 235 stay, each with an INT-294 ruling block naming its gate against the gap. -->
- [x] G2 the plan: for each survivor, one line stating what would fail red, and its depends_on edges
      checked against the rule what would break if I started anyway.
<!-- evidence: 2026-10-22, plan applied under fpatch Seal d8137f5c6416c649. What fails red, one per survivor: 232 a status-agrees-with-directory check (planned/ holds a file with status in-progress, decisions/ holds complete and planned); 212 a zero-gate fixture completes and a stray checkbox outside the criteria blocks; 226 cicomplete is silent when no commit carries the id; 231 a collision fixture passes the existence check; 235 auto-checkpoint reports the deleted restore action while restart-service passes; 233 e733287d shows 15 scripts_dir bypasses; 234 the observable-or-not answer is recorded after a demonstrated attempt. Edges: 212 depends_on [211] became [294], because starting 212 first would code against an unruled gate format; a scan of depends_on and blocks found no other edge touching the nine; 226 231 232 233 234 235 need none. -->
- [x] G3 the ledger changed through the tools per the rulings; core intent validate clean; no file
      edited by hand to cancel or merge.
<!-- evidence: 2026-10-22. Bodies changed by the fpatch Plan with Seal d8137f5c6416c649 (plan text kept at ~/.cache/zero/int294-plan-text-d8137f5c6416c649.txt, seven files, each verified on disk). 211 cancelled with core intent cancel 211 --reason, which wrote the reason into the file. Findings F-0052 and F-0053 filed with core intent find. core intent validate printed All 334 intents valid before and after. No file was edited by hand. -->
- [x] G4 live. `core intent next` and `core intent blocked` give true answers over the new graph.
<!-- evidence: 2026-10-22. core intent blocked lists INT-212 waiting on INT-294 (in-progress) and does not list 211. core intent next recommends INT-223, lists 226 as ready, and names 212 as not recommended, waiting on INT-294. -->
- [x] G5 doors: no code is shipped;
      nsh-test all passing; d 0 failed; core intent validate clean.
<!-- evidence: 2026-10-22. No code changed: git status lists only paths under zero/intents (the INT-280 lifecycle move beside them belongs to INT-280, not this intent), and nothing was shipped. nsh-test with the debug binary: 258 / 258 passed in 51.3 s. d: 26 passed, 2 warnings (Git Repository uncommitted changes, Update Readiness hold off because of tracked changes; both expected before the commit), 0 failed, integrity 100%. core intent validate: All 334 intents valid. -->
- [x] G6 commits: Intent INT-294 and the plan Seal with its note under refs/notes/seals; pushed
      with the pre-push hook passing; main and refs/notes/seals reach GitHub; core intent trace
      reads every seal intact.
<!-- evidence: 2026-10-22. Work commit 1302c2ed carries Intent: INT-294, Finding: F-0052, Finding: F-0053 and Seal: d8137f5c6416c649; its note is under refs/notes/seals, attached with git notes add -C. Pushed with the zero-gate pre-push hook accepting both pushes: main 4e58cd63..1302c2ed in 52.7 s, refs/notes/seals cba1b3ef..e202715a in 52.2 s. git ls-remote origin and git rev-parse agree: main 1302c2edf0cd89a793bb790fa2e953772e40020b, refs/notes/seals e202715a871e387a814866b8d93067b4fdf79b2c. core intent seals, read when this correction was made, lists 1302c2ed with seal d8137f5c6416c649 and verdict sealed, and counts 15 rows: 11 sealed, 4 unsealed (no seal recorded), 0 with any other verdict; core intent trace F-0052 reads the finding as open (no commit on HEAD carries Fixes: F-0052 yet). The close commit and its push follow this tick. -->
- [x] G7 each gate carries evidence per INT-158.
<!-- evidence: 2026-10-22. Checked by the script that applied this plan, on the text as it will read: each gate from G0 to G7 is followed by a line starting with an evidence comment, 8 of 8, and the plan was refused before any write when one was missing. -->

## Non-goals
- No change to the ledger format before G1 rules the gate format.
- Open intents outside the eight are not touched.
- No rebuild of the intent tool and no code in this intent.
- No new crates. No sudo. Omarchy untouched.

## Dependencies
depends_on: none. It orders 211 and 212 and may absorb them; they are inputs, not blockers.
