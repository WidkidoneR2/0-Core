---
id: 102
date: 2026-07-01
type: decisions
title: "Version-bumping Faelight Tools on Nix"
status: planned
tags: [tools, bump, version, Nix, Faelight]
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

---

## Why
Tool versions drift stale because the only bump path is faelight-release, a full
FOREST-RELEASE ceremony (publish TUI, changelog, generation triad, rollback
points). That is far too heavy for "one tool got a patch." Result: faelight-shell
sat at 2.5.0 through BOTH INT-100 (parser fix) and INT-101 (schema fix) -- two
real shipped fixes, no version movement. There is no lightweight per-tool bump.

## The core conceptual split (the root of the problem)
NixOS conflated two DIFFERENT axes:
- TOOL VERSION = "this artifact's code changed" (faelight-shell 2.5.0 -> 2.5.1)
- FOREST GENERATION = "the whole-system state at gen N" (faelight-release's domain)
These are orthogonal. A tool patch should not require a forest release; a forest
release snapshots whatever tool versions exist at that moment. Today they're welded
together through faelight-release, so per-tool versioning has no home.

## Design space (decide before building)
PHILOSOPHY 1 -- per-tool semver, DECOUPLED from forest releases [LEANING]
  Each tool owns its version; bumps when ITS code changes, independent of forest
  generations. faelight-release stays a separate axis that snapshots the forest.
  The bump-versions registry already wants to be this -- the lightweight per-tool
  bump ACTION is what's missing.
PHILOSOPHY 2 -- versions bump ONLY at forest release
  Simpler but coarse; tool versions don't reflect individual fixes. This is the
  current de-facto state and the thing causing the frustration.
PHILOSOPHY 3 -- auto-bump on change via cicomplete
  cicomplete already SUGGESTS bumps ("faelight-shell patch or minor"). Close the
  loop: cicomplete detects which tools an intent's commits touched and bumps them.

## The hard sub-problem: WHO decides patch vs minor vs major?
This is a SEMANTIC judgment a tool can't fully automate. cicomplete suggested
"patch OR minor" for faelight-shell precisely because it can't tell a bug fix from
a feature. Options:
- (a) Human declares the level at cicomplete time (one prompt: patch/minor/major).
- (b) Convention via intent `type:` field -- intents ALREADY have type
  (feature/infrastructure/polish/future/decisions...). Map type -> semver:
  bugfix/polish -> patch, feature -> minor, breaking -> major. The deciding
  METADATA MAY ALREADY EXIST in the ledger. This is the promising lead.

## Integration question
Should cicomplete close the loop end-to-end: on intent completion, detect touched
tools (from the intent's commits), read the intent's type, bump each touched tool
by the mapped semver level, and record it? That makes versioning a natural
byproduct of the intent lifecycle instead of a forgotten manual chore.

## Where versions live (recon needed)
- bump-versions (display alias) reads a registry -- WHERE? (state.db? a versions
  file? faelight-release's triad?) Find the source of truth before designing writes.
- faelight-release commands: publish/plan/preview/status/history/query/gc-check/
  rollback/diff -- understand how it currently reads+writes versions.

## Gates (when built)
- [ ] Source-of-truth for tool versions located + documented
- [ ] A lightweight per-tool bump exists (NOT the full release ceremony)
- [ ] patch/minor/major decision mechanism chosen (human prompt or type-mapping)
- [ ] faelight-shell bumped to reflect INT-100/101 (retroactive first use)
- [ ] Forest-release vs tool-version axes cleanly separated (documented)

## Deferred
This intent CAPTURES the design space; a follow-up (or this one's later phase)
DECIDES the philosophy + builds. Do not build until the philosophy is chosen.

## Notes
Surfaced 2026-07-01 trying to bump faelight-shell after closing INT-100/101 --
found bump-versions is display-only and faelight-release is a full ceremony, with
no lightweight middle. Christian: "we need to figure out a systematic way to where
tool versions could be bumped up -- but deciding how is another thing."
