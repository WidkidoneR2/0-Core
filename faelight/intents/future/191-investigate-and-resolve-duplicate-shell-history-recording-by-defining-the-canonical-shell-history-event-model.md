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

### ★ THE EXECUTED FORM DIFFERS FOR MULTIPLE INDEPENDENT REASONS

Found 2026-07-22 while sampling the unpaired remainder. Alias expansion is only
ONE of the transformations between what was typed and what ran:

    identity           ls              -> ls
    alias expansion    c               -> clear
    path resolution    intent list     -> /run/current-system/sw/bin/intent list
    plugin / wrapper   fg commit       -> ~/0-core/scripts/faelight-git commit   (suspected)

The near-identical counts are the tell: `intent list` 322 / resolved form 321,
`fg commit` 374 / resolved form 370, `deploy faelight-shell` 242 / resolved 195.
Those are pairs, not separate commands.

★ WHAT THIS MEANS FOR THE SCHEMA: the record does NOT need to encode WHY the two
forms differ. It needs to record the two endpoints faithfully -- what the user
entered, and what was ultimately executed. The transformation mechanism can be
inferred later, or logged separately if it earns its place. Encoding the reason in
the primary record would bake in today's list of transformations and break when a
new one appears (as one just did).

### ⚠️ THE PAIRING IS NOT EXPRESSIBLE AS A SET QUERY

Four SQL attempts produced four different wrong answers (49.9% unpaired, 67.7%,
zero alias pairs, then `c` itself landing in the unpaired bucket despite matching
17,293 times standalone). The cause was the same each time: CASE-arm ordering and
correlated-subquery behaviour inside CTEs, not the data.

The reason is structural. Pairing is a STREAMING algorithm:

    read a row -> decide whether the FOLLOWING row belongs to the same command
                -> if yes consume both, else consume one -> continue

SQL can emulate parts of that with window functions, but with multiple pairing
rules it stops being the natural tool. The next step is a small streaming
ANALYZER (not a migration tool) that walks rows in order, classifies each pair by
known transformation, and leaves anything it cannot explain in an UNKNOWN bucket
rather than forcing it into one. The unknowns are where the next insight is.

### ★★ STREAMING ANALYZER RESULT, 2026-07-22 -- every row accounted for

A small read-only Python analyzer that walks rows in id order and CONSUMES pairs
classified 109,042 of 109,042 command rows (5,333 bookkeeping rows held out as a
separate stream):

    pair: alias                        19,926
    pair: identical                    15,793
    pair: path resolution               1,388
    pair: wrapper (program rewritten)     353
    ---------------------------------------
    PAIRS                              37,460   (74,920 rows)
    unpaired (single write)            34,122
    accounted for                     109,042 of 109,042

⚠️ READ THE 69% / 31% SPLIT CAREFULLY. Those percentages describe the CURRENT
IMPLEMENTATION, not the conceptual model. Their value is that they show WHERE the
shell currently bypasses part of the lifecycle -- `cd` (a state-changing builtin),
`exit` (postexec skips it by design), and the prefix handlers that `continue`
before execute_with_context. They do not define how the system should behave.

### ★★★ THE EXECUTED LINE IS A TRANSFORMATION PIPELINE, NOT A SINGLE REWRITE

The unexplained bucket produced a FIFTH transformation -- tilde and variable
expansion in the ARGUMENTS, which the classifier missed because it only compared
program names:

    cd ~/0-core           -> cd /home/christian/0-core        2,137
    git -C ~/0-core add   -> git -C /home/christian/0-core      100
    sqlite3 ~/0-core/...  -> sqlite3 /home/christian/0-core/...  98
    echo $HOME            -> echo /home/christian                83

And a COMPOSED case that matched no single rule:

    d -> core doctor run                                        575

