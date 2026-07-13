---
id: 157
date: 2026-07-13
type: future
title: "fsh VM-based testing: NixOS-test-framework regression tests (parser/aliases/pipes/history) in clean VMs + minimal & debug VM profiles. Personal-scope, deferred until after VM-tool + Friday prerequisites."
status: planned
tags: [fsh, testing, vm, nixos, regression, 027, friday]
---

## Vision
Use the NixOS testing framework to prove fsh's behavior in clean, reproducible VMs: boot a
fresh VM, run fsh, execute commands, compare output, pass/fail -- automatically, no manual
launching. Regression safety for fsh's real logic (parser, aliases, pipes, history), plus a
couple of purpose-built VM profiles for hidden-dependency detection and deep debugging.

## Why
As fsh grows -- especially under Friday work -- regressions get easy to introduce and hard to
catch by hand. A clean-VM test suite catches them automatically. And "does fsh work with almost
nothing installed?" is a real question fsh has failed before (banked lessons: stale-binary trap,
clean-shell interception, hidden PATH/env assumptions). A minimal VM surfaces those before they
bite. Filed now -- even though deferred -- because testing needs tend to appear unpredictably;
better to have a scoped home ready than to scramble mid-crisis.

## In scope (the valuable core -- personal shell)
- NixOS test framework driving fsh: VM boot -> run fsh -> execute commands -> compare output ->
  pass/fail. Exposed as a flake `checks` target so `nix flake check` runs it. Covers parser,
  aliases, pipes, history -- the behavior most prone to silent regression.
- MINIMAL VM profile: fsh with almost nothing installed (kernel + systemd + bash + fsh). Catches
  hidden dependencies / environment assumptions -- the exact bug class from the banked lessons.
- DEBUG VM profile: strace / ltrace / gdb / perf / valgrind / rr available, for deep-debugging
  fsh (and later, Friday components) in an isolated machine.

## Explicitly OUT of scope (do NOT chase -- fsh is a PERSONAL shell, not a distributed product)
- Home Manager INSTALL-testing VM, Release "new user" VM: these test a DISTRIBUTION path fsh
  does not have. No new users to catch problems before. Skip unless fsh ever goes public.
- Multi-VM SSH/SCP/DNS/firewall networking tests: that is testing networked services, not the
  shell. Out unless a concrete Friday need creates one.
- A "500+ tests" target: test COUNT follows real coverage needs, never a number-goal.
- Restructuring fsh into its own flake-repo (the standalone faelight-shell/ tree with its own
  flake.nix): fsh lives in the 0-Core MONOREPO at faelight/rust-tools/faelight-shell/. Keep it
  there. This intent adds tests to the monorepo, it does not extract fsh.

## Sequencing (guardrail -- this is NOT pre-Friday work)
Deferred until AFTER both:
  1. INT-027's VM-tool core work (snapshots, performance, organic Rust migration), and
  2. the Friday prerequisites are done.
Rationale: a VM test suite is infrastructure-for-later. Building a big test pipeline for a
personal shell BEFORE Friday would be a focus rabbit hole (Friday already nudges focus>speed).
This waits its turn. Start small when it does start (a handful of real regression tests), grow
by need.

## Success criteria (when it eventually starts)
- [ ] one NixOS-test check that boots a VM, runs fsh, executes a command, asserts output, exits
      pass/fail -- wired into `nix flake check`
- [ ] parser / alias / pipe / history each have at least one real regression test
- [ ] minimal VM profile: fsh runs with a near-empty environment; a hidden-dependency failure is
      demonstrably caught by it
- [ ] debug VM profile: the debug toolset is present and usable against a running fsh
- [ ] tests start small and real (NOT a count target); grow by actual coverage need

## Relationships
- Depends on INT-027 (faelight-vm tooling) -- needs the VM plumbing first; this builds ON it.
- Serves Friday: regression safety for when fsh grows under Friday work; the debug VM aids
  Friday-component debugging.
- Sibling context (2026-07-13): came from the same brainstorm as INT-156 (keys tester). fsh is
  Christian's PERSONAL shell -- the distribution-flavored testing ideas (HM/release/networking)
  were consciously fenced OUT above. Kept as a SEPARATE intent from 027 (not merged) because
  testing-fsh-behavior is distinct from the VM-tool plumbing, and future testing needs are
  unpredictable -- a separate home lets it grow without muddying 027.

## The Rule
"Prove it in a clean machine. Small, real tests that grow by need -- not a pipeline built for
users who do not exist." 🌲
