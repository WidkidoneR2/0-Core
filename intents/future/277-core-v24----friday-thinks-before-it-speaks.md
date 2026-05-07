---
id: 277
title: "Core v24 -- Friday Thinks Before It Speaks"
status: planned
date: 2026-05-06
tags: [core, friday, attention, intelligence, clarification, v24, architecture]
depends_on: [246, 251]
---
INT-216 taught Friday to observe.
INT-244 taught Friday to speak.
INT-246 teaches Friday when to speak.
Core v24 teaches Friday what deserves its attention in the first place.
The difference between a noisy system and an intelligent one
is not how often it speaks.
It is what it chooses to notice.
---
THE PROBLEM WITH EQUAL ATTENTION
Friday currently treats all events above a confidence threshold equally.
Deploy fires. Friday notices. Pattern matches. Friday speaks.
Health changes. Friday notices. Pattern matches. Friday speaks.
But not all events are equal.
A deploy after 3 days of silence is different from a deploy after 5 minutes.
A health drop at 2am is different from a health drop mid-session.
A contradiction in a new domain is different from a known recurring pattern.
Friday needs to weight significance before deciding to speak.
Not all events deserve equal cognition.
---
PILLAR 1: THE ATTENTION SCORE
Before Friday decides to speak, it computes an attention score.
attention_score = novelty × risk × strategic_relevance × uncertainty × temporal_pressure
NOVELTY (0.0 - 1.0):
  Has Friday seen this exact pattern before?
  First occurrence: 1.0
  Seen 10+ times with same outcome: 0.1
  Novelty decays with repetition.
RISK (0.0 - 1.0):
  What is the potential downside if Friday is wrong or silent?
  Health drop below 90%: 1.0
  Routine deploy: 0.2
  Schema change detected: 0.9
  Commit without cistart: 0.6
STRATEGIC_RELEVANCE (0.0 - 1.0):
  Does this relate to the active intent?
  File changed in active intent domain: 1.0
  Unrelated tool deployed: 0.2
  Friday contradiction in active intent area: 0.9
UNCERTAINTY (0.0 - 1.0):
  How uncertain is Friday about the right action?
  High confidence pattern (>90%): 0.1 uncertainty
  Ambiguous command with multiple interpretations: 0.9
  Unknown pattern: 1.0
TEMPORAL_PRESSURE (0.0 - 1.0):
  Is time a factor?
  Presentation approaching: 0.8
  Normal session: 0.3
  Idle period: 0.1
THRESHOLD:
  attention_score >= 0.6 → Friday considers speaking
  attention_score < 0.6 → Friday observes silently
  attention_score >= 0.9 → Friday interrupts (CHALLENGE tier)
Friday interrupts rarely.
But when it does, it matters.
---
PILLAR 2: CLARIFICATION DIALOGUES
The smartest thing a system can do is ask good questions sparingly.
When Friday detects an ambiguous command with high uncertainty,
instead of guessing or staying silent, it asks once, precisely.
TRIGGER CONDITIONS:
  uncertainty > 0.7 AND strategic_relevance > 0.5
  Command maps to 2+ distinct action trees
  Risk > 0.6 AND outcome is irreversible
FORMAT (always the same structure):
  Friday: "[command] could mean:"
  "  1. [specific action A]"
  "  2. [specific action B]"
  "  3. [specific action C]"
  "  What do you intend? (1/2/3 or Esc to cancel)"
RULES:
  Never open-ended questions -- always numbered options
  Never more than 4 choices -- if more, Friday picks the safest default
  Never asks the same clarification twice in a session
  One keystroke to answer -- no Enter required
  Esc always cancels cleanly
EXAMPLE:
  Christian types: clean system
  
  Friday: "clean system could mean:
    1. remove orphan packages (paru -Qdtq | paru -Rs -)
    2. clear package cache (paru -Sc)
    3. prune logs older than 7 days (journalctl --vacuum-time=7d)
    4. archive old sessions and checkpoints
  What do you intend? (1/2/3/4 or Esc)"
  Christian presses: 1
  Friday executes: paru -Qdtq | paru -Rs -
  Friday records: clean system → orphan removal (feeds future pattern)
This is not chatbot behavior.
This is operational precision.
Friday learns from every clarification answered.
Over time, Friday stops asking -- it already knows what you mean.
---
PILLAR 3: ATTENTION MEMORY
Friday remembers what deserved attention and what did not.
friday_attention table:
  id
  timestamp
  event_type
  attention_score
  novelty
  risk
  strategic_relevance
  uncertainty
  temporal_pressure
  spoke (boolean -- did Friday interrupt?)
  outcome (was speaking the right call?)
  feedback (accepted/rejected/ignored)
This table teaches Friday two things:
  1. Which events genuinely needed attention (outcome = positive)
  2. Which events Friday misjudged (spoke when it should not have)
Over time, attention scoring becomes more accurate.
Friday learns not just what to say, but what to notice.
---
PILLAR 4: THE SILENCE METRIC
Friday's value is measured not just by what it says.
But by what it correctly chose not to say.
silent_correct:
  Friday computed attention_score < 0.6
  Did not speak
  Outcome: nothing went wrong
  This is a positive signal -- Friday correctly filtered noise
silent_wrong:
  Friday computed attention_score < 0.6
  Did not speak
  Outcome: something went wrong that Friday could have predicted
  This is a negative signal -- Friday missed something important
Both are tracked. Both feed the attention model.
A Friday that speaks too often is noise.
A Friday that misses critical signals is dangerous.
The goal is precise, rare, correct interruption.
---
GATES
Pillar 1 -- Attention Score:
[ ] attention_score formula implemented in core
[ ] All 5 dimensions computed per event
[ ] Threshold enforced: score < 0.6 = silent
[ ] Demonstrated: routine deploy does not trigger Friday speech
[ ] Demonstrated: health drop below 90% triggers Friday speech
Pillar 2 -- Clarification Dialogues:
[ ] Ambiguity detection implemented for common command patterns
[ ] Clarification dialogue format implemented in fsh
[ ] One-keystroke response (no Enter required)
[ ] Friday records clarification answer for pattern learning
[ ] Never asks same clarification twice in a session
[ ] Demonstrated: "clean system" triggers clarification, answer feeds pattern
Pillar 3 -- Attention Memory:
[ ] friday_attention table created in state.db
[ ] All 5 dimensions stored per attention event
[ ] spoke + outcome tracked per event
[ ] Attention scoring improves after 50+ events
Pillar 4 -- Silence Metric:
[ ] silent_correct tracked in friday_usefulness
[ ] silent_wrong tracked in friday_usefulness
[ ] Both visible in friday status output
[ ] Friday silence is as measurable as Friday speech
Final Validation:
[ ] Friday speaks noticeably less than before Core v24
[ ] Every interruption is relevant and actionable
[ ] Clarification dialogue used and answered correctly in real session
[ ] Attention score visible in friday debug output
[ ] Christian says: "Friday interrupted me and it was right"
[ ] Graydon asks: how does it decide when to speak?
[ ] The answer is: it computes attention, not just confidence
"Friday does not speak because it detected a pattern.
Friday speaks because the pattern deserved attention,
the moment had urgency,
the risk justified interruption,
and Friday had earned the right to be heard.
The forest is not loud.
The forest is precise." 🌲
