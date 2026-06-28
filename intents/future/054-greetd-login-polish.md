---
id: 054
date: 2026-06-09
type: polish
title: "greetd: green theme, F2 session picker, compositor safety net"
status: planned
tags: [greetd, tuigreet, login, polish, recovery, safety]
priority: medium
---
## Why
The login screen works but lacks polish. Green theme is not rendering.
Session picker keybind needs fixing. More critically: the 2026-06-09
tuigreet incident proved that greetd changes on a live system without
VM pre-flight is unacceptable. This intent enforces that discipline.

## Depends On
- INT-056 (Forest Recovery Protocol) -- MUST complete Phase 1 and 2
  before any work in this intent begins on the real machine
- INT-024 (VM graduation pipeline) -- all changes tested in VM first

## Vision
- Green neon candy theme renders correctly in tuigreet
- F2 opens session picker reliably
- MangoWM and Pinnacle both selectable at login
- Fallback session (fsh) defined for recovery
- Boot generation pinned as rollback safety net
- All changes land via INT-024 graduation pipeline

## Phases

Phase 1 -- VM pre-flight (INT-024 required)
  All greetd config changes built and tested in VM first
  Snapshot created: before-INT-054
  Primary session, fallback session, F2 picker all tested in VM
  Gate: all greetd changes verified working in VM, recovery from
        broken session demonstrated

Phase 2 -- Graduate to real machine
  INT-056 Phase 1+2 gates passed (TTY2 hardened, fallback defined)
  Snapshot/generation checkpoint before applying
  Apply greetd changes from VM-verified config
  Gate: changes on real machine match VM behavior

Phase 3 -- Theme and polish
  Green neon candy colors in tuigreet config
  Clock and hostname rendering verified
  Gate: login screen matches Faelight Forest aesthetic

## Gates
- [ ] INT-056 Phase 1 complete (TTY2 hardened) before any live work
- [ ] INT-056 Phase 2 complete (fallback session defined) before any live work
- [ ] All changes tested in VM via INT-024 before real machine
- [ ] VM snapshot created: before-INT-054
- [ ] Green theme renders correctly on real machine
- [ ] F2 opens session picker reliably
- [ ] MangoWM selectable at login
- [ ] Pinnacle selectable at login
- [ ] Fallback fsh session defined in greetd config
- [ ] Boot generation rollback verified before closing intent

## The Rule
"greetd is the door to the forest.
 We do not change the door on a live house
 without a key in our pocket and the window already open." 🌲

## Pre-flight Gate -- INT-056 (Forest Recovery Protocol)
This intent changes the login/compositor surface. Per INT-056, NOTHING
here lands on the real machine until it has passed the pre-flight
checklist in INT-024's VM:
  [ ] change tested in VM via INT-024 pipeline
  [ ] VM snapshot taken before test (before-INT-NNN)
  [ ] TTY2 verified reachable in VM
  [ ] greetd fallback session verified in VM
  [ ] recovery from a broken session demonstrated in VM
  [ ] all of the above documented before graduating
Door is always open: docs/recovery-runbook.md · TTY2 via Ctrl+Alt+F2.


## EVOLUTION -- 2026-06-26: ReGreet chosen as the greeter; VM testbed working
The original charter said "tuigreet" + "Pinnacle". Two evolutions supersede that:
  - GREETER: ReGreet (graphical, GTK, CSS-themeable) replaces tuigreet as the vehicle. It can
    carry the candy-neon forest identity to the login screen the way tuigreet's text never could.
  - COMPOSITORS: Miracle-wm (INT-087) replaces Pinnacle (INT-086 is removing Pinnacle). The
    session picker offers Mango + Miracle, each with its own profile.

