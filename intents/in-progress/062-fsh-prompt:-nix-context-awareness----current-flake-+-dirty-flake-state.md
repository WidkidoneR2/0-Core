---
id: 062
date: 2026-06-14
type: feature
title: "fsh prompt: nix-context awareness -- current flake + dirty flake state"
status: in-progress
tags: [fsh, prompt, nix, flake, git, nix-native, lane-2, ux]
priority: medium
---

## Why
fsh already knows part of its Nix context -- the prompt shows the devShell
badge (❄ friday-dev) and git branch + dirty state (nixos*). But it shows the
*devshell*, not the *flake* it lives in, and it has no notion of whether the
flake itself is dirty. Lane 2 of the fsh Evolution Roadmap makes fsh think in
Nix; this is the first foundation piece -- the prompt becomes flake-aware.

When you are deep in a subdir, or inside a different flake project, the prompt
should tell you which flake you stand in. And when you have edited flake.nix or
flake.lock, the prompt should say so -- the same way it already flags a dirty
git tree.

## What Already Exists  (rust-tools/faelight-shell/src/prompt.rs)
- render_context (142) -- two-line context block above the input
  - line 1 (156): cwd + git (branch*) cluster
  - line 2 (288): health / intent / commits / friday-hint
- render_line (300) -- the input prompt; nix_indicator (311-327) renders ❄ + devshell name
- git_info (88-110) -- walk up for .git, read branch, git status --porcelain -> dirty bool
  - the "dirty git" half of item 2 is ALREADY DONE (rendered 158-166)
- status_line (355) -- dead code

So this intent adds only: a flake_info() helper, a flake-dirty signal, 2 render hooks.

## Vision
- prompt shows the nearest flake project name (walk-up), next to the ❄ badge
- prompt flags a dirty flake (flake.nix / flake.lock uncommitted) like it flags dirty git
- cheap -- no new subprocess; reuse the git status --porcelain call already run each prompt
- honest -- v1 "dirty flake" means exactly "flake.* has uncommitted changes"

## Approach
- flake_info() -- filesystem walk-up from cwd for flake.nix, mirroring git_info's .git walk;
  returns the flake root's project name, independent of IN_NIX_SHELL / DIRENV_DIR
- flake-dirty -- extend the existing porcelain output check to report whether
  flake.nix or flake.lock appear in the dirty set (no second subprocess)
- render -- current flake near the ❄ badge (render_line); dirty-flake marker
  beside the git (branch*) cluster (render_context line 1)
- colors -- reuse INT-033 semantic tokens; no new hardcoded rgb

Deliberately NOT in v1: system-vs-source drift (running gen vs current flake rev).
That is the meaningful-but-pricier signal -- staged as Phase 3.

## Phases
Phase 1 -- flake_info() + current-flake render
  Gate: prompt shows the current flake name when inside a flake project
  Gate: nothing extra shown when not in a flake project (no false badge)

Phase 2 -- dirty-flake signal (cheap version)
  Gate: editing flake.nix/flake.lock (uncommitted) lights the dirty-flake marker
  Gate: committing it clears the marker
  Gate: no new subprocess added (verified -- reuses git_info's porcelain call)

Phase 3 -- system-vs-source drift (v2, stage later, optional)
  Gate: live system behind source -> drift marker shows
  Gate: rebuild clears it

## Gates
- [ ] flake_info() walk-up helper added (mirrors git_info)
- [ ] prompt shows current flake name inside a flake project
- [ ] no false flake badge outside a flake project
- [ ] dirty-flake detection reuses existing porcelain (no new subprocess)
- [ ] editing flake.nix/flake.lock lights the dirty-flake marker
- [ ] committing clears the dirty-flake marker
- [ ] colors use existing INT-033 semantic tokens (no new hardcoded rgb)
- [ ] builds clean: cargo build in the faelight-shell crate
- [ ] demonstrated live -- not just implemented

## Depends On
  none (self-contained prompt change)

## Lane
  fsh Evolution Roadmap -- Lane 2 (Nix-native), items 1 + 2

## The Rule
"The prompt should know which forest it stands in,
 and whether that forest has been disturbed.
 Show the flake. Flag the drift. Cheaply." 🌲
