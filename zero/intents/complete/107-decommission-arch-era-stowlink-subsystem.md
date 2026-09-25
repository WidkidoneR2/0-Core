---
id: 107
date: 2026-07-02
type: future
title: "Decommission Arch-era stow/link subsystem"
status: complete
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
- [x] link/ domain, dispatcher routes, parser enum, mod.rs decl all removed <!-- STAMP-107-DONE / INT-130 2026-07-10: commit 265706d6 -- 'Removed link/ domain (9-subcommand core link), dispatcher/parser/cli-mod/commands.rs Link plumbing'. -->
- [x] stow-specific paths.rs accessors removed (interfaces/stow/profiles/themes/zshrc) <!-- INT-130 2026-07-10: VERIFIED IN SOURCE -- grep of paths.rs for stow_dir|interfaces_dir|themes_dir|zshrc = 0 matches. Removed per commit 265706d6. -->
- [x] get-version .dotmeta logic + bootstrap/doctor stow scans removed <!-- INT-130 2026-07-10: commit 265706d6 -- 'removed get-version tool (stow .dotmeta reader), bootstrap+doctor stow scans; reframed check_stow -> Dotfile Symlinks: Managed by home-manager'. -->
- [x] Zero-warning build (non-negotiable) <!-- INT-130 2026-07-10: commit 265706d6 -- 'Full workspace clean'. -->
- [x] No alias/script/config.fsh invokes `core link` (grep before + after) <!-- INT-130 2026-07-10: VERIFIED IN SOURCE -- grep -rn 'core link' across .fsh/.toml = 0 matches. Also removed 5 config.fsh aliases + registry entries per commit 265706d6. -->
- [x] `core doctor` still 33/33 -- especially Profile System OK <!-- INT-130 2026-07-10: VERIFIED LIVE this session -- d output shows 32/32 checks 100% healthy, 'Dotfile Symlinks: Managed by home-manager (NixOS)', Profile-area green. (Check count is 32 now vs 33 at 107's time -- later intents adjusted the set; the substance -- Profile System OK, no stow breakage -- holds.) -->
- [x] `cargo test -p faelight-core` not newly broken by removed paths.rs test <!-- INT-130 2026-07-10: commit 265706d6 '33 checks intact; 32/32 resilience'; corroborated by INT-106 (73687ec5) 'cargo test -p faelight-core runs clean (11 passed)'. -->
- [x] config/ move to nix/home/dotfiles/ no longer entangled with stow refs <!-- INT-130 2026-07-10: PROVEN by outcome -- INT-061 FINALE (commit 252b3914) later successfully moved config/ -> nix/home/dotfiles/, which 107 was unblocking. The move happened cleanly, so the stow entanglement was genuinely removed. -->

---
