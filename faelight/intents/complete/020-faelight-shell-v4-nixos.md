---
id: 020
date: 2026-06-03
type: feature
title: "faelight-shell v4: NixOS-native, nix develop aware, forest-first"
status: complete
tags: [fsh, shell, nixos, nix-develop, direnv, improvement]
priority: high
---

## Why

fsh was built on Arch. On NixOS several things need rethinking:
- PATH is assembled differently (Nix profiles, not .bashrc chains)
- direnv integration needs proper hook (currently silent)
- nix develop shells should be visible in the prompt
- Nix-specific vocabulary: rebuild, flake update, nix shell
- The shell should understand it lives in a Nix system

## Vision

fsh on NixOS should feel native. When you cd into 0-core, the prompt
shows the active devShell. When you run rebuild, fsh knows what that
means. When Friday suggests something, it knows you're on NixOS.

## Approach

- Proper direnv hook integration (show active devShell in prompt)
- Nix-aware PATH handling (already improved, continue)
- New fsh vocabulary: rebuild, flake, nix-shell, develop
- Prompt shows active nix devShell name
- fsh understands NixOS generations for rollback suggestions
- Remove remaining Arch assumptions (pacman references in Friday knowledge)

## Gate Check
✅ fsh prompt shows active devShell -- ❄ appears in nix develop
✅ NixOS PATH handling -- /run/current-system/sw/bin always in PATH
✅ faelight-shell candidates updated -- NixOS bin path first
✅ No Arch references in fsh source code
✅ Friday knowledge updated -- 4 pacman facts replaced with NixOS equivalents
⏸ direnv visual activation -- deferred: activates silently, visual hook needs fsh hook API -- approved by: christian 2026-06-04
