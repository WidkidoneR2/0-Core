---
id: 048
date: 2026-06-09
type: infrastructure
title: "local CI gates are disarmed in a fresh clone -- core.hooksPath does not travel"
status: planned
tags: [ci, gitea, hydra, nix, flake, build, infrastructure, nixos]
priority: low
---
## Why
Every push to the nixos branch rebuilds the flake on the real machine.
There is no pre-flight build check.
A broken flake.nix is discovered at rebuild time -- on the live system.

Forest CI gives the forest a local build validator:
  Push to nixos branch
  Gitea receives the push
  Hydra builds the flake derivations
  Build status visible before running rebuild on real machine

This is INT-024's graduation pipeline extended to the NixOS layer.

## Vision
  git push                    -- triggers Gitea webhook
  Hydra picks up the build    -- builds faelight-forest derivation
  Build passes: safe to rebuild
  Build fails: fix before touching live system
  fsh command: ci status      -- shows current build status
  fsh command: ci log         -- shows last build log

## What Already Exists
INT-024: VM-based R&D pipeline, graduation discipline
Gitea: available as NixOS service (services.gitea)
Hydra: available as NixOS service (services.hydra)
flake.nix: already builds faelight-forest as derivation
nixos-lab.qcow2: VM available for isolated builds

## ⚠️ EVERYTHING FROM HERE TO THE RESCOPE IS HISTORY, NOT A PLAN

The approach, the four phases and the nine gates below describe Gitea plus Hydra
building flake derivations. nix/ and flake.nix were deleted with the Omarchy
migration on 2026-08-28, and Hydra is a Nix build farm with nothing left to build.
Not one of those gates is work to do.

They are kept rather than cut because the reasoning is sound and the rule at the end
still holds -- and because the ledger records what was true when written. The title
and the RESCOPED section carry the live finding; read those.

This marker exists because the section head and the gate list read as open work. A
document whose top and middle disagree is the defect ALIASES.md was, and someone
hitting Phase 1 first should know before planning a VM pipeline for a wiped machine.

## Approach
OPTION A -- Gitea + Hydra on local machine (full)
  Run Gitea as NixOS service (local git host)
  Run Hydra as NixOS service (Nix-native CI)
  Mirror 0-Core repo to local Gitea
  Hydra jobset evaluates flake.nix on push
  Pros: fully Nix-native, reproducible
  Cons: resource-heavy, complex setup

OPTION B -- Gitea + simple build script (lightweight)
  Run Gitea as NixOS service
  On push webhook: run nix build .#faelight-forest in VM
  Report pass/fail to state.db
  fsh ci status reads state.db
  Pros: simpler, lower overhead
  Cons: not full Hydra power

Recommended: OPTION B first, graduate to OPTION A if needed.

## Phases

Phase 1 -- Gitea setup (VM first via INT-024)
  Enable services.gitea in VM flake
  Mirror 0-Core repo to local Gitea
  Gate: git push to local Gitea works in VM

Phase 2 -- Build webhook
  Webhook triggers nix build .#faelight-forest on push
  Result (pass/fail + log) written to state.db
  Gate: push triggers build, result in state.db

Phase 3 -- fsh ci commands
  ci status -- shows last build result (pass/fail/pending)
  ci log    -- shows last build output
  ci watch  -- live build progress
  Gate: ci status shows correct result after push

Phase 4 -- Graduate to real machine
  All VM phases pass
  Enable Gitea on real machine via flake
  Gate: full CI pipeline running on Framework 16

## Gates
- [ ] Gitea running in VM as NixOS service
- [ ] 0-Core repo mirrored to local Gitea
- [ ] Push to local Gitea triggers build
- [ ] Build result written to state.db
- [ ] ci status shows pass/fail in fsh
- [ ] ci log shows build output in fsh
- [ ] All VM gates pass before real machine setup
- [ ] Gitea running on real machine via flake
- [ ] Push to real Gitea triggers build and reports to fsh

## Depends On
- INT-024 (VM graduation pipeline) -- all phases tested in VM first
- INT-030 (fsh semantic domains) -- ci as first-class fsh command

## The Rule
"A broken flake should never reach the live machine.
 The forest validates before it deploys.
 CI is not bureaucracy -- it is discipline." 🌲

## RESCOPED 2026-08-27 -- Hydra is gone, and the real defect was found live

FALSE PREMISE: forest-ci was scoped as a local build farm (Gitea + Hydra) for
flake builds. Hydra is a Nix build farm and flakes are gone. Gitea itself runs
anywhere and Gitea Actions has been built in since 1.19, so a runner remains
possible -- but that is not the finding.

THE ACTUAL DEFECT, measured 2026-08-26 during the Omarchy migration:
A FRESH CLONE ARRIVES WITH ITS HOOKS DISARMED. core.hooksPath is LOCAL git
config and is not cloned. Every commit made between the clone and running
git config core.hooksPath .githooks was ungated -- no rustfmt, no fsh-test, no
pre-push check.

WHY THIS IS THE SAME DISEASE THE LEDGER KEEPS FINDING: the gate did not fail.
It was ABSENT, and absence is silent. A skipped hook and a passing hook look
identical from the outside. That is the INT-110 warning arriving from a
direction nobody had checked.

WHAT LOCAL CI ALREADY IS HERE: the pre-push hook runs fsh-test and blocks the
push. It has earned its keep -- it refused the push after the runtime-path move
because two assertions still named the old location. The mechanism works. Its
ACTIVATION is what does not travel.

SUCCESS CRITERIA
- [ ] A fresh clone either arms its own hooks or refuses to commit until armed
- [ ] Watch it fail first: clone to /tmp, commit something rustfmt would reject,
      confirm it goes through today
- [x] The check reports which gates are active, so no output cannot mean not
      installed
      <!-- DONE 2026-09-02, two halves.

      THE DOCTOR ASKS THE QUESTION NOW. check_hooks reports core.hooksPath and whether
      the hooks carry the executable bit, with three distinct failure modes: unset,
      pointed elsewhere, or present but not executable -- git skips that last one
      SILENTLY, which a fresh clone can produce. Unknown when git itself cannot be
      asked, per INT-148.

      Demonstrated by unsetting core.hooksPath and running d: the check went red and
      named the disarmed gate. Restored, and it passes.

      AND THE GATE NAMES ITS OWN BINARY. The wrapper resolves target/debug, then
      target/release, then PATH -- debug FIRST so the gate judges the tree being
      pushed. Nothing reported which won, so a defeated preference looked identical to
      a satisfied one. Measured the same day: target/debug/zero-gate did not exist, so
      every push that day ran the release build, current only because ship happens to
      rebuild it. It now prints one line when it is not the debug build, and a second
      if a NEWER debug build exists beside it. Silent otherwise -- silent on success is
      still the rule.

      ⚠️ THE OTHER THREE CRITERIA STAY OPEN. This makes absence VISIBLE; it does not
      make a fresh clone arm itself, and nothing yet refuses to commit until armed.
      That is the remaining work and it is the harder half. -->
- [ ] Decide separately whether a runner (Gitea Actions) is wanted at all
