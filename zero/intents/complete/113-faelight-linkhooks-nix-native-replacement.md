---
id: 113
date: 2026-07-02
type: future
title: "faelight-link/hooks Nix-native replacement"
status: complete
tags: [nix, hooks, home-manager]
---

## Vision
Whatever legitimate function faelight-hooks (and the old link subsystem) provided
should have a clean Nix-native / home-manager equivalent, so no Arch-era hook
plumbing lingers post-NixOS.

## Context
INT-107 decommissioned the stow/link subsystem (home-manager owns dotfile symlinks
now). But faelight-hooks (v10.2.0) still exists in the workspace. Open question:
what does it actually do on NixOS, and is any of it still needed, or is it
superseded by home-manager activation scripts / NixOS systemd units / git hooks?

## Recon needed (before deciding)
- What does faelight-hooks DO today? (git hooks? deploy hooks? activation hooks?)
- Which of its functions are live vs Arch-era vestige (like link/get-version were)?
- For live functions: is there a Nix-native home?
  - dotfile linking -> home-manager (already)
  - git pre-commit -> nix git-hooks / pre-commit.nix, or the RISK.toml hook (INT-112)
  - system activation -> home-manager activation scripts / systemd units

## Decision space
- RETIRE (like get-version/profile) if fully superseded.
- PORT specific live functions to Nix-native mechanisms.
- KEEP if it does something genuinely forest-specific with no Nix equivalent.

## Related
- Follows INT-107 (stow/link decommission pattern).
- The git-hook enforcement idea in INT-112 (RISK.toml) may absorb some of this.

## RESOLUTION (2026-07-10): RETIRE faelight-hooks; faelight-link already gone
Both tools the title names, resolved:
- **faelight-link**: ALREADY REMOVED. No binary (`which` = not found), no source -- the only
  "faelight-link" left in the tree was this intent file. Retired in earlier work; the engine-side
  link/ domain was separately decommissioned by INT-107. Nothing to do.
- **faelight-hooks**: RETIRE (dead weight). Began as an Arch-era GNU-stow dotfile-linker; on NixOS
  its linking role ceded to home-manager (INT-107) and it was repurposed into a git-hooks checker.
  DECISION FLIPPED KEEP->RETIRE on verification: the flake comments called it "the commit-time
  authority," but its hooks were NEVER INSTALLED -- `.git/hooks/` held ONLY git's default `.sample`
  files (verified 2026-07-10). It built and deployed but nothing invoked it; ~20 commits this
  session passed with no hook firing. Uninstalled dead weight, redundant with git-hooks.nix
  (INT-119, the deliberately-adopted sandboxed flake-check gate). Retired cleanly:
    - crate faelight/rust-tools/faelight-hooks/ (git rm -r)
    - alias `hooks` (config.fsh:241), registry entry (tools.toml), umbrella subtool ref
      (faelight/src/main.rs:201), flake.nix comments updated (git-hooks.nix now sole mechanism)
    - rebuilt (gen 342): `which faelight-hooks` = gone, tool count 58->57, 0 failed checks.
Hook roles now: dotfiles -> home-manager (107); the sole hook gate -> git-hooks.nix (119, flake-
check time). No commit-time hook layer remains -- fine, since faelight-hooks wasn't providing one
(never installed). Continues the migration-simplification arc (get-version/profile/safe-update/stow
-> faelight-hooks). LESSON: I first drafted KEEP off the flake comment "commit-time authority";
Christian pushed to verify it was actually wired in -- it wasn't. Verify installation, not intent.

## Gates (when built)
- [x] faelight-hooks function inventory (live vs vestige) documented <!-- STAMP-113-DONE / INT-130-discipline 2026-07-10: LIVE functions = git-hook installer (pre-commit/pre-push/commit-msg, install.rs) + checks secrets/conflicts/filesize/branch/rustfmt/clippy (main.rs:25-34,123-135). VESTIGE = none: the original GNU-stow/dotfile-linker role (Arch-era) is fully gone -- grep for stow/pacman/.dotmeta/symlink = clean (only #!/usr/bin/env bash shebangs in the hook scripts it installs). The tool was repurposed Arch->Nix from stow-replacement into a git-hooks checker. -->
- [x] Each live function: retire / port-to-Nix / keep decided <!-- 2026-07-10: DECISION = RETIRE (flipped from an initial KEEP draft on verification). faelight-hooks' hooks were NEVER installed -- .git/hooks/ held only git's .sample defaults; it built+deployed but nothing invoked it. Uninstalled dead weight, redundant with the adopted git-hooks.nix (INT-119). Its old stow/dotfile role already ceded to home-manager (INT-107). Retired cleanly + rebuilt (gen 342). -->
- [x] No Arch-era hook plumbing remains; Nix-native equivalents in place <!-- 2026-07-10: VERIFIED after retirement. faelight-hooks fully removed (crate + alias + registry + umbrella ref + flake comments); rebuilt gen 342; `which faelight-hooks` = gone, tool count 58->57, 0 failed checks. Hook roles now have clean Nix-native homes with NO Arch plumbing: dotfiles->home-manager (INT-107), the sole hook gate->git-hooks.nix (INT-119). faelight-link already gone (earlier). -->

---
