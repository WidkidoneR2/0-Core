---
id: 031
date: 2026-06-04
type: feature
title: "faelight-release v2: NixOS-native release manager"
status: in-progress
tags: [faelight-release, nixos, release, changelog, version]
priority: high
---

## Why

faelight-release v1 is Arch-era and dangerous on NixOS:
- Writes to /etc/faelight/VERSION (immutable on NixOS)
- Reads 300+ git commits including Arch-era history
- References 00-meta/CHANGELOG.md (being renamed to meta/)
- No understanding of NixOS generations as release artifacts
- bump alias disabled until this is fixed

## What needs changing

1. Version source: /etc/faelight/VERSION → flake.nix version field
2. Changelog path: 00-meta/CHANGELOG.md → meta/CHANGELOG.md
3. Git log scope: filter to NixOS era commits only (after 2026-06-01)
4. Release artifact: NixOS generation number as part of release identity
5. No Arch assumptions anywhere in changelog generation
6. README update: NixOS-aware, not Arch-aware

## Vision

faelight-release v2 on NixOS:
- Reads version from flake.nix
- Changelog scoped to NixOS era intents (INT-001+)
- Release = NixOS generation + semantic version + intent summary
- Writes to meta/CHANGELOG.md
- bump alias re-enabled after this is complete

## Pre-1.0.0 requirement

This MUST be complete before Faelight NixOS 1.0.0.
The release tool creates the release artifact.
A 1.0.0 release made with broken tooling is not a real release.

## Gate

- [ ] bump runs without errors on NixOS
- [ ] Changelog only includes NixOS era commits
- [ ] Version reads from flake.nix correctly
- [ ] /etc/faelight/VERSION write replaced with NixOS-appropriate mechanism
- [ ] meta/CHANGELOG.md path correct
- [ ] README generation NixOS-aware

## Release Identity Philosophy (2026-06-04)

The release triad:
  Release version = NixOS generation number = Git commit count

Example:
  Faelight NixOS 1.0.0
    NixOS generation: 47
    Git commits: 2984
    Intents complete: INT-001 through INT-025

This means Friday can:
- Trace any bug to exact generation + commit
- Answer "which generation is stable?"
- Cross-reference release artifacts with rollback targets
- Warn when a generation is about to be garbage collected

faelight-release v2 must record all three in state.db on every release.
The triad survives garbage collection. Generations do not.


## Phase 0 -- DONE (2026-06-27): survey faelight-release + triad data + MERGE with INT-034
RECON (current state):
- faelight-release is a real 2678-line tool, 7 modules: main.rs(432) changelog.rs(602)
  intelligence.rs(311) learning.rs(249) readme.rs(223) rollback.rs(218) tui.rs(643).
  Already has generation awareness: show/list/rollback subcommands; main.rs:340 reads
  runtime/generation. So it is a de-Arch + rework (like faelight-update was), NOT from scratch.
- ARCH-ERA TOUCHPOINTS confirmed (031's stated fixes are real):
  * main.rs:147/155/161 -- writes to /etc/faelight/VERSION (immutable on NixOS) -> must move to
    flake.nix version field (read) + a writable mechanism for recording.
  * main.rs:116 + intelligence.rs:294 -- 00-meta/CHANGELOG.md & CHANGELOG.md paths -> meta/CHANGELOG.md.
  * learning.rs:47 -- 00-meta/releases dir -> meta/releases.
- TRIAD data partly present already: rebuild-record (the `rebuild` alias) writes git HEAD to
  ~/.cache/faelight/last-system-rev after each switch (INT-062 drift). So the COMMIT half exists
  (cache, not state.db). Generation is readable via `nixos-rebuild list-generations` /
  runtime/generation. Version is in flake.nix. The triad just needs RECORDING TOGETHER in state.db.

MERGE DECISION: INT-034 (triad tracking) is SUBSUMED INTO INT-031. The triad is the core
deliverable of release v2 -- 031's own "Release Identity Philosophy" section IS 034. Doing them
separately would touch the same 2678-line tool twice. We cistart BOTH, build as ONE effort,
cicomplete TOGETHER. 034's gates are folded into the combined gate-set below.

COMBINED GATE-SET (031 + 034), to complete fully:
  [ ] bump runs without errors on NixOS
  [ ] version reads from flake.nix (not /etc/faelight/VERSION)
  [ ] /etc/faelight/VERSION write replaced with NixOS-appropriate mechanism (state.db record)
  [ ] changelog scoped to NixOS-era commits only (after 2026-06-01)
  [ ] meta/CHANGELOG.md path correct (not 00-meta/)
  [ ] README generation NixOS-aware
  [ ] (034) every release records generation + commit count + intent range in state.db
  [ ] (034) `core release show` displays full triad history
  [ ] (034) Friday can answer "which generation is release X?"
  [ ] (034) GC warning fires before a release generation is collected

SEQUENCING (proposed phases for the build):
  Phase 1 -- de-Arch the paths (VERSION source -> flake.nix; 00-meta -> meta; remove immutable writes)
  Phase 2 -- triad recording: on bump, write {version, generation, commit_count, intent_range} to state.db
  Phase 3 -- triad surfacing: `core release show` history + Friday query + GC warning
  Phase 4 -- changelog/readme NixOS-aware (scope to INT-001+, meta/ paths)
NOTE: NOT cutting a Faelight NixOS 1.0.0 now -- faelight-release is being reworked first; a real
release waits until this tooling is sound (031's own rule: "a 1.0.0 release made with broken
tooling is not a real release").
