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
- [~] crane deps derivation pushed to Cachix: 614 paths -- DOES NOT HOLD as of 2026-06-23 (see finding below: closure not retrievable from cache; 6/691 served) (77 deduped) via cachix push (zstd, All done) -- 2026-06-18
- [ ] clean VM rebuild pulls deps from cache (no local dep recompile)
(fsh cache commands -- former gates cache status / cache push -- SPLIT to INT-068; cache status honest-scoped to present/absent there. 2026-06-18)

## CACHE-INCOMPLETE FINDING (2026-06-23) -- the cache is NOT serving the deps closure
Re-verifying the final gate (clean machine pulls deps from cache) surfaced that the cache
is effectively EMPTY of the current deps closure, despite the 2026-06-18 gate claiming
614 paths pushed. Three independent read-only checks AGREE:
- nix path-info --store https://faelight-forest.cachix.org over the full 691-path deps
  closure: only 6/691 paths served.
- Direct curl of a specific closure narinfo (windows-sys-0.61.2): HTTP 404 from cachix
  (cloudflare). Genuinely absent.
- nix cannot SUBSTITUTE that path ("path is not valid", "don't know how to build").
- nix copy --from cachix into a scratch /tmp store FAILED on that same path.
THE TRAP: `cachix push faelight-forest <deps>` reports "Nothing to push - all store paths
are already on Cachix" -- but this reflects cachix's LOCAL push-record (cachix.dhall),
NOT actual retrievability. The paths are recorded-as-pushed locally but are NOT
publicly substitutable. So the June-18 push either failed silently, went to a different
state, or was GC'd / not persisted cache-side. cachix's "nothing to push" is misleading
and must NOT be trusted as proof the cache works.
IMPACT: the recovery/clean-VM/CI resilience 043 exists to provide is currently ABSENT --
a from-scratch machine would recompile ~685/691 deps paths, not pull them. Found calmly
in verification, not during a real recovery.
deps path measured: /nix/store/v6vq7rwx8dzzxsyz5sgdjd551d7mzwqi-faelight-forest-deps-deps-9.2.0
  (691-path closure, 1.4 GiB).
