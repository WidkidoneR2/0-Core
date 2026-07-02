---
id: 003
date: 2026-06-03
type: infrastructure
title: "NixOS scripts layer: lock-core, unlock-core, deploy, core-protect"
status: complete
tags: [nixos, scripts, deploy, core-protect]
priority: high
---

## Why

deploy, rollback, forest-status, lock-core, unlock-core, auth-health,
reset-auth are all broken -- pointing at missing scripts/ directory.

## Approach

Write NixOS-aware replacements as shell scripts in pkgs/faelight/scripts/
or absorb into core subcommands where they belong.

- deploy → wraps nixos-rebuild switch + core doctor gate
- lock-core / unlock-core → core protect lock/unlock (already working)
- rollback → nixos-rebuild switch --rollback
- forest-status → core doctor quick

## Gate

All 5 placeholder aliases work correctly.
