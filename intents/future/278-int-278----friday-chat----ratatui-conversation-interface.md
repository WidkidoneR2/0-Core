---
id: 278
title: "Friday Chat -- The Forest Speaks Back"
status: planned
date: 2026-05-07
tags: [friday, chat, ratatui, tui, conversation, faelight-term, intelligence]
depends_on: [246, 251]
---
Friday already speaks in the prompt.
Friday already watches in the daemon.
INT-278 gives Friday a voice you can answer.
Not a chatbot. Not a helpdesk.
An operational partner you can question directly.
---
THE MODEL
You work in faelight-term.
Left pane: your shell, your work.
Right pane: friday chat -- always open, always aware.
You type: friday why did health drop
Friday answers: from state.db, from event history, from what it watched.
You type: friday explain the last deploy
Friday answers: what changed, what signals fired, what the outcome was.
You dismiss the pane when you want silence.
You open it when you need the forest to think with you.
---
VOCABULARY
friday chat              -- open Friday chat pane in faelight-term
friday why [event]       -- explain why something happened
friday explain [thing]   -- explain a concept from forest history
friday recall [query]    -- surface past events matching a description
friday trace [signal]    -- follow a signal back to its cause
friday what changed      -- what changed since last session
friday risk              -- what Friday is currently watching with concern
---
ARCHITECTURE
friday chat opens in the right split pane of faelight-term.
Input: single line at bottom, Enter to send.
Output: scrollable conversation history above.
Friday reads: state.db -- events, patterns, knowledge, contradictions.
Friday writes: friday_daemon_messages for persistence.
Friday does not invent. It reports what it has observed.
The conversation is logged to state.db.
Friday learns from what you ask -- questions are signals too.
---
WHAT FRIDAY CAN ANSWER
From state.db:
  Health history -- why did health drop on date X
  Deploy history -- what deployed, what changed, what followed
  Intent history -- what was built during INT-NNN, what signals fired
  Pattern history -- what patterns Friday has learned and their accuracy
  Contradiction history -- what contradictions were active and when
From friday_knowledge:
  Facts Friday has accumulated from 2477+ commits
  Lessons from 227 complete intents
  Decisions from DEC-* records
What Friday cannot answer:
  Anything it has not observed
  Predictions without data
  Questions outside the forest
---
GATES
[ ] friday chat command opens right split pane in faelight-term
[ ] Input field at bottom, Enter sends message
[ ] Friday reads question, queries state.db, returns answer
[ ] friday why [event] returns event history explanation
[ ] friday recall [query] surfaces matching past events
[ ] friday trace [signal] follows signal to root cause
[ ] Conversation logged to state.db
[ ] Pane dismisses cleanly with q or Escape
[ ] Friday answers from data -- never invents
[ ] Demonstrated: friday why did health drop -- returns real answer
"Friday does not guess.
Friday remembers.
Ask it what it saw." 🌲
