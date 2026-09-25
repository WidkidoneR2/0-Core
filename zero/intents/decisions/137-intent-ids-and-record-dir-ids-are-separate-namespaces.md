---
id: 137
date: 2026-07-09
type: decisions
title: "Intent IDs and record-dir IDs are separate namespaces"
status: complete
tags: [ledger, numbering]
---

## The contradiction
Two implementations of the same rule, each with a comment insisting it is correct:

- `engine/src/domains/intent/mod.rs:1281` and `:2359` --
  `active_folders = ["complete","future","in-progress","decisions"]`, commented:
  *"INT-070 fix (regressed): decisions/ SHARES the numeric ID space, so it MUST be
  scanned -- excluding it caused 121 collisions."*
- `rust-tools/intent/src/main.rs:155` -- excludes the record dirs, commented:
  *"INT-077: only the intent-lifecycle dirs are numbered work-intents. The record dirs
  (decisions/, incidents/, experiments/, philosophy/) carry their own numbering."*

They cannot both be right.

## The evidence (2026-07-09, from the directory listing)
- `decisions/002-faelight-bar` and `decisions/002-versioning-strategy` had the SAME number,
  twice, in the SAME directory. Impossible under a shared namespace.
  (RESOLVED 2026-07-10, INT-135 Gate 5: faelight-bar renumbered to decisions/139 --
  its `type: future` revealed it was a misfiled intent, not a decision.)
- `001` exists independently in `decisions/`, `incidents/`, `experiments/`, `philosophy/`.
- `incidents/` also carries date-stamped files (`2026-02-03-...`), no number at all.

## The ruling
**Record dirs number themselves. Intent dirs number themselves. Separate namespaces.**
INT-077 is correct. INT-070's comment memorializes a misdiagnosis: `decisions/121` and an
intent 121 were never a collision -- they are different namespaces, exactly as
`philosophy/001` and `incidents/001` are.

## Consequences -- both implementations violate this ruling. BOTH STILL OPEN.
1. `engine` `active_folders` includes `decisions` -> intent IDs inflate to the decisions
   high-water mark. Must drop `"decisions"`. (Currently masked: `core intent new` is broken.)
2. `rust-tools/intent` computes `next_id` from the INTENT dirs, then writes the file into
   whichever category the user picks. Choose `decisions` and you get an intent-sequence
   number filed in a record dir. DEMONSTRATED today: it created `decisions/135`, colliding
   with `decisions/135-rio-terminal`. Fix: scan the CHOSEN category, not the intent dirs.

## What was fixed today (commit 039e1211)
`rust-tools/intent/src/main.rs:155` listed `"cancelled"` in the scan, so a cancelled
intent's retired number set the high-water mark (`cancelled/277` -> next id 278). Removed.
Proven on the deployed binary: `intent add` offered 135 after 134. A cancelled intent's
number is RETIRED, not reserved.

## Also open (same crate, noted not fixed)
- `intent cancel <id> --reason "x"` stores the literal string `--reason` as the reason.
- `core intent new` errors -- the `templates/` dir was lost in the INT-061 tree move.

## Relationship
Supersedes the reasoning in INT-070's code comment. Confirms INT-077's principle (which was
never fully implemented). The duplicated-scan-drift is INT-115's class: one rule, two copies,
diverged, each documented as settled.
