---
id: 087
date: 2026-06-23
type: future
status: planned
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
## The Rule
"The proof is banked. Now choose what serves the daily work." 🌲
