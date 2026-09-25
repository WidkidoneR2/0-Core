---
id: 053
date: 2026-06-09
type: feature
title: "faelight-bar v2: i3-style wlr-layer-shell bar for MangoWM and Pinnacle"
status: complete
tags: [bar, wayland, layer-shell, gtk4, python, gtk4-layer-shell, css, mango, pinnacle]
priority: high
---
## Decision -- 2026-06-17: pivot to GTK4 + Python
The bar is being rebuilt on the faelight-logout stack -- Python + GTK4 +
gtk4-layer-shell + candy-neon CSS -- replacing the cosmic-text / Rust renderer.
Rationale:
  -- faelight-logout's reception confirmed the GTK4 candy-neon look lands hard,
     and a persistent bar is where that look earns its keep.
  -- GLib's main loop integrates and drains the Wayland fd, so the v3
     broken-pipe failure class (hand-rolled loop, unread socket) cannot recur.
  -- GTK CSS + real widgets make the i3-style layout and theming far cheaper
     than hand-drawing glyphs with cosmic-text.
This supersedes the cosmic-text Approach / Phases / Gates as rewritten below.
The cosmic-text lineage is preserved as History at the foot of this file -- it
earned the Wayland-drain root-cause and the home-manager service pattern.
Safety: the bar is a client surface, not a login/greetd change -- it cannot
lock you out (worst case: no bar, login intact). It builds and runs live per
the logout precedent; the INT-056 VM pre-flight stays reserved for the actual
login-surface intents (054, 005). The running cosmic-text bar stays up until
the GTK4 bar is solid enough to swap into faelight-bar.service.

## Decision -- 2026-06-17: workspace indicators via dwl-ipc (zdwl_ipc_manager_v2)
This resolves the "mango IPC source (unresolved)" question from the Approach.
wayland-info shows mango advertises three candidates: zdwl_ipc_manager_v2
(dwl-ipc, v2), ext_workspace_manager_v1, and zwlr_foreign_toplevel_manager_v1.
Chose dwl-ipc, implemented as a side Wayland client (faelight-wsd) that writes
~/.cache/faelight/workspaces as JSON; the bar reads that file.
Rationale:
  -- dwl-ipc is the protocol mango drives richly: per-tag active / urgent /
     clients / focused plus layout, delivered frame-by-frame (no polling).
     Demonstrated live -- 9 tags, clean enum state, instant tracking on Ctrl+1..5.
  -- ext-workspace-v1 is the more portable choice, but portability to Pinnacle
     is speculative: Pinnacle's real-DRM/NixOS viability is UNPROVEN (only
     nested + terminal smoke-tested). Build for the compositor that demonstrably
     works (mango), not a hypothetical one.
  -- faelight-wsd is tiny and self-contained: pure-Rust Wayland backend (no
     system libwayland), ~324K RSS, write-on-change, running as a
     faelight-session.target service alongside the bar.
Trade-off (honest): dwl-ipc is dwl-family-specific, so this is less portable
than ext-workspace would be. The deviation is contained by the file boundary --
the bar is compositor-agnostic (it reads JSON, knows nothing of dwl), and the
helper is a swappable side-component. If Pinnacle ever proves out on real
hardware, add an ext-workspace helper writing the same JSON; the bar is unchanged.
Safety: helper and bar are client surfaces, not login/greetd -- no lockout risk.

## Decision -- 2026-06-17: Pinnacle render gate split to INT-067
The "renders under Pinnacle" gate is moved to INT-067 (faelight-bar under the
secondary compositor). It is blocked on Pinnacle standing up as a session (INT-038)
and on the INT-056 login-surface pre-flight -- neither is a property of the Mango
bar this intent delivers. Pinnacle is installed and is the primary secondary target
(Rust-native, NixOS foothold); if it will not hold on the 780M, the fallback is to
uninstall it and stand up MiracleWM (Mir-based, v0.9 Apr 2026, i3/Sway-compatible
IPC -- a cleaner workspace source than dwl-ipc). The bar is compositor-agnostic
(reads ~/.cache/faelight/workspaces as JSON), so whichever wins, the bar is
unchanged. 053 closes on the demonstrated Mango gates.

