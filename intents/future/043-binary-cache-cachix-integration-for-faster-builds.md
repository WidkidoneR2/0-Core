---
id: 043
date: 2026-06-09
type: infrastructure
title: "binary-cache: Cachix integration for faster builds"
status: planned
tags: [nix, cachix, binary-cache, builds, performance, nixos, flake]
priority: low
---
## Why
Every `rebuild` recompiles faelight-forest from source.
On a clean system or VM this takes 3-5 minutes.
The faelight-forest derivation does not change between rebuilds
unless the Rust source changes.

A binary cache lets NixOS pull pre-built derivations
instead of compiling from source every time.
rebuild goes from 3-5 minutes to under 30 seconds
when only non-Rust config changes.

## What Already Exists
NixOS flake: faelight-forest as rustPlatform.buildRustPackage
Cachix: available as NixOS service and CLI tool
The 49 Rust tools are the primary rebuild time bottleneck
Cargo.lock committed: deterministic builds already

## Vision
  rebuild                -- checks binary cache first
  cache miss (Rust change) -- compiles, pushes to cache
  cache hit (config change) -- pulls from cache, instant
  fsh command: cache status -- shows cache hit rate
  fsh command: cache push   -- manually push current build

## Approach
OPTION A -- Cachix (recommended)
  Create private Cachix cache for 0-Core builds
  Add Cachix substituter to NixOS flake
  Add Cachix push to CI pipeline (INT-048) or manual push
  Pros: hosted, easy setup, well-integrated with NixOS
  Cons: external service dependency, private cache costs money

OPTION B -- Local Nix store cache (self-hosted)
  Run nix-serve on local machine or VM
  Add as substituter in flake.nix
  Pros: fully local, no external dependency
  Cons: VM must be running for cache to work, more setup

OPTION C -- Attic (self-hosted Cachix alternative)
  Deploy Attic server as NixOS service
  Add as substituter in flake.nix
  Pros: self-hosted, free, NixOS-native
  Cons: more complex setup than Cachix

Recommended: OPTION A (Cachix) for simplicity.
Revisit OPTION C if external dependency is undesirable.

## Phases

Phase 1 -- Cachix account and cache setup
  Create Cachix account and private cache: faelight-forest
  Generate signing key
  Add cachix to NixOS flake as substituter
  Gate: nix build uses Cachix as substituter

Phase 2 -- Push pipeline
  After successful rebuild: push derivation to Cachix
  Script: pkgs/faelight/scripts/cache-push
  Gate: faelight-forest derivation in Cachix after rebuild

Phase 3 -- Cache hit verification
  Clean build in VM without Rust changes
  Verify cache hit: rebuild completes in < 30 seconds
  Gate: cache hit rebuild under 30 seconds in VM

Phase 4 -- fsh integration
  cache status -- show last cache hit/miss rate
  cache push   -- manually push current build
  Gate: cache commands work in fsh

Phase 5 -- CI integration (INT-048 dependency)
  CI pipeline (INT-048) pushes to cache on build success
  Gate: every successful CI build updates cache

## Gates
- [ ] Cachix cache created: faelight-forest
- [ ] Cachix substituter added to flake.nix
- [ ] Signing key configured in NixOS secrets
- [ ] faelight-forest derivation pushed to Cachix after rebuild
- [ ] Cache hit rebuild completes in under 30 seconds
- [ ] cache status shows hit rate in fsh
- [ ] cache push works manually from fsh
- [ ] VM rebuild uses cache (tested clean)

## Depends On
- INT-048 (forest-ci) -- CI pushes to cache on success

## The Rule
"The forest should not recompile what has not changed.
 Build once. Cache everywhere.
 Time saved is time for the work." 🌲
