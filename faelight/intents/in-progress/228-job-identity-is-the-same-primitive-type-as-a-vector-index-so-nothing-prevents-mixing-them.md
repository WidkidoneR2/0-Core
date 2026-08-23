---
id: 228
title: "job identity is the same primitive type as a vector index, so nothing prevents mixing them"
status: in-progress
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
- [ ] G1 SHELL IDENTITY IS TYPE-DISTINCT FROM STORAGE POSITION. `JobId` is an opaque newtype over
      the existing monotonic counter; vector position stays an implementation detail. No new
      allocation, numbering, recycling, or lifecycle semantics are introduced
- [ ] G2 THE VALUES DO NOT CHANGE. A job that was 3 is still 3 -- this is a type change, not a
      renumbering, and existing `jobs` output is unaffected
- [ ] G3 RED FIRST: a test that a `JobId` cannot be used as an index or arithmetic operand, proven
      by the change failing to compile before the newtype and compiling after
- [ ] G4 RENDERING EXPOSES THE EXISTING IDENTITY. If a short human-typable form is added, it encodes
      the SAME counter -- ⭐ Crockford Base32 if so, because it excludes I/L/O/U and accepts the
      confusable characters on input, so a misread `O` still resolves. NOT Base58 (mixed case) and
      NOT Base84 (needs punctuation, which a shell must then quote). ⚠️ Density is the constraint
      fsh has LEAST of: a session has single-digit jobs, and three Base32 characters already give
      32,768
- [ ] G5 NO REGRESSION: `jobs`, `fg`, `kill` and background spawning behave identically. fsh-test
      green
- [ ] G6 each gate carries evidence per INT-158

## Non-goals
- SIGCHLD, stopped-job detection, process groups, terminal foreground transfer. All INT-188.
- Replacing the counter. It works; this only gives it a type.
- Applying the scheme to AST nodes, history rows, sessions, caches or plugins. ⚠️ THAT IS THE
  TEMPTING VERSION AND IT IS OUT OF SCOPE: each of those has its own identity requirements, and
  `command_execution` already uses `session:execution` for a reason INT-191 recorded. One consumer
  first, then evidence.
