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

## Full Specification (2026-06-04)

### Problems to solve
1. `intent list` doesn't work -- requires `core intent list`
2. ID counter reads archive/decisions, creates wrong numbers (was jumping to 277)
3. arch-era/ clutters the active ledger view
4. Deferral override is too rigid -- no simple command to override with reason
5. Friday doesn't understand the intent system deeply enough

### Design decisions

**arch-era retention policy:**
- Keep arch-era/ for ~1 month as migration reference
- After that: move to labs/graduated/arch-era/ or delete
- Incidents stay permanently as learning tools
- Philosophy docs stay permanently

**intent shorthand:**
- `intent list` → works directly without `core` prefix
- `intent show NNN` → works directly  
- `intent next` → works directly
- The `intent` binary needs to call core intent properly

**Better ledger display:**
- Show NixOS era intents cleanly (001-025+)
- Archive/decisions not shown in normal list
- Era indicator in display header
- Friday context shown per intent

**Deferral override:**
- `core intent defer NNN "reason"` → creates proper deferral with timestamp
- `core intent override NNN "reason"` → bypasses gate with logged reason
- Friday learns from both patterns -- deferred vs overridden vs completed

**Friday integration:**
- Friday understands intent velocity (how fast we complete)
- Friday flags contradictions between active intents and values
- Friday learns which intent types complete fastest
- Friday suggests next intent based on forest state

### Gate
- [ ] intent list works without core prefix
- [ ] ID counter only reads active folders
- [ ] arch-era not shown in normal list view
- [ ] core intent defer NNN works cleanly
- [ ] core intent override NNN works with reason logging
- [ ] Friday learns from intent patterns
