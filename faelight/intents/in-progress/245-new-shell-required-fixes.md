---
id: 245
date: 2026-09-09
type: future
title: "New Shell required fixes"
status: in-progress
tags: [nsh, shell]
---

## Vision

A pipeline stage that could not do its job says so, and the reason survives to the terminal
and to the next stage. `ps | reduce median cpu` tells you `median` is not an aggregate this
shell knows. It does not print a blank line.

## The Problem

`Value` has seven variants and none of them means "this did not work":

    Text · Int · Float · Bool · Row · Table · Nothing

So `Nothing` carries two meanings that must never be one value: THE ANSWER IS GENUINELY EMPTY,
and I COULD NOT PRODUCE AN ANSWER. Both render as the empty string, in both renderers.

### Measured 2026-09-10, value.rs

Four sites mint `Value::Nothing`. One is honest and three are refusals wearing an absence:

    509  row.get(field) missing            HONEST -- the field is genuinely not in the row
    624  reduce with no field ("reduce")   REFUSAL -- the expression is malformed
    636  no numeric values in that field   AMBIGUOUS -- empty table? wrong field name?
                                           unparseable text? three facts, one answer
    643  unknown aggregate ("median")      REFUSAL -- the shell does not implement it and knows so

`ps | reduce median cpu` and `ps | reduce sum cpu` against an empty table produce the SAME
VALUE. The first is "I do not know that word". The second is "the answer is nothing".

### And the pipeline has no error channel at all

    pub fn apply_pipeline(value: Value, ops: &[PipeOp]) -> Value

It cannot fail. Every op returns a `Value`, so a stage that cannot do its job must return one
of the seven -- and `Nothing` is the only one that fits, which is why the collapse is here.

### Both renderers flatten it

    value.rs:101   to_pipe_text   Nothing => String::new()
    value.rs:122   render         Nothing => String::new()

So a refusal is silent at the terminal AND silent down a pipe to an external command. A user
who types `reduce median cpu` sees the same thing as a user whose query matched nothing.

## THE CLASS -- fourth sighting in one week

This is not a new defect. It is the same one, in the layer nobody had looked at yet:

| where | the two-state answer | what was missing |
| --- | --- | --- |
| doctor checks (INT-192) | `Vec<T>` | could-not-check vs clean |
| `spine migrate` summary | one total | ruled vs blocking |
| `nsh-test` results | `passed: bool` | could-not-run vs did-not-pass |
| **the value pipeline** | **`Value::Nothing`** | **could-not-compute vs empty** |

Three of those four were found and fixed in the week of 2026-09-05. The pattern is that EVERY
LAYER OF THIS SHELL REACHES FOR A TWO-STATE ANSWER WHEN THE TRUTH HAS THREE, and it is found
afterwards each time. `faelight_core::check` already carries the type -- `Skipped {subject,
reason}` and `Checked<T>` -- but it was built for CHECK OUTCOMES and has never been offered to
anything else.

★ AND NO OTHER STRUCTURED SHELL HAS THIS. Nushell's answer is `Value::Error`: something went
wrong, it flows through the pipe, it short-circuits. That conflates "I do not know that word"
with "I could not reach the process table", which is exactly the distinction the doctor
already makes and the pipeline does not. A pipeline that can carry I COULD NOT LOOK as a value
distinct from both EMPTY and ERROR is not a thing Nu built. This may be the most original idea
in the shell, and today it exists in one crate and one consumer.

## The Solution

### The open design question -- ONE new state or TWO

Not settled, and it must be settled before any code:

  A. ONE state, carrying a reason. `Value::Unknown(Skipped)` or similar. Simple; makes
     "unsupported aggregate" and "could not read the source" the same kind of thing.

  B. TWO states. A REFUSAL (the shell knows exactly what is wrong: `median` is not an
     aggregate) and a SKIP (the shell could not look: the process table was unreadable). The
     doctor already distinguishes these -- Fail vs Unknown -- and it distinguishes them because
     they need different responses from a reader.

## RULED 2026-09-10 (Christian): ONE STATE.

The pipeline gets a single new variant carrying `Skipped {subject, reason}` -- the type
`faelight_core::check` already owns, so no second owner is created.

