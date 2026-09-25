---
id: 248
date: 2026-09-10
type: future
title: "a lifecycle record is silently dropped on a duplicate session_id and execution_id pair"
status: planned
tags: [nsh, history, lifecycle, telemetry]
---

## Vision

Every command that runs has exactly one lifecycle record, opened once. When a record cannot be
written, the shell says so in a way somebody will act on -- not a warning that scrolls past while
the table quietly gains a hole.

## The Problem

Observed 2026-09-12, in ordinary use, running `find` inside an nsh pipeline:

    warning: failed to open command_execution record: UNIQUE constraint failed:
    command_execution.session_id, command_execution.execution_id

That is the lifecycle table rejecting a duplicate on EXACTLY the pair INT-191 chose as its
identity. The command still ran. The record was not written. Nothing else happened.

### TWO OPENERS, TWO SOURCES OF execution_id

    exec.rs:1775   begin_command_execution { session_id: ctx.session_id,
                                             execution_id: ctx.execution_id,  <- ExecContext
    main.rs:1563   begin_command_execution { session_id: crate::exec::session_id(),
                                             execution_id: lifecycle_exec_id, <- a separate binding

Same session, two independent ideas of which execution this is. A line that reaches both paths
opens the record twice, and the second insert is refused.

⭐ THIS IS THE TWO-OWNERS SHAPE, IN THE FUNCTION INT-191 BUILT TO ESTABLISH ONE IDENTITY. The
constraint is not the bug; the constraint is the only reason anybody found out.

### THE DATA IS NOT CORRUPT -- IT IS INCOMPLETE

    SELECT COUNT(*), COUNT(DISTINCT session_id) FROM command_execution;
      -> 15909 rows, 2142 sessions

    SELECT session_id, COUNT(*) c, COUNT(DISTINCT execution_id) d
      FROM command_execution GROUP BY session_id HAVING c != d;
      -> (no rows)

Not one session among 15,909 rows holds a duplicate pair. The constraint held every time. So
nothing in the table is wrong -- ⚠️ COMMANDS ARE MISSING FROM IT, and there is currently no way to
say how many, because a dropped row leaves no trace except a line on stderr.

### WHY THIS MATTERS MORE THAN A WARNING SUGGESTS

INT-191's own criterion was that lifecycle evidence is PRODUCED RATHER THAN INFERRED. A silently
dropped record is that gate leaking: Friday reads this table, the three-failures detector reads
this table, and a command that never got a row is invisible to both. The shell is learning from a
record with gaps it cannot see.

⚠️ AND THE WARNING IS THE WRONG SHAPE FOR THE FAILURE. It goes to stderr, execution continues, and
the user sees it flash past mid-pipeline. Same class as everything INT-192, INT-245 and INT-246
have been about: something that could not be done, reported in a way that does not survive.

## The Solution

### The question to answer FIRST, before any code

WHICH LINES REACH BOTH OPENERS? Neither path is obviously wrong on its own. main.rs opens a record
at the REPL level; exec.rs opens one per executed command. For a simple line those may be the same
event described twice, or two genuinely different events that should each have a record under
DIFFERENT execution ids.

⏭ NOT RULED, and it must be, because the two answers lead to opposite fixes:
  A. ONE EVENT, TWO OPENERS  -> delete one. A REPL line and its execution are the same lifecycle,
     and whichever opener is redundant goes.
  B. TWO EVENTS, ONE ID      -> keep both, fix the id. A pipeline of three commands is arguably
     three executions inside one typed line, and the ids should differ rather than collide.

The measurement decides it: instrument both sites, run one line that triggers the warning, and see
what each was trying to record.

### The failure must not be silent whichever way it goes

Independent of A or B: a lifecycle record that cannot be written is not a warning. Options to
decide between -- surface it in `d` / doctor as a countable health fact, count the drops in the
session so `history` can say the record is incomplete, or fail loudly. Silence is the one answer
that is already known to be wrong.

## Success Criteria

- [ ] The lines that reach BOTH openers are identified and named here, with the command text and
      which path each took. A hypothesis is not a finding
- [ ] ⭐ RULED: A or B. One event with two openers, or two events sharing an id. Written into this
      intent WITH the reason, because the two answers lead to opposite fixes and a future reader
      must not have to re-derive it
- [ ] If A: the redundant opener is DELETED, not guarded. A second owner kept behind an `if` is
      still a second owner -- INT-193 spent a session on exactly that shape in the alias path
- [ ] If B: execution_id is unique per execution and the pair collides for nothing. Proven by
      running the line that produced the warning and seeing two rows with different ids
- [ ] The reproduction is written down and repeatable. Found by accident during a `find` in a
      pipeline; an intent that cannot reproduce its own bug cannot prove it fixed it
- [ ] Zero occurrences of the warning across a full nsh-test run and a full interactive session.
      Evidence: the run, and the absence
- [ ] ⚠️ A DROPPED LIFECYCLE RECORD IS COUNTABLE. Whatever the fix, the shell can answer "how many
      records failed to open this session" rather than leaving the answer on a stderr line nobody
      kept. A hole that can be measured is a different thing from a hole that cannot
- [ ] The existing 15,909 rows are checked for gaps ONCE, and the finding recorded either way.
      If commands are missing, say roughly how many; if the count cannot be established, say that
      instead -- and do not guess

## Relationship

- INT-191 built `command_execution`, the (session_id, execution_id) identity, and the constraint
  that caught this. It is COMPLETE and stays complete -- this is a NEW defect found after it
  shipped, not a reopening of its work. Recorded that way deliberately: the intent audit found 123
  intents marked done that were not, and reopening a finished one to absorb a later bug is how
  that happens
- INT-192 / INT-245 / INT-246 share the failure mode: something that could not be done, reported
  in a form that does not survive contact with a user. Here it is a warning on stderr instead of a
  countable fact
- Friday reads this table. A gap in it is a gap in what the shell believes about its own history
