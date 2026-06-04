---
id: 030
date: 2026-06-04
type: feature
title: "fsh semantic domains: project/intent/experiment as first-class shell objects"
status: planned
tags: [fsh, shell, domains, semantic, vocabulary]
priority: high
---

## Vision

The shell understands concepts, not just filesystem paths.

Instead of:
  cd ~/0-core/intents/future
  ls | grep planned

Just:
  intent list
  project list
  experiment list
  vm list

The filesystem remains underneath.
Humans interact with higher-level objects.

## Why

This is what makes fsh genuinely different from bash + aliases.
The shell should understand the forest's domain model.
Every command should speak forest-native language first.

## Approach

- fsh vocabulary expansion: domain commands as first-class
- intent, project, vm, experiment, sandbox as shell domains
- Tab completion understands domain objects
- Colors convey semantic meaning (intent status, project health)
- Rich table output for list commands
- Pairs with INT-012 (intent ledger) and INT-013 (launcher)

## Philosophy

"Not text streams. Not configuration. Structured wisdom."
The shell is the forest's voice. It should speak human first.

## Gate

- [ ] intent list works natively in fsh (no core prefix)
- [ ] project list shows forest projects with status
- [ ] experiment list shows labs/ entries
- [ ] vm list shows active VMs
- [ ] Tab completion works for all domain objects
- [ ] Colors reflect semantic state not just file type
