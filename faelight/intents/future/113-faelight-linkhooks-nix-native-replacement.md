---
id: 113
date: 2026-07-02
type: future
title: "faelight-link/hooks Nix-native replacement"
status: planned
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

## Gates (when built)
- [ ] faelight-hooks function inventory (live vs vestige) documented
- [ ] Each live function: retire / port-to-Nix / keep decided
- [ ] No Arch-era hook plumbing remains; Nix-native equivalents in place

---
