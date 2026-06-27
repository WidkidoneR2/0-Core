---
id: 084
date: 2026-06-23
type: future
status: in-progress
title: "faelight-launcher: GTK app launcher with faelight-logout-grade polish"
tags: [launcher, gtk, ui, wayland, candy-neon, post-072]
version: TBD
---
## Why
INT-072 retired faelight-palette (the old command-palette launcher, unused since
Niri 11.0.0) and removed the engine launcher dispatch + the broken `launch` alias.
The forest currently has NO launcher. This intent builds a new one -- done right,
Nix-native, with the visual quality of faelight-logout (INT-064, the candy-neon
Wayland power menu). When this ships, app launching returns to the forest as a
first-class, beautiful tool.
## Vision
- GTK (likely GTK4 + gtk4-layer-shell) for a polished, native Wayland surface --
  matching how faelight-logout looks and feels (candy-neon, smooth, intentional).
- Fast fuzzy app search; launches .desktop apps + forest tools.
- Themed with the Faelight Visual Language (reuse faelight-logout's CSS/aesthetic).
- Bound to a compositor keybind (MangoWM) as the daily launcher.
## Must-Do (reconnect what 072 removed)
- Re-add a `launch` alias -> the new faelight-launcher (072 removed the phantom
  `launch` -> faelight-launcher alias; this points it at the REAL new binary).
- Optionally re-add `faelight launch launcher` arm in the umbrella (072 removed it).
- Register in registry/tools.toml with a CORRECT description (072 found palette was
  mislabeled "Color palette and theme management" -- do not repeat that).
## Approach (rough)
Phase 0 -- study faelight-logout's GTK/layer-shell setup + CSS as the template.
Phase 1 -- build the launcher: app discovery (.desktop scan), fuzzy search, launch.
Phase 2 -- theme to candy-neon parity with faelight-logout.
Phase 3 -- wire keybind + `launch` alias + registry entry; deploy; verify.
## Depends On
- AFTER INT-072 (palette decommission -- DONE). Reference INT-064 (faelight-logout)
  for the GTK + candy-neon pattern.
## The Rule
"What the forest launches, it launches beautifully." 🌲


## Progress -- 2026-06-26 (Phases 1-3 core DONE -- working daily launcher)
faelight-launcher is LIVE: glassy candy-neon GTK4 floating launcher, summoned by Super+Space.
Built in Python GTK4 + gtk4-layer-shell, cloned from faelight-logout's (INT-064) proven recipe.
- Phase 0: studied faelight-logout's GTK/layer-shell + CSS -- used as the template.
- Phase 1: app list + fuzzy filter (type to narrow), arrow/Enter/Esc nav, smooth Esc fade.
  (Placeholder app list for now; real .desktop scan is the next milestone -- Phase 1B.)
- Phase 2: candy-neon theme -- glassy panel (rgba 0.55, desktop shows through), PER-APP neon
  colors (lime/coral/aqua/violet/pink/gold, cycled), selected row glows its color. Float-mode
  default (--fullscreen for an overlay variant). This look became a design-language reference --
  Christian wants ReGreet (054) + the bar themed to match.
- Phase 3 (packaging + wiring): packaged in flake.nix as faelight-launcher (wrapGAppsHook4 +
  preFixup baking LD_PRELOAD=libgtk4-layer-shell.so -- a clean binary, no nix-shell/manual
  preload). Installed on framework16. mango keybind SUPER,space -> faelight-launcher (toggle-
  floating moved to SUPER+SHIFT,space). Aliases: launcher + launch -> faelight-launcher
  (re-pointed off the retired faelight-palette, per the 072-cleanup Must-Do).
REMAINING (follow-on milestones, not blocking the working tool):
- Phase 1B: real .desktop scanning + forest-tool entries (replace the placeholder app list).
- Registry entry in registry/tools.toml with a CORRECT description (charter Must-Do).
- Optional: `faelight launch launcher` umbrella arm.
- Design-language capture: the glassy-neon panel recipe as a reusable aesthetic (for 054, bar).
LESSONS: flakes only see git-tracked files (untracked pkg dir = build can't find src);
gtk4-layer-shell must load before libwayland (wrapGAppsHook4 preFixup handles it).


## Phase 1B + finish (2026-06-27): real app discovery, sections, registry -- 084 CORE COMPLETE
- REAL .desktop SCANNING replaced the placeholder list: XDG dirs (XDG_DATA_DIRS + nix-profile +
  /run/current-system/sw/share), parses Name/Exec, strips field codes, terminal apps wrapped in
  alacritty, NoDisplay/Hidden skipped, junk blocklist (NixOS manual, remote-viewer, vim/gvim/nvim,
  etc.).
- SECTIONS: "FOREST TOOLS" (curated forest binaries, A-Z) + "APPLICATIONS" (scanned, A-Z), each
  with a dim candy-neon header. Headers are non-selectable; arrow keys skip them.
- AUTO-SCROLL fix: selected row grab_focus() scrolls it into view -- selection never hides below
  the fold (was the reported bug). Fixed-height ScrolledWindow (420px) so the window stays put.
- REGISTRY: registered in registry/tools.toml with an HONEST description (the 072 Must-Do --
  the old faelight-palette was mislabeled; this one is not).
- Verified live: Super+Space summons the finished launcher; real apps + forest tools, sectioned,
  filtered, auto-scrolling, glassy candy-neon.

CORE COMPLETE. Optional follow-ons remaining (nice-to-have, non-blocking):
- `faelight launch launcher` umbrella arm.
- Capture the glassy-neon panel as a reusable design language (for INT-054 ReGreet + the bar).
- App icons (currently text-only rows).
