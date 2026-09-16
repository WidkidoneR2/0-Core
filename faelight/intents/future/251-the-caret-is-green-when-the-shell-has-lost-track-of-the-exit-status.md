---
id: 251
date: 2026-09-16
type: future
title: "the caret is green when the shell has lost track of the exit status"
status: planned
tags: [prompt, exit-code, telemetry, int-192, pipeline]
---

## Vision

The prompt caret is green after a command that FAILED. Not always -- only when the shell has
lost track of the exit status, which it does on four of its execution paths.

## Measured 2026-09-15

    nsh> false
    x   exited 1 -- that is what false does
    nsh> cat ~/.cache/faelight/last-exit-status
    success                                     <- and the caret stayed GREEN

Reproduced against the previous binary as well, so this predates the INT-247 path work; it was
found BY that work rather than caused by it.

## The chain, and where it breaks

    engine.rs        let exit_ok = engine.last_exit().map(|c| c == 0).unwrap_or(true);
                     writes "success" / "failure" to last_exit_status_file()
    prompt.rs        reads that file, red caret on "failure", green otherwise

Two failure modes, and the second is the bad one:

1. `last_exit()` returns a STALE value -- the previous command's -- because this command's path
   never set it.
2. `last_exit()` returns None, and `.unwrap_or(true)` reads that as SUCCESS.

⭐ THE GAP IS ALREADY DOCUMENTED, IN THE CODE, BY WHOEVER WROTE THE FIX ABOVE IT. engine.rs
carries this note:

    KNOWN GAP, recorded not hidden: four arms of that match never set last_exit_code at all
    (both Value arms, and the two arms that spawn `sh` for pipelines and discard its status),
    so on those paths the value carries over from the previous command.

That comment is the specification for this intent. The work is not diagnosis -- it is deciding
what those four arms should report.

## ⚠️ WHY IT WAS LEFT, AND WHY THAT WAS RIGHT

The same note says fixing it "touches pipeline execution semantics (is pipeline status the last
command? the first failure?) and belongs in its own intent with its own verification."

That is correct and it is the whole difficulty. `false | true` exits 0 in POSIX (the LAST
command) but `set -o pipefail` makes it 1 (the FIRST failure). nsh has to choose, and the choice
is a language decision, not a plumbing one.

## This is INT-192's shape, one layer further in

An unreadable source answering as an empty one; here, an UNKNOWN status answering as a GOOD one.
`.unwrap_or(true)` is the same collapse as `.unwrap_or(100)` in the health cache and
`.unwrap_or_default()` in the /etc/faelight reads (INT-250). Three instances, three files, one
habit: when the answer is not known, supply the happy one.

⭐ AND THE CARET IS THE WORST PLACE FOR IT. It is the one piece of telemetry that is read
hundreds of times a day, by eye, without thinking. A caret that is green when the shell does not
know teaches you to stop believing the caret.

## What is NOT broken

- `$?` and the `x exited N` line are CORRECT -- they come from the real status, not this file.
- The lifecycle table (`command_execution.exit_code`) is correct.
- Only the cached string and the caret colour derived from it are affected.

So this is a DISPLAY defect over a correct substrate, which is why it can wait for a considered
answer rather than a quick one.

## Success Criteria

- [ ] All four arms set `last_exit_code`, or the value is made unavailable rather than stale --
      no arm leaves the previous command's status in place
- [ ] The pipeline question is ANSWERED IN WRITING before any code changes: last command, or
      first failure? With the reason, and with what bash does noted for comparison
- [ ] `false` turns the caret red, demonstrated live -- not asserted
- [ ] `false | true` behaves as the written decision says, demonstrated live
- [ ] ⭐ UNKNOWN IS NOT SUCCESS. If the shell genuinely cannot determine a status, the caret says
      so -- a third colour, a different glyph, anything but the green that means "fine". Replacing
      `unwrap_or(true)` with `unwrap_or(false)` would be the same bug pointed the other way
- [ ] A regression case in nsh-test covers the CLASS: a command whose path does not set the exit
      code does not inherit the previous command's caret
- [ ] The engine.rs note that predicted this is updated rather than deleted -- it was right, and
      the record should show that it was right

## Relationship

- FOUND BY INT-247 Layer 3a while consolidating last-exit-status onto one accessor. The path work
  did not cause it and did not fix it; it walked past it closely enough to see it
- INT-192: unreadable answering as empty. Same collapse, different value
- INT-250: the /etc/faelight reads, found the same night, same `unwrap_or` habit
- INT-201 owns the CommandResult match whose arms are incomplete
- The comment at engine.rs (search: "KNOWN GAP, recorded not hidden") is the primary source

## The Rule

"A status the shell did not establish is not a success. The caret is read more often than any
log line in this system, and it has been saying `fine` whenever the shell lost the thread." 🌲
