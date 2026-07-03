---
id: 115
date: 2026-07-02
type: future
title: "Route hardcoded path strings through paths.rs (per-tool)"
status: planned
tags: [paths, faelight-core, refactor]
---

## Why
Split out of INT-106 (#3) to keep 106 closeable on its two bounded fixes. INT-105
fixed the paths.rs MAP, but ~40 files across engine + rust-tools still BYPASS it --
hardcoding path strings like "intents/future", "registry/tools.toml",
"0-core/runtime/state.db" as literals. Some still use Arch-era names ("01-registry",
"00-meta"). This is why path drift was historically invisible: no single authority.

## The problem
paths.rs is the single path authority, but ~40 call sites hardcode strings instead
of calling paths:: functions. A directory move (e.g. the v2 tree, INT-110/112) would
require a 40-file sweep instead of a one-line change in paths.rs.

## The work (incremental, never big-bang)
Migrate hardcoded path strings to paths:: functions, TOOL BY TOOL, rebuilding +
testing each tool before moving on. Priority order:
1. Arch-era WRONG names first ("01-registry", "00-meta", "02-rules" etc) -- these are
   actively incorrect, not merely unrouted. Find via:
   grep -rn "01-registry\|00-meta\|02-rules\|03-interfaces" faelight --include="*.rs"
2. Then the merely-hardcoded-but-correct strings ("intents/future", "registry/...",
   "runtime/state.db") -- route through the matching paths:: function.

## Approach
- One tool per pass. grep the tool for string literals that duplicate a paths::
  function's target; replace with the function call; rebuild that tool; verify.
- Add paths:: accessors where none exists yet for a needed path.
- Never big-bang all 40 -- each is build-gated, like the INT-061 moves.

## Success criteria
- [ ] Zero Arch-era path names ("0X-*") remain in code
- [ ] Hardcoded path strings that duplicate a paths:: target are routed through it
- [ ] Each tool build-gated + verified as migrated
- [ ] (stretch) a lint/grep guard that flags new hardcoded path strings

## Relationship
- Follows INT-105 (paths.rs map correct) + INT-106 (#1 rename, #2 font -- both done).
- ENABLES the v2 tree (INT-110/112): dir moves become one-line paths.rs changes.
- NOT a 1.0.0 blocker: current tree works; this is debt-reduction for the v2 restructure.

## Notes
Explicitly multi-session / may split into per-tool sub-intents. Was INT-106 #3.
