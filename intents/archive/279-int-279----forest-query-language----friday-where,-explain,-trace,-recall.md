---
id: 279
title: "Forest Query Language -- The Forest Answers Questions"
status: complete
date: 2026-05-07
tags: [fql, query, language, friday, intelligence, state.db, forest]
depends_on: [278, 246]
---
Right now: core predict next
Soon: friday where risk > medium and source = updates
The Forest Query Language is a human-readable query layer
over state.db, intent history, event log, and friday knowledge.
Not SQL. Not natural language.
Something in between.
Forest-native. Readable. Precise.
---
THE LANGUAGE
friday where [condition]
  friday where risk > medium
  friday where source = updates and confidence > 0.8
  friday where domain = fsh and timestamp > today
friday explain [subject]
  friday explain drift
  friday explain why health dropped
  friday explain the last contradiction
friday trace [signal]
  friday trace instability
  friday trace the deploy that failed
  friday trace what caused health to drop to 95
friday recall [memory]
  friday recall "times I ignored warnings"
  friday recall deploys that caused health drops
  friday recall what changed before the PAM incident
friday why [event]
  friday why did health drop
  friday why did that deploy fail
  friday why is this pattern firing
friday show [data]
  friday show patterns with confidence > 0.9
  friday show contradictions active this week
  friday show intents completed this month
---
PARSING APPROACH
Phase 1 -- keyword matching (simple, ships fast):
  Parse first word: where/explain/trace/recall/why/show
  Parse subject from remaining words
  Map to state.db queries
  Return formatted answer
Phase 2 -- condition parsing:
  where [field] [operator] [value]
  Fields: risk, source, domain, confidence, timestamp, health, intent
  Operators: >, <, =, !=, contains
  Values: literals, keywords (today, this_week, medium, high)
Phase 3 -- natural language approximation:
  Fuzzy matching for subjects
  Intent detection from question structure
  Fallback: ask for clarification with specific options
---
IMPLEMENTATION
The FQL parser lives in core.
fsh vocabulary: friday [query] pipes through FQL parser.
Output: formatted forest-style response, never raw SQL.
The parser translates FQL to state.db queries internally.
The human never sees SQL. The human sees forest.
---
GATES
Phase 1 -- keyword commands:
[x] friday why [event] routes through FQL to event bus 2026-05-27
[x] friday explain deploy returns 4 real knowledge entries from friday_knowledge 2026-05-27
[x] friday show decisions returns 5 real decisions, friday show attention returns log 2026-05-27
[x] friday recall faelight-menu returns knowledge + 3 commits; honest empty when no history 2026-05-27
[x] All commands return forest-formatted output directly in fsh 2026-05-27
Phase 2 -- condition parsing:
[x] friday where confidence > 0.9 returns 9 patterns; where risk > medium returns 2 events 2026-05-27
[x] confidence, domain, health, risk fields all working 2026-05-27
[x] > < = operators work for confidence and risk fields 2026-05-27
[x] Deferred to Phase 3 expansion -- approved by: christian 2026-05-27
Phase 3 -- integration:
[x] friday where/show/explain/trace/recall all work directly from fsh 2026-05-27
[x] Results shown inline in fsh -- friday trace deploy shows real signal history 2026-05-27
[x] Questions logged to events table (domain: friday, action: chat_message) 2026-05-27
Final:
[x] friday where risk > medium returns 2 real events: clarification 0.67, health_drop 0.71 2026-05-27
[x] friday trace deploy follows signal -- 5 real deploy events with timestamps; instability returns empty (no such history -- honest) 2026-05-27
[x] friday recall returns honest empty when no history exists -- Friday never invents 2026-05-27
[x] friday where confidence > 0.9 -- forest answers with 9 real patterns 2026-05-27
"The forest is not a database.
But it knows everything that happened.
Ask it in plain language.
It will tell you what it saw." 🌲
