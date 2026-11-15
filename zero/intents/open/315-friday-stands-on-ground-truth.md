---
id: 315
date: 2026-11-15
type: future
title: "Friday stands on ground truth"
status: planned
tags: [friday, ledger]
format: 2
---

## Vision
Stage 1 of six. Friday stays a personal assistant with intelligence and no AI:
no LLM and no embeddings. She suggests only, forever. This stage makes what she
says true, and points her readers at the tables that already hold the truth.
Nothing here runs a command, starts a daemon, or adds a timer.

## The Problem
The brief counts commits from git. The rest of Friday does not. `deploy_without_commit`
counts `events` rows that only zero-git writes, so a `ship` after a plain git
commit can raise a 0.90 anomaly. The habit line walks `shell_history`, which
stores the typed line and the expanded line as neighbors, so the top pair is
often a command and itself.

Spoken lines are still literals: `core friday status` ends on "892h of
observation, 14 patterns, 333 facts"; the milestone fallback is 186; the brief
always says "Healthy." and "No conflicts detected."; `starter_rules` says 10
rules and holds 7; the first-speech insert names `events_v2` columns that do
not exist. The every-10th-command attention path cannot speak. Inside the
30-minute cooldown `get_voice` returns nothing, so `d` goes blank. Session
summaries are stored as facts and keep the doctor's freshness check green.

## The Solution
Friday reads `command_execution` (typed text, cwd, exit, duration), with secrets
redacted before they are stored. Every place she counts commits asks git, the
same way `room::commits_since_local_midnight` does. The habit line and
`extract_patterns` use `typed_text` ordered by `(session_id, execution_id)`.

The literal lines above are deleted or replaced with a measured value. The
every-10th-command path is deleted. When the cooldown is the reason she is
quiet, `d` prints one dim line, `spoke 12m ago`. Session summaries stop counting
as facts, and the freshness check reads the same set. Under the brief, one dim
line names the sources, `from git · ledger · 1,204 commands`.

The path, ruled 2026-11-14. Six stages. Stage 7 is dropped.

- D1. She may read full typed commands and folders. She never stores a token, a
  password, or a URL that carries a secret. Redact before storing.
- D2. Suggest only, forever.
- D3. Learn on each `d` and `core friday` call, incrementally. No daemon timer.
- D4. Feedback is explicit yes/no and implicit taken-within-N. N is a later stage.
- D5. Habit half-life is 30 days. Session summaries are not facts. Old rows expire.
- D6. No every-10th-command voice. The daily brief is shown in `d`. Interruptions
  per hour are limited. No desktop popups.

Stages 2 through 6 are later intents, not gates here. INT-132 is rebuilt as
Stage 4. INT-041 is rebuilt as Stage 6, without a daemon. The 30-day half-life,
the expiry of old rows, and D4's N belong to those stages. INT-191 still owns
the shell-history double-write. This intent only stops Friday from reading it.

## Success Criteria
- [ ] G1 The rulings are in this file before any code. The open questions are
      the ask lines. A ruling line answers each one, or declines it.
      ask: 1. Redact at the command_execution insert, so every reader is safe, or only when Friday copies a command into her own tables? Recommend the insert. 2. Which shapes are secrets: values after --password, --token and --secret, URL userinfo, and Authorization values? Recommend those four. 3. When no doctor health has been measured, the brief says "health unknown" rather than "Healthy." Recommend that word. 4. The snapshot's health=100 and brief_confidence=0.85 are replaced with the last measured health and a confidence that is not a literal. Recommend yes, in this stage, because get_voice treats 0.85 as proof she may speak.
      ruling:
      evidence:
- [ ] G2 Red first, under its own plan Seal. A typed line beside its expansion
      makes the habit pair a command with itself. A ship after a plain git
      commit raises deploy_without_commit. The literal lines are still present.
      The suite is cargo test -p core and cargo test -p novashell.
      ask:
      ruling:
      evidence:
- [ ] G3 Green. The same tests pass. command_execution is the habit source, git
      is the commit source, the literals are gone, the attention path is gone,
      the cooldown names how long ago she spoke, session summaries are not
      facts, and the dim source line is present.
      ask:
      ruling:
      evidence:
- [ ] G4 nsh-test, after cargo build -p novashell -p nsh-test, under a timeout.
      If it stalls, the last case is named.
      ask:
      ruling:
      evidence:
- [ ] G5 Doors, on the laptop, after ship. cargo test -p core, cargo test -p
      novashell, nsh-test, and core intent validate. The deployed brief is what
      was watched.
      ask:
      ruling:
      evidence:
- [ ] G6 cicomplete. The gates above are ticked with evidence, or declined with
      a signed reason.
      ask:
      ruling:
      evidence:

## Out of scope
- Stages 2 through 6, including the 30-day half-life, row expiry, and D4's N.
- Stage 7. Mandates are dropped.
- A daemon, a socket, or a timer.
- An LLM call or an embedding.
- Rewriting INT-191, INT-132, or INT-041 in this file.
