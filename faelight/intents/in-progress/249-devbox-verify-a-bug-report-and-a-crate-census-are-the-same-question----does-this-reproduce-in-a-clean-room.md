---
id: 249
date: 2026-09-14
type: future
title: "devbox verify: a bug report and a crate census are the same question -- does this reproduce in a clean room"
status: in-progress
tags: [devbox, sandbox, testing, reproduction, census, 247, 167]
---

## Vision

One mechanism answers two questions that turned out to be the same one:

    "Does this crate work on a machine that is not mine?"      -- the INT-247 census
    "Does this bug reproduce, and does my fix hold?"           -- every debugging session

Both are: **take a clean room, state what should happen, run it, and believe the answer.** The
clean room already exists. What is missing is a way to WRITE DOWN the second half.

## Verify-first -- what faelight-sandbox ALREADY does, measured 2026-09-14

    run          with --policy, --net-off, --isolate, --profile, --allow-degraded
    snapshot     reflink snapshots of a directory
    restore      from a snapshot
    diff         what changed in the last session
    history      last 10 runs
    audit        the trail, from state.db
    policy-list  seven policies: default, untrusted, network-tool, build, strict,
                 devbox, hardened

1,973 lines across four files. cgroups, seccomp, policies and reflink snapshots are BUILT. Proven
in daily use: `devbox test` snapshots 1,132 files, hides the forest, runs nsh-test, and reports
"no file changes detected" -- six times in one session on 2026-09-14.

⭐ AND THE HARDEST PART IS ALREADY DECIDED, IN THE INTERFACE. `--allow-degraded` reads:

    "Without this, a sandbox that cannot deliver what it promised refuses rather than running
     the command anyway and reporting success."

That is INT-246's ruling, shipped. This intent does not INVENT the undetermined outcome -- it
EXPOSES an existing one to a file.

## The Problem

A reproduction lives in a person's head or in a scrollback buffer. Today's session is the
argument, three times over:

    the -c orphan       "does a stopped child survive?"   -> hand-written pty probe
    the watch leak      "is SIGINT still caught after?"   -> hand-written /proc probe
    diff -> difft       "is the status really dropped?"   -> six shell invocations

Each was answered, none was KEPT. The next session re-derives them, and a fix that quietly
breaks one of them is caught by nobody.

And the census has the same shape: 29 crates, and "does it run in a clean room" asked once per
crate, by hand, is a day nobody will spend twice.

## The Solution -- A CASE IS A FILE, AND THE VERB IS `verify`

### ⭐ RULE 1: THE FORMAT IS DECIDED BY WHAT `run` ALREADY TAKES

Every field maps to a flag that exists today, or it does not go in v1. A case file that can
express something the sandbox cannot do is a promise the tool will break.

    # devbox/cases/188-stopped-child-is-not-orphaned.toml
    description = "a -c shell's stopped child is not left orphaned"
    policy      = "devbox"          # -> --policy
    net         = "off"             # -> --net-off
    command     = "nsh -c 'sleep 30'"

    [expect]
    exit            = 0
    stdout_contains = "..."
    no_process      = "sleep"

⚠️ WHAT IS NOT IN v1, deliberately: no setup scripts, no fixtures directory, no templating, no
case dependencies, no parallelism. Each is a real feature and each can be added once a case file
exists that NEEDS it. Adding them first is designing for imagined cases instead of the nine real
ones this session produced.

### ⭐ RULE 2: THREE OUTCOMES, NEVER TWO

    PASS           ran, and the expectation held
    FAIL           ran, and it did not
    UNDETERMINED   the question could not be ASKED

The third is the whole point, and it is this project's oldest lesson: INT-192 (a check that could
not run reported clean), INT-245 (a pipeline that could not compute returned empty), INT-246 (a
sandbox that could not enforce reported success). A case whose precondition is absent is not
green and it is not red.

nsh-test already does this and says so in words:

    pick_without_fzf_names_the_dependency -- needs a real 0-Core
    repl_206_forest_home_is_still_the_default -- needs a real 0-Core

Two of 195, reported as "a precondition was absent, not a failure". `verify` reports the same way
or it is worse than what exists.

