---
id: 031
date: 2026-06-04
type: feature
title: "faelight-release v2: NixOS-native release manager"
status: planned
tags: [faelight-release, nixos, release, changelog, version]
priority: high
---

## Why

faelight-release v1 is Arch-era and dangerous on NixOS:
- Writes to /etc/faelight/VERSION (immutable on NixOS)
- Reads 300+ git commits including Arch-era history
- References 00-meta/CHANGELOG.md (being renamed to meta/)
- No understanding of NixOS generations as release artifacts
- bump alias disabled until this is fixed

## What needs changing

1. Version source: /etc/faelight/VERSION → flake.nix version field
2. Changelog path: 00-meta/CHANGELOG.md → meta/CHANGELOG.md
3. Git log scope: filter to NixOS era commits only (after 2026-06-01)
4. Release artifact: NixOS generation number as part of release identity
5. No Arch assumptions anywhere in changelog generation
6. README update: NixOS-aware, not Arch-aware

## Vision

faelight-release v2 on NixOS:
- Reads version from flake.nix
- Changelog scoped to NixOS era intents (INT-001+)
- Release = NixOS generation + semantic version + intent summary
- Writes to meta/CHANGELOG.md
- bump alias re-enabled after this is complete

## Pre-1.0.0 requirement

This MUST be complete before Faelight NixOS 1.0.0.
The release tool creates the release artifact.
A 1.0.0 release made with broken tooling is not a real release.

## Gate

- [ ] bump runs without errors on NixOS
- [ ] Changelog only includes NixOS era commits
- [ ] Version reads from flake.nix correctly
- [ ] /etc/faelight/VERSION write replaced with NixOS-appropriate mechanism
- [ ] meta/CHANGELOG.md path correct
- [ ] README generation NixOS-aware
