# Pinnacle WM Migration Plan
**Written:** 2026-06-07
**Based on:** INT-021 study results
**Status:** Pre-migration -- niri remains primary compositor

---

## Study Results Summary

### What Works Under Pinnacle
- ✅ Pinnacle v0.2.2 starts on Framework 16 (AMD Radeon 780M)
- ✅ EGL hardware acceleration enabled
- ✅ alacritty launches and works
- ✅ faelight-notify works
- ✅ faelight-bar renders (partial width -- needs layer-shell config)
- ✅ Keyboard and mouse input work
- ✅ XWayland starts at :0

### What Needs Work
- ⚠️ Pinnacle needs a proper Lua config to stay alive
- ⚠️ faelight-bar needs layer-shell protocol wiring for full width
- ⚠️ faelight-menu exits immediately -- needs Pinnacle workspace integration
- ⚠️ hyprlock does not work under Pinnacle -- needs replacement lock screen
- ⚠️ faelight-login needs session lifecycle changes for Pinnacle

### What Was Not Tested
- fsh inside Pinnacle terminal (Pinnacle kept exiting)
- Multi-monitor behavior
- Workspace switching
- Full keybind migration from niri

---

## Migration Prerequisites

Before touching the real system compositor:

1. **Lua config** -- write a complete Pinnacle config that:
   - Mirrors all current niri keybinds
   - Auto-starts forest services (bar, notify)
   - Has stable session management
   - Tested nested in niri for at least one week

2. **faelight-bar layer-shell** -- update bar to use wlr-layer-shell
   properly under Pinnacle so it renders full width

3. **Lock screen** -- find/build a lock screen that works under
   Pinnacle. Options: swaylock, waylock, or build faelight-lock v2

4. **faelight-menu** -- investigate why it exits under Pinnacle,
   fix workspace integration

5. **VM validation** -- once prerequisites are met, test full
   session in a properly resourced VM (INT-027)

---

## Migration Strategy (when ready)

### Phase 1 -- Parallel testing (nested)
- Run Pinnacle nested in niri daily for 1 week
- All forest tools must work in nested mode
- No regressions

### Phase 2 -- Dual boot generation
- Add Pinnacle session to greetd alongside niri
- Boot into Pinnacle session intentionally
- Niri session remains available as fallback
- Run Pinnacle as daily driver for 1 week

### Phase 3 -- Full migration
- Remove niri from default session
- Pinnacle becomes primary compositor
- Niri kept in packages for emergency rollback
- Document any remaining issues

### Rollback at any phase
rollback    # instant return to previous NixOS generation
NixOS generations protect every phase. No reinstall risk.

---

## Mango WM Evaluation

Mango WM to be evaluated separately via same nested test approach:
- Install via flake input
- Test nested in niri
- Same 6-tool test suite
- Compare stability vs Pinnacle

Decision: adopt whichever passes all 6 tools with least config overhead.

---

## What Must Never Break
- fsh as daily driver shell
- faelight-notify (critical for system events)
- faelight-bar (forest identity)
- State DB (runtime/state.db)
- Friday patterns and knowledge

---

## Timeline
- **Now**: Prerequisites phase (no timeline pressure)
- **Before 1.0.0**: Compositor decision made
- **1.0.0**: Ship with chosen compositor
