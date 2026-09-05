---
id: 241
date: 2026-09-05
type: fix
title: "PWD is never updated so every script reading it gets the directory nsh was launched from"
status: planned
tags: [fix, bugfix]
---

## Vision
`$PWD` names the directory the shell is in. Anything reading it gets the same
answer `pwd` gives.

## The Problem
Found 2026-09-05 while running the test suite. `NSH_BIN=$PWD/target/debug/nsh`
sent the harness to `/home/christian/target/debug/nsh` -- the `0-core` segment
was missing.

Measured directly, in one nsh session:

```
cd ~/0-core
pwd            -> /home/christian/0-core
echo $PWD      -> /home/christian
cd /tmp
echo $PWD      -> /home/christian
```

**`$PWD` is not stale, it is never updated at all.** nsh inherits it from the
bash process that `exec`s it and never touches it again, so it reports the
directory bash was in at login for the entire life of the shell. `cd` does not
update it.

### Why this matters beyond one paste
POSIX requires the shell to set `PWD` on every `cd`, and bash does. Anything
that reads it in nsh gets a wrong answer:

- every script using `$PWD` to build a path
- every `$(...)` substitution that shells out and reads it
- prompt tooling and editor hooks that use it to find the project root
- ⚠️ **a second user's first script**, which is the INT-230 packaging case

📍 It also cost real time in this session: it silently redirected a test harness
to a binary that does not exist, and nsh-test correctly refused to fall back to
the deployed shell -- so the failure was loud, but only because that guard had
already been built.

## The Solution
Set `PWD` in the shell's own environment whenever the working directory
changes, so a child process and a `$PWD` expansion see the same value `pwd`
reports.

⚠️ **THE OWNER QUESTION FIRST.** `set_current_dir` appears at several sites in
`commands/mod.rs` (cd, z_jump and relatives). Setting `PWD` beside each of them
would create the same multi-owner shape this ledger keeps removing. The
directory change and the variable update must have ONE owner, so a caller
cannot move the shell without updating the variable.

📍 `OLDPWD` is the sibling question -- bash maintains it and `cd -` depends on
it. Decide whether it is in scope before building, not after.

## Success Criteria
- [ ] G1 RED FIRST: the divergence is captured verbatim before any fix --
      `pwd` and `echo $PWD` disagreeing, and `$PWD` unchanged across a `cd`
- [ ] G2 EVERY site that changes the working directory is ENUMERATED, not
      grepped for one pattern. The census names each one and what it is for
- [ ] G3 ONE OWNER: the directory change and the `PWD` update cannot be done
      separately. A caller that moves the shell cannot forget the variable
- [ ] G4 `echo $PWD` agrees with `pwd` after: cd, cd -, z_jump, a relative cd,
      a cd through a symlink, and shell startup
- [ ] G5 A CHILD PROCESS SEES IT: `printenv PWD` from a spawned command matches,
      because that is the case that broke the harness
- [ ] G6 Regression test in nsh-test asserting the agreement, driven through the
      REPL door rather than `-c` -- both doors if they can diverge
- [ ] G7 A ruling recorded on `OLDPWD`: implemented, or explicitly out of scope
      with the reason
- [ ] G8 each gate carries evidence per INT-158

## Non-goals
- `cd -` behaviour itself, unless G7 rules OLDPWD in scope.
- Symlink resolution policy (whether `PWD` is logical or physical). Match what
  `pwd` already reports; changing that is a separate decision.
