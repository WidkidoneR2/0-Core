---
id: 010
date: 2026-06-03
type: feature
title: "NixOS environment switching: instant context change without logout"
status: planned
tags: [nix, devshell, environments, direnv, profiles]
priority: medium
---

## Why

NixOS allows switching between completely different environments instantly
via nix develop + direnv profiles. No logout required. This is one of NixOS's
killer features vs Arch.

## Vision

- forest-env: default daily driver
- friday-dev: Rust + AI tools loaded
- secure-env: hardened, VPN enforced, minimal tools
- Switch with: nix develop .#friday-dev

## Gate

Three named devShells in flake.nix. Switching takes one command.
