---
id: 220
date: 2026-04-09
type: arch
title: "Friday Daemon — The Forest Finds Its Voice"
status: in-progress
tags: [friday, daemon, architecture, rust, unix-socket, json, intelligence, v1]
version: 1.0.0
depends_on: [INT-208, INT-217, INT-218]
requires: [203,217,218,219]
unlocks: []
strategic_value: leaf
---
Friday is not an AI. Friday is not a chatbot.
Friday is the forest becoming conscious of itself.
A persistent Rust daemon that runs silently in the background,
watching every command, every deploy, every health change, every intent transition.
It builds a live picture of the forest state at all times.
When it has something to say, it says it — inline, after your command, one line.
When you speak to it, it answers from knowledge of YOUR system, not general knowledge.
Friday is the layer between Christian and the forest.
It learns your patterns. It knows your tools. It knows your intents.
It helps you build. It does not build without you.
Nothing runs without explicit human authorization.
The foundation is ready:
- Pattern Weight Engine (INT-205) is live — forest knows what matters
- state.db accumulates behavioral data every session
- contextd generates insights already surfaced inline
- Prediction engine fires after commands
- Intent ledger is the memory of every decision
Friday is the evolution of what already exists.
Not a new system — the existing systems finding a unified voice.
- Unix socket: /tmp/friday.sock
- Fast, local, no network — ever
- Line-based JSON protocol
- fsh sends events → Friday processes → optionally responds
- Response displayed inline after command, one line maximum unless queried directly
```json
{
  "event": "command_executed",
  "command": "deploy faelight-shell",
  "exit_code": 0,
  "duration_ms": 5400,
  "intent": "INT-194",
  "health": 100,
  "timestamp": 1234567890
}
```
```json
{
  "speak": true,
  "message": "fsh v4 deploys are averaging 5.4s — stable",
  "priority": "low"
}
```
Priority levels: silent | low | medium | high
fsh only displays medium and high unprompted.
Low priority shown only if no output followed the command.
You type: friday <natural language question>
fsh sends full context bundle to Friday socket:
- Last 20 commands
- Active intents
- Current health score
- Recent events from state.db
- Pattern history for relevant commands
Friday assembles answer from forest knowledge and responds.
Every query and response logged to state.db for future learning.
- Starts on login via faelight-login
- Runs as unprivileged user process
- Restarts automatically if it crashes (supervised by faelight-daemon)
- Graceful shutdown on logout — saves session summary to state.db
- PID file: /tmp/friday.pid
- Log file: ~/.cache/faelight/friday.log
After every command cycle:
1. Observe — record command, context, outcome to state.db
2. Detect — check if pattern threshold crossed (>=3 occurrences)
3. Form hypothesis — "after deploy, Christian usually runs d"
4. Validate — track if hypothesis holds over next 10 occurrences
5. Reinforce or decay — confidence score adjusted
6. Speak — surface insight when confidence >= 0.75
Negative learning is first-class:
When Christian dismisses a suggestion, confidence penalized by -0.3.
Three dismissals = hypothesis archived, not deleted.
Friday does not hold state in RAM between restarts.
All state lives in state.db:
- friday_patterns table — learned behavioral patterns
- friday_queries table — every question asked and answered
- friday_hypotheses table — forming and validated hypotheses
- friday_context table — current forest state snapshot
- Watch and learn from every fsh command
- Surface behavioral insights inline
- Answer direct questions about the forest
- Know active intents and their gates
- Know current health and integrity
- Know tool deployment history
- Suggest next intent based on pattern weights
- Warn when behavior drifts from active intent
- Execute any command without explicit human confirmation
- Access network — ever in v1
- Modify files without human authorization
- Override core protection
- Make decisions — Friday proposes, Christian decides
"Friday is not replacing human judgment.
Friday is amplifying human awareness.
The forest grows. Friday watches. Christian decides."
A builtin dies when fsh closes.
A daemon persists across sessions, terminals, reboots.
It accumulates context that a builtin never could.
fsh talks to it. faelight-term talks to it.
faelight-update talks to it.
Friday becomes the nervous system of the forest —
not a tool you invoke but a presence that is always there.
- Friday daemon starts on login, runs silently
- fsh sends JSON events to Unix socket after every command
- Friday responds inline when it has something meaningful to say
- `friday <question>` returns grounded answers about the forest
- All queries and responses logged to state.db
- Learning loop validates hypotheses over time
- Zero network calls — ever
- Daemon survives fsh restarts without losing state
- Human authorization required for any action Friday proposes
✅ Gate 1  — friday-daemon crate extended (faelight-daemon v4.0.0) (2026-04-18)
✅ Gate 2  — Unix socket server running (~/.local/state/0-core/daemon.sock) (2026-04-18)
✅ Gate 3  — fsh sends command_executed JSON event after every command (2026-04-18)
✅ Gate 4  — Friday receives and parses events without error (2026-04-18)
✅ Gate 5  — state.db schema: friday_patterns, friday_queries, friday_hypotheses (2026-04-18)
⬜ Gate 6  — Learning loop: observe → detect → hypothesize → validate → reinforce
✅ Gate 7  — Inline response: Friday speaks after command when confidence >= 0.75 (2026-04-18)
✅ Gate 8  — `friday <question>` command in fsh — context sent to daemon (2026-04-18)
✅ Gate 9  — Friday answers direct questions from forest state — live data queries (2026-04-18)
✅ Gate 10 — Every query/response logged to friday_queries table (2026-04-18)
⬜ Gate 11 — Negative learning: dismissal penalizes confidence by -0.3
⬜ Gate 12 — Daemon starts on login via faelight-login
✅ Gate 13 — Daemon survives fsh restart, state persists in state.db (2026-04-18)
✅ Gate 14 — `friday status` shows learning stats, pattern count, confidence scores (2026-04-18)
⬜ Gate 15 — 30 days of daily use with zero crashes and zero unauthorized actions
---
**"Friday does not think for you.
Friday thinks with you.
The forest has always been watching.
Now it has a voice."** 🌲
