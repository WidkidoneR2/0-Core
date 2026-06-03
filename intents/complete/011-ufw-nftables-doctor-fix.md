---
id: 011
date: 2026-06-03
type: bugfix
title: "UFW→nftables doctor fix: update hardcoded security check for NixOS"
status: complete
tags: [doctor, security, firewall, nftables, nixos]
priority: medium
---

## Why

doctor shows 'UFW ❌' even though NixOS native firewall is active.
The check is hardcoded to look for UFW which doesn't exist on NixOS.

## Fix

Update checks.rs security hardening check to detect NixOS firewall
via networking.firewall systemd service instead of ufw binary.

## Gate

doctor shows 'Firewall ✅' when networking.firewall.enable = true.
