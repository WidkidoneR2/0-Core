---
id: 056
date: 2026-06-09
type: infrastructure
title: "Forest Recovery Protocol: TTY rescue hardening and compositor pre-flight"
status: complete
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
- [x] TTY2 rescue getty guaranteed on tty2 (via autovt -- the explicit-declaration experiment showed autovt is what actually delivers it) <!-- 2026-07-11: VM-tested `systemd.services."getty@tty2".enable = true` per path-c. FINDING: built clean and did NOT conflict with greetd (greetd stayed active/running, 0 failed units -- the documented PR#30893 restart-conflict did NOT occur), BUT the explicit unit sat inactive(dead); when tty2 was selected, `autovt@tty2.service` (NOT our getty@tty2) activated and delivered the login. So the explicit declaration is cosmetic/dormant -- autovt already guarantees tty2 rescue. Removed the inert declaration (path b). Gate INTENT (tty2 rescue guaranteed, not assumed) is MET by autovt, proven functional from mango + Pinnacle + greeter, VM AND metal. -->
- [x] TTY2 reachable via Ctrl+Alt+F2 from MangoWM session
- [x] TTY2 reachable via Ctrl+Alt+F2 from Pinnacle session <!-- 2026-07-11: Pinnacle metal-tested this session (launched from greetd, Alacritty spawned via Super+Return, dynamic tiling + Snowcap quit-confirm all working -- Pinnacle CONFIRMED working on real hardware, first time). From inside the Pinnacle session, Fn+Ctrl+Alt+F2 reached TTY2 successfully (Framework Fn caveat applies, same as mango). Return via Fn+Ctrl+Alt+F1. -->
- [x] TTY2 reachable via Ctrl+Alt+F2 when greetd is the foreground process <!-- 2026-07-11: RESOLVED the 2026-07-01 unconfirmed-chord finding. Tested calmly in the VM (where a failed VT-switch can't strand metal): from the greeter (greetd foreground), Ctrl+Alt+F2 reached a tty2 login (autovt@tty2 activated on switch), Ctrl+Alt+F1 returned. Christian confirmed the SAME works on real metal (both directions). The greeter-state rescue path is now CONFIRMED. Note: in the VM plain Ctrl+Alt+F2 works; on Framework metal the Fn modifier is needed for the media-row F-keys (Fn+Ctrl+Alt+F2), same caveat as the mango/Pinnacle gates. -->
- [x] greetd fallback session defined and tested in VM <!-- 2026-07-11: SafeShell session (Exec=fsh, no compositor) added as environment.etc greetd/sessions/safeshell.desktop. VM-tested (hosts/vm/base.nix): booted VM greeter, picked SafeShell, landed in a working fsh console. Then GRADUATED to metal (hosts/framework16/configuration.nix) + tested on real hardware: logged out, picked SafeShell at the metal greeter, got a working fsh (whoami/pwd/d all fine), returned to mango clean. Metal-proven, exceeding this gate's VM requirement. -->
- [x] Fallback fires when primary compositor session fails <!-- 2026-07-11: PROVEN in VM with a throwaway BadCompositor session (Exec=nonexistent-binary). Picking BadCompositor at the greeter FAILED and bounced back to the greeter (not a lockout); SafeShell then STILL worked. Demonstrates a compositor that fails to launch cannot strand the user -- SafeShell is always the escape. Test session removed after proof; SafeShell shipped (VM + metal). The 2026-06-09 24h-lockout is now structurally defeated. -->
- [x] docs/recovery-runbook.md written and committed
- [x] Pre-flight checklist referenced in INT-053, INT-054, INT-055

## Depends On
- INT-024 (VM graduation pipeline -- all Phase 2+ work goes through VM first)

## Blocks
- INT-053 (faelight-bar v2) -- pre-flight gate must pass first
- INT-054 (greetd polish) -- pre-flight gate must pass first
- INT-055 (faelight-compositor bridge) -- pre-flight gate must pass first

## RESOLUTION (2026-07-11): 056 COMPLETE -- the recovery protocol is proven, end to end.

All 8 gates met. The Forest Recovery Protocol -- born from the 2026-06-09 24h greetd/tuigreet
lockout -- is now structurally in place and demonstrated on real hardware:

1. SafeShell rescue session (the anti-lockout net): a greetd session (Exec=fsh, no compositor)
   that lands the user in a working fsh if every compositor fails. VM-proven (BadCompositor test:
   a failing session bounced to the greeter, SafeShell still worked) AND metal-proven (logged
   out, picked SafeShell, got a working fsh, returned to mango clean).

2. TTY2 rescue from every state: Ctrl+Alt+F2 (Fn+Ctrl+Alt+F2 on Framework metal for the
   media-row keys) reaches a tty2 login from mango, Pinnacle, AND the greeter (greetd
   foreground) -- all confirmed VM + metal. The greeter-state path (the open 2026-07-01
   finding) was re-tested calmly in the VM and confirmed on metal.

3. getty@tty2 finding (path c -> b): VM-tested an explicit `getty@tty2.enable = true`. It did
   NOT conflict with greetd (the feared PR#30893 restart-loop did not occur), but the explicit
   unit sat DORMANT -- autovt@tty2 is what actually activates and delivers the tty2 login on
   VT-switch. So the explicit declaration was cosmetic; removed it. autovt already GUARANTEES
   tty2 rescue (standard NixOS mechanism, always present, proven from every state). The gate's
   intent -- "tty2 rescue guaranteed, not assumed" -- is met by autovt, now explicitly
   understood and documented rather than forced.

4. recovery-runbook.md written + committed; pre-flight checklist referenced in 053/054/055.

Discipline that made this safe: every login-touching change went through the VM first (INT-024
harness), with SafeShell + TTY2 as backups on metal. Nothing that could break login was tested
on metal without a proven escape. The lockout is defeated.

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


## Finding (2026-07-01): greeter-foreground rescue is NOT clean -- needs proper re-test
Attempted the "TTY2 reachable when greetd is foreground" gate live. Result was
NOT a clean pass:
- Fn+Ctrl+Alt+F2 (the chord verified from MangoWM) did NOT behave the same at the
  greeter. Reaching TTY2 from the greeter was difficult; the exact working method
  was not confirmed (possibly plain Ctrl+Alt+F2 WITHOUT Fn -- UNVERIFIED, do not
  trust as runbook gospel yet).
- Recovery ultimately succeeded (returned to mango), but via an unclear path.
HONEST STATE: the greeter-state rescue path is NOT confirmed working. This gate
stays OPEN. The runbook currently implies Fn+Ctrl+Alt+F2 works everywhere; that
appears FALSE at the greeter and must be corrected once the real method is found.
NEXT: a calm, deliberate re-test with someone watching the exact keys -- ideally
in the VM (INT-024) where a failed VT-switch cannot strand the metal machine, OR
on metal with a second device (phone) ready and the exact chord written down
BEFORE logging out. Do NOT check this gate until the working greeter chord is
verified and written into recovery-runbook.md.
## Status (2026-07-11): SafeShell fallback shipped -- VM-proven AND metal-proven

The greetd fallback session (gates: "defined and tested in VM" + "fires when primary fails") is
DONE and now live on real hardware:
- **SafeShell** = a greeter session (`environment.etc."greetd/sessions/safeshell.desktop"`,
  Exec=fsh, no compositor/Wayland). If mango/miracle/pinnacle all fail, SafeShell still lands
  the user in a working fsh to repair from. Default flow (--cmd mango) is unchanged -- it's an
  added OPTION only.
- **VM proof** (base.nix): booted greeter, picked SafeShell -> working fsh. Then a throwaway
  BadCompositor session (Exec=nonexistent binary) FAILED to launch and bounced to the greeter,
  while SafeShell still worked -- proving a broken compositor can't lock the user out. Test
  session removed after.
- **Metal graduation** (framework16/configuration.nix): deployed, logged out, picked SafeShell
  at the real greeter -> working fsh, ran commands, returned to mango clean. Flawless.

Still open on 056: explicit getty@tty2 declaration (gate 1); TTY2-from-Pinnacle (gate 3, Pinnacle
still unverified on metal); TTY2-when-greetd-is-foreground (the 2026-07-01 unconfirmed-chord
finding). SafeShell -- the highest-value anti-lockout mechanism -- is the piece that now makes
testing an uncertain compositor (e.g. Pinnacle/Miracle) on metal SAFE: a failed compositor is
recoverable via the SafeShell pick.

Also cleaned this session: removed stale /etc/greetd/config.toml (pre-June-9
orphan, agetty autologin, unreferenced by the flake, greetd never read it) -- it
would have misled recovery debugging. sudo unlink; greetd store config untouched.