because `d`'s alias is the fully-resolved `/run/current-system/sw/bin/core doctor
run` -- alias expansion AND path resolution, composed. The classifier saw the
composite and matched neither half.

★ THE ARCHITECTURAL CONCLUSION: the executed line is the CUMULATIVE RESULT of a
pipeline, not one rewrite:

    typed
      |
      +-- alias expansion
      +-- plugin / wrapper resolution
      +-- path resolution
      +-- tilde expansion
      +-- variable expansion
      +-- (command substitution, globs, ... whatever the shell learns next)
      |
    executed

So the schema must record the ENDPOINTS, not enumerate the transformations.
Encoding the reason would mean extending an enum forever -- and the list has
already grown twice in one session, from one mechanism to five. `typed` and
`executed` are stable; the pipeline between them is an implementation detail free
to change without a schema migration.

★ THIS SETTLES THE SCHEMA QUESTION. A command lifecycle has exactly two
observable endpoints, and that stays true as the pipeline grows. That stability is
the sign it models something fundamental rather than tracking today's
implementation -- which is precisely why one-row-per-command wins over an event
log or an event_kind column.

### ⚠️ METHODOLOGICAL LESSON (applies beyond this intent)

The streaming analyzer succeeded where four SQL attempts each produced a different
"truth" because it MODELLED THE PROCESS RATHER THAN THE DATA. The shell processes
commands sequentially with state; the analyzer did the same. The SQL attempts tried
to infer that state from sets of rows, which is why they disagreed with each other.

RULE: when analyzing shell execution history, a streaming model aligns with the
system; a purely relational one fights it.

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

## Producer-side repair, 2026-07-26 -- prerequisite work, no gate ticked
None of the criteria below moved. They are all about the history TABLE, and this
work was upstream of it: the producer had to become honest before the table could
be migrated, or the migration would faithfully preserve a false field and lock the
mistake into the new model.

WHAT WAS FOUND. `ExecContext` already declared the two-endpoint model -- `raw`
documented as exactly what the user typed, `expanded` as the form after alias
resolution -- but `from_line` took ONE argument and filled both fields with it,
under a comment promising an update after alias resolution that never ran. Callers
passed a line that had already crossed the execution boundary. So `postexec`
writing `ctx.raw` was never misunderstanding the field; it was faithfully
recording a value that arrived pre-corrupted. The struct never lacked the shape.
The data flow had never caught up to it.

⚠️ AND A NAMING TRAP WORTH RECORDING: `original_line` in the REPL loop is captured
AFTER variable, subshell and glob expansion. Its name means "original relative to
the pipeline rewriting that follows", not "what the user typed". Using it as `raw`
would have committed this intent's own bug by trusting a variable name. The true
user boundary is the segment as it stands before any expansion.

WHAT CHANGED (commits 96d535dc, edf742bf):
  - `from_line(raw, expanded, db)` and `execute_with_context(raw, expanded, ...)`.
    Both REPL call sites now name their stages explicitly.
  - `cmd` and `args` derive from EXPANDED. They are execution identity, and
    preexec's protected-path predicate reads `cmd`.
  - The catastrophic `rm -rf` guard became `blocks_catastrophic_rm(&ExecContext)`,
    reading `cmd` and `expanded` itself so no caller can pass provenance where
    execution text belongs. Six tests; the lock is typed `nuke` / expanded
    `rm -rf /home` -> blocked, which launches no process.
  - `spine migrate` passes `(source, source)`: an INTENTIONAL collapse, because
    that audit compares parsers on identical unexpanded input.

⚠️ THIS WAS NOT BEHAVIOUR-PRESERVING, and the intent should not pretend otherwise.
Restored: postexec still records the executed form, now via `ctx.expanded`. Changed
on purpose: `last_failed_command`, the failure log, and the prediction LIKE
patterns now receive what the user typed rather than the expanded string they got
by accident. Defensible -- provenance should answer what the user asked for -- but
a change. The prediction patterns are the lowest-confidence of those, since they
query a table that currently holds BOTH forms as separate rows; that path is due to
be restructured by the migration phase below.

★★ TWO REGRESSIONS WERE INTRODUCED BY THIS CHANGE AND CAUGHT BY AUDIT, NOT BY
TESTS. Both compiled cleanly. `postexec` would have recorded the typed form,
destroying the executed-form record by redefinition rather than deletion. And
preexec's `rm -rf` scan silently began reading the typed line, so an alias
expanding to a catastrophic command would have presented `cmd = rm` while the scan
saw only the alias name and blocked nothing. A MEANING CHANGE PASSES THE COMPILER
AND FAILS THE SYSTEM: the consumer audit had to precede any test run, and it is
what caught a safety gate being disarmed.

WHAT THE SPLIT BUYS. Collapse is still possible but can no longer be SILENT.
Before, one string filling both fields was the default, and an intentional equality
was indistinguishable from an accidental one. Now a caller has to write it, and the
single place that does carries its justification in place.

⏭ REMAINING PHASES, in order: introduce the record type; route both observations
through one recorder; build the READ-ONLY analyzer producing paired / ambiguous /
unchanged; create the new storage and migrate only proven pairs; move consumers off
raw history and off `id + 1`; archive `shell_history`.
⚠️ The migration must not INVENT data. Historical rows reliably yield typed,
executed and timestamps; they may not yield duration, exit_code or cwd depending on
which writer produced them. Distinguish KNOWN from UNKNOWN and leave unknowns empty
-- filling them with guesses would recreate INT-189 in a different table.

## Execution lifecycle producer, 2026-07-26 -- prerequisite complete, no gate ticked
The remaining criteria are phrased around the MIGRATION OUTCOME, not around the
existence of a replacement producer. That distinction is why none of them move.

THE CORRUPTION, MEASURED RATHER THAN SUSPECTED. `last_history_id` is captured from
the SUBMISSION insert at main.rs:1171, and `postexec` DISCARDS the id of the row it
writes, so the completion update at the top of the next REPL iteration lands on the
typed row. Live: `shell_history` says `c` exited 0 in 96ms while the process that
ran was `clear`. 50,293 rows carry completion metadata and at least 15,957 are bare
alias names -- a FLOOR, not the rate, since the alias test cannot see the
`cd ~/0-core` class, which misattributes identically via tilde expansion.
⚠️ A migration would have preserved this faithfully. The producer had to be
repaired before the table could be reconstructed.

THREE CONCEPTS IN ONE TABLE. `shell_history` carries SUBMISSION (the user entered
this, written before safety checks and before segment splitting), EXECUTION (the
producer ran this, written per segment), and ENRICHMENT (attach exit and duration
to the anchored row). For `a; b` the counts do not even match: one submission, two
executions. The classifier was being asked to infer a boundary the producer never
recorded.

THE IDENTITY EXISTS BEFORE THE DATA. That inversion is the whole change. The old
design derived both command identity and ordering from SQLite insertion order --
`id + 1`, `last_history_id`, `ORDER BY id DESC` -- which is why every consumer
eventually had to guess. Now `execute_with_context` opens a row before anything can
return, and storage is a PROJECTION of a lifecycle rather than a lifecycle inferred
from storage.

⚠️ THE KEY IS THE PAIR, and the recon caught this before it was written.
`execution_id` is an AtomicU64 starting at 1 in EVERY shell process -- its own doc
says so. Persisting it alone would have let two sessions both claim 1, 2, 3 and
silently overwrite each other: a key that looks unique and is not, which is the
exact defect class this intent exists to remove. `session_id` supplies the process
boundary. Nothing owned that question before -- `FSH_SESSION_ID` is read in three
places and set in NONE, which is why `term_commands` holds 42,376 rows under the
fallback string "unknown". Absence should TRIGGER CREATION, not become a value.

POSTEXEC CANNOT BE THE OWNER. Established by recon, not assumed: it never runs for
a blocked command, because `execute_with_context` returns early when preexec
blocks, and it deliberately skips `exit`. Those are the two events most worth
recording. So the row opens at context creation and closes wherever the outcome
becomes known.

TWO PHASES BECAUSE THERE ARE TWO OWNERS. postexec knows the executed form; only the
caller knows the final exit code, since the pipeline arms repaired in INT-189
decide it after `execute_with_context` has already returned. `begin` and `complete`
are separate methods for that reason -- one `save` would have hidden the boundary.
`ExecutionOutcome` carries the identity back out rather than hoisting generation to
the caller, because `from_line` is also used by the `spine migrate` audit, and that
path would then have to supply an execution id for something that never executes.

★ THE NULLS MEAN THINGS. `executed_text` is null when the command never reached
expansion. `exit_code` is null for `exit`, deliberately: that arm never sets
`last_exit_code`, so passing it would record the PREVIOUS command's result -- the
stale-value bug INT-189 removed. `duration_ms` is null on the VAR=value path
because it has no timer, and inventing one is the guess the migration rule forbids.
A row left in state `started` is EVIDENCE the shell died mid-command. Completion
refuses to update zero rows, so it cannot float unattached to a lifecycle.

PROVEN ON METAL, which the unit suite cannot do: `gs` recorded `typed_text='gs'`
with `executed_text='git status'`; a failing command closed as `error` with
`exit_code=1` ON ITS OWN ROW; `exit` closed with `exit_code=None`; execution ids
1..4 monotonic under one session; nothing left in `started`.

🔎 AND THE TABLE IMMEDIATELY MADE SOMETHING VISIBLE: `git status` printed output but
classified as `empty`, so `CommandResult::Empty` conflates "nothing happened" with
"output went straight to the terminal rather than being captured". That is INT-192's
thesis inside fsh's own types. Follow-up, not a regression.

⏭ NEXT IS CONSUMER MIGRATION, not more producer work: inventory every
`shell_history` READ and classify it (submission/display, execution analytics,
suggestions/learning, audit, compatibility); move the EXECUTION consumers first,
starting with the daemon's `id + 1` join; let `shell_history` keep only whatever
concept genuinely remains its own; and only THEN judge the duplication gate, whose
real question is "are two tables storing the same fact?" rather than "do two tables
exist?"

## What `command_execution` represents -- decided 2026-07-26
USER-VISIBLE EXECUTIONS ONLY. Command substitution is an EXPANSION-PHASE
operation and must NOT create child lifecycle records unless a future explicit
expansion-tracing model is introduced.

The temptation is real: once a lifecycle table exists with a stable identity, any
internal invocation can be given a row. Resist it. `echo "branch: $(git branch
--show-current)"` is ONE command the user typed. Recording the substitution as a
sibling would pollute history with text nobody entered, inflate execution counts,
make duration attribution ambiguous, and break the single-lifecycle property the
schema was chosen for -- and nested substitutions would demand a TREE this table
does not model.

If expansion tracing earns its place later it is a DIFFERENT concept -- an
expansion trace or a child-event stream -- not substitutions pretending to be
commands. The rule that keeps this honest is the same one that produced the
schema: record what actually occurred at the boundary where you know it, and do
not let a convenient table become the place hidden work accumulates.

⚠️ The child execution still needs correctness guarantees even without a row: no
mutation of parent shell state, no alias re-expansion from raw text, no bypass of
the safety guards, deterministic capture. That argues for a SHARED INTERNAL
EXECUTOR, not for lifecycle rows.

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

## Deferred 2026-08-17

Moved back to future/ because fsh work is on a deliberate break with the deadline TBA,
while the Zero Core restructuring takes priority. Not blocked and not abandoned -- paused,
with the live fsh bugs still filed. Deferred to respect the cap of three active intents
(decision 142); Friday flagged the contradiction independently.
