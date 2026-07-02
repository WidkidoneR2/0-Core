---
id: 016
date: 2026-06-03
type: housekeeping
title: "Tool audit: Nix/Rust boundary -- what should be Nix vs Rust"
status: complete
tags: [audit, nix, rust, tools, boundary, philosophy]
priority: high
---

## Why

Now that the forest is on NixOS, every tool needs individual review.
The question for each: is this better as Rust or Nix?

Nix is better at: system wiring, service declaration, reproducible
environments, package composition, configuration management.

Rust is better at: intelligence, TUI rendering, performance-critical
tools, Friday brain, shell execution, custom protocols.

## Tools to Audit

- faelight-update → could be a Nix script wrapping nix flake update
- faelight-bootstrap → Nix handles this now
- faelight-cleanup → nix-collect-garbage replaces most of this
- safe-update → wrap nixos-rebuild with health gate
- core-diff → nix-diff integration?
- faelight-diff → keep as Rust, add nix-diff awareness
- faelight-git → keep as Rust, review NixOS path assumptions
- faelight-hooks → review for NixOS compatibility
- faelight-notify → keep as Rust, PAM already fixed
- faelight-idle → keep as Rust, review Wayland assumptions

## Gate

Written decision for each tool: keep Rust / replace with Nix / hybrid.

## Audit Findings (2026-06-03)

### The Boundary Principle
Rust when: needs state.db, Friday awareness, complex TUI, runtime intelligence.
Nix when: wraps another program, manages configuration, system lifecycle NixOS already owns.

### Stays Rust (intelligence required)
core, faelight-shell, faelight-fm, faelight-bar, faelight-git, faelight-notify,
faelight-lock, faelight-daemon, friday-chat, faelight-term, faelight-ade,
faelight-release, faelight-context/contextd, intent/intent-guard, faelight-sandbox,
faelight-vault, db-browse, faelight-menu, faelight-palette, faelight-diff,
faelight-docs, faelight-gen, faelight-clipboard

### Replace with Nix (wrappers around other programs)
- faelight-wallpaper → systemd.user.service calling wpaperd/swaybg
- faelight-idle → services.swayidle in NixOS
- safe-update / latest-update → absorbed into pkgs/faelight/scripts/deploy
- faelight-update → absorbed into deploy script

### Retire (NixOS makes these obsolete)
- faelight-bootstrap → nixos-rebuild IS the bootstrap
- verify-bootstrap → NixOS generations replace this
- core-protect → LUKS + immutable store replaces chattr
- dotctl → home-manager replaces stow

### Needs investigation
- faelight-hooks → what does this do on NixOS?
- faelight-gen → still useful? check usage
- faelight-link → symlink manager, home-manager may replace
- faelight-zone → workspace zones, check if niri handles this natively
- profile → forest profiles, check if still relevant

### Action items
1. Retire: faelight-bootstrap, verify-bootstrap, core-protect, dotctl → INT-??? 
2. Replace with Nix: faelight-wallpaper, faelight-idle → tie into NixOS services
3. Investigate: faelight-hooks, faelight-link, faelight-zone, profile
4. Keep all intelligence tools as Rust

### Verdict
~7 tools can be retired or replaced with Nix.
~5 tools need investigation.
~25 tools stay Rust and are correct as-is.
