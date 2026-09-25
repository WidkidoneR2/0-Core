---
id: 087
date: 2026-06-23
type: future
status: cancelled
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

## Miracle Day scope (added 2026-07-13 -- proven working + build-out map)

### Proven working THIS session (2026-07-13, live in a real Miracle greetd session)
- Config LOADS from ~/.config/miracle-wm/config.yaml (Miracle's own log confirmed the path;
  display.yaml + config.yaml both read). Filename is config.yaml (the module comment saying
  miracle-wm.yaml/miracle-wm.config referred to the separate Mir-level options file -- not this).
- fsh runs in the terminal: `terminal: alacritty -e fsh` WORKS (the "shell doesn't show up"
  symptom is FIXED -- terminal now launches fsh with the full forest banner, "gen 365 - miracle").
- Super+Enter opens a fresh fsh terminal. Super+B launches Brave (custom_actions + action_key:
  super both confirmed live). => Miracle MEETS the communicate-goal NOW (terminal + browser
  reachable from the profile).
- Rounded borders render (radius: 12 applied). startup_apps: [] added for clean launch.

### Known-remaining (the Miracle Day tunes these -- see the dedicated Miracle Day intent)
- Border color renders WHITE, not the configured turquoise focus_color -- color-format issue,
  needs the fix (hex vs r/g/b/a object; confirm which Miracle wants for focus vs unfocused).
- Double-terminal at launch: a black/blank no-prompt Alacritty appeared alongside the working
  one (Miracle auto-spawned a terminal at session start, before the env was ready, so its fsh
  wedged with no prompt). startup_apps: [] SHOULD remove it next launch -- NOT yet verified on a
  fresh launch. First real Miracle-Day gate: launch clean, no black terminal.
- Full keybind set (only Super+B + defaults so far), custom dark-glass alacritty theme, the bar,
  wallpaper -- all still to build.

### The bar decision (settled 2026-07-13)
- Christian builds his OWN bar per profile -- genuinely SEPARATE bars (miracle-bar, pinnacle-bar
  as distinct things), each styled to that profile's mood. NOT one shared themed bar.
- DankMaterialShell (github.com/AvengeMedia/DankMaterialShell) was EVALUATED and REJECTED: it is
  a full external Quickshell/QML+Go desktop shell (bar+launcher+control-center+dynamic wallpaper
  theming, has a NixOS module, supports Miracle) -- powerful, but it would replace faelight-bar
  and dilute the forest identity. Christian wants forest-native, his own code. DMS stays a
  footnote, not a plan.

### Config lives LIVE-EDITABLE until settled (track-when-settled)
- ~/.config/miracle-wm/config.yaml stays a plain editable file during the Miracle Day (fast
  edit-relaunch loop). Home-manager tracking (xdg.configFile."miracle-wm/config.yaml".source =
  ../dotfiles/miracle-wm/.config/miracle-wm/config.yaml, mirroring mango at home.nix:24) is the
  FINAL step once dialed in -- because xdg.configFile makes a READ-ONLY store symlink that kills
  live editing. Also track display.yaml the same way.
- Safety backup of today's working config committed at faelight/miracle-wip/*.working
  (git-only, NOT home-manager-wired -- recovery copy, stays live-editable).

### Sequencing
- The dedicated "Miracle Day" is its own intent (dark glassy forest). Pinnacle gets its own
  "Pinnacle Day" intent (frosted-glass see-through), which depends on INT-142's keybind fix
  landing first. Mango stays the untouched primary work profile.

## Gate Check
🚫 087 -- cancelled: miracle-wm second compositor profile. Hyprland is the compositor and no second profile is wanted. -- approved by: christian 2026-08-27
