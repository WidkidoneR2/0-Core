---
id: 051
date: 2026-06-08
type: infrastructure
title: "NixOS structure: user modules, compositor modules, flake cleanup"
status: complete
tags: [nixos, home-manager, structure, pinnacle, mango, flake]
priority: high
---
## Why
The NixOS configuration needed proper modular structure before building
further. home.nix was monolithic, flake.nix had a broken input, and
compositor config was hardcoded strings.

## What was done
- modules/desktop/pinnacle.nix: proper mkEnableOption NixOS module
- modules/desktop/mango.nix: proper mkEnableOption NixOS module
- Removed broken mango flake input (mangowm/mango does not exist as flake)
- MangoWM now from pkgs.mangowc (correct nixpkgs package name)
- users/christian/fsh.nix: owns fsh config file
- users/christian/alacritty.nix: owns alacritty config
- users/christian/git.nix: programs.git + programs.delta (26.05 API)
- home.nix: imports the three modules, removes owned entries
- home.nix: added home.username + home.homeDirectory
- home.nix: removed niri references, removed duplicate packages
- home.nix: replaced yazi with broot
- git.nix: fixed 5 Home Manager 26.05 deprecation warnings
- Removed dead niri session entry from greetd (automatic via rebuild)
- faelight-shell greeting: neon candy truecolor colors
- faelight-shell greeting: NixOS generation + compositor display
- faelight-shell greeting: philosophy quote bold purple not dimmed
- Friday: migrated all knowledge from arch to nixos domain
- Friday: added 5 NixOS-specific knowledge entries
- Structure audit: 35/39 required paths present

## Gate Check
✅ modules/desktop/pinnacle.nix complete and enabled
✅ modules/desktop/mango.nix complete and enabled  
✅ Broken flake input removed, clean flake.lock
✅ users/christian/fsh.nix wired and deployed
✅ users/christian/alacritty.nix wired and deployed
✅ users/christian/git.nix wired and deployed (zero deprecation warnings)
✅ nixos-rebuild switch clean (zero errors, zero warnings)
✅ Friday: zero arch domain entries remaining
✅ faelight-shell: neon candy greeting building clean (zero warnings)
✅ Intent ledger updated (INT-006, INT-038 moved to in-progress)