THE REASON, recorded so a future reader does not re-open it: every site measured today is a
REFUSAL, and not one is a SKIP, because no value SOURCE currently reports that it could not
read its input. `ps` cannot say "I could not read /proc"; it either produces a table or does
not run. The second state has nothing to describe yet.

⭐ AND WIDENING LATER IS A COMPILE ERROR. The day a source learns to report that it could not
look is the day the second state is earned, and adding it then visits every match site because
it has to -- the same move INT-169 made twice with `CommandResult::Error` and `::Exit`. Taking
two states now would mean guessing which sites are skips before any exist, which is designing
against an imagined caller.

### 636 is a FIX, not a classification

"No numeric values in that field" is three facts today. Before classifying it, split it: a
field that is not present in any row is a different answer from a field present and
unparseable, and the code can still tell them apart at that point. Do not label an ambiguity
that can be removed.

### Scope, measured

    5 files touch Value: value.rs, commands/mod.rs, engine.rs, main.rs, tests/pipeline.rs
    42 CommandResult::Value producers in commands/mod.rs
    1 render entry point (value.rs:107) plus to_pipe_text

⭐ WIDEN THE ENUM, DO NOT ADD A PARALLEL CHANNEL. `CommandResult` records the argument twice
already: a new variant is swallowed by `_` arms, while widening makes every site a COMPILE
ERROR -- completeness proven rather than audited. Same reasoning, same file, same author.

### The external boundary is a SEPARATE question and is already ruled

`gc | grep feat` -- what an external command receives -- is NOT this intent. The ruling
already exists in the crate notes: externals and `sh { }` produce and consume BYTES; `from ps`
/ `from ls` are the only doors from bytes back to structured data. This intent is one layer
in: what the pipeline can EXPRESS, not how it serialises at the edge.

## Success Criteria

- [x] The four Nothing sites are re-read and each is classified refusal / honest-absence /
      ambiguous, with the deciding lines quoted. 636 is SPLIT before it is classified
<!-- DONE 2026-09-12. Line numbers from value.rs at 9ab878e9; the earlier note said 509/625/637/644
     and the code has shifted by one since.

     508  HONEST ABSENCE.  rows[0].get(field).cloned().unwrap_or(Value::Nothing)
          `Get` against a SINGLE row. The field is genuinely not in that row. Nothing is hidden and
          nothing is refused -- this is what Nothing is for.

     624  REFUSAL.  expr.splitn(2, ' ') produced fewer than 2 parts
          `reduce` typed with no field. The shell knows exactly what is wrong: the expression is
          malformed. It answers with the same value as an empty result.

     643  REFUSAL.  the `_` arm of match agg { "sum" | "avg" | "min" | "max" }
          `reduce median cpu`. The shell knows `median` is not an aggregate it implements, and says
          nothing. This is the example in the Vision above.

     636  ⭐ SPLITS FOUR WAYS, NOT THREE. The earlier note called it three facts; reading line 628
          finds four, and only the first is honest:

            a. `rows` is empty                     -- nothing to reduce. HONEST EMPTY.
            b. field ABSENT from every row         -- `r.get(field)?` returns None at 628. The user
                                                      named a column that does not exist. REFUSAL.
            c. field present, Text that won't parse -- `s.parse::<f64>().ok()` is None at 631. That
                                                      column is not numeric. REFUSAL.
            d. field present, Row/Table/Bool        -- `_ => None` at 632. Same as (c) but the type
                                                      is wrong rather than the text. REFUSAL.

          THE COLLAPSE IS AT 628, NOT 636. `filter_map` with `?` inside erases the difference
          between "no such field" and "field is not a number" before 636 ever runs, and 636 then
          erases the difference between those and "no rows". By the time a value is returned, four
          answers have become one.

     ⚠️ (b) AND (c) ARE DIFFERENT REFUSALS AND MUST STAY DIFFERENT. "No such column" and "that
     column is not numbers" need different responses from a reader: one is a typo, the other is a
     misunderstanding about the data. Collapsing them into a single Unknown would rebuild this
     defect one level up, which is the trap this intent's own 636-is-a-fix section warns about.

     ⭐ THE FIX IS TO COUNT, NOT TO CLASSIFY. Lines 626-634 can tell all four apart today by
     counting instead of filtering: how many rows carried the field at all, and how many of those
     parsed as numbers. rows.len() == 0, present == 0, and parsed == 0 with present > 0 are three
     distinct observations available at that point and thrown away. -->
