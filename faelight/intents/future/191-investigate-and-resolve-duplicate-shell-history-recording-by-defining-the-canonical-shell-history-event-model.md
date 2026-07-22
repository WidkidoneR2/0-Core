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

### MEASURED 2026-07-22 -- the diagnosis CHANGED

The timestamp-based measure was the wrong instrument. Measuring by ADJACENCY
instead (the two writes land in consecutive rows whether or not the clock ticked)
gives the real picture:

  adjacent rows with an IDENTICAL command:      16,821  of 114,255
  adjacent rows where the first is an ALIAS and the second is its expansion:

      c      -> clear                    17,283
      d      -> core doctor run             378
      d      -> /run/.../core doctor run    368
      0core  -> cd /home/christian/0-core   214
      intl   -> core intent list             96
      gs     -> git status                   37
      fs     -> faelight-shell               35
      ls     -> eza --icons                  26

★ THE TWO WRITES ARE NOT ALWAYS ACCIDENTAL DUPLICATES. They often represent two
DISTINCT OBSERVATIONS. Whether two ROWS is the right storage model is a separate,
still-open design question -- the measurement does not decide it. One mechanism
explains both numbers:

    main.rs:1170          writes the TYPED line
        |
    alias expansion (~2251)
        |
    execute_with_context(expanded)   ->  ctx.raw = the EXPANDED line
        |
    postexec              writes the EXECUTED line

  No alias fired  -> the two lines are identical -> looks like a duplicate (16,821)
  An alias fired  -> the two lines differ        -> a typed/executed pair (18,000+)

So `shell_history` has been recording a TWO-EVENT MODEL all along -- accidentally,
and with nothing in the schema marking which row is which. The earlier instinct to
"delete the redundant writer" would have destroyed real information: `c` and
`clear` are both true, and they answer different questions.

The remaining work is therefore not deduplication. It is answering: what is the
canonical representation of ONE SHELL INTERACTION?

### The design decision (2026-07-22)

Two conceptual models were considered.

EVENT LOG -- every observation is a row (typed, executed, and later blocked,
rewritten, expanded...). Faithful chronology, extensible, audit-friendly. But
every consumer must understand event types, and adjacency stops meaning "the next
command".

COMMAND RECORD -- one row is one command LIFECYCLE (typed, executed, exit,
duration, alias_used, plan_hash...). More complex writes, and loses the
stream-of-observations abstraction unless a separate event log is added. But
`id + 1` genuinely means the next command, analytics are simple, and consumers do
not reconstruct a command from multiple rows.

★ CHOSEN: COMMAND RECORD -- one row per command. Not because it is simpler, but
because fsh's fundamental unit is increasingly a COMMAND EXECUTION rather than
"someone inserted something into history". The shell already has typed text,
parsed AST, execution plan, executed argv, exit status, telemetry and intent --
all of which belong to ONE lifecycle. That model grows naturally as the shell
does. In an event table, every new field raises "which event owns this?".

### A separate concern: id + 1 is not a sequence

⚠️ Worth challenging INDEPENDENTLY of the schema choice. IDs are INSERTION ORDER,
not semantic sequence. They coincide today only because of how writes happen --
which is exactly the coupling that broke when a second writer appeared. Even
under one-row-per-command, a consumer asking "what came next" should use a
timestamp, or better an explicit sequence number assigned by the shell, so it is
not coupled to a storage detail.

### Dry-run pairing analysis, 2026-07-22 (read-only, wrote nothing)

FINAL NUMBERS, pairing over command rows with bookkeeping EXCLUDED from the
sequence (ROW_NUMBER over non-TIMING/non-SUGGEST rows, delta <= 2s):

    identical pairs    15,799     (no alias fired -- both writes the same text)
    alias pairs        17,943     (typed -> executed)
    -------------------------------
    pairs              33,742  =  67,484 rows
    command rows      108,985     (114,316 total - 5,331 bookkeeping)
    explained by the two-write model:  ~62%
    genuinely single-write:            ~38%

★ STRUCTURAL FINDING -- BOOKKEEPING IS INTERLEAVED WITH COMMANDS. `TIMING:` and
`SUGGEST:` rows live in the same table and land BETWEEN the two halves of a pair:

    116693 | git push            <- typed
    116694 | git push            <- executed
    116695 | TIMING:git:1235     <- bookkeeping

So ROW ADJACENCY IS NOT COMMAND ADJACENCY. Any consumer joining on `id + 1` can
read a TIMING: row as "the command that followed git push" -- including
faelight-daemon's prediction. This is an argument for separating the bookkeeping
stream from the command record regardless of which schema is chosen.

