---
id: 035
date: 2026-01-09
type: decision
title: "Sway Migration - Post-Omarchy Catastrophe"
status: complete
tags: [decision, architecture, sway, omarchy, recovery]
---

## The Incident

On January 9, 2026, an Omarchy system update caused catastrophic configuration conflicts:

- Hyprland configs became incompatible
- Default Omarchy settings overwrote custom 0-Core configurations
- System became unusable after update
- Recovery required complete WM migration

## Root Cause

Omarchy's opinionated defaults conflicted with 0-Core philosophy:
- Omarchy assumes control of user configs
- Updates can overwrite customizations
- Tight coupling between Omarchy components
- No clear separation between "theirs" and "ours"

## Decision

**Migrate from Hyprland (Omarchy) to Sway (vanilla Arch)**

### Why Sway?

| Factor | Hyprland/Omarchy | Sway |
|--------|------------------|------|
| Stability | Bleeding edge | Mature, stable |
| Independence | Tied to Omarchy | Vanilla Arch |
| Config control | Can be overwritten | 100% user-owned |
| Wayland native | Yes | Yes |
| Tiling | Yes | Yes |
| IPC | hyprctl | swaymsg (standard) |

### Migration Scope

1. ✅ Replace Hyprland with Sway
2. ✅ Replace Waybar with faelight-bar (custom Rust)
3. ✅ Replace hyprlock with faelight-lock
4. ✅ Replace hypridle with swayidle
5. ✅ Replace Omarchy themes with Faelight Forest
6. ✅ Remove all Omarchy packages
7. ✅ Rebuild 0-Core from vanilla Arch base

## Outcome

- **Before:** 191 packages (Omarchy bloat)
- **After:** 82 packages (essentials only)
- **Result:** 100% control, no upstream surprises

## Lessons Learned

1. **Never depend on opinionated distro layers** - They can change without warning
2. **Own your entire stack** - If you don't control it, you can't trust it
3. **Document recovery paths** - Intent Ledger saved the migration
4. **Build custom tools** - faelight-* tools replaced Omarchy dependencies
5. **Vanilla base + manual control** - The Faelight Forest philosophy validated

## Philosophy Reinforced

> "Manual control over automation. Understanding over convenience."

The Omarchy incident proved why 0-Core exists. When upstream breaks, we recover because we understand our system completely.

---
_The forest survives the storm._ 🌲
