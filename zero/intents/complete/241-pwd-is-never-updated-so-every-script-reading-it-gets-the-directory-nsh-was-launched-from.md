---
id: 241
date: 2026-09-05
type: fix
title: "PWD is never updated so every script reading it gets the directory nsh was launched from"
status: complete
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
- [x] G1 RED FIRST: the divergence is captured verbatim before any fix --
      `pwd` and `echo $PWD` disagreeing, and `$PWD` unchanged across a `cd`
<!-- evidence: 2026-09-23, interactive session. cd ~/0-core -> pwd said /home/christian/0-core and
     echo $PWD said /home/christian. AND THE -c DOOR AGREED WITH BASH THE WHOLE TIME: nsh -c
     "printenv PWD; pwd" in /tmp printed /tmp twice, identical to bash -c. The bug was the REPL's
     alone, which is why G6 drives that door. -->
- [x] G2 EVERY site that changes the working directory is ENUMERATED, not
      grepped for one pattern. The census names each one and what it is for
<!-- evidence: TEN sites, not nine. commands/mod.rs 2015 session restore, 8444 cd, 8911 z home,
     8931 z query, 14035 crate jump, 14614 scope enter, 14676 scope leave; engine.rs 2199 the cwd
     adopted after yazi exits; main.rs 2333 the forest-home default and 3604 last-directory
     restore. ⚠️ THE TENTH WAS FOUND ONLY BY RE-RUNNING THE CENSUS WITHOUT `head` -- the first
     pass was truncated at nine and the patch failed its anchor check rather than writing a
     partial migration. Proven live after ship: the reloaded shell started in the restored
     directory, so 3604 is not dead code. -->
- [x] G3 ONE OWNER: the directory change and the `PWD` update cannot be done
      separately. A caller that moves the shell cannot forget the variable
<!-- evidence: novashell/src/cwd.rs, one function, chdir<P: AsRef<Path>>. Its signature matches
     std::env::set_current_dir so each call site swapped one for the other without touching its
     error handling. MEASURED AFTER THE MIGRATION: zero occurrences of env::set_current_dir remain
     in commands/mod.rs, engine.rs and main.rs. -->
- [x] G4 `echo $PWD` agrees with `pwd` after: cd, cd -, z_jump, a relative cd,
      a cd through a symlink, and shell startup
<!-- evidence: driven through the REPL against the new binary, 2026-09-23:
       cd /tmp        pwd=/tmp                      var=/tmp
       cd ~/0-core    pwd=/home/christian/0-core    var=same
       cd faelight/scripts (RELATIVE)               var=same
       startup, no cd at all                        var=same
       through a symlink (~/.local/state/zero)      var=same
     ⚠️ SYMLINKS, RECORDED NOT FIXED: nsh reports the PHYSICAL path (.../state/faelight) where bash
     reports the LOGICAL one (.../state/zero). PRE-EXISTING -- nsh's own pwd already resolved
     symlinks, and cwd.rs reads current_dir() back so PWD matches whatever pwd says. The Non-goals
     rule this out of scope; POSIX logical-vs-physical is its own question.
     `cd -` is not implemented in nsh, so it is not in this list -- see G7. -->
- [x] G5 A CHILD PROCESS SEES IT: `printenv PWD` from a spawned command matches,
      because that is the case that broke the harness
<!-- evidence: `cd /tmp` then `printenv PWD` -> /tmp through the REPL on the new binary. On the
     OLD binary the same line returned /home/christian/0-core. -->
- [x] G6 Regression test in nsh-test asserting the agreement, driven through the
      REPL door rather than `-c` -- both doors if they can diverge
<!-- evidence: repl_241_pwd_follows_the_working_directory. PROVEN TO HAVE TEETH: green against the
     new binary (199/199) and RED against the deployed old one, with the failure message naming the
     bug itself -- expected "AGREE pwd=/tmp var=/home/christian/0-core child=/home/christian/0-core"
     to contain "AGREE pwd=/tmp var=/tmp child=/tmp".
     ⭐ THREE CHOICES IN THE CASE, EACH BECAUSE THE OBVIOUS VERSION PASSES ON THE BROKEN SHELL:
     the REPL door (a -c test agreed with bash all along); `cd /tmp` rather than `cd ~/0-core`
     (measured: on the broken binary cd ~/0-core AGREED, because the stale value happened to equal
     the destination); and a child process reading it.
     ⚠️ AND A HARNESS FINDING, worth more than this case: run_repl_lines_status returns the output
     of the LAST command only. The first version asserted across two echo lines and failed against
     BOTH binaries. Any existing case that submits several commands and asserts on earlier output
     is checking less than it appears to. -->
- [x] G7 A ruling recorded on `OLDPWD`: implemented, or explicitly out of scope
      with the reason
<!-- evidence: OUT OF SCOPE, Christian 2026-09-23. OLDPWD exists to serve `cd -`, and nsh has no
     `cd -`: commands/mod.rs dispatches "cd" => cd(args) and the function has no dash handling,
     no OLDPWD and no prev_dir anywhere in the crate. Adding the variable would add one nothing
     reads. If `cd -` is wanted it is its own intent, and cwd.rs is now the obvious owner. -->
- [x] G8 each gate carries evidence per INT-158
<!-- evidence: every gate above carries what was measured. Two things beyond the gates:
     THE FIX IS FOUR LINES AND THE CENSUS WAS THE WORK. cwd.rs is chdir + set_var; finding all ten
     callers, and proving the test could go red, took the session.
     AND THE FIRST PATCH REFUSED RATHER THAN HALF-APPLYING. Indentation guessed from `sed` output
     did not match the file, the anchor check failed, and nothing was written -- so the rebuild
     edited by verified line number instead. INT-215's rule, again: an anchor must be a complete
     line, never a prefix of one. -->

## Non-goals
- `cd -` behaviour itself, unless G7 rules OLDPWD in scope.
- Symlink resolution policy (whether `PWD` is logical or physical). Match what
  `pwd` already reports; changing that is a separate decision.
