---
id: 016
date: 2026-06-03
type: housekeeping
title: "Tool audit: Nix/Rust boundary -- what should be Nix vs Rust"
status: planned
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
