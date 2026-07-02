---
id: 036
date: 2026-06-04
type: housekeeping
title: "config/ cleanup: remove Arch-era configs, retire core-diff and faelight-diff"
status: complete
tags: [cleanup, config, arch-era, nixos, github]
priority: high
---

## Why

INT-008 migrated the structure but did not clean it.
config/ still contains Arch-era directories that have no place on NixOS:
- config/shell-zsh/ -- replaced by fsh + home.nix
- config/wm-sway/ -- replaced by niri
- config/browser-qutebrowser/ -- replaced by Brave
- config/prompt-starship/ -- replaced by fsh prompt
- config/shell-nushell/ -- never used on NixOS
- config/editor-nvim/ -- evaluate: keep or replace with helix post-1.0.0

Root level Arch-era artifacts to remove:
- assets/ -- Arch-era, no longer relevant
- dotfiles/ -- replaced by config/
- status-blocks/ -- Arch-era i3/sway blocks
- TOOLS.md -- Arch-era tool list, outdated

nvd replaces core-diff and faelight-diff:
- nvd diff shows exact package changes between generations
- More accurate than any custom Rust implementation
- core-diff → retire
- faelight-diff → retire

## Approach

1. Move Arch-era config/ dirs to labs/graduated/arch-era/
2. Remove assets/, dotfiles/, status-blocks/, TOOLS.md
3. Retire core-diff and faelight-diff in registry
4. Update README to reflect NixOS era public face
5. Verify health stays 100% after each step

## Gate

## Gate Check
✅ config/ contains only NixOS-relevant configs (alacritty, faelight, faelight-shell, niri, yazi, editor-nvim)
✅ Arch-era root artifacts removed (assets kept for font build dependency)
✅ core-diff retired in registry (nvd replaces it)
✅ faelight-diff retired in registry (nvd replaces it)
✅ Health 100% after all changes
✅ New tools added: wl-clipboard, wpaperd, lazygit, mmv-go, helix
⏸ README updated for NixOS era -- deferred: full README rewrite at 1.0.0 -- approved by: christian 2026-06-04
⏸ assets/fonts/ embed refactor -- deferred: post-1.0.0, tracked in assets/fonts/README.md -- approved by: christian 2026-06-04
