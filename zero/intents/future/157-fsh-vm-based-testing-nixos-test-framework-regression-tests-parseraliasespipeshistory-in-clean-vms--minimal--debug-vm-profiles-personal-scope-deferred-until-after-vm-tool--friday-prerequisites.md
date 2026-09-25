---
id: 157
date: 2026-07-13
type: future
title: "fsh regression tests in clean VMs -- parser, aliases, pipes, history"
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


## Mechanism: the NixOS test driver (the HOW -- added 2026-07-13)
Use the NixOS test framework (`nixosTest` / a flake `checks` target run by `nix flake check`),
NOT hand-rolled QEMU. The driver creates fresh VMs per run, boots them, runs a Python test
script that drives the guest, then destroys them. Reproducible, ephemeral, purpose-built.

Each test file under tests/ defines a complete NixOS VM + a Python script. Three parts:
1. Machine config -- a normal NixOS configuration (services, users, systemPackages incl. fsh).
2. Virtual hardware -- CPU/RAM/disk/NIC/EFI/bootloader/serial, all auto-provided by the driver.
3. Python test script -- drives the guest via `machine.*` objects.

Building blocks (the Python API):
- machine.start()                         -- boot the VM
- machine.wait_for_unit("multi-user.target") -- wait for boot complete
- machine.wait_for_open_port(22)          -- wait until a port is up
- machine.succeed("cmd")                  -- run in guest; FAIL the test if exit != 0
- machine.fail("cmd")                     -- expect NON-zero exit (test invalid input)
- machine.execute("cmd")                  -- run; returns (exit_code, stdout)
- machine.send_chars("help\n")            -- simulate a USER TYPING into the guest
- machine.send_key("ctrl-c")              -- simulate a KEYPRESS (ctrl-c/ctrl-d/tab/arrows/esc)
- machine.copy_from_host("config.fsh")    -- push a host file into the guest (config-load tests)

send_chars / send_key are why this fits a SHELL so well: you can drive fsh interactively and
assert on what it does -- the automated complement to INT-156's interactive `keys` tester.

## Staged rollout (build the suite incrementally -- small, real, grow by need)
- Stage 1 -- one VM: boots, starts fsh, verifies basic commands (fsh runs; `echo hello` -> hello).
- Stage 2 -- language behavior: parser, pipes (`echo hello | cat`), redirection, variables
  (`export TEST=123; echo $TEST` -> 123), history (run commands -> restart shell -> history
  persists), signals.
- Stage 3 -- keybindings via send_key: Ctrl+C, Ctrl+D, Ctrl+L, Tab completion, arrows, Esc.
  Directly complements INT-156 (`keys`): keys is interactive/manual, this AUTOMATES the same
  verification in a clean VM.
(Stages 4-5 -- Home Manager install tests, multi-VM SSH/networking -- remain OUT of scope per the
"Explicitly OUT" section above: distribution testing a personal shell does not need.)

## CRITICAL distinction: test driver is NOT the dev VM (do not conflate)
Two DIFFERENT VM systems for two DIFFERENT jobs -- keep them separate:
- INT-027 `vm` script = a PERSISTENT, stateful DEV VM: build once, boot, ssh in, work, snapshot.
  Long-lived, interactive. For DOING work (Friday experiments). Snapshots/perf/Rust-migration
  live there.
- INT-157 nixosTest = EPHEMERAL TEST VMs: created fresh per `nix flake check`, run assertions,
  destroyed. Short-lived, stateless, automated. For PROVING correctness.
The test driver does NOT replace the dev VM, and the dev VM should NOT try to become the test
driver. They are complementary. This intent (157) is the nixosTest side; 027 is the dev-VM side.
"Build a real VM not just a script" applies to 027's dev VM (Rust migration); testing correctly
uses the driver, which is script-defined NixOS tests by design -- that is the right tool, not a
lesser one.

## The Rule
"Prove it in a clean machine. Small, real tests that grow by need -- not a pipeline built for
users who do not exist." 🌲

## RESCOPED 2026-08-27 -- the want survives, the framework does not

FALSE PREMISE: the mechanism was the NixOS test framework -- declarative VMs
built from the flake, hermetic by construction. That is gone with the store.

WHAT SURVIVES, unchanged: fsh regression tests (parser, aliases, pipes, history)
in a CLEAN machine, so a case cannot pass by accident of this box. That want got
sharper during the migration, not weaker -- six fsh-test failures on Omarchy were
the HARNESS asserting one machine layout (/run/current-system paths, PATH
containing /nix), and a clean-VM suite is exactly what catches that class before
a migration does.

OPEN QUESTION, deliberately not answered here: what provides hermetic VMs on
Arch. The value of the NixOS framework was that the VM was DERIVED from the same
source as the system, so it could not drift. Nothing on Arch gives that for
free, and picking a runner (libvirt, cloud-hypervisor, systemd-nspawn) before
knowing what property is actually needed is how the wrong tool gets adopted.

NOTE: INT-027 (faelight-vm tooling) is COMPLETE, so the old prerequisite is
satisfied -- but it was built against Nix and needs its own check.

NOTE: FAELIGHT_STATE_DIR and FAELIGHT_STATE_DB already exist as isolation
overrides (INT-204). A clean HOME plus those two may deliver most of the
isolation without a VM at all, and that is the cheaper experiment to run first.
