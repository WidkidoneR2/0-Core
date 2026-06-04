---
id: 022
date: 2026-06-03
type: housekeeping
title: "Retire NixOS-obsolete tools: faelight-bootstrap, verify-bootstrap, core-protect, dotctl"
status: planned
tags: [nixos, retirement, cleanup, tools]
priority: medium
---

## Vision

Remove tools that served Arch Linux but are obsolete on NixOS. Clean registry,
clean workspace, clean philosophy. The forest does not carry dead weight.

## Why Now

INT-016 audit identified these 4 tools as NixOS-obsolete:
- faelight-bootstrap: nixos-rebuild IS the bootstrap
- verify-bootstrap: NixOS generations replace this entirely
- core-protect: LUKS + immutable Nix store replaces chattr-based locking
- dotctl: home-manager replaces stow/dotfile management

## Approach

For each tool:
1. Mark retired = true in 01-registry/tools.toml
2. Add to Cargo.toml exclude list if in workspace
3. Add retirement note to the source (// RETIRED: reason)
4. Remove any aliases from config.fsh and state.db

## Success Criteria

- [ ] faelight-bootstrap: retired in registry, excluded from workspace
- [ ] verify-bootstrap: retired in registry, excluded from workspace
- [ ] core-protect: retired in registry, aliases updated
- [ ] dotctl: retired in registry, excluded from workspace
- [ ] No broken aliases remain pointing at retired tools

## Gate Check
⬜ All 4 tools marked retired
⬜ No broken aliases
⬜ Doctor still 100% after changes
