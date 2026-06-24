---
id: 084
date: 2026-06-23
type: future
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
