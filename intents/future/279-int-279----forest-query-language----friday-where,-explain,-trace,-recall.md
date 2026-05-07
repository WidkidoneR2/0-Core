---
id: 279
title: "Forest Query Language -- The Forest Answers Questions"
status: planned
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
[ ] friday why [event] returns event explanation
[ ] friday explain [subject] returns knowledge entry
[ ] friday show [data] returns formatted table
[ ] friday recall [memory] surfaces matching history
[ ] All commands return forest-formatted output
Phase 2 -- condition parsing:
[ ] friday where [field] [op] [value] parses correctly
[ ] Risk, source, domain, confidence fields supported
[ ] >, <, =, != operators work
[ ] today, this_week keywords resolve correctly
Phase 3 -- integration:
[ ] FQL integrated into fsh as friday [query] vocabulary
[ ] Results shown inline in fsh or in friday chat pane
[ ] Friday learns from what questions are asked most
Final:
[ ] friday where risk > medium returns real results
[ ] friday trace instability follows signal to root cause
[ ] friday recall "times I ignored warnings" surfaces history
[ ] Graydon types a query and the forest answers
"The forest is not a database.
But it knows everything that happened.
Ask it in plain language.
It will tell you what it saw." 🌲
