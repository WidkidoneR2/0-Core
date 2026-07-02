---
id: 001
date: 2026-06-02
type: milestone
title: "NixOS Era Begins -- Faelight Forest on NixOS"
status: complete
tags: [nixos, milestone, era, foundation]
version: 14.1.0
priority: high
---

## The Moment

Faelight Forest migrated from Arch Linux to NixOS on 2026-06-01.
This intent marks the boundary between the Arch era and the NixOS era.

## What Was Preserved

- 285 completed intents (arch-era/complete/)
- 2983 commits of forest history
- All of Friday's knowledge -- 412 facts, 14 patterns
- state.db intact -- decisions, incidents, shell history
- All 55 Rust tools compiled and running via Nix flake

## What Changed

- System is now fully declarative and reproducible
- Tools deployed via nixos-rebuild, not scripts/
- Home managed by home-manager
- Flake.nix is the single source of truth

## What This Unlocks

- Reproducible environments (friday-dev shell, etc.)
- NixOS VM tests before every switch
- Pinnacle compositor path
- Conference story: one human + AI building a reproducible OS in Rust

## The Forest Remembers

The Arch era is not deleted. It is graduated.
Every intent, every incident, every decision -- archived and visible.
The forest does not forget. It grows.
