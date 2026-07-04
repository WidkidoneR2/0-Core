---
id: 119
date: 2026-07-04
type: future
title: "Git-Hooks.nix evalution"
status: planned
tags: [nix, git-hook]
---

## Why
faelight-hooks (v10.2.0, our own Rust git-hooks tool: gitleaks/clippy/rustfmt/commitmsg/
prepush/branch/conflicts/filesize/secrets + install.rs) is the AUTHORITY for local
pre-commit checks and stays that way. This intent evaluates cachix/git-hooks.nix as a
COMPLEMENT -- specifically for one capability faelight-hooks can't easily provide: hooks
wired into `nix flake check`, run in a Nix SANDBOX (read-only FS, no network),
reproducible from the flake. That directly serves the roadmap's "improve sandboxing"
goal.

## What git-hooks.nix is (researched 2026-07-04)
- A Nix-flake integration layer (NOT a standalone competing tool). Wires hooks into the
  flake; `nix flake check` runs them sandboxed + reproducibly.
- Ships built-in Rust hooks (clippy, rustfmt, cargo-check) + secret-detection
  (ripsecrets, trufflehog) -- overlaps faelight-hooks, so this is a COMPLEMENT decision,
  not a replacement.

## Explicitly REJECTED alternatives (researched, do not revisit without cause)
- jdx/hk (Pkl-configured general hooks manager) -- would REPLACE faelight-hooks with a
  third-party tool + add a Pkl dependency; less forest control. Skip.
- j178/prek (fast pre-commit reimplementation) -- only valuable if already using the
  pre-commit YAML ecosystem, which we are not. Skip.
Both compete with a tool we built + value controlling. git-hooks.nix is different: it
adds a flake-native sandboxed check LAYER, not a replacement.

## The real design question (decide during eval)
Does faelight-hooks stay the sole authority, OR do we run a HYBRID:
- faelight-hooks = commit-time (rich, local, candy-neon, intent/health-aware)
- git-hooks.nix = `nix flake check`-time (sandboxed, reproducible, CI-style gate)
They serve different MOMENTS. Hybrid keeps faelight-hooks's forest-native experience while
gaining a sandboxed flake-check gate. Evaluate whether that gate earns its place.

## Approach (demonstrated, not declared)
- Try git-hooks.nix in a BRANCH (not main) -- add the flakeModule, wire a couple hooks
  (rustfmt + a secret check) into `nix flake check`.
- Confirm it runs sandboxed (no network, read-only) and reproducibly.
- Compare: does it catch anything faelight-hooks doesn't, at flake-check time?
- Decide: adopt as complement / reject / defer. Keep ONLY if it earns its place.

## Gates
- [ ] git-hooks.nix wired into a branch flake; `nix flake check` runs hooks sandboxed
- [ ] Overlap vs faelight-hooks characterized (what each covers, at which moment)
- [ ] Hybrid-vs-reject decision made + recorded (with rationale)
- [ ] If adopted: faelight-hooks remains commit-time authority; git-hooks.nix is
      flake-check gate only -- boundary documented

## Relationship
- Complements faelight-hooks; does NOT replace it.
- Serves the roadmap "improve sandboxing / VM testing" goal.
- NOT a 1.0.0 blocker.
