---
id: 028
date: 2026-06-04
type: infrastructure
title: "Forest dev tooling: nix-tree, nvd, nh, bacon, cargo-nextest"
status: complete
tags: [nix, tools, bacon, cargo, development]
priority: medium
---

## Vision

The right tools in the right places.
nix-tree for dependency visualization.
nvd for generation diffs.
nh for cleaner rebuild interface.
bacon for smart Rust watching.
cargo-nextest for faster tests.

## Approach

- nix-tree, nvd → home.packages (system-wide inspection tools)
- nh → evaluate as rebuild alias replacement or supplement
- bacon, cargo-nextest → friday-dev devShell in flake.nix
- cargo-watch → already in devShell plan, confirm it's there

## Gate

- [ ] nix-tree available system-wide
- [ ] nvd shows generation diffs cleanly
- [ ] bacon watches and rebuilds on save
- [ ] cargo-nextest runs test suite faster than cargo test
