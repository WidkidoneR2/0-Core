---
id: 029
date: 2026-06-04
type: feature
title: "Autonomous sandboxes: sandbox create orchestrates VM + branch + intent"
status: planned
tags: [sandbox, vm, intent, experiment, isolation]
priority: medium
---

## Vision

sandbox create render-optimization
Creates:
  - VM (isolated environment)
  - git branch (isolated code)
  - intent (tracked goal)
  - experiment entry in labs/
  - notes file
Everything isolated. Graduate it later. Disposable R&D.

## Why

The forest already has all the pieces.
Intents track goals. labs/ holds experiments. VMs provide isolation.
The sandbox command orchestrates them all in one gesture.

## Approach

- core sandbox create <name> command
- Creates: VM via vm create, git branch, intent, labs/experiments/<name>/
- core sandbox graduate <name> moves to labs/graduated/
- core sandbox list shows active sandboxes
- Integration with INT-027 (VM-native dev)

## Gate

- [ ] sandbox create <name> creates all artifacts
- [ ] sandbox list shows active sandboxes with status
- [ ] sandbox graduate <name> promotes to labs/graduated/
- [ ] Sandbox tied to intent for tracking
