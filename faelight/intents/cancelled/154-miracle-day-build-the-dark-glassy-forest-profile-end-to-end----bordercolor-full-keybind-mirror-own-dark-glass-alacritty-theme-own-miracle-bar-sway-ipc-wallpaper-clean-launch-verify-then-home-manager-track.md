---
id: 154
date: 2026-07-12
type: future
title: "Miracle Day: build the dark glassy forest profile end-to-end -- border/color, full keybind mirror, own dark-glass Alacritty theme, own miracle-bar (Sway-IPC), wallpaper, clean-launch verify, then home-manager track"
status: cancelled
tags: [compositor, miracle, theme, bar, alacritty, dark-glass, 087, 067]
---

## Vision
Miracle as the DARK GLASSY FOREST -- the same forest as mango (fsh, keybinds, tooling) wearing
different scenery: deep near-black glass, glossy turquoise/teal accents, subtle transparency, a
sharp dark look. A change of mood, not a change of bones. Mango stays the primary WORK profile;
Miracle is chosen for scenery and pace. This is a DEDICATED full-day build -- unhurried,
edit-relaunch iteration, tuned until it looks right.

## Starting point (proven in INT-087, 2026-07-13)
Config already LOADS and Miracle already MEETS the communicate-goal: fsh runs in the terminal
(alacritty -e fsh), Super+Enter opens a terminal, Super+B opens Brave, rounded borders render,
startup_apps: [] set for clean launch. Working config lives live-editable at
~/.config/miracle-wm/config.yaml (safety backup at faelight/miracle-wip/*.working). This day
turns "works" into "the dark glassy forest."

## Build checklist (rough order -- each verified before the next)
1. CLEAN LAUNCH (first gate): fresh Miracle greetd session comes up EMPTY -- no auto-terminal,
   no black/blank wedged Alacritty (confirms startup_apps: [] fixed the double-terminal). Open
   terminals on demand with Super+Enter.
2. BORDER COLOR: fix white -> turquoise. focus_color isn't rendering as configured; resolve the
   Color format (hex 0xAARRGGBB vs r/g/b/a object) for focus vs unfocused. Dark-glass palette:
   near-black unfocused, turquoise/teal glow on focus.
3. FULL KEYBIND MIRROR: audit mango's keybinds (doctor reports "mango: 36 keybindings") and port
   each into Miracle's custom_actions + default_action_overrides -- terminal, close window,
   workspace switch/move, launcher, media keys, brightness, screenshot, lock, etc. Goal: muscle
   memory carries across profiles. Decide which are universal vs Miracle-specific.
4. OWN DARK-GLASS ALACRITTY THEME: build a Miracle-specific Alacritty theme (deep dark bg,
   glossy accents, subtle opacity) DISTINCT from mango's. Design Q: how to give Miracle its own
   alacritty config without disturbing mango's single tracked alacritty config
   (xdg.configFile."alacritty" at alacritty.nix:5) -- per-profile config? alacritty --config-file
   in the terminal command? separate theme file imported per session? Resolve on the day.
5. OWN MIRACLE-BAR: build a SEPARATE bar for Miracle (miracle-bar, distinct from the future
   pinnacle-bar -- Christian wants genuinely separate bars per profile, NOT one shared themed
   bar). Miracle speaks Sway/i3-compatible IPC (swaymsg/waybar ecosystem), so the bar drives off
   Miracle's IPC. Ties INT-067. Styled dark-glass to match. (DankMaterialShell was evaluated and
   REJECTED -- external Quickshell shell, would dilute the forest identity; build our own.)
6. WALLPAPER: deep glossy dark forest scenery, set at session start (swaybg or Miracle's
   mechanism, via startup once we allow a startup app for the wallpaper only).
7. HOME-MANAGER TRACK (final settling step): once dialed in, move config.yaml + display.yaml +
   the alacritty theme + the bar into home-manager (xdg.configFile pattern, mirroring mango at
   home.nix:24). This makes Miracle reproducible but READ-ONLY (no more live editing) -- so it is
   the LAST step, only when nothing else needs tuning. Retire faelight/miracle-wip/*.working
   after.

## Design questions to resolve ON the day (do not assume)
- Border Color format that actually renders turquoise on focus (test hex vs object).
- Per-profile Alacritty theming mechanism (the cleanest way Miracle gets its own theme without
  touching mango's).
- Bar tech: does an existing faelight-bar already speak Sway-IPC (reusable under Miracle), or is
  miracle-bar a fresh build? Recon INT-067 state first.
- Which mango keybinds are universal (mirror) vs profile-specific.
- Wallpaper mechanism under Miracle (and whether it needs the one allowed startup app).

## Success criteria
- [ ] Fresh Miracle launch is CLEAN -- empty session, no black terminal (startup_apps fix proven)
- [ ] Border renders dark-glass: near-black unfocused, turquoise focus (color-format fixed)
- [ ] Full keybind set mirrored from mango -- each fires in a real Miracle session
- [ ] Own dark-glass Alacritty theme renders in Miracle, mango's theme UNCHANGED
- [ ] miracle-bar shows under Miracle (own bar, dark-glass styled) -- via Sway-IPC (INT-067)
- [ ] Dark forest wallpaper set at session start
- [ ] Everything tracked in home-manager; survives a rebuild + fresh login; miracle-wip retired
- [ ] Daily-drivable as a scenery profile: switch in, communicate, switch out, nothing lost

## Dependencies / relationships
- Builds directly on INT-087 (Miracle enabled + config proven). 087 stays the integration
  record; this is the styling/build-out day.
- INT-067 (faelight-bar under secondary compositor) -- the miracle-bar piece lives here.
- Sibling: the "Pinnacle Day" intent (frosted-glass see-through forest), which depends on
  INT-142's keybind fix landing first. Same structure, different mood.
- Mango (primary work profile) is NOT touched -- guard against changes leaking into mango's
  tracked alacritty/keybind config.

## The Rule
"Same forest, different light. Mango works; Miracle is where the dark glass catches it." 🌲

## Gate Check
🚫 154 -- cancelled: The dark glassy profile end-to-end on miracle-wm, which is not installed. The palette work survives in the Hakker Green single-token-source direction. -- approved by: christian 2026-08-27
