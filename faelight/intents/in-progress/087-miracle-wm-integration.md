---
id: 087
date: 2026-06-23
type: future
status: in-progress
title: "Miracle-wm: second compositor profile (Mir-based, Sway-IPC)"
tags: [compositor, miracle, wayland, mir, profile, 010]
version: TBD
---
## Why
Miracle-wm chosen as second compositor (2026-06-23, pure-Rust constraint relaxed -- judged on
merit). More mature than Pinnacle (v0.8.3 Dec 2025, 723 stars), i3/Sway-compatible IPC
(swaymsg/Waybar -- may slot into faelight-bar), WASM plugins (Rust), YAML config, smooth
animations, hot-reload (Meta+Shift+R). Built on Mir; C++ core.
## Vision
- Miracle as SECOND compositor alongside MangoWM (mango stays daily driver).
- Ties INT-010 (env switching) -- a second compositor IS a switchable environment.
- modules/desktop/miracle.nix (mirror mango.nix). Faelight theme + keybinds; reuse Sway-IPC
  for faelight-bar. greetd session picker: mango or miracle.
## Dependencies / sequencing
AFTER INT-085 + INT-086 (clean compositor ground). Relates INT-010, 055, 005.
Packaging: check nixpkgs for miracle-wm or add flake input.
## Approach (rough)
P0 packaging (nixpkgs miracle-wm? version?). P1 modules/desktop/miracle.nix. P2 YAML config
+ theme + faelight-bar via Sway IPC. P3 greetd session entry. P4 daily-drive trial.
## Status (2026-07-11): Miracle ENABLED + metal-tested -- LAUNCHES on AMD. Config is the remaining work.

The dormant modules/desktop/miracle.nix was enabled (imported + faelight.desktop.miracle.enable
= true in framework16) now that INT-056's SafeShell rescue net is confirmed on metal -- the exact
precondition the module's own comments named. Deployed gen 347; miracle.desktop is live in
/etc/greetd/sessions/ alongside mango, pinnacle, safeshell.

**Metal test result (behind the SafeShell net, keybinds recon'd first from the miracle-wm wiki):**
Miracle-wm LAUNCHES on the AMD 780M. This answers 087's core uncertainty -- last night's VM
black-screen was purely the software-GL/llvmpipe limit (Mir needs a real GPU); on real metal it
runs. Observed:
- Launched to a PURE BLACK session (expected -- Mir shows a blank tiling session until configured;
  no wallpaper/bar/startup apps by default). Not a failure -- an unconfigured compositor.
- Super+Enter DID spawn Alacritty (terminal keybind works), tiled full-screen. But Alacritty came
  up unthemed (white/default) and launched its DEFAULT shell, not fsh -- because the bare Miracle
  session doesn't inherit mango's fsh-as-login-shell + forest theme.
- No bar (expected -- Miracle ships bare like Pinnacle; bar is INT-067's job via Sway-IPC).
- Super+Shift+E (quit_compositor) exited cleanly back to the greeter. Clean, calm, no lockout.

**Verdict: Miracle is a VIABLE 2nd compositor profile on this hardware.** What remains is CONFIG,
not viability. Phase status: P0 packaging done; P1 module done + enabled; P3 greetd session done +
metal-tested (launches); P4 daily-drive NOT yet (needs config); **P2 (YAML config + theme +
faelight-bar via Sway-IPC) is the remaining work.**

P2 config to-do (deferred to a later session, planned as a closing win 2026-07-11 evening):
- ~/.config/miracle-wm/config.yaml (or home-manager equiv): set terminal to launch fsh (not
  default shell), forest theme (candy-neon border/gaps), wallpaper, startup apps.
- faelight-bar under Miracle via Sway-IPC (ties INT-067 -- now 'faelight-bar under whichever
  non-mango compositor is active', three-compositor forest: mango daily + Pinnacle + Miracle).

SEQUENCING NOTE (Christian, 2026-07-11): hold INT-010 (env switching) until BOTH Pinnacle and
Miracle are fully configured -- don't open new fronts while two compositors are half-done.

## The Rule
"The proof is banked. Now choose what serves the daily work." 🌲
