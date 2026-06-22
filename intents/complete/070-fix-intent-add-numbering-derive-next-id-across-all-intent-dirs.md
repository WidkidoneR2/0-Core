---
id: 070
date: 2026-06-20
type: future
title: "Fix intent-add numbering: derive next id across all intent dirs"
status: complete
tags: [intent-ledger, tooling, bug, numbering, rust, nixos]
---

## Why
The `intent add` wizard derives the next id from only the visible dirs (future +
in-progress), not complete/ or cancelled/. So once an id is completed and moved to
complete/, the wizard reuses that number -- it just handed 069 (Faelight-FM) a duplicate
068, corrected by hand. This recurs on every completion: quiet data-corruption in the
core ledger tool.

## What
- Next-id must consider ALL intent dirs: future, in-progress, complete, cancelled.
- Either next id = (max existing id across all dirs) + 1, or a monotonic counter that
  never reuses a number.
- A fresh add right after a completion must not collide.

## Approach
The intent tool is Rust (v3.0.0, rust-tools). Phase 0 locates the next-id function in the
add path. Phase 1 fixes it to scan every intent dir (or read a persistent counter).
Phase 2 demonstrates a non-colliding number with a completed intent sitting in complete/.

## Phases
Phase 0 -- locate the next-id logic; confirm it skips complete/cancelled.
Phase 1 -- derive next id across all dirs (or monotonic counter).
Phase 2 -- demonstrate: a fresh add after a completion assigns a clear number.

## Results (2026-06-22)
- Gate 1 (locate): the bug was get_next_id(category) in rust-tools/intent/src/main.rs -- it
  scanned only get_intent_dir().join(category), the single target dir. Adding a future intent
  maxed future/ alone (067), never seeing complete/068, so it reused 068.
- Gate 2 (fix): get_next_id now scans every subdirectory of the intents dir for the global max
  and drops the category param (the id is global, not per-category). Two passes: the first cut
  scanned all subdirs unguarded and read the date-stamped incident files (incidents/2026-...)
  as id 2026, handing out 2027; fixed by counting only 3-digit numeric prefixes (NNN-), which
  excludes date-named files. Call site updated.
- Gate 3 (demonstrate): dropped a 900- marker in cancelled/ (the most-ignored dir), ran intent
  add -> it assigned 901, proving the cross-dir scan AND that the 2026- incident files are
  ignored. Old code would have given 077. Marker + throwaway removed; validate clean.

## Gates
- [x] Phase 0: next-id function located; complete/cancelled skip confirmed and noted here
- [x] next-id derives from all intent dirs (future + in-progress + complete + cancelled)
- [x] a fresh `intent add` after a completion assigns a non-colliding number

## Notes
- Natural neighbour of INT-031 (release machinery), tracked separately at your request.
- File-based ledger: ids live in frontmatter + filename prefix; fix is in the add path only.

## The Rule
"The ledger must never hand out the same number twice." 🌲
