---
id: 106
date: 2026-07-01
type: future
title: "paths.rs consolidation follow-ups: rename rules_dir, fix hardcoded font, route hardcoded paths through the module"
status: complete
tags: [paths, faelight-core, refactor, cleanup]
---

## Why
Cleanup threads discovered during INT-105 (paths.rs realignment). None block
anything; all are real, small, and belong together as the "finish centralizing
paths" family. Captured so they're not lost.

## The three follow-ups

### 1. Rename rules_dir() -> policy_dir() (cosmetic, naming honesty)
paths.rs `rules_dir()` now points at `policy/` (fixed in INT-105) but is still
NAMED "rules" -- a leftover from the 02-rules era. Rename the function to
`policy_dir()` and update its ~4 call sites (faelight-hooks/checks/secrets.rs,
install.rs, status.rs, etc.). Pure naming; behaviour already correct.

### 2. Fix hardcoded font path in faelight-core/src/lib.rs:29
`include_bytes!("/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf")` is an
absolute path that breaks the TEST build in the Nix sandbox (surfaced when running
`cargo test -p faelight-core` during INT-105). Same hardcoded-filesystem-assumption
class as the paths drift, but for a font asset. Options: embed via a Nix-provided
path, make it a runtime lookup with graceful fallback, or feature-gate the test.
Goal: `cargo test -p faelight-core` runs clean.

### 3. Route the 40+ hardcoded path STRINGS through paths.rs (the big one)
INT-105 fixed the paths.rs MAP, but ~40 files across engine + rust-tools still
BYPASS it -- hardcoding "intents/future", "registry/tools.toml",
"0-core/runtime/state.db", etc. as string literals (some even still say the
Arch-era "01-registry"/"00-meta"). This is why drift was invisible: no single
authority. Migrate these call sites to use paths:: functions, tool by tool,
rebuilding + testing each. This is what makes future dir moves (INT-061 v2 tree)
a one-line change instead of a 40-file sweep. Do incrementally, never big-bang.

## Relationship
- Depends on / follows INT-105 (paths.rs map now correct).
- Enables INT-061 (v2 tree moves become cheap once #3 is done).
- Bridge toward the horizon: structure declared in Nix, code reads it.

## Success criteria
- [x] rules_dir() renamed to policy_dir(); call sites updated; builds clean. <!-- STAMP-106-DONE / INT-130 2026-07-10: VERIFIED IN SOURCE -- paths.rs:80 policy_dir() (rules_dir gone), call sites at 85/89, test assert at 420 passes ('policy'). Commit 73687ec5. -->
- [x] faelight-core test build no longer fails on the font path. <!-- INT-130 2026-07-10: commit 73687ec5 -- font path changed from include_bytes! absolute to runtime std::fs::read with graceful skip-if-absent; 'cargo test -p faelight-core runs clean (11 passed)'. -->
- [~] Hardcoded path strings migrated to paths:: (tracked per-tool; may span
      multiple sessions -- this intent can complete in stages or spawn sub-intents). <!-- INT-130 2026-07-10: DEFERRED via authorized split -- this gate's own text permits 'spawn sub-intents'. #3 was split to INT-115 (future/115-route-hardcoded-path-strings-through-pathsrs-per-tool.md, VERIFIED to exist) per commit 73687ec5 ('#3 split to INT-115, not a 1.0.0 blocker'). Marked [~], not [x]: the migration itself is NOT done -- it lives on in INT-115. 106's quick wins (#1,#2) are done; the big item is legitimately tracked elsewhere. -->

## Notes
Surfaced 2026-07-01 during INT-105. #1 and #2 are quick; #3 is substantial and
may warrant its own dedicated sessions or a split into per-tool intents.
