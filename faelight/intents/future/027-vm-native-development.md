---
id: 027
date: 2026-06-04
type: feature
title: "VM-native development: vm create/enter/snapshot/rollback"
status: planned
tags: [vm, nixos, qemu, development, sandbox]
priority: high
---

## Vision

VMs as first-class forest objects, not external tools.
vm create rust-lab
vm enter rust-lab
vm snapshot stable
vm rollback stable

## Why

INT-005 (faelight-login) showed we need VM iteration without rebooting real system.
INT-021 (Pinnacle VM) is the immediate need.
Long term: all high-risk work happens in VMs first.

## Approach

- Wrap libvirt/QEMU with forest-native commands
- NixOS VMs built from flake configurations
- Snapshots mapped to NixOS generations
- Integration with intent system: vm tied to active intent
- fsh vocabulary: vm is a first-class domain

## Gate

- [ ] vm create <name> spins up NixOS VM from flake
- [ ] vm enter <name> opens shell in VM
- [ ] vm snapshot <tag> saves state
- [ ] vm rollback <tag> restores state
- [ ] INT-021 Pinnacle VM uses this system
