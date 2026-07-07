---
id: 119
date: 2026-07-04
type: future
title: "Git-Hooks.nix evalution"
status: complete
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

## DECISION (gates 3-4) -- 2026-07-07: ADOPT AS HYBRID (git-hooks.nix = flake-check gate)
Verdict: **Adopt git-hooks.nix as a COMPLEMENT. It earned its place -- proven, not assumed.**
faelight-hooks stays the commit-time authority; git-hooks.nix is the sandboxed flake-check gate.

## How it was evaluated (same discipline as INT-043 Attic, INT-122 nixCats)
Spiked on branch experiment/git-hooks-119: added inputs.git-hooks, wired a
`checks.${system}.pre-commit-check` (rustfmt + ripsecrets) via the plain-flake API
`inputs.git-hooks.lib.${system}.run { src = ./.; hooks = {...}; }`. Ran `nix flake check`.

## The decisive demonstration (gate 1 + the "does it catch anything" question)
`nix flake check` FAILED -- rustfmt (sandboxed, pinned) caught REAL unformatted code
committed on main: faelight/rust-tools/teach/src/main.rs lines 184 & 217 (chained
.to_string_lossy().to_string() not wrapped). Host rustfmt --check confirmed the same
diffs -- genuine drift, not a version quirk. faelight-hooks had PASSED this code.
=> The reproducible gate caught what the host-dependent commit-time checker missed.
Not theoretical redundancy: a demonstrated, real gap closed.

## Why faelight-hooks missed it (the structural difference, concretely)
- faelight-hooks: checks STAGED files at COMMIT time, shells out to host rustfmt,
  SILENTLY SKIPS if the tool is absent. If code is committed while a tool is skipped
  (or via a bypassed hook), it slips in permanently -- nothing re-checks it.
- git-hooks.nix: `nix flake check` re-validates the WHOLE TREE every run, in a Nix
  sandbox (read-only FS, no network), with PINNED tools -- unskippable + reproducible.
Different MOMENT, different GUARANTEE. The drift on main is the proof the gate matters.

## Gates -- ALL MET
- [x] git-hooks.nix wired into a branch flake; `nix flake check` runs hooks sandboxed
      (it's a Nix build derivation -> read-only/no-network by construction)
- [x] Overlap vs faelight-hooks characterized: BOTH do rustfmt + secret-scan, but
      faelight-hooks = commit-time/staged/host-tool/skippable (rich UX, intent-aware);
      git-hooks.nix = flake-check/whole-tree/pinned/unskippable (reproducible gate).
      Overlap in WHAT, difference in the GUARANTEE + MOMENT. Not redundant.
- [x] Hybrid-vs-reject decision: ADOPT HYBRID, recorded here with rationale
- [x] Boundary documented: faelight-hooks = commit-time authority (rich/local/intent/
      health-aware); git-hooks.nix = flake-check gate ONLY (sandboxed CI-style).

## The wider lesson (reinforces 043/122; the honest guardrail direction FLIPPED)
043 & 122 = "don't stay anchored on the incumbent when it can't do the job."
119 = the MIRROR discipline: "don't ADD a tool just because it exists -- make it PROVE
a real gap." It did (caught real drift faelight-hooks passed). AND the deeper note
Christian named: sometimes the forest-serving move is to let an external tool do a job
BETTER than your own hand-built one -- ceding ground is not weakness when it serves the
whole. faelight-hooks doesn't have to do everything to be valuable; git-hooks.nix owns
the reproducible-gate role it's built for. Not-invented-here is its own anchor to resist.

## Serves
- Roadmap "improve sandboxing" -- literal sandboxed checks.
- INT-048 CI -- `nix flake check` IS the CI gate; this is the CI hook layer, ready.

## Follow-on (not part of this eval)
- The gate found real drift: fix teach/main.rs formatting (done on this branch).
- Consider which hooks to enable long-term (rustfmt + ripsecrets proven; clippy/others TBD).