⚠️ AND UNDETERMINED MUST CARRY ITS REASON. "Skipped" is not a result. "Needs bwrap, which is not
installed" is.

### ⭐ RULE 3: THE CENSUS IS THE SAME MECHANISM, NOT A SECOND ONE

A crate's census entry is a case file:

    description = "faelight-git runs on a machine that is not the author's"
    policy      = "devbox"
    command     = "faelight-git --version"
    [expect]
    exit = 0

If the census needs different machinery than a bug report, the design is wrong and this rule is
how that gets caught early rather than after both exist.

## Non-goals -- each with the reason

  - ⛔ NO TUI IN v1. INT-167's vision has one and it can have one later. A file format and a verb
    are the thing with a caller today; a TUI is a way to look at results that do not exist yet.
  - ⛔ NOT A TEST FRAMEWORK. nsh-test tests the SHELL, in-process, fast, 193 cases. `verify` runs
    whole commands in a clean room and is slow by construction. If a case can be an nsh-test
    case, it should be one.
  - ⛔ NO CASE DEPENDENCIES OR ORDERING. A case that needs another case to have run first is
    stateful, and the clean room exists to remove exactly that.
  - ⛔ DO NOT INSTRUMENT THE SHELL FOR THIS. 167 imagined an instrumentation API feeding the
    debugger. `observe.rs` (INT-207) already exists and `verify` observes from OUTSIDE -- exit
    codes, output, the process table. Outside-in is what makes a case portable to a machine that
    does not have the source.

## ⚠️ THE NAME

The binary is `faelight-sandbox`. `devbox` is an ALIAS, not a crate -- measured 2026-09-14,
`which devbox` finds nothing.

Per INT-247 Layer 2: a crate is renamed when it is REWRITTEN, not for spelling. This intent does
not rename it. If `verify` grows large enough to justify a split, the new crate is `zero-devbox`
-- and `0-devbox` is illegal, because cargo refuses a package name starting with a digit.

## Success Criteria

- [ ] Verify-first: what the sandbox does today is documented, with the policy list and the flags
      `run` accepts. A case format that cannot be expressed by those flags is not written
- [ ] The case format is DECIDED and written down, with every field mapped to an existing flag
- [ ] `verify <dir>` runs every case in a directory and reports PASS / FAIL / UNDETERMINED
- [ ] UNDETERMINED carries a REASON, and a case with an absent precondition produces it rather
      than a pass or a fail. Demonstrated by removing a precondition on purpose
- [ ] At least three cases exist from REAL bugs this project already had, not invented ones.
      Candidates measured 2026-09-14: the -c stopped child, the watch SIGINT lifetime, and a
      suspended job being resumable
- [ ] ⭐ A FIX IS PROVEN BY A CASE BEFORE IT REACHES THE SHELL. One real change goes
      case-first: the case fails, the fix lands, the case passes. That is the whole point and it
      is the gate that proves the tool rather than the format
- [ ] The census runs: every crate in rust-tools gets a case, and the results are the evidence
      INT-247's Week 1 census asks for
- [ ] `verify` exits non-zero when any case FAILS, and exits non-zero when any case is
      UNDETERMINED unless `--allow-undetermined` is given. An unaskable question must not pass CI
      silently
- [ ] No regression: nsh-test still green, `devbox test` still works unchanged
- [ ] Each gate carries evidence per INT-158

## Relationship

- INT-247 -- this is the census instrument that intent names. Its Week 1 deliverable
  (`docs/inventory.md`) is a `verify` report
- INT-167 -- the DevBox vision. This is its first shippable piece, and deliberately the smallest
  one. The TUI, the event log and the profiler are not blocked by this and are not part of it
- INT-192, INT-245, INT-246 -- the UNDETERMINED arm is their shared ruling, applied again
- nsh-test (INT-202) -- the sibling. Fast and in-process; this is slow and outside-in. A case
  that can live there, lives there

## The Rule

"A reproduction that lives in a scrollback buffer is not a reproduction. The clean room already
exists -- what was missing was somewhere to write down what should have happened." 🌲
