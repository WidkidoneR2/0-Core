---
id: 235
title: "Friday Daemon v2 -- Anticipation Engine: Always Watching, Always Ready"
status: planned
date: 2026-04-16
tags: [friday, daemon, anticipation, continuous, v2, background, intelligence]
---
INT-220 describes Friday finding its voice.
Friday Daemon v2 is Friday never sleeping.
Not a cron job. Not a polling loop.
A persistent intelligence that watches the forest in real time,
builds context continuously, and speaks at the right moment.
v1 (faelight-daemon): aggregates context, serves on request
v2: proactively pushes insights, anticipates needs, holds state
Watches forest_events_v2 continuously via SQLite WAL
Watches shell_history in real time (inotify on state.db)
Maintains rolling 30-minute session context
Triggers Friday speech when:
- Confidence >= 0.85 AND pattern matches current state
- Contradiction detected (immediate alert)
- Build fails (knowledge engine lookup + speak)
- Intent milestone reached
- Health drops below 95%
Every 60 seconds: run planning layer (Core v21)
Compare prediction to what actually happened
If diverging: surface warning before problem occurs
"You usually commit after deploy -- 3 deploys without commit detected"
Daemon holds conversation context across fsh sessions
When you open a new terminal: Friday already knows your context
No cold start -- Friday is always warm
→ fsh prompt: subtle indicator when Friday has something to say
→ faelight-notify: urgent alerts (health drop, contradiction)
→ faelight-term Friday pane: full context always visible
→ state.db friday_daemon_messages: persistent message queue
⬜ Daemon starts on login, survives session restart
⬜ Watches forest_events_v2 continuously without polling
⬜ 30-minute rolling context maintained
⬜ Speaks when confidence >= 0.85 (not before)
⬜ Build failure triggers knowledge engine + speak within 5 seconds
⬜ Contradiction detection triggers immediate notify
⬜ Anticipation engine runs every 60 seconds
⬜ Divergence from prediction surfaced before problem occurs
⬜ Conversation context persists across fsh sessions
⬜ fsh prompt shows Friday indicator when message pending
⬜ faelight-notify integration for urgent alerts
⬜ Friday Daemon v2 replaces and supersedes INT-220
⬜ No more than 1 unsolicited message per 5 minutes (no spam)
"Friday never sleeps.
Friday is always watching.
Friday speaks when it matters." 🌲
