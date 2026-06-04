---
id: 035
date: 2026-06-04
type: infrastructure
title: "Forest safety net: pre/post health gate, VM-first workflow, rebuild guard"
status: planned
tags: [safety, health, rollback, vm, rebuild, guard]
priority: high
---

## Why

Rollback is not a complete safety net:
- Generation 46 behavior bled into clean boot after rollback
- Rollback can misbehave on first clean boot
- We cannot always rely on generations alone

## The Real Safety Net (layered)

Layer 1 -- VM testing first:
  Nothing high-risk touches real system until VM-proven.
  faelight-login, compositor changes, login flow = VM first.

Layer 2 -- rebuild guard (smart rebuild alias):
  Before rebuild: snapshot health score + generation number
  After rebuild: check health again
  If health drops: auto-rollback with reason logged to state.db
  Friday learns which types of changes are high-risk

Layer 3 -- release triad awareness:
  Friday knows which generation is stable
  Warns before garbage collection removes a stable generation
  Warns before high-risk rebuilds near stable releases

Layer 4 -- INT-027 VM-native development:
  vm create, vm snapshot, vm rollback as first-class commands
  Every risky intent gets a VM first

## Rebuild Guard Implementation

New alias: rebuild-safe
  1. Record current generation + health
  2. Run rebuild-dry first
  3. If dry-run passes, rebuild
  4. Check health after
  5. If health < pre-rebuild health: auto-rollback
  6. Log result to state.db with generation numbers

## Gate

- [ ] rebuild-safe alias works with health gate
- [ ] Auto-rollback fires if health drops after rebuild
- [ ] Friday logs rebuild outcomes to state.db
- [ ] VM workflow documented and working (INT-027)
- [ ] rebuild-safe used by default for high-risk changes
