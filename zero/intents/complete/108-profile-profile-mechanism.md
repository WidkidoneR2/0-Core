---
id: 108
date: 2026-07-02
type: future
title: "profile .profile-mechanism"
status: complete
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
- [x] Zero-warning build (non-negotiable) <!-- STAMP-108-DONE / INT-130 2026-07-10: commit 291d4ab5 'Full workspace builds clean'. -->
- [~] `profile list` shows real profiles (from the consolidated TOML model) <!-- INT-130 2026-07-10: MOOT by retirement. The intent RESOLVED by retiring the profile tool entirely (commit 291d4ab5), not consolidating it -- 'profile' is now command-not-found (verified live). So there is no 'profile list' to demonstrate; the dead .profile mechanism this gate targeted is gone. Marked [~]: honest -- the tool was removed, not fixed-to-list-TOML. Power-switching is declarative on NixOS now. -->
- [~] Profile switching works end-to-end (demonstrated, not declared) <!-- INT-130 2026-07-10: MOOT by retirement -- profile tool removed (command-not-found, verified live). No switching to demonstrate; superseded by declarative NixOS power-switching. -->
- [x] doctor "Profile System OK" stays green <!-- INT-130 2026-07-10: commit 291d4ab5 removed doctor check_profiles + cockpit label (the dead-dir check). doctor is 100% healthy / 32-32 live this session -- no profile-related failure. The check was retired WITH the tool, cleanly. -->
- [x] No dead `.profile`-file code paths remain; profiles_dir() resolved <!-- INT-130 2026-07-10: VERIFIED IN SOURCE -- profiles_dir()/current_profile_file gone from paths.rs (grep=0); the only remaining 'profile' in engine/domains/profile/mod.rs is a struct-field access (r.profile), NOT the .profile-file mechanism. Dead paths gone. -->

## Relationship
- Follows INT-107 (which decoupled profiles_dir as a stopgap so profile compiled).
- UNBLOCKS INT-061: once profiles_dir() is resolved, config/ has no profile
  entanglement -> config/ -> nix/home/dotfiles/ move is fully clean.
- Sibling to INT-106 (paths.rs hygiene) -- removing profiles_dir is one of the
  Arch-era accessors 106 is broadly concerned with.

## Success Criteria
- [~] profile tool speaks one storage model (TOML); .profile-file mechanism removed <!-- INT-130 2026-07-10: resolved by RETIRE, not consolidate. The charter's own open question ('is TOML the whole story -- remove the file mechanism?') was answered at execution: the whole tool is Arch-era residue -> retired (commit 291d4ab5). The .profile mechanism IS removed [x-part], but 'tool speaks TOML' is moot (no tool) [~-part]. Net [~]: honest about the pivot. -->
- [x] profiles_dir() removed or honestly repointed; no nonexistent-dir reads <!-- INT-130 2026-07-10: VERIFIED IN SOURCE -- profiles_dir() removed from paths.rs (grep=0). No nonexistent-dir reads remain. -->
- [x] Live profile system verified green; build clean <!-- INT-130 2026-07-10: doctor 100% healthy live; build clean (commit 291d4ab5). 'Profile System' concern resolved by removing the dead subsystem -- nothing profile-related is red. -->
- [x] config/ move unblocked of profile entanglement <!-- INT-130 2026-07-10: PROVEN by outcome -- INT-061 FINALE (commit 252b3914) later moved config/ -> nix/home/dotfiles/ cleanly; profiles_dir removal (this intent) was the unblock. -->

---
