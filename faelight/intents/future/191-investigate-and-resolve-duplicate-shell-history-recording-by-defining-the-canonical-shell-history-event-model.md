---
id: 191
date: 2026-07-22
type: fix
title: "investigate and resolve duplicate shell history recording by defining the canonical shell history event model"
status: planned
tags: [fix, bugfix, telemetry, friday, data-model]
---

## Vision

`shell_history` records a well-defined set of events, each written exactly once by
a single owner, with a schema that says what each row means.

## The Problem

Measured 2026-07-22: `shell_history` holds 114,105 rows against 99,944 distinct
`(command, timestamp)` pairs -- roughly 14,161 excess. Duplicates go back to at
least 2026-04-05, so this long predates the INT-169 spine work.

⚠️ THAT ~12% IS A FLOOR, NOT THE RATE. The suspected writers each call
`SystemTime::now()` independently, so any command that spans a second boundary
lands its rows under different timestamps and escapes a
`GROUP BY command, timestamp` measure entirely. Fast commands cluster into the
same second and are counted; slow ones are not. The true duplication rate is
unknown and probably higher.

A grep found two writes that both fire for an ordinary command
(`main.rs` REPL, unconditional per line; and `postexec` via
`save_history_entry(&ctx.raw)`), plus `SUGGEST:` cooldown rows, `TIMING:` rows,
a confirmed-suggestion path, and a doctor test row. `save_history_entry` is a
plain INSERT with busy-retry and no dedupe.

⚠️ BUT "two independent writers" REMAINS A HYPOTHESIS. A grep is not an
enumeration. It is enough evidence to justify investigating; it is not enough to
name the root cause.

### The deeper question

This may not be a duplicate-write problem at all. The shell may be recording TWO
DIFFERENT FACTS:

  typed     -- the literal line the user entered
  executed  -- what actually ran after interpretation

    typed: intl        executed: core intent list
    typed: ll          executed: eza -la --icons

Those answer different questions. Frequency analysis arguably wants the typed
form; failure attribution wants the executed one. They are distinct observations,
not accidental duplicates, and today they land in one table with nothing marking
which is which.

This is the same category as the finding that a capability gap
(`spine exec ll` -- alias expansion not implemented) was recorded as `ll | 127`,
indistinguishable from a process that genuinely exited 127. The principle:
RECORD WHAT ACTUALLY OCCURRED AT THE BOUNDARY WHERE YOU KNOW IT. Summarising to
a single notion of "a command happened" is a VIEW, not the storage format.

## The Solution

Not decided -- deliberately. The measurement should drive the design.

Candidate shapes, once the data is in: two tables; one table with an
`event_kind` column; or one table with distinct typed/executed columns.

⚠️ ONE APPROACH TO AVOID: "only record the executed form when it differs from the
typed form." It saves rows but destroys a clean event model -- the absence of a
second row becomes ambiguous between "omitted because identical" and "execution
never reached that stage". If provenance matters, record both events explicitly
and let analysis collapse them.

## Success Criteria

- [ ] Every code path that writes to `shell_history` is enumerated (not grepped -- enumerated)
- [ ] Measured: how often the typed and executed forms actually differ in real usage
- [ ] Measured with a method that does NOT depend on the two writes sharing a timestamp
- [ ] Decided and recorded: what `shell_history` represents -- user input, executed commands, or both as distinct event types
- [ ] Every history write has a single, well-defined owner
- [ ] The schema expresses the chosen event model rather than relying on duplicate inserts
- [ ] Frequency analysis, failure attribution and downstream learning consume semantically correct history instead of inferring meaning from duplicate rows
- [ ] Existing accidental duplication eliminated WITHOUT discarding information that is intentionally distinct
- [ ] `spine audit` / `spine migrate` volume figures re-checked afterwards (their shape metrics are unaffected -- dedup absorbs this -- but the volume metric has been inflated)
