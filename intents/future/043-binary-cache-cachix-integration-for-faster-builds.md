---
id: 043
date: 2026-06-09
type: infrastructure
title: "faster builds: crane dep-split + Cachix binary cache"
status: planned
tags: [nix, cachix, crane, rust, binary-cache, builds, performance, nixos, flake]
priority: low
---
## Why
Every `rebuild` recompiles faelight-forest from source.
On a clean system or VM this takes 3-5 minutes.
The faelight-forest derivation does not change between rebuilds
unless the Rust source changes.

A binary cache lets NixOS pull pre-built derivations
instead of compiling from source every time.
crane attacks that Rust compile directly by splitting dependencies into a
separately-cached derivation; a binary cache then distributes built
derivations to clean, recovering, and CI machines.

Honest scope (2026-06-14): on THIS machine the local /nix/store already serves
unchanged derivations, so a config-only rebuild here is already fast -- the
binary cache does NOT speed the local daily loop. crane is the lever that
reduces the actual Rust compile; the cache is resilience for recovery-from-USB,
clean VMs, and CI (INT-048).

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
Two complementary layers, NOT alternatives:

LAYER 1 -- crane (build structure)  [the actual compile lever]
  Today faelight-forest is one rustPlatform.buildRustPackage derivation: deps +
  our source compile together (~180s) on every clean build. crane splits this
  into a deps-only derivation (third-party crates from Cargo.lock, none of our
  source) plus our-source built against those cached artifacts. A source-only
  change becomes a deps cache hit; only our crates recompile.
  CAVEAT: crane caches third-party deps, not our own code -- faelight-forest is
  ~98% our own Rust, so the local win is bounded by the deps-vs-own-code split.
  MEASURE first. The deps derivation is also the cacheable unit LAYER 2 pushes.

LAYER 2 -- binary cache backend (distribution)
  Where built derivations live so a clean/recovering/CI machine downloads
  instead of compiling. Backend options:
  OPTION A -- Cachix (recommended): hosted, easy, NixOS-integrated; private
    cache costs money; free tier ~5GB.
  OPTION B -- nix-serve (self-hosted): fully local, no external dep; host/VM
    must be running.
  OPTION C -- Attic (self-hosted Cachix alternative): free, NixOS-native; more
    setup.
  Recommended: A (Cachix); revisit C if the external dependency becomes
  undesirable.

Honest scope: on THIS machine the local /nix/store already serves unchanged
derivations, so LAYER 2 does not speed the local daily loop -- its value is
recovery-from-USB, clean VMs, and CI. LAYER 1 (crane) reduces the actual Rust
compile, locally and everywhere.

## Phases
Phase 0 -- measure the win (gates crane)
  cargo build --timings (or a cold full build minus an incremental one-crate
  rebuild) to find the deps-vs-own-code split of the ~180s.
  Gate: deps-vs-own-code split measured and recorded in this charter

Phase 1 -- crane build structure
  Convert faelight-forest from rustPlatform.buildRustPackage to crane:
  deps-only derivation + source derivation.
  Gate: nix build of faelight-forest uses crane (deps derivation split out)
  Gate: a source-only change recompiles our crates only; deps are a cache hit

Phase 2 -- Cachix account and cache setup
  Create Cachix private cache: faelight-forest. Generate signing key. Add
  cachix to the NixOS flake as substituter.
  Gate: nix build uses Cachix as substituter

Phase 3 -- push pipeline
  After a successful rebuild, push derivations (especially crane's deps) to
  Cachix. Script: pkgs/faelight/scripts/cache-push
  Gate: crane deps derivation present in Cachix after rebuild

Phase 4 -- clean/recovery hit verification
  Clean build in a VM with no Rust changes; verify the cache hit.
  Gate: clean VM rebuild pulls deps from cache (no local dep recompile)

Phase 5 -- fsh integration
  cache status -- last hit/miss rate; cache push -- manual push.
  Gate: cache commands work in fsh

Phase 6 -- CI integration (INT-048 dependency)
  CI pushes to cache on build success.
  Gate: every successful CI build updates cache

## Gates
- [ ] deps-vs-own-code split of the 180s measured and recorded (Phase 0)
- [ ] faelight-forest converted to crane (deps-only derivation split out)
- [ ] source-only change rebuilds our crates only; deps are a cache hit
- [ ] Cachix cache created: faelight-forest
- [ ] Cachix substituter added to flake.nix
- [ ] signing key configured in NixOS secrets
- [ ] crane deps derivation pushed to Cachix after rebuild
- [ ] clean VM rebuild pulls deps from cache (no local dep recompile)
- [ ] cache status shows hit rate in fsh
- [ ] cache push works manually from fsh

## Depends On
- INT-048 (forest-ci) -- CI pushes to cache on success

## Related -- Lix (separate intent, not in scope here)
The 2026-06-14 review also covered Lix, the third layer: a fork of the Nix
CLI/daemon (faster evaluation, friendlier errors, better DX). It does NOT
change compile time and is orthogonal to both crane and Cachix. Mature (v2.95,
Mar 2026), a system-level swap -- reversible but a core component. Track as its
own small future intent; recorded here so the research is not lost.

## The Rule
"The forest should not recompile what has not changed.
 Build once. Cache everywhere.
 Time saved is time for the work." 🌲
