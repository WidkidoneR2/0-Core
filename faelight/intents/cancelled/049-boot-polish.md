---
id: 049
date: 2026-06-09
type: polish
title: "boot-polish: clean quiet boot, fix display handoff, greetd stability"
status: cancelled
tags: [boot, plymouth, greetd, display, tuigreet, polish, amd, framework]
priority: medium
---
## Why
Generation 92/93 caused blank screen after LUKS unlock.
The 2026-06-09 tuigreet incident proved live boot changes without VM
pre-flight cost 24 hours of lost work.
This intent fixes boot cosmetics and hardens the boot path.
Nothing lands on the real machine without VM validation first.

## Known Facts
- Generation 91: stable (Plymouth enabled, no quiet params)
- Generation 92: broke (Plymouth + quiet + splash added)
- Plymouth bgrt theme conflicts with AMD Radeon 780M display handoff
- libvirtd onBoot=start was resuming nixos-lab VM -- caused blank screen (FIXED gen 93)
- Ctrl+Alt+F2 did not work during blank screen -- full display lock
- INT-056 must complete first (TTY2 hardening, greetd fallback)

## The Problem Stack
1. Plymouth bgrt reads firmware ACPI logo -- conflicts with AMD KMS handoff
2. quiet + splash suppress output during failure -- no recovery signal
3. No fallback TTY during blank screen -- full lockout
4. greetd appears too slowly after LUKS unlock

## Options (test in VM via INT-024 first)

Option A -- Disable Plymouth entirely (recommended first)
  Remove boot.plymouth.enable
  Use systemd-boot silent mode only
  Cleanest approach, no display handoff risk
  Risk: none -- cosmetic only

Option B -- Replace Plymouth bgrt with text/spinner theme
  bgrt uses ACPI BGRT (firmware logo) -- known AMD conflict
  spinner or text theme avoids the handoff issue
  Risk: low -- theme swap only

Option C -- quiet without Plymouth
  boot.kernelParams = ["quiet"] only
  No splash, no Plymouth
  systemd suppresses most output naturally
  Risk: low

Option D -- Fix AMD KMS handoff
  Add amdgpu.dc=1 or drm.modeset kernel params
  Force KMS earlier in boot sequence
  Risk: medium -- kernel param changes

## Phases

Phase 1 -- INT-056 pre-flight (HARD DEPENDENCY)
  TTY2 hardened, greetd fallback session defined
  Ctrl+Alt+F2 verified working before any boot changes
  Gate: INT-056 Phase 1 and 2 complete

Phase 2 -- VM boot testing (INT-024 required)
  Snapshot VM: before-INT-049
  Test Option A in VM: disable Plymouth
  Boot VM 5 times, verify clean greetd handoff
  Test recovery: intentionally break, verify TTY2 escape
  Gate: clean boot in VM, recovery demonstrated

Phase 3 -- Graduate to real machine
  VM gates passed
  Generation checkpoint before applying
  Apply chosen option to framework16 flake
  Boot and verify: greetd within 3 seconds of LUKS unlock
  Gate: clean boot on Framework 16, no blank screen

Phase 4 -- Boot time optimization
  systemd-analyze to measure boot time
  Identify slow units
  Target: boot to greetd in under 10 seconds
  Gate: systemd-analyze shows < 10s to greetd

## Gates
- [ ] INT-056 Phase 1+2 complete before any boot changes
- [ ] VM snapshot created: before-INT-049
- [ ] Option A (disable Plymouth) tested in VM -- 5 clean boots
- [ ] Recovery from broken boot demonstrated in VM
- [ ] Clean boot on Framework 16 -- no blank screen
- [ ] tuigreet appears within 3 seconds of LUKS unlock
- [ ] Ctrl+Alt+F2 works as emergency escape at all boot stages
- [ ] No white text flash during password entry
- [ ] Boot time under 10 seconds to greetd (systemd-analyze)

## Depends On
- INT-056 (Forest Recovery Protocol) -- MUST complete first
- INT-024 (VM graduation pipeline) -- all changes tested in VM

## The Rule
"The boot screen is cosmetic.
 Stability is priority.
 Test in the VM. Graduate to the machine.
 Never the other way around." 🌲

## RESEARCH + STRATEGY (2026-07-01) -- grounded, anti-band-aid

### Measured problem (from this-boot journalctl, not memory)
Boot timeline on gen 272 (framework16, AMD 780M):
- 06:50:34.49  Plymouth TERMINATES ("Terminate Plymouth Boot Screen" finishes)
- 06:50:34.54  greetd.service starts (~50ms after plymouth dies -- handoff is TIGHT here)
- 06:50:37.36  greeter session opens (tuigreet actually paints)
=> ~2.8s window between Plymouth gone and tuigreet visible. THIS is the rough
   screen. It is NOT slow greetd-start; greetd starts immediately. It is the
   time for tuigreet to initialise + present its first frame after plymouth quit.

