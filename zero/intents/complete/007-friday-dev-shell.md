---
id: 007
date: 2026-06-03
type: infrastructure
title: "friday-dev shell: nix develop environment for Friday/forest development"
status: complete
tags: [nix, devshell, friday, development, direnv]
priority: high
---

## Why

Dev tools (rustc, cargo, clang, cmake, python, cargo-audit, cargo-flamegraph)
shouldn't be in home.packages permanently. A dedicated devShell keeps the
system lean and the dev environment explicitly declared.

## Approach

Add devShells.x86_64-linux.default to flake.nix:
- rust stable toolchain (via fenix)
- clang, cmake, pkg-config
- cargo-audit, cargo-flamegraph, cargo-watch
- python3 with pip
- sqlite
- direnv activates automatically via .envrc

## Gate

nix develop drops into forest dev environment.
cd ~/0-core activates it automatically via direnv.
cargo build works without nix shell workarounds.