- [x] ONE state or TWO is DECIDED and written into this intent, with the reason. A decision to
      start with one and widen later is a valid discharge -- declining with reasons is proof
<!-- DECIDED 2026-09-10 by Christian: ONE state. See the RULED section above for the reason --
     no value source can currently report that it could not read its input, so the second state
     has nothing to describe. Widening later is a compile error, so it stays available. -->
- [ ] `Value` carries the decided state, and the widening is proven by the compiler rather
      than by audit: every match site is visited because it had to be

## THE WORKED EXAMPLE -- `ls` on a file, found 2026-09-10

The instances above are query-language edges. This one is `ls`, which every user types, and it is
the clearest statement of the defect in the whole intent.

    commands/mod.rs:7410  fn sys_files(...)   -- the `files` / `ls` builtin
    commands/mod.rs:7424  std::fs::read_dir(&path).ok()
    commands/mod.rs:7453  .unwrap_or_default()

`read_dir` on a FILE fails. `.ok()` discards WHY. `unwrap_or_default()` turns the failure into an
empty Vec, which becomes `Value::Table(vec![])`, which renders as "No results." at value.rs:174.

So `ls somefile.txt` reports AN EMPTY DIRECTORY for a file that exists and is two bytes long.

### Why nobody had seen it

    nsh> alias | grep ls
    ls = eza --icons=auto   shadows builtin

The alias shadows the builtin on this machine, so `ls` has always been eza, which handles files
correctly. Inside the devbox sandbox NSH_CONFIG points at a config that does not exist, no aliases
load, and `which ls` reports the forest builtin FIRST and /usr/bin/ls second. The builtin wins,
and the lie appears.

    devbox: ls ~/.local/state/faelight/state.db   -> "No results."
    devbox: ls ~/.local/state/faelight            -> a table listing state.db, 2 bytes, file

⭐ THE INSTRUMENT FOUND IT, WHICH IS THE POINT. This was invisible for as long as the suite only
ran on one machine with one config. Converting nsh-test to run against a fixture (phases 1-4,
2026-09-10) surfaced it on the first clean-room run. `state_db_exists` is LEFT RED in the suite
with the reproduction in a comment, deliberately, rather than rewritten to ask an easier question.

### AND THE HONEST FIX HERE MAY NOT BE A THIRD STATE

⏭ Not ruled. `ls <file>` arguably should LIST THE FILE, the way every other ls does, and then the
third state is reserved for when it genuinely cannot look -- a directory it lacks permission to
read, say. A refusal is the right answer when the shell cannot do what was asked; listing one file
is something it can do and simply does not. Deciding that is part of this intent's work, and the
answer may be BOTH: list the file, and use the new state for the permission case that `.ok()`
currently swallows too.
- [ ] BOTH renderers say what happened. `render` and `to_pipe_text` must not print an empty
      string for a refusal -- that is the defect, and fixing the enum without fixing the
      renderers changes nothing a user can see
- [ ] Proven by watching it fail: `ps | reduce median cpu` names the unsupported aggregate,
      and `ps | where cpu > 999 | reduce sum cpu` still reports an honest empty result. The
      two must not look alike
- [ ] The pipe boundary is decided: what an external command receives when the value is
      "could not compute". Silence is a choice and must be an explicit one
- [ ] `$?` is decided for a refusing pipeline. A stage that refused did not succeed, and the
      exit status is how a script finds out
- [ ] Nothing here becomes a second owner. `faelight_core::check` already has the type; use it
      or state why the pipeline needs its own

## Relationship

- INT-192 built `Checked<T>` / `Skipped` and proved the shape on the doctor. This is that
  type's second consumer, and the intent that tests whether it generalises
- INT-322 built the pipeline stages this intent widens
- The crate notes rule the EXTERNAL boundary (bytes, one `from ps` door). Not this intent
- Raised indirectly by the 2026-09-10 Omarchy/beta review item 7, which asked what `grep`
  receives. That question has a settled answer. Reading the code to answer it found this,
  which does not
