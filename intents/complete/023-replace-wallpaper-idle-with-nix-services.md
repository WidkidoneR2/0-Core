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

- [ ] faelight-wallpaper replaced with systemd.user.service
- [ ] faelight-idle replaced with services.swayidle
- [ ] Both removed from niri spawn-at-startup
- [ ] Both marked retired in registry
- [ ] Wallpaper still shows on login

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