Second issue found in /proc/cmdline: `... splash loglevel=4 ...` -- `splash` is
present but `quiet` is NOT. loglevel=4 lets kernel warnings print to console, so
the 2.8s gap can also show kernel text, a SEPARATE noise source from the black gap.

### Why this is a KNOWN AMD problem (research, not guessing)
- AMDGPU driver is large/slow to load; kernel uses EFI framebuffer for early
  console then SWITCHES to amdgpudrmfb when ready -- that framebuffer switch is a
  documented flicker point on AMD (Gentoo/Arch). bgrt plymouth theme reads the
  ACPI firmware logo and is repeatedly named as the AMD-handoff-conflict theme
  (matches 049's existing "Problem Stack" item 1).
- SimpleDRM: on UEFI, plymouth uses SimpleDRM on the EFI framebuffer to avoid
  flicker waiting for amdgpu. Tunable via `plymouth.use-simpledrm=0/1`. CAUTION
  (Arch): with SimpleDRM, SECONDARY monitors may not light during boot and a
  docked-laptop LUKS prompt may be invisible -- relevant for a Framework 16 in a
  dock. Test docked + undocked.

### The HONEST ceiling (why "seamless" is not a config tweak)
Per the greetd maintainer (kennylevinsen greetd issue #17): a truly seamless,
zero-black, GDM-style FADE from plymouth -> greetd is NOT a solved declarative
feature today. "plymouth quits as soon as greetd starts"; the black screen is
just greetd+greeter coming up. GDM's smoothness is bespoke. A proper fix needs
EITHER upstream plymouth using drmModeCloseFB to retain KMS state/framebuffer on
exit (WIP: plymouth MR-173) OR a libseat/logind BACKGROUND-session approach:
start the session in the background, DISABLE greetd's VT switch, then manually
chvt + `plymouth quit`. That is experimental, compositor-specific (must verify
with mango), and belongs in labs/ + VM -- NOT a quick param change.
=> Anyone promising a clean seamless handoff on greetd today via a couple kernel
   params is band-aiding. Naming that honestly is the point of this section.

### STRATEGY -- three tiers (pick per honest cost/benefit; all VM-gated)
TIER 1 -- Kill the avoidable NOISE (reliable, available now; cosmetic-only risk)
  - Add `quiet` to boot.kernelParams (currently missing) so kernel text stops
    printing during the gap. Keep `splash`.
  - Set boot.plymouth.theme deliberately; get OFF bgrt (AMD ACPI-logo conflict)
    -> spinner/breeze-class theme avoids the firmware-logo/framebuffer clash.
  - Evaluate plymouth.use-simpledrm (test docked + undocked, LUKS prompt visible).
  Outcome: cleaner boot (no text flash), though a brief black gap may remain.
  This is essentially 049 Option B/C done with eyes open. LOW risk, real win.

TIER 2 -- Shrink the 2.8s GAP (moderate; needs measurement)
  - `systemd-analyze critical-chain greetd.service` to see the greetd path; check
    if anything can start earlier/in parallel.
  - The gap is tuigreet first-paint, not greetd-start -> investigate tuigreet init
    cost; plymouth ShowDelay / quit-timing so splash isn't torn down before the
    greeter is ready to present.
  Outcome: smaller black window even without a true fade.

TIER 3 -- ACTUAL seamless handoff (real R&D; VM-heavy; labs/)
  - The greetd background-session + disable-VT-switch + manual chvt + plymouth quit
    dance, and/or track upstream plymouth MR-173 (drmModeCloseFB KMS-retain).
  - Compositor-specific: must prove it works with mango, not just sway.
  - This is a RESEARCH SPIKE, not a config edit. Snapshot-heavy VM work. May
    conclude "not worth it until upstream lands" -- a legitimate outcome.

### Recommended path
Do TIER 1 first (VM-proven, then metal, rescue-armed) for an immediate honest
improvement. Measure with TIER 2 to decide if the residual gap is worth chasing.
Treat TIER 3 as a separate, clearly-scoped labs/ research spike -- do NOT block a
real Tier-1 win on the perfect Tier-3 fade. "Clean black boot" (049 Option A) also
remains a valid Tier-1-adjacent answer: a fast clean black is smoother than a
flickering splash.

### Dependencies UNCHANGED (research reinforces them)
- INT-056 (TTY rescue) MUST precede any boot change -- the AMD framebuffer/KMS
  handoff is EXACTLY what blanked gen 92; a working Ctrl+Alt+F2 is the safety net.
- INT-024 VM testing for every option. simpledrm's docked-monitor caveat makes
  VM + real docked/undocked testing mandatory before metal.

### NOT the fix
- Lanzaboote (INT-059) is SECURE BOOT, not visual smoothness. It will not remove
  the gap or add a fade. Decoupled from 049; don't pin smoothness hopes on it.

## PLYMOUTH vs ALTERNATIVES + DECISION (2026-07-01)

### Is there something BETTER than Plymouth? (researched)
No tool to switch TO. Plymouth is effectively the ONLY mature Linux boot-splash
(Fedora/Arch/Gentoo/Ubuntu all use it). "Alternatives" are only: (a) Plymouth
themes (still Plymouth), or (b) NO splash at all. There is no "Plymouth-but-better"
competitor. Per ArchWiki/CachyOS: Plymouth is "not a system-critical component"
and "has a nasty habit of breaking boot under various circumstances" -- which is
literally our gen-92 history. So the real axis is NOT "Plymouth vs X"; it is
"tuned Plymouth vs NO Plymouth."

### DECISION (2026-07-01): KEEP Plymouth, TUNE it
Rationale: we want to preserve the GRAPHICAL LUKS passphrase prompt. Dropping
Plymouth (Option A) would give a text-console LUKS prompt -- functional, but we
value the graphical prompt. So execution path = Option B/C (theme swap + quiet),
NOT Option A (disable).
- Trade-off accepted: keeping Plymouth keeps the AMD framebuffer-handoff conflict
  surface + the "breaks boot sometimes" fragility. Mitigated by VM-gating + the
  bgrt->spinner theme swap (removes the specific ACPI-logo AMD conflict) + rescue.
- Option A (drop Plymouth) is NOT chosen, but RETAINED as the fallback if tuning
  cannot make Plymouth stable on the 780M -- resilience beats splash if forced.

### CONSTRAINT this decision creates (important)
Keeping the graphical LUKS prompt INTERACTS with plymouth.use-simpledrm: with
simpledrm, a DOCKED-laptop LUKS prompt may be INVISIBLE (Arch). Framework 16 may
dock -> the simpledrm choice must be tested DOCKED + undocked so the kept prompt
is never silently hidden. This makes the docked/undocked real-hardware test a HARD
gate, not optional. Add to gates below.

### Refined Tier-1 execution (given KEEP-and-tune)
1. Add `quiet` to boot.kernelParams (keep `splash`). Stops kernel-text flash.
2. Swap boot.plymouth.theme OFF bgrt -> spinner/breeze-class (kills the ACPI-logo
   AMD conflict; a candy-neon-recolored spinner could match the forest later).
3. Decide plymouth.use-simpledrm by DOCKED test: prompt must stay visible docked.
4. VM-prove all three (5 clean boots + recovery), THEN metal, rescue-armed.
Gate additions:
- [ ] Graphical LUKS prompt still visible -- tested BOTH docked and undocked
- [ ] Chosen plymouth theme is not bgrt; boots clean on 780M in VM

## LIFECYCLE EXTENSION (2026-07-13 discussion) -- boot-polish grows to the WHOLE arc
This intent is boot-focused (Plymouth -> greetd -> session display handoff). A design discussion
extended the vision to the FULL system lifecycle: power-on -> boot -> login -> session -> logout
-> SHUTDOWN. Two intertwined goals:

1. VISUAL SEAMLESSNESS (the aesthetic soul): flicker-free handoffs, smooth fades, no jarring stage
   transitions, no flash-to-black-console, no resolution blink -- the screen flows gracefully from
   firmware through greeter into session and back out. NOT about speed (boot is already fast at
   ~7s -- that is fine); about GRACE. This is "flicker-free boot," a real discipline.
2. LIFECYCLE CORRECTNESS (system health -- the more important half): orderly, CONSISTENT startup
   AND shutdown. Key observation: the three compositor profiles (mango / Miracle / Pinnacle) each
   shut down DIFFERENTLY -- different teardown order, different process-kill behavior, different
   display release. Inconsistent/ungraceful shutdown stresses the system (half-written state, fs
   stress, occasional boot problems downstream) and "does not give software a moment to disconnect
   properly." Goal: a graceful, CONSISTENT shutdown across all three profiles.

WHERE IT IS BUILT: proven in the faelight-vm (INT-027) for the LOGIC/ORDERING (systemd shutdown
ordering, compositor stop hooks, boot sequence) -- the part that can brick/corrupt on metal. Final
VISUAL flicker-tuning happens on real metal (VM virtual GPU != Framework 16 AMD 780M display
path). VM-for-logic, metal-for-visual. (Supersedes the older "test in VM via INT-024" pointer:
the concrete VM is now 027's OVMF-capable faelight-vm.)

NOTE: 049 is the boot/login/shutdown EXPERIENCE (per INT-078's own boundary: 078 = boot
INFRASTRUCTURE, 049/054 = experience). Tonight's seamlessness+shutdown vision lives HERE, not in
Everglow.

## Gate Check
🚫 049 -- cancelled: boot-polish targeted greetd stability and the NixOS display handoff. Omarchy boots through limine with its own greeter; nothing transfers. -- approved by: christian 2026-08-27
