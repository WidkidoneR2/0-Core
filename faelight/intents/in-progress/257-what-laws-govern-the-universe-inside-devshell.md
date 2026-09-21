---
id: 257
date: 2026-09-20
type: future
title: "what laws govern the universe inside devshell"
status: in-progress
tags: [devbox, devshell, novashell, nsh, nova]
---

## Vision

A sandbox you LIVE IN. Not a harness you point at the shell -- a world you enter, work in, break,
and leave without having broken anything outside it.

★ THE SPECIFICATION IS A SENTENCE: inside the devshell I want X to be true, while outside it
remains impossible for the experiment to Y.

```text
INSIDE                              OUTSIDE MUST REMAIN IMPOSSIBLE
  I am root                           damaging the real filesystem
  / looks like a normal Arch system   killing host processes
  I can install packages              reading host credentials
  I can compile anything              reaching host network services
  I can kill processes                surviving my exit -- leaving restores the prior state
  I can change networking
  I can modify /etc
  I can experiment with my shell
```

⭐ THAT IS NOT A SHELL WRAPPER. It is a small Linux execution environment with the shell as its
interface, and the architectural decision is NOT "bubblewrap or namespaces or chroot". It is
WHAT LAWS GOVERN THE UNIVERSE INSIDE. Write the laws down and the primitives that enforce them
become obvious.

## The three questions every law answers

Inside the devshell: what must be IMPOSSIBLE, what must be REVERSIBLE, what must be OBSERVABLE.

## What already exists -- measured 2026-09-14 (INT-249)

This is not a greenfield. `faelight-sandbox` is 1,973 lines and in daily use:

```text
    run          --policy, --net-off, --isolate, --profile, --allow-degraded
    snapshot     reflink snapshots of a directory
    restore      from a snapshot
    diff         what changed in the last session
    history      last 10 runs
    audit        the trail, from state.db
    policy-list  default, untrusted, network-tool, build, strict, devbox, hardened
```

cgroups, seccomp, policies and reflink snapshots are BUILT. `devbox test` snapshots 1,132 files
and reports no changes, six times in one session.

⭐ AND THE HARDEST DECISION IS ALREADY MADE, IN THE INTERFACE. `--allow-degraded` reads: "without
this, a sandbox that cannot deliver what it promised REFUSES rather than running the command
anyway and reporting success." That is the law behind all the others.

## RECON, 2026-09-20 -- what is real, measured by violating it

### The sandbox is more honest than it is complete, and that is the good news

```text
    network      REAL      drives unshare --net
    memory       REAL      cgroup v2 memory.max, swap.max=0
    environment  REAL      allow_env is already a capability allowlist
    fs_write     DECLARED  and policy.rs SAYS SO: "Marking them is not a substitute for
                           enforcing them -- it is what makes the gap visible instead of
                           letting the header do the lying."
    process      BROKEN    see below
    identity     PARTIAL   --map-root-user is passed
    hardware     ABSENT
    promote      ABSENT
```

### ⚠️ PID ISOLATION IS CLAIMED AND DOES NOT WORK

`--isolate full` prints "pid: isolated". Measured:

```text
    host                                              411 processes
    faelight-sandbox run --isolate full -- ps aux     415
```

⭐ THE CAUSE IS ONE MISSING STEP, NOT A WRONG DESIGN. main.rs:989 passes `--pid --fork` and
:993 passes `--mount`, which is right -- but a PID namespace changes what PIDs ARE, while `ps`
reads /proc, and /proc is still the host's. Without a fresh proc mounted inside the new mount
namespace, ps walks the host tree regardless.

Both fixes measured:

```text
    unshare --net --map-root-user --pid --fork --mount --
        sh -c 'mount -t proc proc /proc; ps aux | wc -l'        4
    bwrap --unshare-all --dev-bind / / --proc /proc --
        sh -c 'ps aux | wc -l'                                  5
```

★ bubblewrap 0.12.0 IS INSTALLED, and policy.rs already notes it: "bwrap could do this properly
and is installed; nobody has wired it." It mounts /proc as part of namespace setup, which is
exactly the step unshare leaves to the caller.

### ⚠️ AND TWO LAWS ALREADY CONFLICT

```text
    seccomp: not applied because network isolation is on
    (unresolved: the filter may block the syscalls unshare needs)
```

`--isolate full` gives namespaces OR seccomp, never both, and says so at run time. A law that
cannot coexist with another law is a design decision this intent has to make, not a bug.

### ⚠️ AND THE NAMESPACE BLOCK ONLY RUNS IF THE NETWORK IS ISOLATED

main.rs:985: the whole unshare path is inside `if network_isolated`. So `--isolate full` on a
policy with `allow_net = true` gets NO namespaces at all -- pid, mount and user isolation are
silently conditional on a network decision that has nothing to do with them.

## The eight dimensions -- each one a law to be chosen, not inherited

### 1. Filesystem reality

Options: a separate root; a normal filesystem with overlays; the real HOME with writes
redirected; a synthetic home holding only what is exposed; copy-on-write over the host.

★ THE INVARIANT WORTH REACHING FOR: inside, `/` LOOKS REAL, BUT EVERY MUTATION IS DISPOSABLE
UNLESS EXPLICITLY PROMOTED. That is a different mental model from a wrapper -- it makes the
sandbox transactional rather than merely separate.

### 2. Process reality

A process started inside CANNOT DISCOVER OR SIGNAL processes outside its namespace. That law
pushes toward Linux namespaces rather than shell-level tricks: PID, mount, IPC, UTS, cgroup,
user. Not all of them are needed on day one, and choosing which is the work.

