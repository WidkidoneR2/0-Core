---
id: 246
date: 2026-09-09
type: future
title: "the sandbox declares four restrictions and enforces one"
status: planned
tags: [devbox, sandbox, policy, enforcement]
---

## Vision

A policy that says `memory: 256MB` limits memory to 256MB. Every restriction a policy prints
before a command runs is a restriction that applies to it, and the ones that cannot be applied
are not printed as though they could.

## The Problem

`SandboxPolicy::restrictions()` prints to the user BEFORE their command executes:

    Policy:  untrusted
             network: isolated
             filesystem: read-only
             cpu: 60s limit
             memory: 256MB limit

Measured 2026-09-12: **one of those four is real.**

    allow_net           LIVE      drives unshare --net
    allow_fs_write      warning   prints a line, detects writes afterwards, blocks nothing
    max_cpu_seconds     nothing   read only to print itself
    max_memory_mb       nothing   read only to print itself

And the values are not accidents. Six policies in sandbox-policies.toml set memory deliberately
-- 128, 256, 512, 1024, 4096 -- so somebody thought about each one. Every one of them is
decoration. `restrictions()` was marked with DECLARED / NOT ENFORCED on 2026-09-12 so the header
stops lying, but marking a gap is not closing it.

⚠️ THIS IS THE THIRD LAYER OF THE SAME DEFECT IN ONE TOOL. The same session found seccomp
silently dropped whenever network isolation was on, and the unshare fallback running with NO
isolation while the report claimed otherwise. Both are fixed. This one is the fields underneath
them.

## The Solution

### RULED: ONE MECHANISM, NOT FOUR HALF-MEASURES

Four fields enforced four different ways is how a tool ends up with four different failure modes
and a header nobody trusts. The policy engine gets ONE enforcement layer.

### RULED: CGROUP v2, AND NOT RLIMIT_AS

`RLIMIT_AS` is the easy answer and it is the wrong one. It limits ADDRESS SPACE, not resident
memory:

  - a Rust or JVM process reserving 1GB of address space while using 30MB resident DIES under
    `memory: 128MB`
  - a process that mmaps a large file and touches almost none of it DIES
  - a process that genuinely leaks resident memory under the cap SURVIVES

So the field would say "memory" and enforce something else. ⭐ A WRONG ENFORCEMENT IS WORSE THAN
AN ABSENT ONE, because it mostly works -- which means it is discovered by a confusing crash months
later rather than by reading the code. That is precisely the shape this whole intent exists to
remove, and adopting it here would be the fourth instance rather than the fix.

cgroup v2 `memory.max` limits resident memory, which is what the field NAMES. Arch runs systemd
with the unified hierarchy by default, so the machinery is present on this machine and on any
Omarchy install. `cpu.max` covers max_cpu_seconds by the same argument.

### RULED: allow_fs_write GOES TO BWRAP, NOT A FIFTH MECHANISM

bwrap is installed (`/usr/bin/bwrap`, confirmed 2026-09-06) and expresses read-only mounts
directly. A seccomp filter on `openat` is the alternative and it is a bad one: seccomp sees
pointers, not paths, so path matching is unreliable by construction.

⚠️ TWO MECHANISMS, NOT ONE, AND THAT IS THE HONEST ANSWER. cgroups govern RESOURCES; bwrap governs
the FILESYSTEM. They are different kinds of limit and one tool does not do both well. The ruling
above is against four ad-hoc half-measures, not against two mechanisms that each own a coherent
domain.

### RULED: A FIELD THE MECHANISM CANNOT ENFORCE IS DELETED

`allow_fs_read` was removed on 2026-09-12 under this rule: declared in the struct, set by NO
policy in the registry, read by nothing, and expressible by bwrap in a different shape anyway. It
was imagined and never used.

The rule generalises. A policy field is a promise; if nothing can keep it, it does not get to sit
in the struct looking like a control.

## Success Criteria

- [ ] Every field in SandboxPolicy is classified ENFORCED / DECLARED / DELETE, with the mechanism
      named for each enforced one. No field stays in the middle
- [ ] `max_memory_mb` is enforced by cgroup v2 memory.max. **Proven by watching it fail:** a
      policy with a small cap runs something that allocates past it and IS KILLED, and the report
      says the cap was the reason rather than reporting a bare non-zero exit
- [ ] `max_cpu_seconds` is enforced by cpu.max, or DELETED with the reason written here. A
      declared-forever field is a decision and needs to be made rather than inherited
- [ ] `allow_fs_write` is enforced by bwrap. **Proven by watching it fail:** a policy with
      allow_fs_write = false runs a command that writes, and the WRITE FAILS -- not "is detected
      afterwards"
- [ ] `restrictions()` loses its DECLARED / NOT ENFORCED markers, because there is nothing left to
      mark. If a marker survives, that field belongs in the DELETE column instead
- [ ] The cgroup is CLEANED UP when the run ends, including on a red run. INT-204 already records
      this trap for the test harness: clean on the path that always executes, not the happy one
- [ ] Enforcement failing is a DEGRADATION, not a warning. It joins the `degraded` list added
      2026-09-12 and can be named in a policy's `require` list, so a policy that must have its
      memory cap refuses rather than running without it
- [ ] What happens on a machine WITHOUT cgroup v2 is decided and written down. Refuse, degrade, or
      require -- silence is not one of the options

## Relationship

- Extends the degradation and `require` machinery added 2026-09-12 (policy.require,
  --allow-degraded, session.degraded). The limits become things that can be required
- INT-245 is the same defect in the value pipeline: a state that cannot be expressed becomes a
  value that means something else. Here it is a control that cannot be applied becoming a line
  that says it was
- INT-167's guardrail applies to the mechanism choice: do not reinvent what the system already
  provides. cgroups and bwrap are the system providing it
