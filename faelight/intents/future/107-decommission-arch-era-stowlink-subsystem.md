---
id: 107
date: 2026-07-02
type: future
title: "Decommission Arch-era stow/link subsystem"
status: planned
tags: [link, Arch-era]
---

## Vision
The tree tells the truth about how dotfiles actually work: home-manager owns them,
declaratively, with atomic generations and rollback. No redundant second mechanism.
Removing the Arch-era stow/link subsystem is structural honesty -- and it unblocks
the INT-061 config/ -> nix/home/dotfiles/ move.

## The Problem
`core link` is a fully-wired GNU-Stow reimplementation (9 subcommands: status, list,
audit, plan, deploy, undeploy, adopt, redeploy, sync) that manages dotfile symlinks
from config/ into ~. On NixOS this job belongs entirely to home-manager.

VERIFIED 2026-07-02: ~/.config/mango/config.conf -> /nix/store/...home-manager-files/...,
with a config.conf.prenix backup marking where HM displaced the pre-Nix file.
`core link status` reports 10 "deployed" links, but home-manager created them --
`core link` only OBSERVES. It is live-but-obsolete: a redundant second dotfile
mechanism duplicating home-manager.

It is also the entanglement BLOCKING INT-061's config/ move: ~50 references
(link/ domain + stow accessor family + get-version .dotmeta) point at config/ as
a stow directory.

## The Solution
Remove the subsystem, verifying obsolete before each cut:

REMOVE:
1. link/ domain -- engine/src/domains/link/mod.rs (~40 internal stow refs)
2. dispatcher routes -- app/dispatcher.rs:106-132 (whole LinkCommand match arm)
3. parser enum -- cli/parser.rs LinkCommand + subcommand defs (~604-621+)
4. domains/mod.rs:27 -- `pub mod link;`
5. paths.rs Arch-era accessors (95-111): interfaces_dir(), stow_dir(),
   profiles_dir() (config/profiles -- doesn't exist), themes_dir()
   (config/themes -- doesn't exist), zshrc() (~350, zsh removed), + test at ~463
6. get-version .dotmeta logic -- get-version/src/main.rs:65,92-95
7. bootstrap + doctor config/-as-stow scans -- bootstrap/mod.rs:380,
   doctor/mod.rs:110, doctor/checks.rs:11 (check_stow)

CAUTION -- verify does NOT break:
- profile tool uses profiles_dir() (profile/src/main.rs:62). Dashboard shows
  "Profile System OK" -- confirm it stays green after removing profiles_dir().
- doctor checks.rs:45 (`0-core/config/{}` symlink-target check) -- decide route vs remove.
- checkpoint/mod.rs:503 reads config/faelight-shell/...config.fsh -- real read, must
  route to NEW dotfiles location (coordinate with the config/ move).

PRESERVE (optional, vision-positive):
The per-package `core link status` view is useful. Consider folding a READ-ONLY
dotfile-status report into doctor (reporting home-manager's /nix/store-backed links),
keeping observability while dropping the deploy machinery.

## Relationship
- Sibling to INT-106 (paths.rs accessor hygiene): THIS intent takes the stow-specific
  accessors (interfaces/stow/profiles/themes/zshrc); 106 keeps rules_dir rename, font
  path, and general 40+ string routing.
- UNBLOCKS INT-061: once removed, config/ -> nix/home/dotfiles/ is clean.
- The "faelight-link REMOVED" thread was a DIFFERENT tool; this is the engine-side
  link/ domain + stow accessor family that survived.

## Success Criteria
- [ ] link/ domain, dispatcher routes, parser enum, mod.rs decl all removed
- [ ] stow-specific paths.rs accessors removed (interfaces/stow/profiles/themes/zshrc)
- [ ] get-version .dotmeta logic + bootstrap/doctor stow scans removed
- [ ] Zero-warning build (non-negotiable)
- [ ] No alias/script/config.fsh invokes `core link` (grep before + after)
- [ ] `core doctor` still 33/33 -- especially Profile System OK
- [ ] `cargo test -p faelight-core` not newly broken by removed paths.rs test
- [ ] config/ move to nix/home/dotfiles/ no longer entangled with stow refs

---
