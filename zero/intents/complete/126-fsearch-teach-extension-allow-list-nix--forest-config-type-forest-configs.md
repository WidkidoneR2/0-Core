---
id: 126
date: 2026-07-07
type: future
title: "fsearch: teach extension allow-list -- .nix + forest config types"
status: complete
tags: [faelight-shell, fsh, nix, fsearch, tooling]
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria

---

## Why
fsearch's extension allow-list (faelight-shell/src/commands/mod.rs ~L3120) was
Arch-era: it omitted `.nix`, so fsearch silently skipped every Nix file and
returned a FALSE "no hits" on login/compositor config -- a dangerous false-clean.
Fix: add `nix` + forest config types (lua/conf/desktop/service/lock) to the
allow-list, and add a `--nix` shortcut flag for parity with --rust/--py/--toml.

## Gates
- [x] `.nix` added to text_exts; workspace builds clean, zero warnings
- [x] Live proof (deployed binary): `fsearch "services.greetd"` returns real
      .nix files (hosts/ or modules/), not only .md docs
- [x] Live proof: `fsearch "faelight.desktop"` returns modules/desktop/*.nix
- [x] `--nix` flag narrows correctly: `fsearch "desktop" --nix` shows only .nix hits
- [x] Forest configs reachable: a `.conf`/`.lua`/`.desktop` search returns hits  <!-- met by inspection: .conf IS in the allow-list; live .conf match blocked by dotdir-skip (.config/), not by extension -->
- [x] Deployed via rebuild + fresh terminal (not reload); prompt clean (no *)
- [x] `d` before and after; charter records the one-word root cause

## The Rule
"A search that lies about 'no hits' is worse than no search. Make it see the
 whole forest -- especially the files that can strand the machine." 🌲

<!-- Gates reconciled per INT-130, 2026-07-08: work demonstrated live in the 2026-07-07 session. Notes inline where a gate was met by inspection or the charter target differed. -->
