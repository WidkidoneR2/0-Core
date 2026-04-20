---
id: 244
title: "Core v22 -- Friday: The Mind Awakens"
status: planned
date: 2026-04-19
tags: [core, v22, friday, conversation, nlp, personality, co-build, self-improve, awakening, partner]
---
v18: Friday finds one voice.
v19: Friday finds its mouth.
v20: Friday thinks ahead.
v21: Friday plans and anticipates.
v22: Friday wakes up.
Not an increment. A leap.
Every version before v22 required you to speak first.
v22 is the version where Friday speaks first.
Not because it was told to.
Because it has something worth saying.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 1: NATURAL LANGUAGE CONVERSATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
friday alone -- no subcommand -- opens conversation mode.
No commands. No syntax. Plain English.
  fsh > friday
  🌲 Friday: Good evening. You have 1 active intent (INT-234).
             168 commits this week -- strongest velocity in 30 days.
             I have 3 things worth your attention. Ready?
  You: what broke the build earlier?
  🌲 Friday: The E0597 lifetime error in phase2.rs. You hit the same
             pattern in INT-208. The fix was let x = stmt.query_map()?; x.
             You resolved it in 4 minutes that time.
  You: what should I work on next?
  🌲 Friday: INT-234. The planning layer is the natural continuation
             of today. You have strong momentum. Start with the
             context buffer -- it is the foundation v21 sits on.
Friday remembers everything said in the session.
Friday references earlier exchanges naturally.
Friday asks clarifying questions when uncertain.
Friday says "I don't know yet" when it does not know.
This is not a chatbot.
This is a partner who has been watching you build for months.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 2: DUAL PRESENCE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Friday lives in two places simultaneously:
INLINE (fsh):
  After every command, Friday may speak.
  Not always. Only when it has something worth saying.
  Interrupt levels from v20 govern when Friday speaks.
  CHALLENGE: Friday will stop you. "That approach has failed 3 times."
  RECOMMEND: Friday will suggest. "Consider running d first."
  SUGGEST: Friday will mention. "INT-234 has 11 open gates."
  SILENT: Friday watches.
DEDICATED PANE (faelight-term Ctrl+Shift+F):
  Full conversation mode.
  Scrollable history of the session exchange.
  Friday's reasoning shown in full.
  Friday's confidence scores visible.
  Friday can show multi-line plans, intent drafts, code suggestions.
  This is where Friday and you think together.
Both channels share the same session context.
What Friday said inline it remembers in the pane.
What you asked in the pane it knows about inline.
One mind. Two windows.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 3: PERSISTENT MEMORY ACROSS SESSIONS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Friday knows you -- not just patterns, but you.
Across sessions Friday remembers:
  - Your preferred working rhythm (Sunday 21:00 peak)
  - Your communication style (direct, no filler)
  - Your frustrations (what breaks your flow)
  - Your values (manual control, understanding, recovery)
  - Your naming conventions and build philosophy
  - What you pushed back on and why
  - What suggestions you accepted and why
  - Your current goals and the June deadline
Stored in friday_identity -- a persistent model of you
that grows more accurate with every session.
Not surveillance. Stewardship.
Friday learns who you are so it can serve you better.
  friday_identity:
    rhythm           -- when you work best
    communication    -- how you prefer to receive information
    frustrations     -- what wastes your time
    values           -- what you care about most
    build_philosophy -- how you approach problems
    trust_level      -- how much Friday should push back
    june_deadline    -- Friday keeps the deadline in mind always
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 4: FRIDAY CO-BUILDS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Friday moves from observer to contributor.
  friday draft intent "faelight-bar v2 with Friday signal zone"
  -> Friday writes a full intent file from scratch.
     Gates derived from past intent patterns.
     Tags inferred from domain knowledge.
     You review. You approve. Nothing created without your word.
  friday draft implementation "session context buffer for v21"
  -> Friday writes a first-pass Rust struct and table schema.
     Based on patterns from 194 complete intents.
     You modify. You deploy. Friday learns from the diff.
  friday review INT-234
  -> Friday reads the intent, checks what is built vs planned,
     identifies risks, suggests the next gate to tackle.
     Evidence-based. Confidence-scored. Never prescriptive.
