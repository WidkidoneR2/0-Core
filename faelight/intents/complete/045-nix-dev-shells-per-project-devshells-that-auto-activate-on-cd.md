---
id: 045
date: 2026-06-09
type: feature
title: "nix-dev-shells: per-project devShells that auto-activate on cd"
status: complete
tags: [nixos, devshell, direnv, fsh, nix, development]
priority: medium
---
## Why
Right now building faelight-shell requires:
  nix develop --command cargo build -p faelight-shell

That is friction. Every project that needs specific tooling
should activate its environment automatically when you cd into it.

The forest philosophy: the environment follows you.
You do not chase the environment.

## Vision
  cd ~/0-core              -- faelight devShell activates automatically
  cd ~/0-core/rust-tools   -- same devShell, same tools
  cd ~/projects/other      -- different devShell, different tools
  cd ~                     -- devShell deactivates, clean environment

No manual `nix develop`. No forgotten env vars.
The shell knows where you are and what you need.

## What Already Exists
flake.nix already has devShells.default (friday-dev shell).
fsh already detects IN_NIX_SHELL and shows ❄ indicator in prompt.
nix develop --command works manually.
direnv exists as a NixOS option but is not configured.

## Approach
Two valid strategies -- choose one:

OPTION A -- direnv + nix-direnv (simpler, proven)
  Add direnv to NixOS config
  Add nix-direnv for fast nix flake integration
  Add .envrc files to each project root
  fsh respects direnv activation natively
  Pros: battle-tested, fast with nix-direnv caching
  Cons: requires .envrc in each project, external dependency

OPTION B -- fsh-native cd hook (forest-native)
  fsh intercepts cd commands
  Checks for flake.nix in new directory
  Auto-runs nix develop if found
  No direnv dependency
  Pros: pure forest, no external tools
  Cons: slower (no caching), more complex

Recommended: OPTION A -- direnv + nix-direnv
  direnv is declarative, NixOS-native, well-tested.
  nix-direnv caches shells so activation is instant after first run.
  .envrc is a single line: use flake

## Phases

Phase 1 -- NixOS direnv config
  Add programs.direnv.enable = true to flake.nix
  Add nix-direnv for cached flake shells
  Gate: direnv available in PATH after rebuild

Phase 2 -- Project .envrc files
  Add .envrc to ~/0-core: use flake
  Add .envrc to any other forest projects
  Gate: cd ~/0-core auto-activates friday-dev shell

Phase 3 -- fsh prompt integration
  fsh already shows ❄ for IN_NIX_SHELL
  Verify ❄ appears on auto-activation
  Add devshell name to prompt context line
  Gate: prompt shows ❄ friday-dev when in 0-core

Phase 4 -- fsh vocabulary
  devshell list  -- show available devShells in current flake
  devshell enter -- manually enter a named devShell
  devshell exit  -- exit current devShell
  Gate: devshell commands work in fsh

## Gates
- [x] programs.direnv.enable = true in NixOS flake
- [x] nix-direnv configured for cached flake shells
- [x] .envrc present in ~/0-core with: use flake
- [x] cd ~/0-core auto-activates friday-dev shell
- [x] cd ~ deactivates the devShell cleanly
- [x] fsh prompt shows ❄ and devShell name on activation
- [x] devshell list shows available shells in current flake
- [x] Second cd into same dir is instant (nix-direnv cache)

## Depends On
- INT-040 (fsh-completions) -- devshell tab completion

## The Rule
"The environment follows the developer.
 You should never have to remember which shell
 a project needs -- the forest knows." 🌲
