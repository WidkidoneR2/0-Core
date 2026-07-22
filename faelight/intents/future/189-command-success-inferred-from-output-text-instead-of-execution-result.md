---
id: 189
date: 2026-07-21
type: fix
title: "command success inferred from output text instead of execution result"
status: planned
tags: [fix, bugfix, telemetry, friday]
---

## Vision

Exactly one place decides whether a command succeeded, and it is the execution
result. Nothing downstream re-derives that verdict from what the command printed.

## The Problem

fsh had TWO sources of truth for command success.

The authoritative one is correct: `exec::execute_with_context` returns a
`CommandResult`, and the match in main.rs sets `last_exit_code` from it
(Output / Empty / NotBuiltin -> 0, Error -> 1).

The second one was wrong. An INT-201 block (Arch-era, "track last command exit
status for the faelight-term indicator") RE-DERIVED success by scanning the
command's OUTPUT TEXT for a leading cross-mark, the substring "error", or the
substring "not found" -- and then OVERWROTE `last_exit_code` with that guess.

It was wrong in BOTH directions:

- FALSE FAILURE. A command that succeeded but whose legitimate output mentions
  one of those words was recorded as a failure. Found via `spine migrate`: its
  report legitimately COUNTS parse errors, so printing "Spine parse error: 47"
  made a successful audit register as a failed command.
- FALSE SUCCESS. A builtin that genuinely failed but whose message happens not
  to contain those words was recorded as a success.

That value is written to `term_commands.exit_code`, which Friday's
three-failures-in-a-row detector reads. The shell was learning from fabricated
observations and printing conclusions drawn from them
("spine failed 3 times in a row -- check the command", which was false).

### Data integrity

Four fabricated rows (`spine migrate`, exit_code 1) were CORRECTED to 0 rather
than deleted -- only the exit_code field was fabricated; session, timing and cwd
on those rows are true. `term_commands` has no validity column, so a targeted
correction was the available repair.

A wider sweep was investigated and REJECTED on evidence. Other exit_code=1 rows
turned out to be genuine: deliberate test failures (invalidcmd123, badcmd999,
`core intent cancel 999`, `sqlite3 /nonexistent/nope.db`), and `intent list`,
which is a RETIRED command that legitimately fails now (`intl` replaced it).

CUTOFF, recorded rather than patched over: builtin exit codes recorded BEFORE
generation 421 are NOT authoritative -- they may reflect output-text inference
rather than execution result. For any individual historical row it is impossible
to reconstruct what that command printed at the time, so no further repair is
honest. Friday's pattern data should be read with that qualifier.

## The Solution

Delete the second source of truth. Keep the feature it was attached to.

The INT-201 block did two things -- the wrong re-derivation, and a legitimate
cache write of the status for the faelight-term indicator. Only the
re-derivation is gone; the block now CONSUMES `last_exit_code` instead of
recomputing it. DONE, commit 6543649b.

### Remaining work: the stale exit-code paths

Removing the scan exposed a pre-existing gap it had been papering over. Four arms
of the `CommandResult` match never set `last_exit_code` at all:

- both `Value` arms
- `Value(_) if has_external_op`
- `Output(out) if !pipeline_ops.is_empty()`

The last two spawn `sh` for the pipeline and DISCARD its exit status. On those
paths `last_exit_code` carries over stale from the previous command. Making the
staleness visible rather than guessed is an improvement, but it is still wrong.

This was deliberately NOT bundled with the telemetry fix. It touches pipeline
execution semantics and needs its own answers first:

- Is the spawned shell's status the status we want?
- Is pipeline status the last command, the first failure, or something else?
- Does existing fsh behaviour intentionally differ from POSIX here?

INT-143's double-execution scars are on this path, so it needs its own
verification rather than riding along with an unrelated repair.

## Success Criteria

- [x] Output-text inference of command success removed; the CommandResult verdict is the only source
<!-- commit 6543649b -- main.rs INT-201 block now reads `let exit_ok = last_exit_code.map(|c| c == 0).unwrap_or(true);`; grep for contains("error") / contains("not found") in that path returns no matches -->
- [x] faelight-term last-exit-status cache write preserved
<!-- commit 6543649b -- the cache_dir write survives; only the re-derivation was deleted -->
- [x] Fabricated telemetry rows repaired, scope justified by evidence
<!-- 4 `spine migrate` rows corrected 1 -> 0; verified before/after (0|2 + 1|4 -> 0|6). `spine parse` exit 1 left untouched: bare `spine parse` genuinely returns CommandResult::Error -->
- [x] Verified on metal: a successful audit no longer registers as a failure
<!-- demonstrated gen 421: `spine migrate` returns exit 0, no cross-mark in the prompt indicator, no Friday failure line -->
- [ ] Decide pipeline exit-status semantics (last command / first failure / other) and record the decision
- [ ] All four unset `CommandResult` arms assign `last_exit_code` from a real source
- [ ] Pipeline exit status verified end-to-end without reintroducing INT-143 double execution
