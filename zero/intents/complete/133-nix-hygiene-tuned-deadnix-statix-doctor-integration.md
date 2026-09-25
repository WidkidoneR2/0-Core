---
id: 133
date: 2026-07-08
type: future
title: "Nix hygiene in the forest: tuned deadnix/statix + doctor integration"
status: complete
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
- [x] deadnix + statix tuned config committed (noise suppressed, real signal kept); tuned deadnix + statix check on nix/ return clean-or-real-findings-only <!-- 2026-07-11: statix.toml at repo root disables `repeated_keys` (idiomatic flat-dotted boot.* keys) while keeping real lints. deadnix runs with --no-lambda-pattern-names. PROVEN: tuned statix dropped the repeated-keys false positive and kept 5 real empty-pattern findings; tuned deadnix returned CLEAN. Signal, not noise. -->
- [x] Doctor "Nix Hygiene" check runs the tuned linters; green when clean, warns on genuine findings <!-- 2026-07-11: check_nix_hygiene(core_root) in checks.rs shells out to deadnix (--no-lambda-pattern-names) + statix check (reads statix.toml via current_dir), parses stdout (both exit 0 even with findings -- verified, exit codes useless). Registered after check_nix_store; added to nixos_names in cockpit.rs to render in ❄ NixOS. Live on gen 350: '✅ Nix Hygiene  Nix code clean'. -->
- [x] Check catches a real issue -- demonstrated, not declared <!-- 2026-07-11: no seeding needed -- caught 5 REAL empty-patterns ({ ... }: -> _) across faelight-insightd/notify/wsd/bar.nix + nix/tests/boot.nix. Doctor showed '⚠️ Nix Hygiene  5 findings'. statix fix (--dry-run-verified) resolved all 5; doctor went '✅ Nix code clean'. Full loop real-finding -> fix -> green, demonstrated. -->
- [x] Check runs fast enough not to slow the doctor dashboard <!-- 2026-07-11: full `d` (33 checks incl. Nix Hygiene shelling out to deadnix + statix over nix/) ran in ~0.57s deployed. No perceptible slowdown. -->
- [x] (optional) Friday knowledge-feed layer -- DEFERRED with reason <!-- 2026-07-11: explicitly DEFERRED. Nix-hygiene lint rules are static/declarative; Friday's engine is for situated behavioral patterns (command sequences, failure loops). Static lint rules don't fit its model -- would be noise, not knowledge. Charter itself flagged this as least-fitting. Defer unless a concrete Friday-relevant Nix pattern emerges. -->

## RESOLUTION (2026-07-11): SHIPPED -- the forest self-monitors its Nix code quality.

deadnix + statix are now FOREST INFRASTRUCTURE, not manual tools. A tuned "Nix Hygiene" check
runs in the doctor's ❄ NixOS section: green when clean, warns on genuine dead code / anti-patterns
only. The whole game was tuning (raw = 40+ false warnings; tuned = signal): statix.toml disables
`repeated_keys` (idiomatic flat-dotted keys), deadnix runs --no-lambda-pattern-names (idiomatic
module headers). Both tools exit 0 even with findings, so the check parses stdout, not exit codes.

Earned its keep immediately: caught 5 real empty-pattern issues ({ ... }: -> _:) nothing else
tracked, across faelight-insightd/notify/wsd/bar.nix + a test node. statix fix (dry-run-verified)
resolved them; the check went green (live on gen 350). Friday knowledge-feed explicitly deferred
(static lint rules don't fit Friday's situated-pattern engine).

Extends faelight-deadwood (structural orphans) + doctor-v2 into Nix CODE hygiene.

## Relationship
- Extends: faelight-deadwood (structural orphans) + doctor-v2 (INT-050) into Nix CODE hygiene.
- Companion to: the deadnix/statix install + empty-profiles cleanup (2026-07-08, commit 6b107233).
- Relates to: INT-073 (generation prune policy) + the broader forest-hygiene theme.

## Notes
deadnix/statix installed 2026-07-08 as a warm-up; this intent makes them FOREST
INFRASTRUCTURE (self-monitoring) rather than manual tools. The whole game is tuning:
raw = noise, tuned = signal. Filed directly (core intent new errors -- templates/ dir
missing, lost in the INT-061 tree move; flagged for a hygiene-pass fix).
