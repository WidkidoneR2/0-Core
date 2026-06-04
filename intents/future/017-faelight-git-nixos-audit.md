---
id: 017
date: 2026-06-03
type: improvement
title: "faelight-git NixOS audit: review paths, assumptions, improvements"
status: planned
tags: [faelight-git, nixos, audit, git, improvement]
priority: medium
---

## Why

faelight-git was built on Arch. On NixOS several assumptions may have
changed. The fg alias works but the tool needs a full review.

## What to Check

- Hardcoded paths to scripts/
- Deploy pipeline assumptions
- Intent commit linking still works
- Event bus integration on NixOS
- Any pacman/Arch-specific code

## Gate

faelight-git works cleanly on NixOS. No stale Arch assumptions.
fg alias works for all git operations.
