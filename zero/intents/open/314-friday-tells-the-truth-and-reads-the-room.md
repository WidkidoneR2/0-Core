---
id: 314
date: 2026-11-15
type: future
title: "Friday tells the truth and reads the room"
status: in-progress
tags: [friday, brief, ledger]
format: 2
---

## Vision
Friday stays a personal assistant with no model behind it. The brief is recomputed
when it is read, the commit count comes from git, and the room (open gates, a
stall, an anomaly, a habit) is said in the colors Friday already uses. Nothing
is written that Friday does not already write. There is no daemon.

## The Problem
The brief counts `events` where `domain='git'` and `action='commit'` for the last
86400 seconds (`friday_arch/mod.rs`). Only zero-git's confirm path inserts that
row. A plain git commit never does, so the line says 0 commits today while the
log has some. `get_voice` then replays the stored `synthesis_snapshots` row
(`doctor/mod.rs` calls it) instead of counting again.

In-progress work is listed, not ranked. `reasoning::reason()` already returns
anomalies and the brief does not print them. `shell_history` and `friday_patterns`
hold the habit and the brief does not say it.

## The Solution
One read path. Commits are `git log` since local midnight. The snapshot is not
the source of that sentence. In-progress intents are ordered by open gates and
by days since a commit named the id. Fourteen days with no gate change and no
naming commit is stalled. Anomaly lines are whatever `reasoning::reason()`
returns, and they are omitted when that list is empty. The habit line is the
top command and its usual follow-up, and it is omitted below the 50-row
`shell_history` floor `get_voice` already uses. New lines use Friday's colors:
a magenta mark, a cyan id, a green count. INT-132 (teaching) follows this
intent. It is not part of it.

## Success Criteria
- [x] G1 The brief is recomputed on read. Commits are counted with git log since
      local midnight, not the events table, so a day with commits does not read
      as 0 and a stored snapshot is not replayed as today's count.
      ask:
      ruling: RULED 2026-11-15 (Christian): one function, used by run and get_voice. Keep the INSERT for the health trend. Never print the stored friday_brief. A git failure prints unknown.
      evidence: red e480525b returned 0 and STALE SNAPSHOT. Green: room::sentence_for, used by run and get_voice. cargo test -p core 304 passed.
- [x] G2 In-progress intents are ranked by open gates and by days since a commit
      named the id. An intent with no gate change and no naming commit for 14
      days is flagged stalled.
      ask:
      ruling: RULED 2026-11-15 (Christian): sort by open gates, then days. A tie is the lower id. stalled is a red mark. The line is a magenta star, a cyan id, and a green done/total. Both clocks at 14 days is stalled.
      evidence: red e480525b left the rows in file order. Green: rank_room and print_room. Same cargo test -p core, 304 passed.
- [x] G3 The brief surfaces what reasoning::reason() returns, in Friday's colors,
      and prints no anomaly line when that list is empty.
      ask:
      ruling: RULED 2026-11-15 (Christian): at most 3, highest confidence, skip normal. Yellow warning, the confidence, and the conclusion. Causal is red. Empty prints nothing.
      evidence: red e480525b kept the normal row. Green: pick_anomalies and print_room. Same cargo test -p core, 304 passed.
- [x] G4 One habit line from shell_history and friday_patterns: the top command
      and its usual follow-up.       Quiet when shell_history is under 50 rows.
      ask:
      ruling: RULED 2026-11-15 (Christian): the most common consecutive shell_history pair, first word only. The line is a star, the two words, and the count in green. Quiet under 50 rows, or when that pair happened fewer than 3 times.
      evidence: red e480525b returned no line. Green: habit_line. Same cargo test -p core, 304 passed.
- [x] G5 Red first. The four behaviours above fail under their own plan Seal
      before the fix. The suite is cargo test -p core.
      ask:
      ruling: RULED 2026-11-15 (Christian): red first, then the brief.
      evidence: commit e480525b, cargo test -p core FAILED. 0 passed; 6 failed.
- [x] G6 Green. The same tests pass.
      ask:
      ruling: RULED 2026-11-15 (Christian): the same tests pass.
      evidence: cargo test -p core 304 passed, including the 6 that failed in e480525b.
- [ ] G7 Doors, on the laptop, after ship. cargo test -p core passes. cargo test
      -p novashell passes. nsh-test passes. core intent validate reports every
      intent valid. The deployed brief is what was watched.
      ask:
      ruling:
      evidence:

## Out of scope
- A morning brief.
- Hiding the usefulness percent.
- A daemon, a socket, or a timer.
- Any write beyond the tables Friday already has.
- An LLM call or an embedding.
- INT-132. Teaching follows this intent.

## Open for Christian
Ruled 2026-11-15. G1 one function, snapshot kept, stored brief never spoken. G2 open gates then days, stalled is a mark. G3 three anomalies. G4 consecutive first words. The stall window is 14 days.