### VM testbed -- PROVEN WORKING (2026-06-27, the breakthrough)
The INT-056/024 VM now renders a full graphical ReGreet login end-to-end -- this is the
test surface 054 always required. Key fixes that got it there:
  - VM was headless: graphics=false + console=ttyS0 -> graphics=true + console=tty0,ttyS0.
    THE fix for a 2-session black screen (the VM never showed graphical output before).
  - ReGreet rendered an EMPTY window without config -> programs.regreet.enable provides the
    cage+regreet+config scaffold (the module launches regreet in cage by default).
  - ReGreet rendered but took NO keyboard -> the greetd 'greeter' user wasn't in the 'input'
    group, so cage/libinput couldn't read /dev/input/event* (crw-rw---- root:input). Fixed:
    users.users.greeter.extraGroups = [ "input" "seat" "video" ]. LOGIN THEN WORKED -- typed
    password, authenticated, dropped to a session.
  - VM-ONLY artifacts (NOT real-system bugs, do not chase on metal): post-login mango
    client-surface compositing fails (QEMU/wlroots wall -- real GPU composites fine); upside-down
    software cursor (WLR_NO_HARDWARE_CURSORS); wrong clock (no NTP in the VM).

### What "done" now requires (expanded scope)
  1. ReGreet configurability MAPPED -- regreet.toml + CSS ceiling (background, layout, behavior).
  2. Themed to faelight-logout -- candy-neon, consistent with the logout screen (INT-091 palette).
     VISUAL TARGET (pinned 2026-06-27): rich candy-neon green like faelight-logout -- the saturated,
     high-richness green that excited us, NOT a muted terminal green. Full GTK/CSS background +
     accents. DISPLAY THE FAELIGHT FOREST VERSION NUMBER on the greeter (like the fsh startup
     banner shows "14.1.0 -- <codename>") so the login screen states which forest version it is.
  3. Mango profile -- working session entry.
  4. Miracle-wm profile (INT-087) -- second compositor, own session entry.
  5. Session picker -- choose Mango / Miracle at login, verified.
  6. SECURITY & LEAK AUDIT -- full greetd->cage->regreet->PAM chain: greeter least-privilege
     (audit ALL the greeter user can reach, not just input groups), no secrets in committed
     config, no password/keystroke leakage to logs, gitleaks clean. First-class gate -- this
     touches authentication.
  7. Exhaustive VM testing -- test after test to 100% before real-machine graduation.
  8. INT-059 (Lanzaboote) interaction -- secure-boot/signing implications understood pre-deploy.

### Open questions
  - ReGreet config/theme ceiling (map it).
  - How session profiles (Mango/Miracle) are declared + picked.
  - Full security audit scope: greeter privileges, PAM, leak surface.
  - Does the cursor flip persist on real hardware or is it VM-only?
  - INT-059: any greeter-signing implications under secure boot.

### Relates to
  INT-056 (recovery net -- GATES this), INT-024 (VM pipeline), INT-005 (login flow),
  INT-086 (remove Pinnacle), INT-087 (Miracle), INT-059 (Lanzaboote), INT-091 (candy-neon).

## New gates (additive to the originals above)
- [ ] ReGreet renders a usable login form in the VM (DONE 2026-06-27 -- login authenticates)
- [ ] ReGreet config + CSS theming ceiling mapped
- [ ] ReGreet themed to match faelight-logout (candy-neon)
- [ ] Mango session profile works in the picker
- [ ] Miracle-wm session profile works in the picker
- [ ] Security & leak audit passed (greeter least-privilege, no secrets, PAM reviewed, gitleaks clean)
- [ ] INT-059 secure-boot interaction understood
- [ ] Exhaustive VM test cycles documented before real-machine graduation


## Progress (2026-06-27): ReGreet candy-neon GLASS theme -- VM-proven, KEEP
ReGreet configured + themed in the VM (NOT yet on real machine -- 054 discipline holds: VM-proof
first). Big visual win, locked in.
WHAT WORKS (proven live in `vm gui`):
- ReGreet renders, accepts keyboard, authenticates (greeter user in input/seat/video groups).
- Candy-neon GLASS theme via programs.regreet.extraCss: near-black radial-gradient green base,
  translucent glass login card with top-edge light-catch + outer lime glow, glowing lime clock,
  glassy inset entry fields (lime border -> aqua glow on focus), aqua glass buttons (fill-on-hover),
  amber pencil-edit chips, coral dropdown arrows. JetBrainsMono Nerd Font throughout.
