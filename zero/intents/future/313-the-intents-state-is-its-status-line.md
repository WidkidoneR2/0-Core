---
id: 313
date: 2026-11-13
type: future
title: "The intent's state is its status line"
status: planned
format: 2
tags: [intent, ledger]
---

## Vision
One glance tells the truth. An intent's state is the `status:` line in its
frontmatter. A folder is only a shelf. The shelves that remain are few, and
none of them means "done" or "started." Git history is the archive. cistart
changes one line. cicomplete changes one line. The file does not move.

## The Problem
Re-read 2026-11-13 against 7264b56f. The study's citations hold, with one
count corrected below.

start (intent/mod.rs:1005-1020) replaces `status:` across the whole file, then
`let _ = fs::rename` into in-progress/. It does not call create_dir_all.
complete_intent (2594-2600) and cancel (4710-4711) do call create_dir_all,
then write the file into complete/ or cancelled/. validate (553-561) requires
the folder to match the status, and a file in future/ must say planned.
load_all (110-136) and in_progress_ids (99-108) already trust the status line.
folders.rs:139-154 fails if a lifecycle name is not a directory.

Readers still key off folder names. `it` joins future, in-progress, and
complete (intent_tui.rs:468). Friday reads ~/0-core/intents/future
(planning.rs:458-466). The same path shape is in strategy, partner, journal,
integrity, doctor, zero-git, completion, nsh-test, and zero-daemon
(daemon.rs:606 and 714, which search the text `status: in-progress`). G0
lists every one.

On disk: future 43, planned 1, in-progress absent, complete 232, cancelled 31
plus .gitkeep. Records stay separate: decisions, experiments, incidents,
philosophy, findings. The study said 12 bodies quote `status: planned`. This
tree has that phrase in the body of two files, INT-308 and INT-310, four
times. A whole-file replace can edit those sentences.

INT-308 is on hold until this layout is settled. This intent does not edit
it. INT-310 wants Friday and `it` on one reader. That reader has to be the
loader this intent leaves behind, or 310 will retarget two screens onto
folders this intent is about to drop.

## The Solution
Status is the state. complete/ is dropped; history is the archive.
in-progress/ is dropped; cistart and `core intent start` change only the
frontmatter status line. What is still open is the shape of the shelves, the
fate of the 232 and the 31, and the order of the move. Each open gate is one
question. A ruling is a line in its ruling field, dated, in his words. No
code before G4, G5, G6, and G7 have those lines.

The end, if the recommendations below are accepted:

```
zero/intents/open/          every live intent
zero/intents/decisions/     records, unchanged
zero/intents/experiments/
zero/intents/incidents/
zero/intents/philosophy/
zero/intents/findings/
```

Nothing in that tree means done. Done is `status: complete` in a commit that
is already history.

## Out of scope
- The format-wide cleanup: type-field values, heading names, and the v1 to v2
  rewrite of old files. That is the later ledger-format rebuild. INT-309 owns
  the reader for format 2.
- INT-308's other work: non-interactive add, and the seal-note command.
- Editing INT-308 or INT-310 in this filing.

## Success Criteria
- [ ] G0 Re-read The Problem against the tree. The citations and the counts
      either stand or are corrected here before any other gate.
      ask: do the file:line citations and the folder counts still match?
      ruling:
      evidence:
- [x] G1 The status line is the state. Folders no longer encode it. Only a
      few folders remain.
      ask: where an intent's state lives.
      ruling: RULED 2026-11-13 (Christian): The frontmatter status: line is the single source of truth for an intent's state. Folders no longer encode lifecycle state. Keep only a few folders.
      evidence: the ruling line above.
- [x] G2 complete/ is not a shelf. Git history is the archive.
      ask: where a finished intent lives.
      ruling: RULED 2026-11-13 (Christian): Drop complete/. Git history is the archive.
      evidence: the ruling line above.
- [x] G3 in-progress/ is not a shelf. cistart does not move the file.
      ask: what cistart writes.
      ruling: RULED 2026-11-13 (Christian): Drop in-progress/. Christian delegated this to Jarvis and then ruled it. cistart and core intent start stop moving files and change only the frontmatter status line.
      evidence: the ruling line above.
- [ ] G4 The shelves that remain.
      ask: one folder, named open/, holding every live intent (today's future, planned, and in-progress). The record directories stay: decisions, experiments, incidents, philosophy, findings. future/ and planned/ are emptied into open/ in the same move. Reject, or name a different shelf.
      ruling:
      evidence:
- [ ] G5 The 232 complete files and the 31 cancelled files.
      ask: git rm both directories in the migration gate, after no reader opens them, so the only copy is history. Cancelled is included because a folder that means a status is the thing G1 ends. A different fate for cancelled is a ruling.
      ruling:
      evidence:
- [ ] G6 validate, and the status edit.
      ask: validate stops requiring folder == status. status must still be a known word, and the id must still match the filename. start, complete, and cancel replace the status line inside the frontmatter block only. INT-308 and INT-310 keep the sentences in their bodies.
      ruling:
      evidence:
- [ ] G7 One loader.
      ask: every reader asks load_all, or the one function that replaces it. No private folder list. INT-310 does not grow a second design: Friday and `it` are the first two callers of this loader, and 310's own gates close by pointing at that.
      ruling:
      evidence:
- [ ] G8 The order. Each stage is green on d, nsh-test, doors, and core intent
      validate before the next stage starts. A stage that has not deleted
      files reverts with a commit.
      ask: this order. 1, frontmatter-only status edit, files stay put. 2, the loader, and the readers call it. 3, validate no longer matches folder to status. 4, live files move into the shelf G4 named. 5, the migration in G9. Reject a step, or reorder.
      ruling:
      evidence:
- [ ] G9 Christian runs the migration.
      ask: one command he runs himself, after stage 4 is green: git rm of complete/ and cancelled/, and the commit message names the range those files still have in history. Nothing is pushed until he says so.
      ruling:
      evidence:
- [ ] G10 The laptop. He opens `it`, asks Friday, runs d, cistart, and
      cicomplete. The file does not move. The status line does.
      ask: the five checks, on the machine, after the migration.
      ruling:
      evidence:
