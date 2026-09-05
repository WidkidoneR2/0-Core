#!/usr/bin/env python3
"""
Write full bodies and gates for INT-241, 242, 243 and 244.

⚠️ WHY THIS SCRIPT EXISTS. Four intents were filed on 2026-09-05 as TITLE-ONLY
SCAFFOLDS with no gates -- which is precisely the hole INT-212 exists to close
("cicomplete blocks open gates but never requires gates to EXIST"), and it is
the door the 123-intent audit came through. They were filed to avoid folding
unrelated work into INT-230, which was right on scope and wrong on the ledger:
a stub is a promise with no receipt.

Every body below is written from evidence gathered while the defect was found --
reproductions, line numbers, measured counts, and in 244's case a git-stash
proof that the defect predates today's work. That evidence is what a stub lacks
and it is what goes stale first.

Each intent gets: The Problem (with the measurement), The Solution, Success
Criteria with real gates, and Non-goals. INT-158's evidence rule is in the
template already; these carry gates it can attach to.
"""

import glob
import io
import sys


def write_body(number, body):
    matches = glob.glob("faelight/intents/future/" + number + "-*.md")
    if len(matches) != 1:
        print("ABORT: " + number + " matched " + str(len(matches)) + " files", file=sys.stderr)
        sys.exit(1)
    path = matches[0]
    with io.open(path, "r", encoding="utf-8") as fh:
        text = fh.read()

    # Keep the frontmatter, replace everything after it.
    parts = text.split("---", 2)
    if len(parts) < 3:
        print("ABORT: " + path + " has no frontmatter fence", file=sys.stderr)
        sys.exit(1)
    frontmatter = "---" + parts[1] + "---\n"

    with io.open(path, "w", encoding="utf-8") as fh:
        fh.write(frontmatter + body)
    print("wrote " + path)


# ============================================================ INT-241
write_body("241", """
## Vision
`$PWD` names the directory the shell is in. Anything reading it gets the same
answer `pwd` gives.

## The Problem
Found 2026-09-05 while running the test suite. `NSH_BIN=$PWD/target/debug/nsh`
sent the harness to `/home/christian/target/debug/nsh` -- the `0-core` segment
was missing.

Measured directly, in one nsh session:

```
cd ~/0-core
pwd            -> /home/christian/0-core
echo $PWD      -> /home/christian
cd /tmp
echo $PWD      -> /home/christian
```

**`$PWD` is not stale, it is never updated at all.** nsh inherits it from the
bash process that `exec`s it and never touches it again, so it reports the
directory bash was in at login for the entire life of the shell. `cd` does not
update it.

### Why this matters beyond one paste
POSIX requires the shell to set `PWD` on every `cd`, and bash does. Anything
that reads it in nsh gets a wrong answer:

- every script using `$PWD` to build a path
- every `$(...)` substitution that shells out and reads it
- prompt tooling and editor hooks that use it to find the project root
- ⚠️ **a second user's first script**, which is the INT-230 packaging case

📍 It also cost real time in this session: it silently redirected a test harness
to a binary that does not exist, and nsh-test correctly refused to fall back to
the deployed shell -- so the failure was loud, but only because that guard had
already been built.

## The Solution
Set `PWD` in the shell's own environment whenever the working directory
changes, so a child process and a `$PWD` expansion see the same value `pwd`
reports.

⚠️ **THE OWNER QUESTION FIRST.** `set_current_dir` appears at several sites in
`commands/mod.rs` (cd, z_jump and relatives). Setting `PWD` beside each of them
would create the same multi-owner shape this ledger keeps removing. The
directory change and the variable update must have ONE owner, so a caller
cannot move the shell without updating the variable.

📍 `OLDPWD` is the sibling question -- bash maintains it and `cd -` depends on
it. Decide whether it is in scope before building, not after.

## Success Criteria
- [ ] G1 RED FIRST: the divergence is captured verbatim before any fix --
      `pwd` and `echo $PWD` disagreeing, and `$PWD` unchanged across a `cd`
- [ ] G2 EVERY site that changes the working directory is ENUMERATED, not
      grepped for one pattern. The census names each one and what it is for
- [ ] G3 ONE OWNER: the directory change and the `PWD` update cannot be done
      separately. A caller that moves the shell cannot forget the variable
- [ ] G4 `echo $PWD` agrees with `pwd` after: cd, cd -, z_jump, a relative cd,
      a cd through a symlink, and shell startup
- [ ] G5 A CHILD PROCESS SEES IT: `printenv PWD` from a spawned command matches,
      because that is the case that broke the harness
- [ ] G6 Regression test in nsh-test asserting the agreement, driven through the
      REPL door rather than `-c` -- both doors if they can diverge
- [ ] G7 A ruling recorded on `OLDPWD`: implemented, or explicitly out of scope
      with the reason
- [ ] G8 each gate carries evidence per INT-158

## Non-goals
- `cd -` behaviour itself, unless G7 rules OLDPWD in scope.
- Symlink resolution policy (whether `PWD` is logical or physical). Match what
  `pwd` already reports; changing that is a separate decision.
""")


# ============================================================ INT-242
write_body("242", """
## Vision
A command that reports success has changed something. `flow focus` sets the
focus that every reader reads.

## The Problem
Found 2026-09-05 while migrating the shell's intent readers onto the INT-230
adapter. **`flow focus` writes one storage and every reader reads another.**

```
db::set_focus_intent   ->  INSERT INTO shell_state (key='focus_intent', ...)
db::get_focus_intent   ->  reads ~/.local/state/0-core/intent/focus.toml
```

A setter and a getter sharing a name and not a storage. Measured: `shell_state`
contains **no `focus_intent` key at all**, while `focus.toml` holds
`id = "230"`, written by `cistart` at 23:51:17.

### Three defects, not one
1. **`flow focus INT-231` prints `focus set -> INT-231` in green and changes
   nothing any reader observes.** A success message for a write nobody reads.
2. **`flow clear` clears a key nobody reads**, so focus survives being cleared.
3. **The formats differ.** `flow` requires and stores `INT-231`; `focus.toml`
   stores `230` bare, and `main.rs:2547` does
   `get_focus_intent().map(|i| format!("INT-{}", i))` -- confirming readers
   expect the bare form. Even sharing a storage, the values would not match.

### Who reads focus
Eleven call sites use `get_focus_intent`, including `db.rs:329`, which stamps
**every shell-history row** with it. So the value is not cosmetic; it is
attached to the historical record.

### The source of truth is not in doubt
`engine/src/domains/friday/attention.rs:98` says it outright: *"read focus.toml
(written by cistart, source of truth)"*. The file carries id, title, started and
workflow.

## The Solution
`focus.toml` wins -- it is what `cistart` writes, what the engine calls
canonical, and what every reader already reads.

⭐ **SO THE FIX IS MOST LIKELY A DELETION**, which is the shape that has been
right repeatedly in this codebase: remove `set_focus_intent` and
`clear_focus_intent`, and have `flow focus` / `flow clear` either write
`focus.toml` through one owner or refuse and point at `cistart`.

⚠️ **A RULING IS NEEDED BEFORE CODE**: should `flow focus` be able to set focus
at all, or is focus something only `cistart` sets? If the shell may set it, the
write needs the same owner the engine uses -- a second writer of `focus.toml`
would reproduce this defect in a new place.

📍 `db.rs:495` hand-builds the focus.toml path from `env::var("HOME")` and never
asks `paths.rs`. INT-230 added `core_integration::focus()` which routes through
`paths`, so there are currently THREE readers of that file. Consolidating them
is part of this.

## Success Criteria
- [ ] G1 RED FIRST: `flow focus INT-999` prints success, and `flow status`
      afterwards reports something else. Captured verbatim
- [ ] G2 THE RULING IS RECORDED: may the shell set focus, or is that `cistart`
      only? Written here before any code
- [ ] G3 ONE STORAGE. Whatever the ruling, exactly one location holds focus and
      `grep` proves no second writer exists
- [ ] G4 ONE READER. The three current readers of focus.toml
      (`db.rs:495`, `commands/mod.rs:14753`, `core_integration::focus`) become
      one, routed through `paths.rs`
- [ ] G5 THE ID FORMAT IS SETTLED and stated: bare `230` or prefixed `INT-230`,
      with every producer and consumer agreeing
- [ ] G6 `flow clear` demonstrably clears what `flow status` reads
- [ ] G7 If `flow focus` is retained, a regression test asserts that setting
      focus is observable by `get_focus_intent` in the same session
- [ ] G8 each gate carries evidence per INT-158

## Non-goals
- The focus feature's design. This is about a write and a read disagreeing.
- INT-230's adapter boundary. `core_integration::focus()` reads correctly today;
  it is one of the three readers this intent consolidates.
""")


# ============================================================ INT-243
write_body("243", """
## Vision
Piping any command into `head` ends the pipeline. It does not kill the shell and
it does not panic.

## The Problem
Found 2026-09-05 while proving an unrelated fix:

```
nsh -c "dash forest" | head -4
->  thread 'main' panicked at library/std/src/io/stdio.rs:1166:9:
    failed printing to stdout: Broken pipe (os error 32)
```

`head` exits after four lines and closes the pipe; the next `println!` panics.

### ⚠️ THIS IS INT-299's SYMPTOM, RETURNING BY A DIFFERENT ROUTE
INT-299's comment records the original: *"`ls ~/path | head -5` would previously
panic with 'failed printing to stdout'"*. Its arch-era fix was a process-wide
`signal(SIGPIPE, SIG_DFL)`, which traded a visible panic for a SILENT FATAL
SIGNAL -- and that was corrected on 2026-08-21 by removing the process-wide
reset and restoring `SIG_DFL` per child in `spawn_pipeline`'s `pre_exec`.

⭐ **THAT FIX ASSUMED THE SHELL NEVER WRITES INTO A CLOSED PIPE**, because both
pipeline stages are spawned as real children with real pipes. **A PRINTING
BUILTIN BREAKS THAT ASSUMPTION**: it writes to stdout directly with `println!`,
inside the shell process, so there is no child to take the signal.

### The connection to the structured pipeline
`peel_builtin_first_stage` handles a builtin at the head of a pipeline by taking
its `CommandResult::Output(text)` and feeding it in on a thread with
`let _ = write_all(...)` -- EPIPE-safe by construction. But a builtin that
PRINTS rather than returning `Output` never reaches that path; it falls into
`Peeled::Finished` and its bytes go straight to stdout.

⭐ So this is the **same root cause as `history | head` dropping its pipe**
(fixed 2026-08-21 by giving tables a `to_pipe_text`): builtins split into those
that RETURN text and those that PRINT it, and the printing ones are outside the
pipeline machinery. That fix made table commands return `Value`. This one is
about the commands that still print.

### Scope
Any builtin that writes with `println!` is a candidate. `dash forest` is
confirmed. The census is the first gate because the count decides whether the
answer is per-command or structural.

## The Solution
Two candidate shapes, and the choice is the intent's real content:

**(a) Make printing builtins return text.** Consistent with the `to_pipe_text`
work and makes them pipeable as a side effect. ⚠️ Large: every printing builtin
changes shape, and some print incrementally by design.

**(b) Handle EPIPE at the write boundary.** Smaller, but it is a guard rather
than a fix, and it leaves the two classes of builtin permanently different.

⚠️ **DO NOT REINTRODUCE A PROCESS-WIDE SIGPIPE RESET.** The 2026-08-21 work
established that the shell must IGNORE SIGPIPE (so writes return EPIPE and can
be handled) while children get `SIG_DFL` restored in `pre_exec`. Both halves are
load-bearing: without the per-child restore, `yes | head -3` spins forever.

## Success Criteria
- [ ] G1 RED FIRST: the panic is captured verbatim, and the three-case control
      is re-run -- `yes | head -3` stops, `ls ~ | head -5` does not panic,
      `dash forest | head -4` panics
- [ ] G2 EVERY builtin that writes to stdout with `println!` instead of
      returning `Output` is ENUMERATED. The count decides (a) versus (b)
- [ ] G3 THE RULING between (a) and (b) is recorded here with its reason
- [ ] G4 `dash forest | head -4` completes without panic and without killing the
      shell, on the DEPLOYED binary
- [ ] G5 THE CONTROLS STILL HOLD: `yes | head -3` still terminates its child,
      and no process-wide SIGPIPE reset has returned. `grep` proves the second
- [ ] G6 A NESTED-SHELL test: the failure originally took the PARENT shell down
      too, so the fix is verified from inside a child nsh
- [ ] G7 Regression tests in nsh-test for at least one printing builtin piped
      into `head`
- [ ] G8 each gate carries evidence per INT-158

## Non-goals
- Making every builtin pipeable. That is the structured-pipeline work; this is
  about not crashing.
- Revisiting INT-299's original decision. It is already corrected.
""")


# ============================================================ INT-244
write_body("244", """
## Vision
`timeline` shows the snapshots that exist. A database built from source has the
same columns as this machine's.

## The Problem
Found 2026-09-05. **Two separate defects in one table, and both report as an
empty result.**

### ① The table has four columns nothing creates
`PRAGMA table_info(shell_snapshots)` on the live database returns **thirteen**
columns. The `CREATE TABLE` at `commands/mod.rs:13585` defines **nine**.

```
missing from source:  command, git_hash, cwd, intent_id
```

They were added by `ALTER TABLE` and exist only on machines that ran the version
that added them. **A database built from source cannot run the query at
`commands/mod.rs:14953`**, which selects all four. `Err(_) => "No snapshots
yet."` reports that failure as an empty result.

⭐ Same shape as INT-214 (`events.source_tool` / `correlation_id` never created
by any commit), in a second table.

### ② `timeline` is broken on THIS machine, right now
```
nsh -c "timeline 3"   ->  "○ No snapshots yet. Run: snapshot"
sqlite3 ... "SELECT COUNT(*) FROM shell_snapshots"   ->  574
```

574 rows, and the command says there are none.

**The cause, measured:** `db::capture_snapshot` writes name, timestamp, health,
command, git_hash, cwd and intent_id. It NEVER writes `commits`, `processes` or
`load_avg`. So every automatic snapshot (`auto-git`, `auto-rm`) has NULL in
those three columns:

```
574 | auto-git | 1783041981 | 100 | NULL | NULL | NULL
```

`timeline`'s reader binds them as non-null (`r.get::<_, i64>(4)`, `(5)`,
`(6)`), so the row conversion fails and the whole command returns the empty
message.

⚠️ **TWO WRITERS, ONE TABLE, DISAGREEING ABOUT WHICH COLUMNS EXIST.**
`snapshot_cmd` fills all nine; `capture_snapshot` fills seven different ones.

📍 **PROVEN NOT TO BE TODAY'S WORK.** `git stash` back to `f8d44e6d`, rebuild,
`timeline 3` -> the identical "No snapshots yet". The defect predates the
INT-230 G4 changes entirely.

### Why it stayed invisible
`Err(_) => return CommandResult::Output("No snapshots yet")` discards the
reason. A failed query and an empty table produce the same output -- INT-192's
subject exactly, and this is a live instance of it.

## The Solution
Three parts, and they are separable:

1. **The canonical schema gains the four ALTER-only columns**, so a fresh
   database is born able to run every query against this table. (INT-214's fix,
   applied here.)
2. **The two writers agree.** Either `capture_snapshot` fills the operational
   columns, or the readers accept that automatic snapshots do not have them --
   a ruling, not a default.
3. **The reader stops swallowing the reason.** A query that fails says so
   instead of reporting an empty table.

## Success Criteria
- [ ] G1 RED FIRST, ALREADY CAPTURED: `timeline 3` prints "No snapshots yet"
      against 574 rows, and a git-stashed build does the same -- so the defect
      is pre-existing rather than introduced
- [ ] G2 THE COLUMN CENSUS: every column the live table has, every column the
      source creates, and every column each writer fills. As a table in this
      intent
- [ ] G3 THE CANONICAL SCHEMA CREATES ALL THIRTEEN COLUMNS. Proven against a
      FRESH database, not this one
- [ ] G4 THE RULING on the operational columns is recorded: does
      `capture_snapshot` fill them, or do readers treat them as optional?
- [ ] G5 `timeline` LISTS THE 574 EXISTING ROWS, including the NULL-bearing
      automatic ones
- [ ] G6 A FAILED QUERY IS DISTINGUISHABLE FROM AN EMPTY TABLE. "No snapshots
      yet" is only printed when there are genuinely no snapshots
- [ ] G7 Proven on a database built FROM SOURCE with a clean HOME and
      `FAELIGHT_STATE_DB`, since that is the case the ALTER-only columns break
- [ ] G8 each gate carries evidence per INT-158

## Non-goals
- Migrating the 574 existing rows. They are the historical record; the columns
  are nullable and reading them is what must work.
- The snapshot feature's design.
- INT-192's tri-state contract. This intent is one instance; 192 owns the class.
""")

print("")
print("All four bodies written. Verify with: core intent validate")