NEXT SESSION (focused "make the cache actually work"):
1. Force a real re-upload bypassing cachix's stale skip-record -- e.g. `nix copy --to
   'https://faelight-forest.cachix.org' <deps>` (does not consult cachix.dhall), or
   investigate why cachix.dhall thinks it's pushed.
2. Re-verify with nix path-info over the closure until it shows ~691/691 served.
3. THEN the clean-pull gate: nix copy --from cachix into a scratch store succeeds end-to-end.
4. Understand WHY the original push didn't persist, so the re-push sticks (auth scope?
   public-vs-private visibility? cachix GC? partial-upload on a dropped connection?).
Phase 4 host edit (hosts/vm/configuration.nix Cachix substituter, backup .bak-20260623T184125)
is DONE and correct -- keep it; it just can't be verified until the cache actually serves.

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
<!-- INT-071 backfill (2026-06-22): 043 commit history restored to intent_commits; queryable via genealogy again. -->


## SESSION 2026-07-07 -- CHURN ROOT-CAUSE FOUND + FIXED (Tier 1 live); cache-serve gap re-characterized

This session CORRECTS the 2026-06-23 "cachix push silently failed" theory. The push
was not the (whole) problem -- the deps derivation hash was never stable, so every
push was for an instantly-dead path.

### REAL ROOT CAUSE (of the churn)
flake.nix `faelightDeps = buildDepsOnly (faelightCommonArgs // {...})` inherited
`src = ./.` (line 23) -- the WHOLE repo tree. So the deps hash changed on EVERY repo
change (.rs edits, intent .md files, version bumps). Evidence: ~25+ distinct
`faelight-forest-deps-deps-9.2.0` output paths in /nix/store. Every push was stale on
arrival. buildDepsOnly is meant to depend only on Cargo.toml/Cargo.lock, not source.

### TIER-1 FIX (DONE, committed 7eb0075f, deployed live)
Give faelightDeps its own source:
  - manifests-only via `pkgs.lib.fileset.fileFilter` (Cargo.toml + Cargo.lock only)
  - stub targets via faelight/packages/faelight/scripts/normalize-deps-versions.sh
    (creates src/main.rs + src/lib.rs per [package] crate, plus any explicit
    `path = "src/bin/*.rs"` -- e.g. faelight-lock/auth.rs, faelight-daemon/test-client.rs)
  - the script ALSO normalizes [package] versions to 0.0.0 (Cargo.toml + Cargo.lock,
    section-aware: only OUR workspace members, identified by no `source=` line)
  - pass the prepared source as `dummySrc` (NOT `src`) so crane uses it verbatim and
    SKIPS its own mkDummySrc.
PROVEN: a .rs edit does NOT move the deps hash (identical before/after). Deps build
clean (148s). faelight-forest builds against cached deps (~55s vs ~156s cold).

### mkDummySrc BLOCKER (why we can't use crane's built-in) -- SOLVED via dummySrc
crane's mkDummySrc sets `package.build = <store-path-to-dummyBuild.rs>` in each dummy
Cargo.toml (crane issue #117 workaround for cargo timestamp checking), then writes it
via `builtins.toJSON` -> Nix 26.05 REJECTS "string is not allowed to refer to a store
path". Present even on crane master (checked). ALSO: crane's dummy injects a
`#[panic_handler] fn panic(_info: &::core::panic::PanicInfo)` which fails to compile
because our engine crate is literally NAMED `core` (`::core::` resolves to our crate).
Both avoided by supplying our own dummySrc (buildDepsOnly.nix:37 uses args.dummySrc
directly when present).

### TIER 2 (version-bump stability) -- DEFERRED, not achievable this way
Normalizing versions inside the runCommand happens at BUILD time, but the deps .drv
HASH is computed from the runCommand's INPUTS at eval time -- which include the
un-normalized manifests fileset. So a version bump still moves the deps hash
(verified: 3.0.2->3.0.4 changed the hash). To fix, versions must be laundered at
EVAL time (read+rewrite in Nix before hashing, like mkDummySrc's
unsafeDiscardStringContext trick). Tier 1 (.rs stability) is the frequent case and is
enough for now; version bumps (cicomplete) re-push occasionally.

### CACHE-SERVE GAP -- RE-CHARACTERIZED (this is the actual remaining blocker)
With a stable deps path now, pushed fresh:
  - the deps DERIVATION narinfo SERVES: `curl -sI .../zhsyaypl....narinfo` -> HTTP 200
  - but its CLOSURE cargo-package paths 404 on BOTH our cache AND cache.nixos.org
    (crane builds each crate with crane-specific hashes nixpkgs' cache never has, so
    they genuinely must live in OUR cache)
  - `cachix push` reports "Pushing 4 paths (663 already present) ... All done" but
    those 663 paths 404 -> the cachix "already present" claim is FALSE (the 06-23 trap,
    now confirmed with a stable target)
  - NO local cachix skip-record in ~/.cache/cachix or ~/.local/share/cachix (so the
    stale record is server-side, not a local file we can delete)
KEY METHOD CORRECTION: verify retrievability with narinfo HTTP HEAD
(`curl -sI https://faelight-forest.cachix.org/<hash32>.narinfo`), NOT
`nix path-info --store https://...` -- the latter FALSE-NEGATIVES ("path is not valid")
and likely produced the 06-23 "6/691" figure. Re-verify all future cache claims via
narinfo HTTP.
KEY METHOD CORRECTION 2: `nix copy --to https://faelight-forest.cachix.org` CANNOT push
to Cachix -> HTTP 405 "Method not supported" (the public URL is read-only; uploads MUST
go through the `cachix push` protocol). This RULES OUT the 06-23 "use nix copy --to"
suggestion.

### NEXT SESSION (focused "make cachix actually serve the closure")
1. Force cachix to re-push the FULL closure ignoring its false "already present" --
   investigate `cachix push` flags (e.g. --compression, or push the whole closure via
   `nix-store -qR <deps> | cachix push faelight-forest`), watch for real upload counts.
2. Signing key: cachix.dhall has `binaryCaches = []` (no local signing key). Confirm
   whether this cache needs a signing key generated (`cachix generate-keypair`) for
   pushes to persist/serve, and configure it if so.
3. Re-verify via narinfo HTTP HEAD over the whole ~667-path closure until served=667.
4. Then the clean-pull gate: `nix copy --from https://faelight-forest.cachix.org <deps>`
   into a scratch /tmp store succeeds end-to-end (this DOES work for pulling; only
   pushing via nix copy is blocked).
5. If the cachix skip-record/serve behavior stays opaque -> pivot to Attic (Option C,
   self-hosted, NixOS-native) as the cleaner backend.

### SECURITY
The Cachix authToken in ~/.config/cachix/cachix.dhall was exposed during this session
-> ROTATE it (regenerate at cachix.org, then `cachix authtoken <new>`). Scope is `tx`
push on a public cache, low-risk, but rotate for hygiene.

### HONEST SCOPE NOTE
The LOCAL crane win (rebuild ~156s->~55s) is delivered and live -- NOT at risk from any
of the above. The remaining work is purely clean-VM/recovery/CI RESILIENCE (the cache
actually serving a from-scratch machine), which does not speed the daily loop.
