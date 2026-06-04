---
id: 036
date: 2026-06-04
type: housekeeping
title: "config/ cleanup: remove Arch-era configs, retire core-diff and faelight-diff"
status: planned
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

- [ ] config/ contains only NixOS-relevant configs
- [ ] Arch-era root artifacts removed
- [ ] core-diff retired in registry
- [ ] faelight-diff retired in registry
- [ ] README updated for NixOS era
- [ ] GitHub looks clean and organized
- [ ] Health 100% after all changes
