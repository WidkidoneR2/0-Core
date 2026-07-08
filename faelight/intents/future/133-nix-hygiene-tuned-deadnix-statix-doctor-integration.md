---
id: 133
date: 2026-07-08
type: future
title: "Nix hygiene in the forest: tuned deadnix/statix + doctor integration"
status: planned
tags: [nix, hygiene, deadnix, statix, doctor, friday, linting]
---

## Vision
The forest self-monitors its own Nix code quality -- dead code and anti-patterns surfaced
automatically, the way faelight-deadwood surfaces structural orphans. deadnix + statix,
TUNED to signal-not-noise, wired into the doctor health check, with findings optionally
feeding Friday's knowledge. Christian's framing: "deadnix and statix will benefit Friday"
-- doctor runs them, they keep the forest's Nix clean, Friday learns the patterns.

## Why Now (learned in the 2026-07-08 warm-up)
Installed deadnix 1.3.1 + statix. Findings:
- RAW output is mostly NOISE: deadnix flags idiomatic `{ config, pkgs, lib, ... }:` module
  headers as "unused lambda pattern" (40+ warnings across hosts/ and home/). statix flags
  flat-dotted-keys (boot.loader.x / boot.plymouth.y) as a style opinion (repeated keys).
- BUT `deadnix --no-lambda-pattern-names` returns CLEAN -- the forest's Nix has NO real
  dead code once idiomatic headers are excluded.
- KEY INSIGHT: the tools are only useful TUNED. A doctor check running RAW linters would
  cry wolf (40 false warnings). Tuning is the prerequisite, not an afterthought.
- They DID earn their keep: surfaced 5 empty orphan profiles/*.nix files (base/desktop/
  laptop/development/security) -- cleaned + verified same session (commit 6b107233). That
  is the value when noise is filtered: real deadwood nothing else was tracking.

## The Solution (3 layers)
1. TUNE (prerequisite): a deadnix invocation/config with --no-lambda-pattern-names; a
   statix.toml disabling the style lints not wanted (repeated-keys), keeping the real ones
   (empty patterns -> `_`, genuine anti-patterns). Without this, integration is noise.
2. DOCTOR INTEGRATION: a "Nix Hygiene" health check running the TUNED linters -- flags real
   dead code / anti-patterns / empty files, like deadwood does orphans. Green when clean,
   warns on genuine findings only. (Extends INT-050 doctor-v2.)
3. KNOWLEDGE FEED (optional, thin, lowest priority): Friday learns Nix-hygiene patterns.
   Honestly the least-fitting layer -- static lint rules are not the situated pattern
   knowledge Friday's engine is built for. Evaluate; adopt or explicitly defer.

## Success Criteria (gates -- refine at cistart)
- [ ] deadnix + statix tuned config committed (noise suppressed, real signal kept); tuned deadnix + statix check on nix/ return clean-or-real-findings-only
- [ ] Doctor "Nix Hygiene" check runs the tuned linters; green when clean, warns on genuine findings
- [ ] Check catches a seeded real issue (empty file or dead let-binding) -- demonstrated, not declared
- [ ] Check runs fast enough not to slow the doctor dashboard (parallel with other checks)
- [ ] (optional) Friday knowledge-feed layer evaluated -- adopt or explicitly defer with reason

## Relationship
- Extends: faelight-deadwood (structural orphans) + doctor-v2 (INT-050) into Nix CODE hygiene.
- Companion to: the deadnix/statix install + empty-profiles cleanup (2026-07-08, commit 6b107233).
- Relates to: INT-073 (generation prune policy) + the broader forest-hygiene theme.

## Notes
deadnix/statix installed 2026-07-08 as a warm-up; this intent makes them FOREST
INFRASTRUCTURE (self-monitoring) rather than manual tools. The whole game is tuning:
raw = noise, tuned = signal. Filed directly (core intent new errors -- templates/ dir
missing, lost in the INT-061 tree move; flagged for a hygiene-pass fix).
