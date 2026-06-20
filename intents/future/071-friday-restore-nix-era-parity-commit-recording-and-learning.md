---
id: 071
date: 2026-06-20
type: future
title: "Friday: restore Nix-era parity (commit-to-intent recording, then learning)"
status: planned
tags: [friday, learning, commit-recording, intent-commits, parity, migration, nixos]
---

## Why
Friday's commit->intent recording has been dark since the Arch->NixOS migration: the
intent_commits table froze at an Arch-era commit, so Friday no longer learns from new
commits, and INT-034 (generation + commit + intent triad tracking) is blocked behind it.
More broadly, Friday kept doing its job but "not the Nix way." This intent restores
Friday's Arch-era parity on NixOS.

## What
- Repair commit->intent recording so each new commit records again (intent_commits
  advances past the Arch-era freeze).
- Restore Friday learning from commit history.
- Audit and close the remaining "worked on Arch, not on Nix" Friday gaps.
Scope boundary: this is PARITY RESTORATION (recover what the migration broke), distinct
from INT-039 (friday-daemon) and INT-041 (shell-context), which are NEW Friday features.

## Approach
Phase 0 audits Friday's Arch-era behaviour vs current Nix behaviour and records the gap
list (commit-recording is the headline; find the rest). Phase 1 diagnoses why the
recording hook stopped firing after the migration (path, db location, or the sync step
that moved with the repo) and re-wires it. Phase 2 confirms Friday ingests commits again.
Phase 3 closes or formally defers the remaining Phase-0 gaps.

## Phases
Phase 0 -- parity audit: record the Arch->Nix gap list here.
Phase 1 -- repair commit->intent recording (intent_commits frozen since migration).
Phase 2 -- learning resumes: Friday consumes recent commit history again.
Phase 3 -- close remaining parity gaps (or formally defer).

## Gates
- [ ] Phase 0: Friday Arch->Nix parity-gap list recorded in this charter
- [ ] commit->intent recording repaired: a new commit records an intent_commits row past the Arch-era freeze
- [ ] Friday learns from recent commits again (demonstrated, not just wired)
- [ ] remaining Phase-0 parity gaps resolved or formally deferred

## Notes
- Unblocks INT-034 (triad tracking needs live commit-recording).
- Distinct from INT-039 (daemon) and INT-041 (shell-context): new features, not parity.
- Your framing: "doing its job, just not the Nix way" -- this is the recovery.

## The Rule
"The forest remembers -- including why each commit was made." 🌲
