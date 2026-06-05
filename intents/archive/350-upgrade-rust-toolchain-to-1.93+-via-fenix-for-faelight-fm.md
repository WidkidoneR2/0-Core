---
id: 350
title: "Upgrade Rust toolchain to 1.93+ via fenix for faelight-fm"
status: planned
date: 2026-06-01
tags: [rust, toolchain, fenix, faelight-fm, nixos, nix-overlay]
---

## Why

faelight-fm v0.2.1 depends on libcosmic (git) and wgpu v28.0.0.
Both require rustc >= 1.93. Current system toolchain is 1.91.1 (nixos-25.11 pin).
faelight-fm is excluded from the workspace Cargo.toml for this reason.

## Approach

Use fenix flake overlay to provide a specific Rust toolchain version.
fenix is the standard NixOS approach for pinned Rust versions.

Steps:
1. Add fenix to flake.nix inputs:
   inputs.fenix.url = "github:nix-community/fenix";
   inputs.fenix.inputs.nixpkgs.follows = "nixpkgs";

2. Pass fenix to configuration.nix via specialArgs

3. Replace pkgs.rustc + pkgs.cargo in systemPackages with:
   fenix.packages.x86_64-linux.stable.toolchain
   (or "complete" if rust-src needed for debugging)

4. Remove faelight-fm from Cargo.toml exclude list

5. Verify faelight-fm builds in workspace:
   nix build .#faelight-forest
   ls result/bin/faelight-fm

6. Update flake.nix faelight-forest derivation if needed

## Risk

Medium -- touches flake.nix and systemPackages.
Always dry-run first: sudo nixos-rebuild dry-run --flake .#framework16
Keep a rollback generation available: sudo nixos-rebuild switch --rollback

## Gate

faelight-fm launches and shows the forest file manager.
