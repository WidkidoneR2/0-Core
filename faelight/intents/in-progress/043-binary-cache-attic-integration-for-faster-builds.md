---
id: 043
date: 2026-06-09
type: infrastructure
title: "faster builds: crane dep-split + Attic binary cache"
status: in-progress
tags: [nix, attic, crane, rust, binary-cache, builds, performance, nixos, flake]
priority: low
---
## Why
Every `rebuild` recompiles faelight-forest from source. On a clean system or VM
this takes 3-5 minutes. The faelight-forest derivation does not change between
rebuilds unless the Rust source changes.

crane attacks the Rust compile directly by splitting third-party dependencies into
a separately-cached derivation; a binary cache then distributes those built
derivations to clean, recovering, and CI machines.

Honest scope: on THIS machine the local /nix/store already serves unchanged
derivations, so a config-only rebuild here is already fast -- the binary cache does
NOT speed the local daily loop. crane is the lever that reduces the actual Rust
compile (locally and everywhere); the cache is resilience for recovery-from-USB,
clean VMs, and CI (INT-048).

## Approach -- two complementary layers, NOT alternatives
LAYER 1 -- crane (build structure)  [the actual compile lever]
  faelight-forest was one rustPlatform.buildRustPackage derivation: deps + our
  source compiled together (~156s) on every clean build. crane splits this into a
  deps-only derivation (third-party crates from Cargo.lock) plus our-source built
  against those cached artifacts. A source-only change becomes a deps cache hit;
  only our crates recompile. DELIVERED + LIVE (see Phase 0/1 + Gates).

LAYER 2 -- binary cache backend (distribution)
  Where built derivations live so a clean/recovering/CI machine downloads instead
  of compiling.
  CHOSEN -- Attic (self-hosted, NixOS-native): single-tenant (ours), full control of
    what is stored/served, no external dependency. Correct fit; adopted 2026-07-07.
    Fits the forest: own the full stack.
  Rejected -- Cachix (hosted): multi-tenant content-dedup silently refuses to serve
    crane deps paths (marks them "already present" via another tenant's identical NAR,
    then 404s them from our namespace, no client-side override). Proven dead-end.
  Rejected -- nix-serve: simpler but no signing/dedup/retention; Attic is better.

  LESSON (the most valuable thing this intent produced): "recommended / first /
  popular" is an anchor, not an answer. Cachix was the original OPTION A and cost this
  intent real time across several sessions before the pivot. When a tool resists past a
  couple of genuine attempts, question the TOOL, not just the technique. The forest's
  own values -- own the full stack, understanding over convenience -- pointed at Attic
  from the start; we just didn't apply them to the tool choice early enough. Both the
  human and the AI reinforced the sunk-cost anchor; the corrective is to periodically
  step back and ask "is this even the right thing, or am I just committed to it?"

## Phase 0 Results (2026-06-17)
Method: clean release build of the full workspace, then touch every repo .rs (deps
stay fingerprint-clean in target/) and rebuild -- isolating deps-compile from
our-code-compile, which is exactly the crane split.
- T_total   (clean: deps + our crates):            156.8s  (2m36)
- T_ourcode (our crates only, deps cached):         47.8s
- T_deps    (T_total - T_ourcode, crane-cacheable): 109.0s  (~70%)
Verdict: deps are ~70% of clean-build compile time. compile TIME is dominated by the
dep tree (smithay, wayland, git2, tokio, serde, clap, ratatui). Because nix builds are
hermetic, every Rust-source change pays the full ~156s. crane caches the deps
derivation in /nix/store, dropping a source-change rebuild from ~156s to the ~48s
our-code floor (~70% off). crane is a real local win, not just CI/recovery.

## Phases
Phase 1 -- crane build structure  [DONE 2026-06-17]
  Gate: nix build uses crane (deps derivation split out)  [MET]
  Gate: source-only change recompiles our crates only; deps a cache hit  [MET]

Phase 1b -- crane deps-hash stability  [DONE 2026-07-07, commit 7eb0075f]
  Root cause of prior cache churn: faelightDeps inherited src=./. so the deps hash
  moved on EVERY repo change, making every cache push instantly stale. Fixed: a
  manifests-only fileset source (Cargo.toml+Cargo.lock) + stub targets via
  normalize-deps-versions.sh, passed as dummySrc so crane uses it verbatim (skipping
  its own mkDummySrc, which embeds a store-path build line Nix 26.05 rejects and a
  panic-handler that collides with our crate named core).
  Gate: a .rs edit does NOT move the deps hash  [MET -- proven identical before/after]
  Deferred: version-bump stability (needs eval-time version laundering; .rs stability
  is the frequent case and is enough).

