---
id: 005
date: 2026-06-03
type: feature
title: "faelight-login + faelight-menu: proper NixOS login flow with greetd"
status: in-progress
tags: [faelight-login, faelight-menu, greetd, tuigreet, nixos]
priority: high
---

## Why

Current login flow: LUKS decrypt → login prompt → manually type niri-session.
This is three steps that should be one clean forest greeting.

## Approach

1. Add services.greetd with tuigreet to configuration.nix
2. Configure greetd to auto-start niri session
3. faelight-login becomes the greeter face
4. faelight-menu works properly as power/session menu

## Gate

Boot → LUKS → faelight-login greeting → niri session starts automatically.
faelight-menu opens cleanly with Mod+Escape.
