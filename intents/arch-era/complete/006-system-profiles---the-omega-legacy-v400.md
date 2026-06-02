---
id: 006
date: 2026-01-05
type: future
title: "System Profiles - The Omega Legacy (v4.0.0)"
status: complete
tags: [profiles, gpu, omega, v4.0.0, flagship]
---

## The Vision

Inspired by Omega Drivers — community-built, user-empowering, better than default.

System Profiles lets you switch your entire system state with one command.
No magic. No auto-detection. You choose. You control.

## Why

Current state:

- GPU settings scattered across configs
- No unified way to switch "modes"
- Services started manually, forgotten
- VPN toggled separately from workflows
- Every context switch is 5-10 manual steps

0-Core way:

- One command switches everything
- Explicit, documented, reversible
- You define what each profile does
- Nothing hidden, nothing automatic

## Goals

### Goal 1: Profile Infrastructure

**What:** Core system for defining and switching profiles
**Success:** `profile <name>` command works

### Goal 2: Default Profiles (4 minimum)

**What:** Ship with useful starting profiles
**Profiles:**

- `default` — Daily driver, baseline state
- `gaming` — GPU performance, notifications off
- `work` — VPN on, focus mode
- `low-power` — Battery optimization

**Success:** Each profile switches cleanly and reverts cleanly

### Goal 3: Profile Definition Format

**What:** Simple, readable profile files
**Location:** `~/0-core/profiles/`
**Format:**

```bash
# profiles/gaming.profile
# Gaming Profile - Performance mode
#
# ACTIVATES:
#   - GPU performance mode
#   - Disable notifications
#   - Game mode compositor settings
#
# DEACTIVATES:
#   - VPN
#   - Background services

[activate]
# Commands run when profile activates
supergfxctl -m dedicated
notify-send "Gaming mode active" "GPU at full power"
makoctl mode -a dnd

[deactivate]
# Commands run when switching away
supergfxctl -m hybrid
makoctl mode -r dnd
```

**Success:** Non-programmers can read and modify profiles

### Goal 4: Profile Status in Prompt

**What:** Starship shows active profile
**Example:**

```
┌─   christian 🔓 🎮 0-core  main
```

**Success:** Current profile visible at all times

### Goal 5: Profile Safety

**What:** Guardrails for profile switching
**Rules:**

- Warn before switching from modified state
- `profile status` shows current state
- `profile default` always works (escape hatch)
- Log all profile switches

**Success:** Can't get stuck in broken state

### Goal 6: Documentation

**What:** Complete guide for profiles
**Includes:**

- How to create custom profiles
- All default profiles explained
- Troubleshooting
- Philosophy explanation

**Success:** Someone new can create a profile in 10 minutes

## Implementation Plan

### Session 1: Foundation (Saturday morning, ~2 hrs)

- [ ] Create `profiles/` directory structure
- [ ] Write `profile` script (switch, status, list)
- [ ] Create profile file format parser
- [ ] Add `default.profile`

### Session 2: Core Profiles (Saturday afternoon, ~2 hrs)

- [ ] Create `gaming.profile`
- [ ] Create `work.profile`
- [ ] Create `low-power.profile`
- [ ] Test all switches and reverts

### Session 3: Integration (Sunday morning, ~1.5 hrs)

- [ ] Starship profile indicator
- [ ] Profile logging
- [ ] Safety checks and warnings
- [ ] `dot-doctor` Check 12: Profile health

### Session 4: Polish (Sunday afternoon, ~1.5 hrs)

- [ ] Documentation (PROFILES.md)
- [ ] Tab completions for `profile` command
- [ ] Update README
- [ ] Test full workflow

## Commands

```bash
profile list              # Show all available profiles
profile status            # Show current profile + state
profile gaming            # Switch to gaming profile
profile default           # Return to baseline
profile create <name>     # Create new profile (interactive)
profile edit <name>       # Edit existing profile
profile history           # Show recent profile switches
```

## File Structure

```
0-core/
├── profiles/
│   ├── default.profile
│   ├── gaming.profile
│   ├── work.profile
│   ├── low-power.profile
│   └── custom/           # User-created profiles
├── scripts/
│   └── profile           # Main profile script
└── docs/
    └── PROFILES.md       # Documentation
```

## Philosophy Alignment

| Principle              | Implementation                               |
| ---------------------- | -------------------------------------------- |
| Manual control         | You run `profile`, nothing auto-switches     |
| Intent over automation | Each profile declares what it does           |
| Visible state          | Prompt shows active profile                  |
| Recovery               | `profile default` is always the escape hatch |
| Explicit structure     | Profile files are readable text              |
| No magic               | Just runs commands you defined               |
| Documented             | Every profile explains itself                |

## The Omega Feeling

What made Omega Drivers special:

- Community built, not corporate
- Better than the default
- Gave control back to users
- Shared freely
- Made a real difference

System Profiles carries this forward:

- You build your profiles
- Share them with the community
- Better than scattered configs
- Control your system, don't fight it

## Success Criteria

**v4.0.0 is complete when:**

- [ ] `profile` command works reliably
- [ ] 4 default profiles ship and work
- [ ] Profile format is documented and simple
- [ ] Starship shows current profile
- [ ] Anyone can create a profile in 10 minutes
- [ ] README updated with profiles section

## Risk & Mitigation

| Risk                          | Mitigation                                            |
| ----------------------------- | ----------------------------------------------------- |
| GPU commands vary by hardware | Document common options, make profiles hardware-aware |
| Stuck in broken profile       | `profile default` always reverts to safe state        |
| Profile conflicts             | Warn when switching with uncommitted state            |
| Complexity creep              | Keep format simple, resist feature additions          |

## Future (Not v4.0.0)

- Profile sharing (export/import)
- Profile dependencies
- Scheduled profiles (manual trigger, scheduled execution)
- Community profile repository

These are NOT in scope for v4.0.0. Ship simple first.

---

_The Omega legacy lives on. The user is in control._ 🎮

---

_Part of the 0-Core Intent Ledger_ 🌲