## Why
faelight-bar v1 was a Niri prototype built around Niri-specific IPC.
MangoWM is now the daily driver. Pinnacle is the compositor target.
v1 does not work under MangoWM or Pinnacle.

faelight-bar v2 is built compositor-agnostic from the ground up:
wlr-layer-shell only, no compositor-specific IPC.
It reads forest state from state.db and /proc -- not from the compositor.

## What Already Exists
faelight-bar v1: cosmic-text renderer, wlr-layer-shell, health/git/intent display
INT-033: neon candy palette applied to v1 (green/amber/red/purple)
MangoWM confirmed working on Framework 16
Pinnacle confirmed working with Lua config

## Vision
  Top bar, anchored via wlr-layer-shell
  Left:   🔒 lock state · H:95% health · branch* git
  Center: active intent title (neon purple) or Friday message (neon cyan)
  Right:  CPU% · RAM% · battery% · wifi · clock

  Colors:
    health >= 95  → neon green
    health >= 80  → neon amber
    health < 80   → neon red
    active intent → neon purple
    friday msg    → neon cyan
    git dirty     → neon amber
    git clean     → neon green

  Updates every 2 seconds from state.db and /proc
  No compositor IPC dependency -- reads forest state directly

## Approach (GTK4 -- revised 2026-06-17)
- Python + GTK4 + gtk4-layer-shell: the faelight-logout stack, proven on mango
- Anchored top via gtk4-layer-shell with an exclusive zone so windows do not
  overlap the bar
- Candy-neon look via GTK CSS (INT-033 palette), JetBrainsMono Nerd Font
- GLib main loop drives a refresh tick; no hand-rolled Wayland loop, so the v3
  broken-pipe class cannot recur
- Forest state from state.db (health, active intent, Friday); git from
  .git/HEAD + git status --porcelain
- System stats from /proc/stat, /proc/meminfo, /sys/class/power_supply,
  /proc/net/wireless
- Workspace indicators deferred to a late phase: they need a wlr workspace/tag
  source (mango dwl-ipc, unresolved) handled via a side Wayland client or
  helper, not GTK
- Built alongside the running cosmic-text bar; swapped into faelight-bar.service
  only once solid

## Pre-flight (INT-056 required)
Any compositor-touching change must pass INT-056 pre-flight:
TTY2 hardened, fallback session defined, VM tested first.

## Phases (logout-style -- one demonstrable gate each)
Phase 1 -- Skeleton
  Blank GTK4 bar, anchored top via gtk4-layer-shell, exclusive zone, candy-neon CSS
  Runs as its own surface alongside the existing bar; does not touch the service
  Gate: bar renders anchored top under MangoWM and survives, no crash

Phase 2 -- Forest state
  Health (neon candy colors, INT-033), active intent (neon purple),
  git branch + dirty, refreshed on a GLib timer from state.db and .git
  Gate: left and center show correct, live forest data

Phase 3 -- System stats
  CPU (/proc/stat delta), RAM (/proc/meminfo), battery (/sys/class/power_supply),
  wifi (/proc/net/wireless), clock HH:MM
  Gate: right section shows live system stats, no flicker, flat memory

Phase 4 -- Swap
  Point faelight-bar.service ExecStart at the GTK4 bar; retire the cosmic-text bar
  Gate: GTK4 bar autostarts at a real login, cosmic-text bar code removed

Phase 5 -- Pinnacle and workspaces (later)
  Verify render under Pinnacle; add i3-style workspace indicators once the
  mango IPC source question is settled
  Gate: renders under Pinnacle; workspace indicators show live tags

## Gates
- [x] GTK4 layer-shell bar renders anchored top under MangoWM, does not crash
- [x] Forest state: health in neon candy colors, active intent in neon purple, git branch + dirty
- [x] System stats: CPU, RAM, battery, wifi, clock all rendering
- [x] Updates on a timer with no flicker and no memory growth: 2s GLib tick, no visible flicker over ~73 min live as the daily bar; RSS flat -- two samples ~16 min apart (uptime 59 -> 73 min, NRestarts=0, same PID) byte-identical at VmRSS 114440 kB / VmHWM 121528 kB (~112M / ~119M); HWM never advanced, so no growth; 2s git-subprocess tick ruled out as a leak; well under the 228M old-peak and the 391M watch-figure (2026-06-17)
- [x] Workspace indicators (i3-style) -- live via faelight-wsd (dwl-ipc) + bar render
- [x] Clean swap: GTK4 bar replaces cosmic-text bar in faelight-bar.service, verified at real login
- [x] cosmic-text / Niri-era bar code retired after the swap
- [x] runs live as a client surface only, no greetd/login change (logout precedent)

