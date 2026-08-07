---
id: 204
date: 2026-08-06
type: future
title: "fsh-test isolation is accidental -- config alias pruning is what cleans up between cases, and nothing says so"
status: in-progress
tags: [fsh-test, fsh, clean up]
---

## Vision
A suite whose cases cannot reach each other. Today they can, and the only thing stopping them is a
side effect nobody designed, documented, or tested.

## The Problem
Five unrelated tests failed on a push that added vim mode.

    repl_quoted_redirect_is_not_an_operator   saw  MARK193 zzq > zzmark
    conform_echo_one___grep_one___wc__c       got  12   where bash got 4
    conform_echo_one____tmp_fsh_conform_b     got  MARK193 one / MARK193 two
    repl_builtin_first_pipeline_with_redirect the file held the prefixed text
    repl_plain_builtin_redirect_still_works   the file held the prefixed text

One cause: `alias echo='echo MARK193'`, created by repl_193_expansion_happens_exactly_once and never
removed. It reached five later cases and made each of them lie about the shell.

## THE MECHANISM, and it is the finding
Every case spawns a fresh fsh. That fsh runs config::apply, which PRUNES any alias not present in
config.fsh (config.rs 283-296, INT-060 G9). So an alias a test creates is swept away before the next
case starts.

THAT PRUNING IS THE SUITE'S ISOLATION. Nobody wrote it for that purpose. No test asserts it. No
comment in fsh-test mentions it. It has been holding the suite together as a side effect of a
feature built for a different reason -- config.fsh being the source of truth for aliases.

HOW IT BROKE. The push ran from a nested shell started with an inline FSH_CONFIG pointing at a
one-line scratch config. fsh sets inline assignments into its own environment, children inherit
them, so every test's fsh loaded a config with zero aliases. INT-060's guard then did exactly the
right thing -- never prune from an empty config, because a parse failure must not wipe the live set
-- and the alias survived. A correct guard, a legitimate new capability, and five false failures.

## Why this ranks
A suite whose cases can poison each other across invocations is INT-202's problem one layer down: not
a flaky harness, a STATEFUL one. INT-202 made the numbers mean something by fixing capture, timing
and routing. This is the same claim about state, and it is currently held up by an accident.

⚠️ AND IT WILL RECUR SILENTLY. The next thing that changes alias handling, config loading, or the
order of cases will produce failures that look like bugs in whatever is being worked on -- as this
one did, for the better part of an hour.

## THE DESIGN QUESTION THIS INTENT EXISTS TO ANSWER
Not "how do we isolate" but "isolate from WHAT".

  REALISTIC   the shell under test uses the real database, so it behaves like the shell that is
              actually run. Some cases exist BECAUSE of that environment -- one records that `cat`
              is aliased to bat, whose box-drawing output forced the assertion to change. Isolation
              would silently invalidate the reason that test is written the way it is.

  REPRODUCIBLE  the shell under test gets its own database, so no case can see another's writes and
              a green run means the same thing on any machine. But the shell then differs from the
              one anyone uses, and a class of bug -- the interaction between real config and real
              behaviour -- becomes untestable.

The answer may be BOTH, split explicitly: most cases isolated, a small named set declared as running
against the live environment with the reason recorded. What must not happen is choosing implicitly,
which is the situation today.

## THE DECISION (2026-08-07): isolate the shell under test, and say so out loud
ISOLATE FROM WHAT: from the USER'S DATABASE. The shell under test gets its own, pointed at by
FAELIGHT_STATE_DB. The harness keeps writing its own RESULTS to the live one -- isolation is about the
shell being tested, not about the reporting.

THE OBJECTION THAT MADE THIS QUESTION HARD IS ANSWERED BY THE SOURCE. The worry was that several
cases exist because the real environment shaped them -- cat is aliased to bat, and a case had to be
written around that -- so a scratch database would change what those cases see. It does not.
config::apply REGISTERS the config aliases into the table before it prunes anything, and it runs at
every startup. A scratch database therefore arrives seeded from config.fsh exactly as a fresh login
shell does. Isolation costs nothing in realism, which is why this stopped being a trade-off.

WHERE IT GOES: paths::state_db, and nowhere else. Its own doc calls it the seam that makes moving
runtime a one-line change, so canonical means ONE DECIDER rather than unoverridable. A second place
resolving the database is precisely what that comment forbids.

⚠️ THE NAME IS FAELIGHT_STATE_DB, NOT FSH_ANYTHING. paths.rs lives in faelight-core and every tool
resolves through it, so an fsh prefix would describe a reach it does not have.

⚠️⚠️ AND THE DANGER IS THIS INTENT'S OWN STORY, WITH HIGHER STAKES. INT-204 exists because
FSH_CONFIG leaked out of an inline assignment into a child process and silently changed behaviour. A
variable pointing at the DATABASE that leaked the same way would send the shell to a scratch file --
no history, no aliases, no session memory -- and say nothing.

⭐ SO IT IS LOUD. When the resolved database is not the canonical one, the shell says so on every
start. That is the model this project already runs on: you cannot make it unbreakable, you can make
breakage loud. The notice derives from comparing the resolved path against the default rather than
reading the variable again, so the decision keeps one owner and the message cannot disagree with it.

## Success Criteria
- [ ] The dependency is PROVEN before it is removed: a run with a zero-alias config reproduces the
      five failures, and that reproduction is recorded. Unreachable isolation is a claim; a red run
      is evidence.
- [ ] The isolate-from-what question above is ANSWERED and written down, with the reason. If the
      answer is "both", the split is explicit and each live-environment case says why it is one.
- [ ] The shell under test can be pointed at a database other than the user's, by an env var, the
      way FSH_CONFIG already points it at another config.
- [ ] A case that creates an alias leaves NOTHING behind: after a full run, the live shell_aliases
      table holds exactly what config.fsh defines. Verified by count and by name, not by eye.
- [ ] The suite is green with FSH_CONFIG set to a zero-alias config -- the proof that isolation no
      longer depends on pruning. This is the gate that would have caught today's failure.
- [ ] fsh-test still writes its own RESULTS to the live state.db. Isolation is about the shell under
      test, not the harness's reporting; a gate saying "the suite never writes to state.db" would be
      the wrong invariant.
- [ ] Each gate carries evidence per INT-158.

## Scope guardrails
- Do NOT fix this by making each test unalias in its own session. A red run skips its own cleanup,
  which is exactly how this litter accumulated in the first place -- the mechanism must not depend on
  the case succeeding.
- Do NOT remove or weaken INT-060's prune guard. It behaved correctly throughout; the accident is
  that something else was relying on the pruning, not that the pruning is wrong.
- Isolation is not a licence to change what cases assert. If a case only passes in an isolated
  database, that is a finding about the case, not a reason to keep isolating quietly.

## Relationship
- INT-202 (fsh-test 2.0) fixed capture, timing, speed and routing. This is the state axis of the same
  argument, found the day after 202 closed.
- INT-060 owns config.fsh as the source of truth for aliases, including the prune and its guard.
- ⚠️ A SEPARATE FINDING, recorded here because it was found here and belongs elsewhere: in fsh,
  `FOO=bar cmd` sets FOO in the SHELL'S OWN environment and everything spawned afterwards inherits
  it. In bash the assignment affects only that command. That divergence is what carried FSH_CONFIG
  into the pre-push hook, and it is INT-143 bug B's territory -- it deserves its own intent rather
  than a line in this one.
