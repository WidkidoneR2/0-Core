---
id: 056
date: 2026-06-09
type: infrastructure
title: "Forest Recovery Protocol: TTY rescue hardening and compositor pre-flight"
status: in-progress
tags: [recovery, tty, greetd, compositor, safety, preflight, rescue]
priority: critical
---
## Why
On 2026-06-09 a live greetd/tuigreet change broke the login session
for approximately 24 hours. The system had no reliable TTY escape
hatch and no enforced pre-flight gate before compositor-touching changes
landed on the real machine.

This intent makes that failure structurally impossible to repeat.
It does not add polish. It adds discipline.

## The Two Failures This Fixes

FAILURE 1 -- No reliable rescue path
  TTY2 must be a guaranteed, hardened rescue shell.
  Not assumed present. Explicitly declared in NixOS config.
  Reachable via Ctrl+Alt+F2 regardless of compositor or greeter state.
  getty@tty2.service enabled and verified before any session work lands.

FAILURE 2 -- Live system used as test environment
  INT-024 exists for exactly this reason.
  Any change touching: greetd, tuigreet, compositor session,
  login flow, or display manager MUST pass INT-024's VM graduation
  pipeline before touching the real machine.
  This is not optional. It is a hard gate.

## What This Intent Delivers

1. TTY2 hardening
   - getty@tty2.service explicitly enabled in flake (not assumed)
   - Verified reachable via Ctrl+Alt+F2 from any compositor state
   - Verified reachable when greetd fails or hangs
   - Documented in forest recovery runbook

2. greetd fallback session
   - A bare fsh or bash session defined as fallback in greetd config
   - If primary compositor session fails to launch, fallback fires
   - Fallback session tested in VM before live system
   - Prevents total lockout if compositor fails at login

3. Pre-flight checklist (mandatory gate for INT-053, 054, 055)
   Before ANY compositor or login flow change lands on real machine:
   [ ] Change tested in VM via INT-024 pipeline
   [ ] VM snapshot created before test (before-INT-NNN naming convention)
   [ ] TTY2 verified working in VM
   [ ] Fallback session verified working in VM
   [ ] Recovery from broken session demonstrated in VM
   [ ] All above documented before graduating to real machine

4. Forest recovery runbook
   A short doc: docs/recovery-runbook.md
   - How to reach TTY2
   - How to rebuild NixOS from TTY
   - How to roll back a generation
   - How to manually start a session from TTY
   This doc lives in the repo. It is never more than one TTY away.

## Phases

Phase 1 -- TTY2 hardening (NixOS config)
  Verify or add getty@tty2.service to flake
  Test Ctrl+Alt+F2 from MangoWM, from Pinnacle, from greetd
  Gate: TTY2 login confirmed from all three states

Phase 2 -- greetd fallback session
  Add fallback session entry to greetd config (VM first via INT-024)
  Test: break primary session, verify fallback fires
  Gate: fallback session lands user in fsh from greetd failure

Phase 3 -- Pre-flight checklist formalized
  Write checklist into docs/recovery-runbook.md
  Add checklist reference to INT-053, 054, 055 gate sections
  Gate: checklist exists, is referenced, is understood

Phase 4 -- Recovery runbook complete
  docs/recovery-runbook.md written and committed
  Gate: another person could recover this system using only that doc

## Gates
- [ ] getty@tty2.service explicitly declared in NixOS flake
- [x] TTY2 reachable via Ctrl+Alt+F2 from MangoWM session
- [ ] TTY2 reachable via Ctrl+Alt+F2 from Pinnacle session
- [ ] TTY2 reachable via Ctrl+Alt+F2 when greetd is the foreground process
- [ ] greetd fallback session defined and tested in VM
- [ ] Fallback fires when primary compositor session fails
- [x] docs/recovery-runbook.md written and committed
- [ ] Pre-flight checklist referenced in INT-053, INT-054, INT-055

## Depends On
- INT-024 (VM graduation pipeline -- all Phase 2+ work goes through VM first)

## Blocks
- INT-053 (faelight-bar v2) -- pre-flight gate must pass first
- INT-054 (greetd polish) -- pre-flight gate must pass first
- INT-055 (faelight-compositor bridge) -- pre-flight gate must pass first

## The Rule
"Nothing that can break the session lands on the real machine
 without surviving the VM first.
 TTY2 is always the door.
 The door is always open." 🌲

## Status (2026-06-13)

Verified on bare metal:
- TTY2 reachable from MangoWM. Super+Ctrl+2 when mango is responsive; Fn+Ctrl+Alt+F2 (kernel VT switch) when mango is frozen; return via Fn+Ctrl+Alt+F1. Framework caveat: the top row defaults to media keys, so the Fn modifier is required for real F-keys.
- docs/recovery-runbook.md written, committed (7b2814e2), and pushed.

Relying for now on the NixOS autovt default to provide getty@tty2. It works, but the explicit declaration named in Failure 1 is NOT yet added. That is a login-touching change, deferred to the INT-024 VM.

Not yet tested (honest gaps):
- Pinnacle session: installed, but the TTY2 escape from Pinnacle has never been verified.
- TTY2 escape while greetd is the foreground process: not tested.
- greetd fallback session: not defined or tested (VM-gated via INT-024).

Reproducibility gap: the rescue chords live in an untracked ~/.config/mango/config.conf. A fresh install from the flake would not have them. Wiring config/mango into home-manager is pending.