Phase 2 -- Attic setup (self-hosted, local-only)  [DONE 2026-07-07]
  attic flake input + nix/modules/services/atticd.nix (127.0.0.1:8080, SQLite + local
  storage, required chunking block, JWT RS256 secret in root-only /etc/atticd.env).
  Substituter + public key wired into hosts/framework16/configuration.nix
  (extra-substituters http://127.0.0.1:8080/faelight ; extra-trusted-public-keys
  faelight:oyzBMXRQvmCpv7tXJHstiYm/4C+kDjH8rjfEe1sZecU=). attic-client on PATH
  (replaced pkgs.cachix).
  Gate: atticd running + nix uses Attic as substituter  [MET -- active on 127.0.0.1:8080]

Phase 3 -- push pipeline  [DONE 2026-07-07]
  attic push faelight <deps closure>.
  Gate: crane deps closure present in Attic  [MET -- 667 paths pushed, 0 already-cached,
  0 in-upstream, 0 skipped -- vs Cachix which false-skipped ~663]

Phase 4 -- clean/recovery hit verification  [DONE 2026-07-07]
  nix copy --from the local Attic into a fresh scratch store.
  Gate: clean-store pull of the full deps closure succeeds  [MET -- 662 cargo-package
  paths landed in /tmp/scratch-store; the EXACT operation Cachix failed]

Phase 5 -- fsh integration  [OPTIONAL / future]
  cache status (hit/miss), cache push (manual). Not required for the core gate.

Phase 6 -- CI integration  [depends on INT-048]
  CI pushes to Attic on build success. Give CI its OWN scoped push-only token
  (never the admin token). Deferred to INT-048.

## Gates
- [x] deps-vs-own-code split measured: deps 109s (70%) / our code 48s (30%) of 156.8s (Phase 0, 2026-06-17)
- [x] faelight-forest converted to crane (deps-only derivation split out) (Phase 1, 2026-06-17)
- [x] source-only change rebuilds our crates only; deps are a cache hit (Phase 1, 2026-06-17)
- [x] crane deps-hash stable across .rs edits (Phase 1b, 2026-07-07, commit 7eb0075f)
- [x] Attic self-hosted cache stood up, atticd running on 127.0.0.1:8080 (Phase 2, 2026-07-07)
- [x] crane deps closure pushed to Attic: 667 paths, 0 skipped (Phase 3, 2026-07-07)
- [x] clean-store pull of full closure from Attic succeeds: 662 paths into scratch store (Phase 4, 2026-07-07)

## Why not Cachix (historical -- proven dead-end 2026-07-07)
Cachix is multi-tenant with content-addressed GLOBAL dedup. Our crane crate paths are
not on cache.nixos.org (404) but ARE in Cachix's global store from other tenants'
identical NARs. Cachix therefore reports "already present" and skips upload, but never
links the NAR into OUR cache's served namespace -> the path 404s on our URL. Proven:
cachix push (even single-path) says "Nothing to push - all store paths are already on
Cachix" while the path 404s on both our cache and cache.nixos.org; nix copy --from into
a scratch store fails ("...is not valid"); nix copy --to cannot push to Cachix (HTTP
405, URL read-only). No client-side override exists. Correct verify method = narinfo
HTTP HEAD (curl -sI .../<hash32>.narinfo), NOT nix path-info --store (false-negatives).
The old faelight-forest.cachix.org substituter/token were removed; the exposed auth
token was rotated then retired.

## Depends On / Related
- INT-048 (forest-ci) -- CI pushes to the cache on success (Phase 6).
- Lix (separate future intent) -- Nix CLI/daemon fork; orthogonal to crane + cache.

## Security notes (Attic, local-only)
- JWT RS256 secret: /etc/atticd.env, root-only (600), NOT in git/store. Localhost-only
  listener (127.0.0.1:8080) is the real security boundary -- confirmed via ss -tlnp.
- Cache is --public: safe ONLY because it is loopback-bound. If ever networked
  (deferred "network Attic"), revert to private + scoped tokens + TLS + firewall.
- Admin token is broad (pull/push/create/configure/delete, 10y). Fine for solo local
  use; give CI a narrow push-only token if INT-048 uses the cache.

## The Rule
"The forest should not recompile what has not changed.
 Build once. Cache everywhere. And do not marry the first tool -- there is always
 something that fits better." 🌲