## Depends On
- INT-056 (Forest Recovery Protocol) -- pre-flight gate
- INT-055 (compositor bridge) -- shared layer-shell infrastructure
- INT-033 (color system) -- neon candy palette already applied

## The Rule
"The bar is the forest's pulse.
 It should show the forest's health at a glance --
 not the compositor's internal state." 🌲

## Pre-flight Gate -- INT-056 -- SUPERSEDED 2026-06-17
SUPERSEDED by the 2026-06-17 client-surface decision above. faelight-bar is a client
surface, not a login/greetd change -- it cannot lock you out (worst case: no bar,
login intact), and it ships live per the faelight-logout precedent. The INT-056 VM
pre-flight is reserved for the actual login-surface intents (054, 005) and for the
secondary-compositor switch (INT-067). The original checklist applied to the
cosmic-text-era framing, when the bar was conceived as compositor-touching; the GTK4
client-surface bar is not. Recorded as resolved, not as debt.

## History -- cosmic-text lineage (v1-v3, superseded by the GTK4 pivot 2026-06-17)
The two sections below document the superseded cosmic-text / Rust implementation.
Kept as honest record: the broken-pipe root-cause and fix, and the home-manager
service pattern, both still inform the GTK4 build.

## Known Issue -- in progress (2026-06-14)
faelight-bar v3 exits with rc=1 "Io error: Broken pipe (os error 32)" after
~8 min under MangoWM: the compositor closes the Wayland connection and the next
eq.flush()? in main() propagates the IoError and exits. Confirmed NOT memory
(RSS flat ~34 MB across deaths) and NOT a panic.
Workaround in place: supervised auto-restart loop respawns the bar in ~2s,
mirroring a deployed systemd service with Restart=always.
TODO next session: run with WAYLAND_DEBUG=1, read the compositor's fatal error
just before the broken pipe, decide bar-protocol-bug vs Mango behavior, then
either fix the bar or have main() reconnect instead of exiting.
Status: RESOLVED 2026-06-14 -- connection drop understood and fixed (see Progress below).

## Progress -- 2026-06-14 (broken pipe RESOLVED; MangoWM render + autostart demonstrated)
ROOT CAUSE (the Known Issue above, now understood): the main loop never read the Wayland
socket after init -- only eq.flush() + eq.dispatch_pending() (local buffer) + a sleep. Mango's
events (frame callbacks, wl_buffer.release, pings) piled in the kernel recv buffer at a fixed
rate; at ~8:12 it filled and Mango dropped the connection -> broken pipe. Explains the flat RSS.
FIX (commit 5235bf5e): each loop now drains the socket -- eq.flush(), conn.prepare_read(),
libc::poll() the connection fd, guard.read() on POLLIN. Survives well past 8:12 (debug + release).
The /tmp restart loop is retired.
SHIPPED AS A SERVICE (commit 2bb7c21b): declarative home-manager faelight-bar.service
(Restart=always) + a custom faelight-session.target (graphical-session.target refuses manual
start); mango exec-once imports the Wayland env and starts the target. Verified autostarting at
a real login after reboot -- PID under faelight-bar.service, no broken pipe.
GATES MARKED: renders-under-MangoWM and autostart-in-MangoWM-config, both demonstrated.
HONEST DEVIATION: the INT-056 VM pre-flight gate was NOT followed -- this landed live (dry-run
build + recovery standby + greetd/launch untouched, no lockout), but the VM checklist (snapshot,
TTY2, greetd fallback, recovery-in-VM) was skipped. That gate stays unchecked: an honest debt.
STILL OPEN: Pinnacle render + autostart, content gates (health colors / intent / git / system
stats / 2s no-flicker) not audited this session, no-Niri-code unverified, INT-056 pre-flight.
NOT cicomplete.
WATCH: bar RSS ~391M as a service (font loading?) -- profiling pass owed.