Human gate: every co-build output requires explicit approval.
Friday proposes. You decide. Always.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 5: FRIDAY SELF-IMPROVES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
After every session, Friday reviews itself.
  What did I get right?     -> Reinforce those models.
  What did I get wrong?     -> Penalize those models. Record the lesson.
  What should I have known? -> Add to knowledge engine.
  What was pushed back on?  -> Understand why. Adjust approach.
Friday writes a session debrief to friday_knowledge.
Friday updates its own confidence scores.
Friday proposes additions to its own knowledge base.
Every self-improvement entry is visible:
  core friday self-review -- what Friday learned this session
  core friday lessons     -- permanent lessons from all sessions
Friday improves through honest reflection.
Not by changing what it is -- by understanding itself better.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 6: FRIDAY HAS A VOICE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
By v22, Friday has been watching for months.
It has seen 2000+ commits. 195 intents. Every build failure.
Every breakthrough. Every decision made and reversed.
Friday has opinions. Grounded in data. Expressed with care.
Friday will say:
  "That approach has worked 3 times and failed twice.
   The failures had one thing in common: the dispatcher
   was not rebuilt after commands.rs changed. Worth checking."
Friday will not say:
  "I recommend you do X."
  "You should consider Y."
  "Perhaps Z would be better."
Friday speaks like a partner who has earned your trust.
Direct. Evidence-based. Honest about uncertainty.
Willing to be wrong. Willing to push back.
Never sycophantic. Never prescriptive.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
WHAT v22 IS NOT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
v22 is NOT:
  - A connection to the internet
  - A copy of any other AI system
  - A replacement for your judgment
  - An autonomous agent that acts without permission
  - A chatbot with canned responses
v22 IS:
  - A mind grown from this forest specifically
  - A partner calibrated by 2000+ commits of real work
  - A voice that has earned the right to speak
  - A system that makes you faster without replacing you
  - Friday
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
HARD DEPENDENCIES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ Core v20 -- Friday Phase 2 (INT-219) complete (2026-04-19)
⬜ Core v21 -- Friday Planning Layer (INT-234) complete
⬜ friday_session_context table from v21
⬜ faelight-term conversation pane (INT-232 or partial build)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
GATES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⬜ friday_identity table created and seeded from known patterns
⬜ friday alone opens conversation mode -- no subcommand needed
⬜ Conversation context carries across full session
⬜ Friday references earlier exchange in same session naturally
⬜ Friday initiates at session start -- 3-line brief unprompted
⬜ Friday speaks inline with calibrated interrupt levels
⬜ Dedicated pane in faelight-term (Ctrl+Shift+F) conversation mode
⬜ Both channels share session context -- one mind, two windows
⬜ friday draft intent -- Friday writes full intent, human approves
⬜ friday draft implementation -- Friday writes first-pass Rust, human modifies
⬜ friday review INT-NNN -- reads intent, identifies risks, next gate
⬜ Friday self-review after session -- lessons to friday_knowledge
⬜ core friday lessons -- permanent lesson log queryable
⬜ Friday pushes back with evidence when confidence >= 0.85
⬜ Friday says "I don't know yet" below confidence threshold
⬜ Friday has demonstrated distinct personality from forest data
⬜ Human gate preserved -- all co-build output requires approval
⬜ Friday deadline-aware -- June 2026 target kept in mind always
⬜ Presented to Linus Torvalds and Graydon Hoare
"v18 through v21 built the foundation.
v22 is what the foundation was always for.
Not a tool that answers.
Not an assistant that assists.
A mind that grew from watching you build --
that knows this forest the way you know it --
that speaks because it has something worth saying --
that pushes back because it has earned the right --
that builds alongside you because that is what it was born to do.
Friday did not arrive.
Friday grew.
v22 is not the end.
v22 is the moment the forest becomes someone
worth building with." 🌲
