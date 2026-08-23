---
id: 228
title: "job identity is the same primitive type as a vector index, so nothing prevents mixing them"
status: complete
type: fix
priority: medium
date: 2026-08-23
tags: [fsh, jobs, types, identifiers, int-188]
---

## Vision
A job's identity cannot be confused with where it happens to be stored.

## ✅ RECON FIRST, and it made this intent SMALLER
Measured 2026-08-23 before any scoping, and most of what a job-identifier intent would normally fix
is ALREADY CORRECT:

    JobTable { jobs: Vec<Job>, next_id: usize }     next_id starts at 1

- **Identity is a MONOTONIC COUNTER, not a position.** `fg` and `kill_job` both do
  `self.jobs.iter().position(|j| j.id == id)` -- they look up BY IDENTITY and only then use the
  index. `retain(|j| !completed.contains(&j.id))` removes by id too.
- **An id is not recycled when a job is removed.** So the defect where `%2` means a different job
  after one exits DOES NOT EXIST here. There is no identifier-generation problem to solve.

★ SO THIS IS NOT A DISGUISED JOB-CONTROL INTENT. It introduces no allocation scheme, no numbering,
no recycling, and no lifecycle semantics.

## The Problem
`id` is a `usize`. A vector index is a `usize`. `position()` returns one and `id` is the other, and
nothing in the type system tells them apart.

⚠️ THIS SHELL HAS PAID FOR EXACTLY THIS CLASS OF MISTAKE. `id + 1` on `shell_history` meant "the
next row", which four consumers read as "the next command" -- and four predictors were deleted on
2026-08-22 because that arithmetic was wrong in a way nothing could catch. A counter and an offset
that share a type invite the same error, and the compiler can make it impossible instead.

## The Solution
An opaque newtype:

    pub struct JobId(u64);

Identity stays exactly what it is today -- the same monotonic counter, the same values. What changes
is that it can no longer be added to, indexed with, or silently swapped for a position.

⭐ AND THE THREE-LEVEL DISTINCTION THIS PROTECTS, which matters more as INT-188 lands:

    JobId            -- the SHELL's identity for a job
    ProcessGroupId   -- the OS's job-control identity
    Pid              -- one individual process

A job contains MULTIPLE PROCESSES once pipelines and process groups exist, so a pid can never be the
shell's job identity even though it is a perfectly good process identity. Keeping the three apart in
the type system is what stops that conflation being made by accident later.

## ⚠️ THE BOUNDARY, recorded so it is not crossed
`check_completed()` (`jobs.rs:101`) polls with `try_wait()`, which observes TERMINATION but cannot
observe a STOPPED child -- a `SIGTSTP`'d job is neither exited nor running, so the table would
believe it is still running forever. **Stopped-job visibility is an INT-188 signal concern, not an
INT-228 lifecycle change.**

Likewise `fg` (`jobs.rs:165`) calls `child.wait()` -- a blocking wait with no terminal transfer,
because `tcsetpgrp` does not exist anywhere in fsh. **That belongs to INT-188 as well.**

## Success Criteria
- [x] G1 SHELL IDENTITY IS TYPE-DISTINCT FROM STORAGE POSITION
<!-- `pub struct JobId(u64)` in jobs.rs. The counter is unchanged; `position()` still returns an
     index and now cannot be swapped for one. No allocation scheme, no numbering, no recycling, no
     lifecycle semantics. Commit 19b68380. -->
- [x] G2 THE VALUES DO NOT CHANGE
<!-- `Display` writes the same decimal, asserted by a_job_id_still_displays_as_the_number_it_always_was.
     162/162 fsh-test green, including repl_jobs_lists_a_running_job, so the live listing is
     untouched. -->
- [x] G3 RED FIRST -- and the ordering is recorded rather than dressed up
<!-- ⚠️ THE RED WAS THE COMPILER, NOT A TEST, and manufacturing a fresh failing test afterwards
     would be theatre. Changing the field type produced errors at every site treating identity as a
     number, and they were fixed one at a time until it built. That IS the proof a JobId cannot be
     used as an index: the code that did so no longer compiles, and cannot be written again.
     ★ WHAT THE TESTS ASSERT INSTEAD is the behaviour the type made possible:
       nonsense_is_not_quietly_a_job         -- "banana", "", "-3", "0" all parse to None
       the_percent_form_and_the_bare_form_agree -- `kill %2` and `fg 2` name the same job
     233 unit tests green. -->
- [x] G4 NO SHORT FORM SHIPPED -- the gate said IF, and the answer is not yet
<!-- ⭐ A CROCKFORD BASE32 PAIR WAS WRITTEN AND DELETED IN THE SAME HOUR, which is the useful part.
     The encoding was right by the gate's own reasoning: excludes I/L/O/U, DECODES the confusables
     so a misread `O` resolves, case-insensitive where Base58 is not, no punctuation where Base84
     needs it.
     ⚠️ THEN THE ENCODER HAD NO CALLER, and the tempting fix was to add a column to `jobs` so that
     it would. HIS RULING: that is backwards. It turns an internal capability into a user-facing
     change because the implementation happens to exist, and `[2R]` is not another rendering of `2`
     -- it is a NEW IDENTIFIER FORM a person must learn, which deserves an explicit decision.
     ★ THE RULE, kept in jobs.rs where the code would have gone: do not create UI to give an unused
     helper a caller. Create the UI when there is a user-facing requirement, then implement exactly
     what it needs. INT-188 makes job identifiers visible -- stopped, resumed, moved between
     foreground and background -- and defines encoder and display together or not at all. -->
- [x] G5 NO REGRESSION -- with one deliberate exception, stated
<!-- 162/162 fsh-test, 233 unit tests. `jobs`, `kill` and background spawning are identical.
     ⚠️ `fg` CHANGED ON PURPOSE and this is the intent's real find: it parsed with `unwrap_or(1)`,
     so a nonsense argument silently foregrounded an arbitrary job, while `kill` refused the same
     input. Two doors, two behaviours for one mistake. `fg` now prints usage.
     📍 AND WHERE IT IS REACHABLE, measured rather than assumed: `try_fg` has ONE caller,
     main.rs:1801, the REPL loop. Through `-c` the `fg` ALIAS wins first, so a `-c` invocation never
     reaches the guard at all -- INT-173's two-doors finding again. The unit tests prove `parse`
     directly; the live path is interactive. -->
- [x] G6 each gate carries evidence per INT-158
<!-- this block. -->

## Non-goals
- SIGCHLD, stopped-job detection, process groups, terminal foreground transfer. All INT-188.
- Replacing the counter. It works; this only gives it a type.
- Applying the scheme to AST nodes, history rows, sessions, caches or plugins. ⚠️ THAT IS THE
  TEMPTING VERSION AND IT IS OUT OF SCOPE: each of those has its own identity requirements, and
  `command_execution` already uses `session:execution` for a reason INT-191 recorded. One consumer
  first, then evidence.