### 3. Network reality

Full host networking / a namespace with Internet / a namespace with none / an allowlist / a
synthetic local network. Two candidate laws:

```text
    the devshell reaches the Internet but NOT the host LAN or localhost
    networking does not exist unless the experiment asks for it
```

The second makes package installers, build systems and untrusted scripts genuinely testable.

### 4. Identity reality

A user namespace so that apparent root inside is not host root.

★ ROOT INSIDE HAS NO ROOT AUTHORITY OVER THE HOST. That is what lets software expecting root be
tested without being handed the machine.

### 5. Environment reality

PATH HOME USER SHELL TERM LANG XDG_* SSH_* GPG_* DISPLAY WAYLAND_DISPLAY DBUS_* -- each decided
ONE VARIABLE AT A TIME.

⭐ NOTHING CROSSES IN UNLESS DECLARED SHAREABLE. That turns the devshell from an accidental
inheritance of the user environment into a CAPABILITY BOUNDARY, which is the same shape the
capability system already uses elsewhere in this project.

### 6. Time and hardware reality

/dev, GPU, USB, audio, camera, clocks, hostname, kernel info, CPU topology, memory. Profiles
name universes rather than flags:

```text
    devshell minimal    devshell linux    devshell gui
    devshell gpu        devshell network  devshell hostile
```

### 7. Reversibility -- probably the most important property

```text
    $ devshell
    dev> touch /etc/oops
    dev> pacman -S something
    dev> rm -rf ~/project
    dev> exit
    $ git status      # host completely unchanged
```

With an explicit escape hatch -- `promote /etc/my-shell.conf` or `devshell commit` -- the
sandbox stops being isolation and becomes A TRANSACTION: start, diff, checkpoint, rollback,
commit, discard.

### 8. Observability -- first class, not an afterthought

```text
    dev> rm -rf build/
    devshell diff
      M  /home/me/project/foo.c
      A  /tmp/test-output
      D  /home/me/build/cache
```

Recording processes created, files modified and opened, network connections, capabilities
requested, syscalls. THAT IS A LABORATORY RATHER THAN A CONTAINER.

## What it looks like to use

```text
    $ enter experiment/foo
    foo> status
    filesystem: clean    network: restricted    identity: isolated
    processes: isolated  changes: 14
    foo> checkpoint alpha
    foo> run ./dangerous-test
    foo> diff
      + /etc/foo.conf
      ~ /home/dev/project/build/
      + process: foo-worker
    foo> rollback alpha
    foo> promote /home/dev/project
```

⭐ THE DEVSHELL IS AN OBJECT THE SHELL UNDERSTANDS, not a command that launches another process.

## Success Criteria

⭐ EVERY LAW IS PROVEN BY VIOLATING IT. A law nobody tested is a wish. Each gate below is
demonstrated by ATTEMPTING the forbidden thing from inside and recording what happened.

- [ ] THE LAWS ARE WRITTEN DOWN BEFORE ANY PRIMITIVE IS CHOSEN. Each of the eight dimensions
      gets a decided law with a reason, including the ones deliberately left permissive.
- [ ] The census first: what faelight-sandbox already enforces, per dimension, measured rather
      than assumed. This intent EXTENDS a working sandbox; it does not rebuild one.
- [ ] ⭐ PROCESS: from inside, `kill` a host PID and `ps` for host processes. Both must fail,
      and the failure output is recorded here.
- [ ] ⭐ FILESYSTEM: from inside, write to a real file outside the sandbox, then exit and
      confirm from the host that it is unchanged. The host check is the evidence, not the
      sandbox's own report.
- [ ] ⭐ NETWORK: from inside, reach whatever the chosen law forbids -- localhost, the LAN, or
      the Internet -- and record the refusal.
- [ ] ⭐ IDENTITY: from inside as root, attempt something only host root could do, and record
      that it fails.
- [ ] ENVIRONMENT: the variables that cross are a DECLARED LIST, and a variable not on it is
      absent inside. Proven by printing the environment in there.
- [ ] REVERSIBILITY: the session in the Vision runs end to end -- touch /etc, install a
      package, rm -rf a project, exit -- and `git status` on the host is clean.
- [ ] PROMOTION EXISTS AND IS EXPLICIT. Something survives only because it was named, and the
      diff before and after shows exactly that one thing.
- [ ] OBSERVABILITY: `diff` reports what changed, and its report is checked against the real
      filesystem rather than trusted.
- [ ] ⚠️ A SANDBOX THAT CANNOT DELIVER ITS LAWS REFUSES. The --allow-degraded rule already
      shipped for the existing policies extends to every new law here: a namespace that could
      not be created is a REFUSAL, never a quiet downgrade.
- [ ] nsh-test green, and the suite gains a case for at least one law.

## Not in scope

Replacing `devbox test` or `devbox shell` -- they work and they stay. INT-249's `devbox verify`
is a different intent: a case is a file, and this is a world you sit in.

## Relationship

Filed 2026-09-20 after INT-255 retired the Nix `devshell` -- which listed and entered flake
devShells, a thing with no Arch counterpart. THIS IS NOT ITS REPLACEMENT. The old one answered
"which toolchain does this project want", already covered here by mise and rustup. This one
answers a question NovaShell development actually has: WHERE CAN I BREAK THE SHELL WITHOUT
BREAKING THE SESSION I AM BREAKING IT FROM.
