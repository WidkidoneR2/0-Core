---
id: 140
date: 2026-07-11
type: future
title: "friendly deploy-error messages"
status: planned
tags: [messages, errors, deploy]
---

## Vision
Deploy output currently shows every warning with equal alarm. A benign "Git tree is dirty"
warning looks just as scary as a failed system activation. There is no signal for "this is
fine, ignore it" vs "this needs a known one-line fix" vs "stop and look, something is really
broken." INT-140 triages deploy output into three severities -- like telling the sniffles
from a cold from a heart attack -- so at a glance you know whether to ignore, apply a known
fix, or investigate.

## The Problem
The raw `nixos-rebuild` cascade mixes benign noise, easily-fixed errors, and genuine failures
into one wall of red/yellow text. Every deploy this project prints, unavoidably:
- `warning: Git tree ... is dirty` (benign -- always appears with uncommitted changes)
- `evaluation warning: buildDepsOnly will ignore src ...` (benign -- crane dep-split noise)
- post-rebuild health drop to ~90% ADVISORY (benign -- generation drift, clears on reboot)
These SNIFFLES look identical in tone to real problems (stale Cargo.lock, untracked flake
files) and to genuine HEART ATTACKS (activation failure, infinite recursion). The human has
to know from memory which is which. That memory is exactly what a tool should hold.

## The Solution
A severity classifier for deploy output, with three tiers:

- SNIFFLES (benign -- deploy is fine): recognized noise (dirty-tree, buildDepsOnly, the
  post-rebuild health advisory, the churn "declining health" forecast). Shown calmly or
  collapsed to a one-line "(benign: N informational warnings)".
- COLD (real, but a known fix): recognized failures with a known one-line remedy --
  stale Cargo.lock after adding a crate -> `cargo check --workspace` then retry; an
  untracked .rs/.nix the flake can't see -> `git add <file>` then retry; a Nix syntax
  error -> point at the file/line. Printed as a calm message WITH the exact fix.
- HEART ATTACK (serious -- stop and look): activation/switch failure, infinite recursion,
  evaluation aborted -- AND, critically, anything UNRECOGNIZED. Shown as the full raw error,
  flagged serious.

## Core principle (the honest part)
INT-140 only downgrades errors it POSITIVELY RECOGNIZES as benign -- the sniffles list is a
whitelist, not a catch-all. Anything it does not recognize is treated as SERIOUS by default
and passed through raw. It never silences a mystery. A wrong "this is fine" is far more
dangerous than a false alarm, so the tool errs toward alarm on the unknown.

## Approach (phased, demonstrated not declared)
Recon the deploy script first (faelight/packages/faelight/scripts/deploy) -- find where it
runs nixos-rebuild and where output is printed. Then: P1 capture rebuild output + a signature
table (sniffles/cold patterns). P2 classify + render the three tiers. P3 wire into the real
deploy flow. Each phase demonstrated on a real deploy (trigger a known cold to prove it).

## Success Criteria
- [ ] Triage classifier recognizes the known SNIFFLES (dirty-tree warning, buildDepsOnly note, post-rebuild health advisory) and marks them benign -- shown calmly or collapsed, not alarming
- [ ] Recognizes the known COLDS (stale Cargo.lock after a new crate; untracked .rs/.nix flake file) and prints the exact one-line fix, not the raw cascade
- [ ] Unrecognized errors pass through RAW and are flagged serious -- 140 never silences an error it does not positively recognize (whitelist, not catch-all)
- [ ] Wired into the actual deploy flow (faelight/packages/faelight/scripts/deploy) -- triage runs on a real `dep`, demonstrated live
- [ ] Demonstrated end to end: deliberately trigger a known COLD (e.g. add a crate without updating Cargo.lock) and show 140 catches it with the correct fix, then a clean deploy shows only benign sniffles

## Relates To
- INT-114/130 lessons: the Cargo.lock-stale and untracked-flake-file errors are the two COLDs
  hit most often this project -- they are the highest-value catches.
- faelight-shell exec layer already has some deploy-outcome awareness (exec.rs ~514); check
  whether triage belongs in the script, the shell, or a thin wrapper.
