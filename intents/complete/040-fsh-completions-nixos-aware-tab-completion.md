---
id: 040
date: 2026-06-09
type: feature
title: "fsh-completions: tab completion for domain objects and NixOS vocabulary"
status: complete
tags: [completion, fsh, domains, nixos, tab, vocabulary]
priority: high
---
## Why
INT-030 established intent, project, vm, experiment as first-class shell objects.
Tab completion must follow. A domain object without completion is half-built.
The shell should know what you can do before you finish typing it.

## Depends On
- INT-030 (fsh semantic domains) -- domain objects must exist before completing them

## What Already Exists
completion.rs already handles:
  - intent show <TAB> -- dynamic intent ID completion (all three dirs)
  - cistart/cicomplete <TAB> -- intent ID completion
  - core intent <TAB> -- subcommand completion
  - git branch completion
  - path completion (~/ expansion)
  - binary completion ($PATH scan)

## What This Intent Adds

1. Native domain verb completion
   intent <TAB>        -- show, list, search, new, edit
   intent show <TAB>   -- all intent IDs (already works, verify)
   intent list <TAB>   -- (no args needed, no-op)
   vm <TAB>            -- list, start, stop, snapshot, restore
   vm start <TAB>      -- qcow2 names from ~/vms/
   vm restore <TAB>    -- snapshot names from qemu-img snapshot -l
   project <TAB>       -- list, status, health
   experiment <TAB>    -- list, new, graduate

2. NixOS-aware completion
   rebuild <TAB>           -- framework16 (from flake.nix hosts)
   nix develop <TAB>       -- flake outputs from flake.nix
   nixos-rebuild <TAB>     -- switch, boot, test, dry-run, dry-activate

3. fsh vocabulary completion
   Any word from fsh vocabulary (INT-261) completes with description
   delete <TAB>  -- shows: Delete(File) -- confirm required
   find <TAB>    -- shows: Find(File) -- safe

## Phases

Phase 1 -- Domain verb subcommand completion
  Add to completion.rs: intent, vm, project, experiment subcommands
  intent show <TAB> already works -- extend to all intent subcommands
  vm start/stop/snapshot/restore + qcow2 name completion
  Gate: all four domain verbs complete subcommands on TAB

Phase 2 -- NixOS-aware completion
  Parse flake.nix for nixosConfigurations keys (e.g. framework16)
  rebuild <TAB> completes with known host names
  nix develop <TAB> completes with devShell names from flake
  Gate: rebuild <TAB> shows framework16

Phase 3 -- fsh vocabulary completion
  Cross-reference semantic.rs verb list for completion hints
  Gate: all vocabulary verbs complete with semantic description

## Gates
- [x] intent <TAB> completes: show, list, search, new, edit
- [x] vm <TAB> completes: list, start, stop, snapshot, restore
- [x] vm start <TAB> completes with qcow2 names from ~/vms/
- [x] project <TAB> completes: list, status, health
- [x] experiment <TAB> completes: list, new, graduate
- [x] rebuild <TAB> completes with flake host names
- [x] nix develop <TAB> completes with devShell names
- [x] All completions tested in live fsh session

## The Rule
"If the shell knows the object exists,
 it should know what you can do with it.
 TAB is the forest speaking back." 🌲