⚠️ THE ~38% SINGLE-WRITE REMAINDER IS NOT YET EXPLAINED and must be before any
migration. Known contributors: `exit` (postexec skips it explicitly --
`if status != "exit"`), and every prefix-handled command (export/unset/persist/
spine-exec) which `continue`s before reaching execute_with_context. Whether those
account for all of it is unmeasured.

⚠️ PROCESS NOTE, worth as much as the number: the first three attempts at this
analysis produced 49.9% unpaired, then 67.7% unpaired, then zero alias pairs --
all from CASE-arm ordering bugs in the classifier, not from the data. The alias
condition matched 17,293 rows when tested STANDALONE while returning zero inside
the CTE. A dry run that reports an alarming number should be suspected of
measuring itself before it is believed about the data.

### Migration of existing rows

The 114k existing rows are TELEMETRY, not user-authored data, so rewriting them is
acceptable provided the migration:
  - preserves the original database
  - documents the pairing heuristic
  - produces a REPORT rather than silently forcing every row into a pair:
        paired automatically / ambiguous / left unchanged

### A consumer this structurally breaks

faelight-daemon's prediction_precompute (daemon.rs ~499) infers "what command
follows what" by self-joining on adjacency:

    JOIN shell_history h2 ON h2.id = h1.id + 1

Given the two-event model, `id + 1` does not mean "the next command". It means
"the other half of the same command". Measured against real history, the daemon's
top predictions are:

      clear   17,305
      c       17,283

That is one keystroke and its own alias expansion -- the single most frequent
"adjacency" in the entire corpus, and not a behavioural pattern at all. The
daemon is not learning what follows what; it is learning the alias table. That is
not degraded accuracy, it is a specific wrong answer produced confidently.

The daemon is NOT currently running (lost during the Nix migration), so nothing
acts on this today. But it would return and immediately learn a false pattern
from months of duplicated rows -- which is the argument for fixing the recording
before reconnecting any consumer.

Enumeration of writers, completed 2026-07-22 (gate 1):
  save_history_entry has exactly THREE callers -- exec.rs:314 (postexec),
  main.rs:1170 (REPL, per line), main.rs:1187 (multi-line/pty path). No hidden
  helper. Plus four special-purpose direct INSERTs: SUGGEST: cooldown rows,
  TIMING: rows, the confirmed-suggestion path, and a doctor test row.
  faelight-daemon and db-browse only READ; history_tui, completion and semantic
  contain no INSERT/UPDATE/DELETE at all. So fsh is the only writer.

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

- [x] Every code path that writes to `shell_history` is enumerated (not grepped -- enumerated)
<!-- 2026-07-22: save_history_entry has 3 callers (exec.rs:314 postexec, main.rs:1170 REPL, main.rs:1187 multi-line) + 4 special-purpose direct INSERTs. faelight-daemon and db-browse read only; history_tui/completion/semantic have no writes. fsh is the sole writer. -->
- [x] Measured: how often the typed and executed forms actually differ in real usage
<!-- 2026-07-22: 16,821 adjacent-identical pairs (no alias fired) + 18,000+ adjacent typed/executed pairs (alias fired), led by c -> clear 17,283. Both from ONE mechanism: REPL writes typed, postexec writes ctx.raw which is post-alias-expansion. -->
- [x] Measured with a method that does NOT depend on the two writes sharing a timestamp
<!-- 2026-07-22: self-join on id+1 rather than GROUP BY timestamp. The two writes are always consecutive rows regardless of whether the clock ticked between them, so adjacency is the correct instrument and the earlier ~12% timestamp figure was an artifact of the method. -->
- [x] Decided and recorded: what `shell_history` represents -- user input, executed commands, or both as distinct event types
<!-- 2026-07-22: COMMAND RECORD, one row per command lifecycle (typed + executed + exit + duration + ...), chosen over an event log because fsh's fundamental unit is a command execution and the model grows with the shell. See "The design decision". -->
- [ ] Migration produces a REPORT (paired / ambiguous / unchanged), never silently forcing pairs
- [ ] Original database preserved before any rewrite
- [ ] Consumers asking "what came next" use a timestamp or explicit sequence, not id + 1
- [ ] Every history write has a single, well-defined owner
- [ ] The schema expresses the chosen event model rather than relying on duplicate inserts
- [ ] Frequency analysis, failure attribution and downstream learning consume semantically correct history instead of inferring meaning from duplicate rows
- [ ] Existing accidental duplication eliminated WITHOUT discarding information that is intentionally distinct
- [ ] `spine audit` / `spine migrate` volume figures re-checked afterwards (their shape metrics are unaffected -- dedup absorbs this -- but the volume metric has been inflated)
