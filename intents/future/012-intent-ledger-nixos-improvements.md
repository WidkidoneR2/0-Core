---
id: 012
date: 2026-06-03
type: improvement
title: "Intent ledger NixOS improvements: intent shorthand, display, workflow"
status: planned
tags: [intent, ledger, nixos, workflow, fsh]
priority: medium
---

## Why

intent list, intent show, intent next all broken -- point at old scripts/core.
The intent binary needs NixOS path awareness.
Ledger display should show NixOS era cleanly.

## Approach

- Fix intent binary to use /run/current-system/sw/bin/core
- Or absorb intent binary into core intent domain fully
- Update display to show arch-era/ as historical, not active
- intent shorthand works from fsh without core prefix

## Gate

intent list shows NixOS era intents.
intent show NNN works for any intent.
intent next recommends NixOS-relevant work.
