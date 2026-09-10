---
id: 246
date: 2026-09-09
type: future
title: "the sandbox declares four restrictions and enforces one"
status: in-progress
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

### ⚠️ MEASURED 2026-09-12, BEFORE ANY CODE: memory.max ALONE IS NOT A MEMORY LIMIT

The ruling below was right about the mechanism and would still have shipped a cap that does not
cap. Two probes on this machine, same cgroup, same 32MB value, same 200MB allocation:

    memory.max = 33554432                          -> "ALLOCATED -- cap did NOT hold", rc=0
    memory.max = 33554432, memory.swap.max = 0     -> Killed, rc=137
                                                      memory.events: oom 1 oom_kill 1

cgroup v2 reclaims before it kills. With swap available, a process over `memory.max` is SWAPPED
rather than stopped, so the cap behaves as a swap threshold and the allocation succeeds.
`memory.swap.max = 0` is what makes it a memory limit.

⭐ THIS IS THE INTENT'S OWN THESIS ARRIVING FROM AN UNEXPECTED DOOR. RLIMIT_AS was rejected below
for enforcing the WRONG THING. This would have enforced the RIGHT THING and then not enforced it
-- a `memory: 256MB` line that prints, writes a real kernel file, and lets a 2GB process run.
Strictly worse than the decoration it replaced, because it would have looked verified.

RULED: `max_memory_mb` is TWO writes, not one. If the `memory.swap.max` write fails, the cap is
NOT enforced -- that is a DEGRADATION, it joins the degraded list, and a policy with
`require = ["memory"]` refuses rather than running under a limit that does not limit.

### ALSO MEASURED, and it decides the implementation

    /sys/fs/cgroup                     cgroup2fs
    root controllers                   cpuset cpu io memory hugetlb pids rdma misc dmem
    user@1000.service controllers      cpu memory pids     <- DELEGATED
    user@1000.service subtree_control  cpu memory pids     <- ENABLED FOR CHILDREN
    mkdir under user@1000.service      rc=0, rmdir rc=0    <- NO ROOT NEEDED

So DevBox writes the cgroup files DIRECTLY. `systemd-run --user --scope -p MemoryMax=` was the
fallback if delegation were absent; it is not needed here. Recorded because the fallback becomes
correct again on any machine where that mkdir fails, which is the same question as the
no-cgroup-v2 gate below.

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

- [x] Every field in SandboxPolicy is classified ENFORCED / DECLARED / DELETE, with the mechanism
      named for each enforced one. No field stays in the middle
- [x] `max_memory_mb` is enforced by cgroup v2 memory.max. **Proven by watching it fail:** a
      policy with a small cap runs something that allocates past it and IS KILLED, and the report
      says the cap was the reason rather than reporting a bare non-zero exit
- [x] `max_cpu_seconds` is enforced by cpu.max, or DELETED with the reason written here. A
      declared-forever field is a decision and needs to be made rather than inherited
- [x] `allow_fs_write` is enforced by bwrap. **Proven by watching it fail:** a policy with
      allow_fs_write = false runs a command that writes, and the WRITE FAILS -- not "is detected
      afterwards"
- [x] `restrictions()` loses its DECLARED / NOT ENFORCED markers, because there is nothing left to
      mark. If a marker survives, that field belongs in the DELETE column instead
- [x] The cgroup is CLEANED UP when the run ends, including on a red run. INT-204 already records
      this trap for the test harness: clean on the path that always executes, not the happy one
- [x] Enforcement failing is a DEGRADATION, not a warning. It joins the `degraded` list added
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

## What landed, 2026-09-12

    allow_net         ENFORCED  unshare --net
    max_memory_mb     ENFORCED  cgroup v2 memory.max + memory.swap.max=0   (961f84a1)
    allow_fs_write    ENFORCED  bwrap --ro-bind / / + one writable HOME    (83d6230a)
    max_cpu_seconds   DELETED   cpu.max is a quota, not a time limit       (fce202f9)
    allow_fs_read     DELETED   no policy ever set it
    emit_events       UNEXAMINED -- read only to print itself. Not checked this session.

Proofs, each by watching it fail:

    400MB under a 256MB cap  -> rc=137, report names the cap as the reason
    10MB under the same cap  -> rc=0, no degradation (the control)
    touch ~/zz under untrusted -> Read-only file system, file absent afterwards
    devbox test              -> 193/193, 2 skipped -- bwrap breaks nothing
    cgroup leftovers after 0/1/137 -> 0 every time (cleanup is in Drop)

Two wrong turns worth keeping:

- memory.max ALONE let a 200MB allocation through under a 32MB cap. cgroup v2 reclaims before
  it kills, so with swap available the cap is a swap threshold. memory.swap.max=0 makes it real.
- The first bwrap wiring bound the REAL HOME read-write when a policy set none, so the
  untrusted policy
  made /home/christian writable while claiming to block writes. Found by running it.
- The first cgroup wiring read cgroup.controllers instead of cgroup.subtree_control, creating
  the cap somewhere it could never apply. The degradation message located it.

## FOUND ALONGSIDE: the sandbox did not exit with its child

Running the untrusted policy against the command false reported Exit: 1 and returned rc=0.
The status was displayed
and thrown away, so no script, no and-chain, and no devbox test run could tell pass from
fail. Fixed in
2b611b70: true->0, false->1, OOM->137. Exit 3 stays reserved for a refusal.

## NEXT SESSION: the general-error message on a zero grep count

    cargo build ... | grep -cE "^warning"   ->  0,  then  x exited 1 -- general error

grep -c printed 0 and exited 1, because grep returns non-zero when it finds no matches. Zero
warnings is the result we wanted -- the exit 1 is grep reporting "nothing found", which is
success here. A grep -c as the last command in a chain will ALWAYS look like a failure when the
count is zero.

NOT a sandbox defect and not this intent's work -- recorded so it is not forgotten. The thing to
look at is nsh's MESSAGE: "general error" is what it says for any exit 1, and for grep that is a
well-known "no matches". Whether nsh should know that, or be less confident when it does not,
is a NovaShell question and needs its own home.
