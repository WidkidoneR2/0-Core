---
id: 030
date: 2026-06-04
type: feature
title: "fsh semantic domains: project/intent/experiment as first-class shell objects"
status: complete
tags: [fsh, shell, domains, semantic, vocabulary]
priority: high
completed: 2026-06-09
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

## What Was Built
- intent list: reads all three dirs (in-progress, complete, future)
  with correct status from frontmatter -- no core prefix required
- project list: shows 0-core stats, commits, branch, intent counts
- experiment list: reads labs/ directory, shows active and graduated
- vm list: replaced virsh dependency with ~/vms/*.qcow2 scanner
  shows disk name, run state (via pgrep qemu), size on disk

## Gate
- [x] intent list works natively in fsh (no core prefix)
- [x] project list shows forest projects with status
- [x] experiment list shows labs/ entries
- [x] vm list shows active VMs (qcow2 scanner, no virsh)
- [ ] Tab completion for domain objects -- deferred to INT-040
- [ ] Colors reflect semantic state -- deferred to INT-033

## Deferred
- Tab completion (INT-040): NixOS-aware tab completion for domain objects
- Semantic colors (INT-033): Forest-aware color system for intent status

## The Rule
"The shell is the forest's voice.
 It speaks human first.
 UNIX is the fallback, not the interface." 🌲
