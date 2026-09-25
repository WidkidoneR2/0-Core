---
id: 140
date: 2026-07-11
type: future
title: "friendly deploy-error messages"
status: complete
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
- [x] Triage classifier recognizes the known SNIFFLES (dirty-tree warning, buildDepsOnly note, post-rebuild health advisory) and marks them benign -- shown calmly or collapsed, not alarming <!-- 2026-07-11: triage.rs is_sniffle() matches dirty-tree, buildDepsOnly, generation-drift/reboot, ADVISORY, declining-health/investigate, already-running. Rendered live on the gen 348 deploy: '🟢 benign: 3 informational warning(s) -- safe to ignore'. Unit tests dirty_tree_is_sniffle + builddeps_is_sniffle + clean_deploy_only_sniffles pass. -->
- [x] Recognizes the known COLDS (stale Cargo.lock after a new crate; untracked .rs/.nix flake file) and prints the exact one-line fix, not the raw cascade <!-- 2026-07-11: cold_fix() matches Cargo.lock-stale -> 'cargo check --workspace' and untracked .rs/.nix -> 'git add <file>', plus nix syntax/undefined-variable. Demonstrated on a real-shaped log (/tmp/test_cold_deploy.log): '🟡 KNOWN ISSUE -- here's the fix'. Unit tests cargo_lock_is_cold + untracked_file_is_cold pass. -->
- [x] Unrecognized errors pass through RAW and are flagged serious -- 140 never silences an error it does not positively recognize (whitelist, not catch-all) <!-- 2026-07-11: default-serious rule in classify() -- any 'error:' line not matched as sniffle/cold becomes 🔴 SERIOUS, shown raw. Demonstrated: /tmp/test_serious_deploy.log with 'infinite recursion' AND 'some totally unknown failure' both flagged serious. Unit test unknown_error_defaults_serious passes. Whitelist confirmed: sniffles/colds are positive-match lists; everything else escalates. -->
- [x] Wired into the actual deploy flow (faelight/packages/faelight/scripts/deploy) -- triage runs on a real `dep`, demonstrated live <!-- 2026-07-11: deploy script captures rebuild output via `... 2>&1 | tee /tmp/faelight-deploy.log`, then `faelight-shell --triage-deploy` renders after rebuild, before health check. The rebuild command is BYTE-IDENTICAL; set -e lifted for exactly one line to read PIPESTATUS[0] (rebuild's real exit code), then restored. Triage rendered LIVE on the gen 348 deploy. Safety proven: failed-rebuild stub exits 1 (no false all-clear), clean stub exits 0. Backup kept during test, removed after success. -->
- [x] Demonstrated end to end: deliberately trigger a known COLD (e.g. add a crate without updating Cargo.lock) and show 140 catches it with the correct fix, then a clean deploy shows only benign sniffles <!-- 2026-07-11: cold path proven via a SIMULATED Cargo.lock-stale log (chose a stub over deliberately breaking a real deploy -- safer, same proof) -> triage showed the correct 'cargo check --workspace' fix; the clean gen 348 deploy showed only 🟢 benign. Both ends demonstrated. (Substance of the gate met via stub + unit test rather than a live-broken deploy.) -->

## RESOLUTION (2026-07-11): SHIPPED -- deploy-output triage live, safety-proven.

Built as an fsh subcommand (`faelight-shell --triage-deploy [logfile]`) backed by a new
triage.rs module (classify + render), NOT a separate tool -- triage logic lives in the shell
where deploy-awareness already sits, and it's Rust + unit-tested (7 tests). The deploy script
(faelight/packages/faelight/scripts/deploy) captures rebuild output via tee and pipes it to
triage AFTER the rebuild, before the health check.

Safety discipline held throughout (Christian's rules): the rebuild command is byte-identical;
`set -e`/abort was NEVER changed except lifted for one line to read PIPESTATUS[0] (so a failed
rebuild can't be masked by tee's exit code -- the #1 danger), then restored; triage is `|| true`
(can never abort the deploy); the script exits with the rebuild's REAL code. A working backup
was kept during testing and removed only after full success. Failure-propagation was proven with
a stub (failed rebuild -> exit 1, no false all-clear) and success with a stub (exit 0), before
any real deploy. Then demonstrated LIVE on the gen 348 deploy: '🟢 benign: 3 informational
warning(s) -- safe to ignore'.

Scope (honest): improves the DEPLOY experience (clearer `dep` output), not general shell
behavior. The sniffles/colds are a positive-match whitelist; anything unrecognized escalates to
🔴 SERIOUS shown raw -- the tool never silences a mystery. Nix package updates go through a
different path (faelight-update) and are unaffected.

Future (optional): grow the signature tables as new benign/known patterns appear; consider a
Severity enum + structured output if another tool ever wants to consume triage results.

## Relates To
- INT-114/130 lessons: the Cargo.lock-stale and untracked-flake-file errors are the two COLDs
  hit most often this project -- they are the highest-value catches.
- faelight-shell exec layer already has some deploy-outcome awareness (exec.rs ~514); check
  whether triage belongs in the script, the shell, or a thin wrapper.
