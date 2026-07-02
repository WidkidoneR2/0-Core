---
id: 108
date: 2026-07-02
type: future
title: "profile .profile-mechanism"
status: planned
tags: [profile]
---

## Vision
The profile tool speaks ONE storage model, honestly. Right now it has two competing
models fighting: a dead Arch-era file-per-profile mechanism and the live TOML config.
Consolidate on the TOML model (the one that actually works on NixOS), remove the dead
mechanism. Naming and location derivable, no phantom code paths.

## The Problem
`profile` has TWO storage models:

1. DEAD (file-per-profile): `get_profile_dir()` -> `paths::profiles_dir()` ->
   `config/profiles` -- a directory that DOES NOT EXIST and contains ZERO `.profile`
   files (verified 2026-07-02: `find ~/0-core/config -name "*.profile"` and
   `find ~/.config -name "*.profile"` both empty). Yet 7 call sites depend on it:
   cmd_list (list), cmd_edit (edit a .profile in $EDITOR), cmd_export, import (dest),
   the doctor dir-check, plus lines 268/370. These operate on a nonexistent dir --
   effectively no-ops that report nothing.

2. LIVE (TOML): `current_profile_file()` = `faelight_state_dir().join("current-profile")`
   + `profiles.toml` (home-manager-deployed to ~/.config/faelight/profiles.toml). THIS
   is what reports "Profile System OK (current: default)" on the dashboard.

The `.profile`-file mechanism is Arch-era residue. INT-107 left `profiles_dir()` as a
stopgap (`core_dir().join("config/profiles")`, decoupled from the removed stow
interfaces_dir) purely so profile kept compiling. INT-108 owns the real fix.

## The Solution
Resolve the two-model split -- consolidate on TOML. Decide per operation:
- cmd_list -> read available profiles from profiles.toml (not a .profile dir scan)
- cmd_edit -> edit the TOML entry (or drop if TOML is declarative-only)
- cmd_export / import -> operate on TOML entries, or remove if not meaningful under
  home-manager's declarative model
- doctor dir-check (line ~176) -> check the TOML/current-profile state, not a dir
- get_profile_dir() + paths::profiles_dir() -> REMOVE once no caller remains

Open design question for this intent: does the file-per-profile UX have value worth
keeping (repoint at a real, existing, populated location) OR is TOML-declarative the
whole story (remove the file mechanism entirely)? Decide at execution, demonstrated
against the live profile system.

## Scope
- profile/src/main.rs: 7 get_profile_dir() call sites (177, 268, 370, 434, 445, 504) +
  get_profile_dir() def (61)
- paths.rs: profiles_dir() (INT-107 stopgap) -- remove or repoint to real profile home
- Verify no regression: `profile list`, profile switching, doctor "Profile System OK"

## Gates
- [ ] Zero-warning build (non-negotiable)
- [ ] `profile list` shows real profiles (from the consolidated TOML model)
- [ ] Profile switching works end-to-end (demonstrated, not declared)
- [ ] doctor "Profile System OK" stays green
- [ ] No dead `.profile`-file code paths remain; profiles_dir() resolved

## Relationship
- Follows INT-107 (which decoupled profiles_dir as a stopgap so profile compiled).
- UNBLOCKS INT-061: once profiles_dir() is resolved, config/ has no profile
  entanglement -> config/ -> nix/home/dotfiles/ move is fully clean.
- Sibling to INT-106 (paths.rs hygiene) -- removing profiles_dir is one of the
  Arch-era accessors 106 is broadly concerned with.

## Success Criteria
- [ ] profile tool speaks one storage model (TOML); .profile-file mechanism removed
- [ ] profiles_dir() removed or honestly repointed; no nonexistent-dir reads
- [ ] Live profile system verified green; build clean
- [ ] config/ move unblocked of profile entanglement

---
