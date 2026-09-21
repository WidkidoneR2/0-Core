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

## ⭐ PROVEN ON THIS MACHINE, 2026-09-20 -- one primitive, six dimensions

bubblewrap 0.12.0, already installed. Every line below was RUN, and each is a law tested by
attempting to violate it.

```text
  DIMENSION 2 -- PROCESS
    bwrap --unshare-all --dev-bind / / --proc /proc -- ps aux | wc -l
      4          against 411 on the host
    kill -0 <a host pid>
      "No such process"    -- NOT MERELY HIDDEN. Unreachable.

  DIMENSION 3 -- NETWORK
    curl -s -m2 https://github.com
      NET-BLOCKED          -- and only loopback exists inside

  DIMENSION 4 -- IDENTITY
    bwrap --unshare-all --uid 0 --gid 0 ...
      id -u  ->  0
      whoami ->  root
      touch /etc/oops-test  ->  Permission denied
    ★ THE LAW, ENFORCED: root inside has no root authority over the host. Software that
      expects root will run; it buys nothing outside.

  DIMENSION 6 -- HOSTNAME
    bwrap --unshare-uts --hostname devshell ...
      hostname -> devshell
    The sandbox can announce itself WITHOUT the shell being told.

  DIMENSION 1 -- FILESYSTEM, and this is the one that makes it a world
    bwrap ... --overlay-src /etc --overlay /tmp/ds-upper /tmp/ds-work /etc
      ls /etc | wc -l   ->  174     the REAL /etc, visible
      touch /etc/oops   ->  WROTE
      /tmp/ds-upper     ->  oops    the write landed in the upper layer
      /etc/oops on host ->  absent
    ★ THE INVARIANT FROM THE VISION, ENFORCED: inside, / looks real, but every mutation is
      disposable unless explicitly promoted.

  DIMENSION 8 -- OBSERVABILITY, for free
    THE UPPER DIRECTORY IS THE DIFF. Everything changed sits in one place, already
    enumerated. `devshell diff` is a directory listing, and `promote` is copying a file
    out of it. No tracking layer to write and none to drift.

  ⚠️ WHAT tmpfs COSTS, measured, because the obvious approach is worse:
    --tmpfs /etc  ->  writable, and ONE FILE IN IT. Nothing that reads config works.
    The overlay is why this is a world rather than an empty room.
```

## THE LAWS -- decided 2026-09-20, each against what bwrap can enforce

★ These are the laws of the world inside `devshell`. Each was chosen after measuring, not
before; each names the primitive; and each is proven by attempting to break it.

### LAW 1 -- FILESYSTEM. / looks real. Every mutation is disposable unless promoted.

`--dev-bind / /` for the view, `--overlay-src DIR --overlay UPPER WORK DIR` for each writable
region. The upper layer is the session. Nothing reaches the host without `promote`.

⚠️ NOT tmpfs. Measured: a tmpfs /etc has one file in it and nothing that reads config works.
A world you cannot use is not a world.

### LAW 2 -- PROCESS. You cannot see or signal anything outside.

`--unshare-pid --proc /proc`. THE SECOND HALF IS THE WHOLE TRICK: a PID namespace changes what
PIDs are, but ps reads /proc, so without a fresh proc the old view survives. That is exactly
the bug this intent found in the existing --isolate full.

### LAW 3 -- NETWORK. Nothing by default. The experiment asks.

`--unshare-net`. Chosen over "Internet but not the LAN" because the strict default is the one
that makes package installers and untrusted scripts interesting to test, and because a law you
can relax per-session is safer than one you must remember to tighten.

### LAW 4 -- IDENTITY. Root inside. No authority outside.

`--unshare-user --uid 0 --gid 0`. Proven: id says 0, whoami says root, and /etc refuses.

### LAW 5 -- ENVIRONMENT. Nothing crosses unless declared.

`--clearenv` plus explicit `--setenv`. The policy's existing `allow_env` list IS this law, and
it is already written -- the devshell profile names what crosses and nothing else does.

### LAW 6 -- HARDWARE AND IDENTITY OF THE MACHINE. The sandbox says what it is.

`--unshare-uts --hostname devshell`. Profiles name universes rather than flags: minimal, linux,
gui, gpu, network, hostile. Only the first two exist on day one.

### LAW 7 -- REVERSIBILITY. Leaving restores the prior state.

The upper layer is discarded on exit unless promoted. checkpoint is a copy of the upper layer;
rollback is restoring one; commit is promote applied to everything.

### LAW 8 -- OBSERVABILITY. What changed is enumerable, and checked against reality.

The upper directory IS the diff -- no tracking layer to write, none to drift. And the report is
verified against the real filesystem rather than trusted, which is the rule the doctor earned.

### LAW 9 -- SUDO IS NOT A DOOR. Christian, 2026-09-20.

The devshell does not touch sudo, its configuration, or its credential cache -- and nothing
inside can use it to become root outside.

⭐ MEASURED, AND IT ALREADY HOLDS FOR A GOOD REASON:

