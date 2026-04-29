---
id: 258
date: 2026-04-28
type: feature
title: "Ctrl+D Health Display as Ratatui TUI"
status: planned
tags: [feature, rust, faelight, tui, ratatui, health, doctor, fsh, ux, condensation]
version: TBD
---

## Vision

`d` (the doctor health summary) currently emits 22-23 checks with full
explanatory text every time it's called. The information is correct but
lengthy — 30+ lines of output for a status check that's run multiple
times per session. Most of those lines say "✅ everything fine" which
provides little signal compared to the noise.

`Ctrl+D` (or `dt` from fsh prompt) opens a ratatui-based health TUI:
- Compact at-a-glance view by default
- Drill-down on any check that's failing or warning
- Live-refreshing while open
- One screen, scannable in 3 seconds

This is the same pattern proven by INT-250 (Ctrl+R), INT-253 (gt), and
INT-254 (it). Ratatui + crossterm inside fsh's REPL loop, single-purpose
TUI for a workflow that's currently command-output noise.

This intent ALSO encompasses a real upgrade to what `d` measures and
how it presents — not just a TUI wrapper around the existing checks.

## Why Now

Three real signals from daily use:

1. **`d` runs multiple times per session.** Every deploy ends with
   "run d to verify health." That's correct guidance, but the output
   is too noisy for repeated use. Compact form is a quality-of-life
   win every time.

2. **22 of 23 checks pass on a healthy day.** The signal is in the
   ONE that's not green. Today that signal is buried under 22 lines
   of confirmation. Compact form puts it front.

3. **Ctrl+R, gt, it patterns are maturing.** Adding Ctrl+D extends a
   coherent UX language: function keys open focused TUIs, return
   cleanly to fsh prompt, share the forest's color palette and tone.

## Approach

### Invocation
- `Ctrl+D` from fsh REPL line edit -> opens TUI
- `dt` from fsh prompt -> opens TUI
- `d` continues to work as today (CLI output) for scripting / piping
- TUI exits cleanly back to fsh prompt with a one-line summary

### Layout (initial)

**Compact mode (default):**
┌─ 🏥 Faelight Forest 11.9.0 ─ 100% ─ 22/23 ─ Integrity 100% ──┐
│                                                              │
│  System    [✅✅✅✅✅]  5/5  ▸                              │
│  Git&Code  [✅✅✅]      3/3  ▸                              │
│  Tools     [✅✅✅✅]    4/4  ▸                              │
│  Forest    [✅✅✅✅✅✅✅] 7/7 ▸                            │
│  Security  [✅✅⚠️]      2/3  ▸  (Core unlocked)             │
│                                                              │
│  Friday: 13 patterns · 172 facts · 1 contradiction           │
│  Forecast: 24h 100% · 7d 100% · trend stable                 │
│                                                              │
│  q quit · ▸ expand · r refresh · v verbose                   │
└──────────────────────────────────────────────────────────────┘

**Expanded section (when user presses ▸ on Security):**
┌─ 🔒 Security ────────────────────────────────────────────────┐
│  ✅ Security Hardening    UFW ✅  fail2ban ✅                │
│  ✅ Security Audit        28 findings, all upstream pending  │
│  ⚠️  Core Protection       Core is UNLOCKED                  │
│      └─ Action: lock-core before shutdown                    │
└──────────────────────────────────────────────────────────────┘

### Real upgrades to health (beyond TUI wrapper)

**1. Severity tiering**
Current `d` treats all checks equally. Reality: "core unlocked"
is informational; "stow symlinks broken" is critical. New tiers:
- 🔴 critical (blocks work)
- 🟡 warning (worth knowing)
- 🔵 informational (status fact)
- ✅ healthy

Compact view colors borders by highest tier present.

**2. Trend awareness**
Each check carries a trend:
- ↗ improving (was warning yesterday, healthy today)
- ↘ degrading (was healthy, now warning)
- → stable

Trend pulled from doctor cache history.

**3. Smart aggregation**
Where multiple checks are conceptually one (binary deps + tool
install + path resilience all mean "tools are present"), TUI shows
ONE row with a count. Drill-down expands the underlying checks.

**4. Action shortcuts**
For checks with known fixes, TUI shows the fix command in expanded
view AND lets user press [a] to copy it to clipboard. No more
manually retyping "lock-core."

**5. Time-since-last-check freshness**
Compact view shows when each section was last verified. If a
section is computed from cache > 5 minutes old, show with dim color.

### Implementation modules (suggested)
- `rust-tools/faelight-shell/src/health_tui/mod.rs` -- entry point
- `rust-tools/faelight-shell/src/health_tui/state.rs` -- doctor cache reader
- `rust-tools/faelight-shell/src/health_tui/render.rs` -- ratatui rendering
- `rust-tools/faelight-shell/src/health_tui/aggregate.rs` -- check grouping logic
- `rust-tools/doctor/` -- minor changes to emit severity + trend in cache

Or as standalone tool `rust-tools/dt/` if scope grows.

## Hard Dependencies

- ratatui 0.28 + crossterm 0.28 (already in fsh)
- doctor's health cache (already exists at ~/.cache/faelight/health-status)
- ConditionalEventHandler pattern (proven in INT-250)
- Doctor changes for severity tiering: backwards-compatible, default to
  "warning" for unannotated checks

## Success Criteria

- [ ] `Ctrl+D` from fsh REPL opens a working health TUI
- [ ] `dt` from fsh prompt opens the TUI (alias path)
- [ ] `d` continues to work as today's CLI output
- [ ] Compact view shows all 5 sections in <12 lines
- [ ] Each section shows a checkmark-strip summary (✅✅✅✅✅ 5/5)
- [ ] Border color reflects highest-severity issue present
- [ ] Friday status line shows patterns + facts + contradictions
- [ ] Forecast line shows 24h / 7d / trend
- [ ] Pressing ▸ (or arrow) on a section expands its detail rows
- [ ] Severity tiers (critical/warning/info/healthy) shipped in doctor cache
- [ ] Trend indicators (↗ ↘ →) shown for each check in expanded view
- [ ] Smart aggregation collapses related checks (e.g. tool checks merged)
- [ ] Expanded warning rows show suggested fix command
- [ ] [a] key copies fix command to clipboard (wl-copy)
- [ ] Time-since-last-check freshness shown in compact view
- [ ] [r] refreshes (re-runs doctor)
- [ ] [v] toggles to verbose mode (current full-text output)
- [ ] [q] / Esc returns cleanly to fsh prompt

## Scope

### In scope
- Ratatui TUI for health display (Ctrl+D, dt)
- Severity tiering in doctor's check definitions
- Trend tracking via cached history
- Smart aggregation of related checks
- Action-shortcut copy-to-clipboard for fixes
- Compact + drill-down layout

### Out of scope (separate intents or future expansion)
- New health checks (this intent reshapes presentation, doesn't add metrics)
- Live auto-refresh (manual [r] only)
- Health alerts via faelight-notify (separate intent if wanted)
- Historical health graphs (could be panel later, not v1)
- Cross-host health (single-host only)
- Voice readout of health summary (Friday voice integration, separate)

### Deliberately deferred
- Doctor's health-check Rust rewrite if currently shell-based (only
  if it gets in the way of severity tiering; otherwise keep)

## Gate Check
⬜ Not started

---

*"22 of 23 checks pass on a healthy day.
The signal is in the one that doesn't.
Show that one. Hide the rest until asked."* 🌲
