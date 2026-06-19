---
id: 043
date: 2026-06-09
type: infrastructure
title: "faster builds: crane dep-split + Cachix binary cache"
status: in-progress
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

## Phase 0 Results (2026-06-17)
Method: clean release build of the full workspace, then touch every repo .rs
(deps stay fingerprint-clean in target/) and rebuild -- isolating deps-compile
from our-code-compile, which is exactly the crane split.

- T_total   (clean: deps + our crates):            156.8s  (2m36)
- T_ourcode (our crates only, deps cached):         47.8s
- T_deps    (T_total - T_ourcode, crane-cacheable): 109.0s  (~70%)

Verdict: deps are ~70% of clean-build compile time. The "98% our own Rust" is by
line volume; compile TIME is dominated by the dep tree (smithay, wayland, git2,
tokio, serde, clap, ratatui). Because nix builds are hermetic (no incremental
target/ reuse across rebuilds), every Rust-source change today pays the full
~156s -- which is why this session's rebuilds were all ~183s. crane caches the
deps derivation in /nix/store, dropping a source-change rebuild from ~156s to the
~48s our-code floor: ~109s / ~70% off every source-change rebuild, projecting
rebuild ~183s -> ~75s. crane is a real local win, not just CI/recovery.

## Gates
- [x] deps-vs-own-code split measured: deps 109s (70%) / our code 48s (30%) of 156.8s clean (Phase 0, 2026-06-17)
- [x] faelight-forest converted to crane (deps-only derivation split out): cold build 352s, full workspace built (40+ bins, makeWrapper-wrapped, postFixup intact); core also on crane sharing deps (Phase 1, 2026-06-17)
- [x] source-only change rebuilds our crates only; deps are a cache hit: 1-line src change -> 54.3s vs 352s cold (faelightDeps cached); core -p build 66.8s confirms shared deps. NOTE: nix non-incremental on our code (full engine recompile per build); dev loop stays on cargo devshell ~5-7s, crane win is the rebuild/deploy path ~183s -> ~54-66s (Phase 1, 2026-06-17)
- [x] Cachix cache created: faelight-forest (live at faelight-forest.cachix.org, public open-source free tier; 2026-06-18)
- [x] Cachix substituter configured (hosts/framework16/configuration.nix: extra-substituters + extra-trusted-public-keys; verified live -- nix config show substituters lists faelight-forest.cachix.org; 2026-06-18)
- [x] auth token configured (cachix CLI credential, NOT a NixOS secret -- Cachix-hosted cache needs only the public key for pull (done above); push uses CACHIX_AUTH_TOKEN held locally by the cachix CLI via cachix authtoken; verified -- cachix push authenticated OK, 2026-06-18)
- [x] crane deps derivation pushed to Cachix: 614 paths (77 deduped) via cachix push (zstd, All done) -- 2026-06-18
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