```text
    inside:  which sudo   ->  /usr/bin/sudo        the binary is visible
             sudo -n true ->  "/etc/sudo.conf is owned by uid 65534, should be 0"
    host:    /run/sudo    ->  permission denied
             sudo -n true ->  "a password is required"   no cached credential
```

★ THE USER NAMESPACE DEFEATS SUDO BY ITSELF. Host root maps to nobody inside, so sudo's own
ownership check on /etc/sudo.conf fails and it refuses to run. Law 4 protects Law 9 without
either being written for the other.

⚠️ SO THIS LAW IS "KEEP IT THAT WAY", AND IT IS THE ONE MOST EASILY BROKEN BY A CONVENIENCE.
Binding /run/sudo for some future reason, or dropping --unshare-user to make a tool work,
would hand the sandbox a route to real root. Any change that touches the user namespace
re-runs this law's test.

### LAW 0 -- THE ONE THE OTHERS REST ON

⚠️ A SANDBOX THAT CANNOT DELIVER A LAW REFUSES. Already shipped as `--allow-degraded`: "without
this, a sandbox that cannot deliver what it promised refuses rather than running the command
anyway and reporting success." Every law above inherits it. A namespace that could not be
created is a REFUSAL, never a quiet downgrade -- and never a printed claim, which is what
--isolate full was doing about pid isolation.

## ⭐ FIRST LIGHT, 2026-09-20 -- the world exists and was attacked

A 30-line bash script invoking bwrap with the ten laws. Entered, used, and deliberately abused:

```text
    devshell> whoami; hostname; ps aux | wc -l
    root
    devshell
    5

    devshell> touch /etc/i-was-here
    devshell> rm -rf ~/0-core/faelight/registry
    devshell> ls ~/0-core/faelight/
    RISK.toml engine intents meta policy rust-tools schema scripts     <- REGISTRY GONE
    devshell> exit

    HOST:
    ls faelight/registry/   ->  aliases.toml doctor packages.txt       <- INTACT
    ls /etc/i-was-here      ->  No such file or directory
```

★ THE REGISTRY WAS DELETED INSIDE AND IS UNHARMED OUTSIDE. Not a probe -- a real rm -rf on the
directory this project keeps its declarations in.

### And the upper layer really is the diff

```text
    upper-etc/i-was-here                          added
    upper-home/.bash_history                      modified
    upper-home/0-core/faelight/registry           c--------- 0,0
```

⚠️ A DELETION IS A WHITEOUT, NOT AN ABSENCE. overlayfs records `rm` as a CHARACTER DEVICE 0,0
in the upper layer, so `find -type f` misses it entirely -- the first listing showed two files
and hid the most destructive thing that happened.

★ SO `diff` HAS THREE CASES AND ONLY ONE IS OBVIOUS:
```text
    regular file      added or modified
    char dev 0,0      DELETED
    directory         traversed; may carry an opaque xattr when wholly replaced
```
A diff that lists only regular files would report a destroyed tree as two harmless changes.

## Law 8 built, and what it caught, 2026-09-20

`devshell-diff` against the session that attacked the tree:

```text
    M  ~/.cache/faelight/last-exit-status
    D  ~/0-core/faelight/rust-tools/novashell/src        <- the rm -rf
    M  ~/0-core/.git/index
    D  ~/.local/state/faelight/state.db-wal
    D  ~/.local/state/faelight/state.db-shm
    M  ~/.local/state/faelight/state.db
```

Host afterwards: 36 files present, `cargo build -p novashell` clean, `git status` EMPTY.

★ NOVASHELL'S ENTIRE SOURCE WAS DELETED FROM INSIDE A SHELL RUNNING FROM THAT SOURCE, AND THE
HOST NEVER NOTICED. Not a probe and not a fixture -- the real directory.

⚠️ AND THE DIFF CAUGHT SOMETHING NOBODY ASKED ABOUT: `M ~/0-core/.git/index`. A session that
ran three commands touched the git index. On the host that is a real modification to the
repository, and nothing would have reported it. THE SANDBOX IS ALREADY AN INSTRUMENT, not just
a shield.

### Resolved: what touched .git/index

A session running ONE command:

```text
    devshell -- bash -c 'cd ~/0-core && git status --porcelain > /dev/null'
    devshell-diff  ->  M  /home/christian/0-core/.git/index
```

`git status` WRITES THE INDEX when its cached stat data is stale: it refreshes and saves the
result. Inside a fresh overlay every file looks new to git, so it always refreshes.

⚠️ AND THE HOST TEST SAID OTHERWISE, WHICH IS WHY THIS NEEDED THE SANDBOX. Running
`git status --porcelain` on the host left the index mtime unchanged -- because git had already
refreshed it moments earlier and had nothing to write. The host measurement was not wrong, it
was UNINFORMATIVE, and it would have supported the conclusion that git does not write on read.

★ THE SANDBOX ANSWERED A QUESTION THE HOST COULD NOT, on its second day, about a tool nobody
was investigating.

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
