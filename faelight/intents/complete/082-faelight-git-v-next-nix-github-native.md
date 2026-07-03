---
id: 082
date: 2026-06-23
type: future
status: complete
title: "faelight-git v-next: Nix + GitHub-native rewrite, shed Arch-era lock model"
tags: [faelight-git, rewrite, nix, github, productivity, post-025]
version: TBD
---
## Why
faelight-git (fg) is the daily git lifecycle tool -- `fg commit`, `fg done`, `fg sync`,
and the cistart/cicomplete intent lifecycle + deploy-pipeline integration. It is Arch-era:
it still carries core-protect lock-guards (`bail!("Core is locked. Run 'unlock-core'...")`
in commit.rs/done.rs/quick.rs/main.rs) that assume the chattr +i immutability model LUKS +
the Nix store made obsolete. INT-025 removes those guards surgically; this intent is the
deliberate, clean rebuild that makes fg *think Nix + GitHub* natively and smarter.
## Must-Keep (non-negotiable -- these are daily muscle memory)
- `fg commit` (with intent-aware messages), `fg done`, `fg sync` (pull/commit/push).
- The cistart / cicomplete intent lifecycle integration.
- Deploy-pipeline integration (commit -> deploy flow, health gate awareness).
- Intent-to-commit recording (genealogy; see INT-071 parity work).
## Goals (the upgrade)
- NIX-NATIVE: understands generations, flake dirty-state, `nixos-rebuild` context; ties
  commits to generations where useful (pairs with INT-034 triad tracking).
- GITHUB-NATIVE: PRs, releases, issue refs; clean auth (token already cached); maybe gh CLI.
- SMARTER / PRODUCTIVITY: better commit-message assistance, faster sync, clearer status,
  fewer prompts in the happy path, intent-aware suggestions.
- NO Arch-era lock model (post-025: no core-protect, no lock-core/unlock-core guards).
## Depends On / Sequencing
- AFTER INT-025 (core-protect retirement) -- 025 removes the lock-guards from current fg;
  this intent rebuilds on the cleaned base. Do NOT start until 025 is complete.
- Relates: INT-017 (prior faelight-git NixOS audit, complete), INT-071 (Friday commit
  recording parity), INT-034 (release triad), INT-008 (GitHub org).
## Phases (rough -- refine when started)
Phase 0 -- audit current fg surface (every subcommand, every db/registry touchpoint).
Phase 1 -- design v-next architecture (Nix + GitHub modules, keep lifecycle API stable).
Phase 2 -- implement, preserving fg commit/done/sync + cistart/cicomplete contracts.
Phase 3 -- deploy-pipeline + generation/PR integration.
Phase 4 -- migrate, verify daily loop unchanged for muscle memory, retire old paths.
## The Rule
"The forest's hands should think in the forest's substrate." 🌲
