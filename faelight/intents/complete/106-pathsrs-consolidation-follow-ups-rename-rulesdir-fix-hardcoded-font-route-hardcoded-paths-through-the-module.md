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
- [ ] rules_dir() renamed to policy_dir(); call sites updated; builds clean.
- [ ] faelight-core test build no longer fails on the font path.
- [ ] Hardcoded path strings migrated to paths:: (tracked per-tool; may span
      multiple sessions -- this intent can complete in stages or spawn sub-intents).

## Notes
Surfaced 2026-07-01 during INT-105. #1 and #2 are quick; #3 is substantial and
may warrant its own dedicated sessions or a split into per-tool intents.
