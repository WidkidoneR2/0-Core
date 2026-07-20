---
id: 173
date: 2026-07-18
type: fsh
title: "Complete fsh-test drives the REPL, not just -c"
status: in-progress
tags: [fsh, faelight-shell, fsh-test]
---

## Vision
fsh-test drives the REAL interactive REPL -- the shell Christian actually types into --
not just `fsh -c`. A green suite should mean "fsh works", not "the shell behind fsh works".

## The Problem
fsh has TWO front doors, and until INT-172/171 the test suite only knocked on one of them.

`fsh -c "cmd"` hands the whole line to `/bin/sh` (main.rs, the -c branch): fsh's own parser,
dispatch, redirect handling, and &&/|| chaining never run. So every `-c` test executes under
sh. The suite's ~83 `-c` tests largely verify that SH works -- which it does -- while telling
you almost nothing about fsh's interactive behaviour.

This was proven concretely during INT-171 gate 3. The six INT-143 regression bugs (double-exec
on redirect, typo-&&-leak, python3 flag stripping, bash-script non-exec, env passthrough, inline
var scope) are ALL INVISIBLE through `-c`: run through sh, every one produces the correct answer.
The suite was "83/83 green" on a demonstrably broken fsh. The suite was honest about what it ran;
what it ran was sh. Two doors, one tested.

The interactive REPL is the door that has the bugs -- and the door Christian uses every day.
Testing it is not optional polish; it is the difference between a suite that measures fsh and a
suite that measures /bin/sh.

## The Solution
The pty harness already exists. `repl.rs` (built for INT-172 gate 7, extended for INT-171 gate 3)
exposes `repl::run_repl(cmd) -> Result<String, String>`: it spawns fsh under a pseudo-terminal,
feeds a command to the REAL prompt, and returns what the running shell actually emitted. The
`Category::Repl` tests use it. Eleven exist today (5 from INT-172's pipe/redirect work, 6 from
INT-171's INT-143 regressions), all passing on the deployed binary.

This intent FORMALISES that harness as the way fsh-test tests fsh, DOCUMENTS the two-doors finding
so it is never re-learned the hard way, EXTENDS REPL coverage to core interactive behaviours that
`-c` cannot see, and WRITES DOWN the policy that keeps the suite from drifting back to testing sh.

SCOPE GUARDRAIL -- this is NOT "convert all 83 `-c` tests to REPL". Many `-c` tests legitimately
exercise the `-c`/sh path, which is real and load-bearing (INT-190: niri-session boot depends on
`fsh -c`). 173 is about having the REPL door TESTED AT ALL and DEFAULT for interactive behaviour --
not deleting the `-c` suite. The `-c` path stays tested as the `-c` path.

## Success Criteria
- [ ] The pty harness (repl.rs) carries a module header documenting the two-doors finding: WHY
      `fsh -c` tests /bin/sh (the -c branch in main.rs hands the line to sh) and why the REPL door
      is the one that exercises fsh's dispatch. So the next person does not re-derive it from a
      broken-but-green suite. Evidence: the header exists and names the mechanism (file:line).
- [ ] `repl::run_repl` is the sanctioned interactive-test entry point, proven by the existing
      Category::Repl tests passing on the DEPLOYED binary (not target/debug). Evidence: fsh-test
      run against the deployed fsh, REPL tests green, count stated.
- [ ] The core interactive behaviours that are INVISIBLE through `-c` have REPL tests -- at minimum
      fsh-builtin dispatch and alias expansion at the prompt, added where INT-171 gate 3 did not
      already cover them (chain-stop, redirect, inline-var scope are already covered by the
      repl_143_* tests -- do not double-count). Each new test proven to exercise fsh's own path,
      not sh's: green on HEAD, and demonstrably red if the fsh behaviour it checks is broken.
- [ ] A written policy (in repl.rs's header and/or docs/CONVENTIONS.md): new tests for interactive
      behaviour go through the REPL door; `-c` tests are explicitly for the `-c`/sh path. The rule
      that stops the suite drifting back to measuring sh. Evidence: the policy text + location.
- [ ] Each gate carries evidence per INT-158.

## The Rule
"A green suite that runs the wrong shell is not a test -- it is a comment that compiles.
Test the door you type into." 🌲
