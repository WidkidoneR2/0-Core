---
id: 002
date: 2026-06-03
type: infrastructure
title: "Fenix: Rust 1.93+ toolchain via flake overlay"
status: complete
tags: [rust, fenix, toolchain, nixos, faelight-fm]
priority: high
---

## Why

faelight-fm requires rustc 1.93+. Current nixpkgs-26.05 provides 1.91.1.
Fenix is the standard NixOS approach for pinned Rust toolchains.

## Approach

1. Add fenix input to flake.nix
2. Replace pkgs.rustc + pkgs.cargo with fenix stable toolchain
3. Remove faelight-fm from Cargo.toml exclude list
4. Rebuild and verify faelight-fm compiles

## Gate

faelight-fm launches from /run/current-system/sw/bin/faelight-fm
