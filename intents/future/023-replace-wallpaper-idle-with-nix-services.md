---
id: 023
date: 2026-06-03
type: improvement
title: "Replace faelight-wallpaper and faelight-idle with NixOS services"
status: planned
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
⬜ Wallpaper service running via systemd
⬜ Idle service running via systemd
⬜ niri config cleaned
⬜ Doctor still 100%
