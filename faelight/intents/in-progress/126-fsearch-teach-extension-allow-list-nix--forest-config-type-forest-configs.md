---
id: 126
date: 2026-07-07
type: future
title: "fsearch: teach extension allow-list -- .nix + forest config types"
status: in-progress
tags: [faelight-shell, fsh, nix, fsearch, tooling]
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

---

## Why
fsearch's extension allow-list (faelight-shell/src/commands/mod.rs ~L3120) was
Arch-era: it omitted `.nix`, so fsearch silently skipped every Nix file and
returned a FALSE "no hits" on login/compositor config -- a dangerous false-clean.
Fix: add `nix` + forest config types (lua/conf/desktop/service/lock) to the
allow-list, and add a `--nix` shortcut flag for parity with --rust/--py/--toml.

## Gates
- [ ] `.nix` added to text_exts; workspace builds clean, zero warnings
- [ ] Live proof (deployed binary): `fsearch "services.greetd"` returns real
      .nix files (hosts/ or modules/), not only .md docs
- [ ] Live proof: `fsearch "faelight.desktop"` returns modules/desktop/*.nix
- [ ] `--nix` flag narrows correctly: `fsearch "desktop" --nix` shows only .nix hits
- [ ] Forest configs reachable: a `.conf`/`.lua`/`.desktop` search returns hits
- [ ] Deployed via rebuild + fresh terminal (not reload); prompt clean (no *)
- [ ] `d` before and after; charter records the one-word root cause

## The Rule
"A search that lies about 'no hits' is worse than no search. Make it see the
 whole forest -- especially the files that can strand the machine." 🌲
