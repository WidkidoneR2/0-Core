---
id: 048
date: 2026-06-09
type: infrastructure
title: "forest-ci: local CI with Gitea and Hydra for flake builds"
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
