---
id: 118
date: 2026-07-02
type: future
title: "Friday engine resumption"
status: planned
tags: [core engine, Friday]
---

## Why
Friday's reasoning-engine development paused ~1 month during the NixOS migration +
restructure. This intent RECONSTRUCTS the thread (from git history + past sessions) so
resuming is picking up a defined task, not staring at a blank canvas. The concrete next
step was already identified pre-pause; it just needs re-homing in the new tree/ledger.

## Where Friday was (reconstructed 2026-07-03)
- Vision intact: Friday = a living intelligence grown from the forest -- observation,
  memory, reasoning, its own emerging vocabulary. Not a chatbot, not internet-trained.
  Learns from the forest's own data (intents, commits, commands, health, decisions).
- Capability ladder (Arch-era "Core vN" naming, historical): v9 Intent -> v10 Reaction
  -> v11 Prediction -> v12 Strategy -> v13 Autonomy. (NOTE: crate semver is separate --
  engine Cargo.toml is 3.1.0. Do NOT reset either; capability-eras are history, semver is
  tooling. See INT-111 gen-vs-version axiom.)
- The specific next step (old INT-342, lost in the restructure renumber -- RE-HOMED here):
  "Friday Reasoning Engine -- Causal Chain Rule for Tool Retirement Regression Detection."

## The concrete resumption task (old INT-342, reconstructed)
PROBLEM: the event bus captures DEPLOYS but not `tool_retired` or `health_check_failed`
as discrete events. So the reasoning engine cannot connect A->B->C (e.g. "retired a tool
-> health dipped -> 3 rapid deploys to fix it").
SOLUTION (two parts):
  1. Emit `health_check_failed` events from the doctor domain (discrete, on the event bus).
  2. Add a causal-chain RULE that detects rapid deploy cycles following a health dip.
GATE IT UNLOCKS: INT-251 Pillar 2 -- causal-chain reasoning DEMONSTRATED on a real
failure case (not simulated). Evidence pattern from the original: "tool retired -> d
failed -> 3 core deploys in 5 min to fix." Those events must all be on the bus for the
chain to be reconstructable.

## Strategic direction (banked, longer-horizon)
- "fsh as authority, [visual layer] as presentation, Friday as the reasoning engine."
  (Original note said COSMIC as the visual layer; on NixOS the visual layer is Mango now /
  Miracle later -- the PRINCIPLE holds: fsh = authority, compositor = presentation,
  Friday = reasoning. Re-evaluate the visual-layer choice; keep the separation of concerns.)
- Causal Chain Rule for Tool Retirement Regression Detection = the first concrete causal
  capability; the doctor-events work above is its foundation.

## Suggested re-entry sequence (calm, after a month away)
Phase 0 -- Re-read: `core intent show 251` (Pillar 2), grep the events domain for the
  current event types emitted, confirm what the event bus captures TODAY on NixOS (the
  Arch-era assumptions may have shifted in the restructure).
Phase 1 -- Emit health_check_failed from the doctor domain as a discrete event.
Phase 2 -- Add the causal-chain rule (rapid-deploy-after-health-dip detection).
Phase 3 -- Demonstrate on a real (or safely reproduced) failure case -> unlock INT-251
  Pillar 2. Demonstrated, not declared.

## Recon needed on resume (a month of drift)
- Does the current NixOS engine still have the same event-bus / events domain shape the
  Arch-era 342 assumed? (Restructure moved engine -> faelight/engine; verify event types.)
- Is INT-251 still the live "Pillars" tracker, or was it renumbered too?
- What causal infrastructure already exists (why_chain / why_correlate / why_suggest were
  built in the Arch "Core v5" era -- do they survive in faelight/engine)?

## Relationship
- INT-117 (Friday language cleanup) is the natural warm-up before this.
- Ties to INT-251 (reasoning Pillars), the old INT-342 (re-homed here), and the Friday
  learning-architecture (curriculum: blocks -> puzzles -> tic-tac-toe -> why/causal).
- NOT a 1.0.0 blocker -- post-release engine development.

## Gates
- [ ] Event-bus / events domain state re-confirmed on NixOS (Phase 0 recon)
- [ ] health_check_failed emitted as a discrete event from doctor
- [ ] causal-chain rule detects deploy-cycle-after-health-dip
- [ ] demonstrated on a real failure case (INT-251 Pillar 2 unlocked)
