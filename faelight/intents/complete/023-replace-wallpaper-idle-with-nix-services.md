---
id: 023
date: 2026-06-03
type: improvement
title: "Replace faelight-wallpaper and faelight-idle with NixOS services"
status: complete
tags: [nixos, services, wallpaper, idle, declarative]
priority: medium
---

## Vision

Replace two Rust wrapper tools with proper NixOS declarative services.
faelight-wallpaper just calls a wallpaper setter. faelight-idle just calls swayidle.
NixOS owns these better than Rust wrappers do.

## Why Now

INT-016 audit identified these as Nix-replaceable.
Currently they are in niri spawn-at-startup which works but is not declarative.
NixOS services survive reboots, restarts, and rebuilds cleanly.

## Approach

faelight-wallpaper → systemd.user.service in home.nix:
  systemd.user.services.faelight-wallpaper = {
    description = "Forest wallpaper";
    wantedBy = [ "graphical-session.target" ];
    serviceConfig.ExecStart = "${pkgs.swaybg}/bin/swaybg -m fill -i /path/to/wallpaper";
  };

faelight-idle → services.swayidle in home.nix:
  services.swayidle.enable = true;
  services.swayidle.timeouts = [ ... ];

Remove both from niri spawn-at-startup after services are wired.
Retire both tools in registry once replaced.

## Success Criteria

- [~] faelight-wallpaper replaced with systemd.user.service <!-- INT-130: DEFERRED -- investigation reversed the plan: tool is health-reactive (wlr-layer-shell direct), not a swaybg wrapper. KEEP AS RUST. approved by christian 2026-06-04 -->
- [~] faelight-idle replaced with services.swayidle <!-- INT-130: DEFERRED -- tool uses ext-idle-notify-v1 natively, not a swayidle wrapper. KEEP AS RUST. approved by christian 2026-06-04 -->
- [x] Both removed from niri spawn-at-startup <!-- INT-130: verified 2026-07-10 -- absent from all startup config; niri itself uninstalled (which niri -> not found; only a stale GPU/Wayland comment remains in configuration.nix:54) -->
- [x] Both marked retired in registry <!-- INT-130: VOID -- superseded by the KEEP decision (2026-06-04). Both tools intentionally NOT retired: registry shows retired=false for both. Gate no longer applies. -->
- [x] Wallpaper still shows on login <!-- INT-130: verified 2026-07-10 -- wallpaper live on the running session -->

## Gate Check
⏸ Wallpaper service -- deferred: tool has health-reactive intelligence, not replaceable -- approved by: christian 2026-06-04
⏸ Idle service -- deferred: tool uses native Wayland protocol directly -- approved by: christian 2026-06-04
✅ niri config already clean
✅ Doctor 100% confirmed

## Investigation Finding (2026-06-04)

These tools are NOT simple wrappers -- they are genuine Rust tools with forest intelligence:

faelight-wallpaper:
- Uses smithay-client-toolkit + wlr-layer-shell directly
- Health-reactive: color shifts with forest health score
- This is forest intelligence, not a swaybg wrapper

faelight-idle:
- Uses ext-idle-notify-v1 Wayland protocol natively
- Direct Wayland integration, not a swayidle wrapper

Decision: KEEP BOTH AS RUST. Replacing with NixOS services would lose
health-reactive wallpaper and native Wayland idle detection.

INT-016 boundary analysis was correct in spirit but wrong about these two
specifically -- they have forest awareness that a Nix service cannot replicate.

The only NixOS improvement needed: ensure they start reliably via systemd
user services rather than niri spawn-at-startup (more robust on session start).

<!-- Gates reconciled per INT-130, 2026-07-10: G1/G2 deferred (KEEP-as-Rust decision, approved christian 2026-06-04); G3/G5 ticked (verified live: niri uninstalled, wallpaper showing); G4 voided (superseded by KEEP -- both retired=false). NOTE: the investigation named 'declarative startup via systemd user service' as the one improvement worth doing; it was never implemented (no systemd.user.service exists). Recorded, not a success criterion -- 023 closes as-is. SEPARATE observation: configuration.nix:54 has a stale 'so niri can run' comment; niri is gone. -->