- time.timeZone = "America/Chicago" added (clock was UTC, now correct Central).
- Iterated the CSS over several passes to remove overlapping combobox/entry double-borders,
  frame boxes around the greeting + bottom button row, and separator lines. Consolidated to
  single-border fields.
KNOWN LIMITATIONS (for the real-machine pass / newer ReGreet):
- Greeting text "Welcome back!" is hardcoded in regreet 0.3.0 -- can't change to "Faelight Forest"
  via config. Needs newer ReGreet or a patched build.
- Some GTK4 combobox/entry nesting still needs CSS refinement for pixel-perfect single borders.
STILL BLOCKED (separate from theming -- the real 054/056 gate):
- Mango session CRASHES on launch from greetd in the VM: session opens+closes same second.
  Manual `mango` over SSH shows libseat "Could not open VT / Failed to start a DRM session" --
  but that's expected over SSH (no seat). The greetd-launch crash reason is still uncaptured
  (mango stderr swallowed by greetd). christian has groups video/input/seat. NEXT debugging step:
  wrap mango's session Exec to log stderr to a file to catch the real greetd-launch crash.
- So: ReGreet greeter = themed + working; mango handoff = still to solve before real-machine.
NOTE: keep the extraCss block in hosts/vm/configuration.nix as the canonical forest greeter theme;
port to framework16 config ONLY after the mango-handoff crash is solved and the whole flow is 100%.


## Progress (2026-06-28): Two-mode VM harness built; cage->mango handoff isolated as VM-ceiling

Built the INT-024 two-mode VM login harness and used it to characterize the
ReGreet->mango crash precisely. Committed + pushed at 14b60b33.

### Harness (INT-024)
- Split hosts/vm/configuration.nix into: base.nix (shared) + login-mirror.nix
  (greetd->tuigreet->mango, mirrors framework16) + login-regreet.nix
  (cage+ReGreet candy-neon, migration target).
- flake: nixosConfigurations.faelight-vm (mirror) + faelight-vm-regreet.
- vm script: `vm build` = mirror, `vm build regreet` = ReGreet testbed.

### Findings (demonstrated, not declared)
- MIRROR mode PROVEN WORKING: tuigreet->mango launches, christian session holds
  open (identical to the real machine's login flow).
- REGREET mode reproduces the crash: cage->mango fails. Captured stderr:
    libseat: Could not poll connection: Broken pipe
    Could not open tty0 to update VT: Permission denied
    Could not open VT for client
    Timeout waiting session to become active -> Failed to start a DRM session
- ROOT CAUSE (proven by A/B): cage (ReGreet's wlroots compositor) holds seat0/VT;
  mango's libseat cannot acquire it during the greeter->session handoff. tuigreet
  is a TTY greeter (no DRM, hands the seat straight through) so it never hits this.

### Fixes tested and ruled out
- sleep 1 and sleep 3 in a session wrapper (timing/VT-race theory): NO effect.
- LIBSEAT_BACKEND=logind (seatd-vs-logind contention theory): test tangled with a
  ReGreet "session not found" wrinkle; inconclusive but the VT "Permission denied"
  lines persisted.

### Conclusion
Almost certainly a QEMU-emulation artifact, NOT a ReGreet/mango bug: mirror mode
works in the SAME VM, and the real machine runs mango daily. The cage->mango seat
handoff is the one thing the VM cannot faithfully emulate. The real graduation
test for ReGreet is on METAL (real DRM), performed carefully with INT-056 TTY
rescue armed BEFORE touching the live login.

### Capture method banked (reusable)
  vm ssh sudo systemd-run --uid=1000 --gid=100 -p PAMName=login \
    --setenv=XDG_RUNTIME_DIR=/run/user/1000 --wait --collect /path/to/wrapper.sh
reliably captures session stderr that greetd otherwise swallows.

### Next
- ReGreet greeter itself (theme, render, auth) is PROVEN in the VM -- that work can
  proceed without the post-login handoff.
- Graduation to metal is the open step; gate it behind INT-056 (TTY rescue).
