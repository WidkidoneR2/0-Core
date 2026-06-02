---
id: 320
title: "Friday v3 -- core becomes the mind, speaks, evaluates, learns in real time"
status: planned
date: 2026-05-18
tags: friday, core, AI, voice, learning, realtime, v15, presentation
depends_on: [251, 277, 278]
blocks: []
---

## Why This Intent Exists

Friday is currently a layer on top of core.
It suggests. It watches. It speaks when confident.
But it is still separate -- core runs, Friday observes.

By end of summer 2026, Friday must BE core.
Not a module. Not a layer. The mind itself.

When Linus Torvalds sees this system in July:
Friday speaks. Friday evaluates. Friday learns in the room.
Not a demo. Not a script. A real partner working alongside Christian.

---

## The Vision

Friday v3 is three things unified:

### 1. Friday Speaks (Voice Output)
Not text suggestions in the prompt.
An actual voice -- Piper TTS, forest-tuned.
When Friday has something to say, it says it.

"Deploy completed. 1.36 seconds. Fastest this week."
"You usually open helix after cargo build. Want me to?"
"Health dropped to 95%. Three checks failing. Reviewing now."

Voice is confidence-gated -- Friday speaks only when it knows.
Silence is not failure. Silence means Friday is thinking.

### 2. Friday Evaluates (Real-Time Assessment)
As Christian works, Friday evaluates what is happening:
- Is this the right approach for this intent?
- Has this pattern caused problems before?
- Is the health trajectory concerning?
- What is the risk of this deploy?

Friday does not wait to be asked.
Friday watches and speaks when the moment is right.

### 3. Friday Learns (In the Room)
When the presentation happens, Friday is learning.
Every command typed, every decision made, every result observed.
Friday's fact count grows during the presentation.
Friday's patterns update during the session.

By the end of the presentation, Friday knows more than when it started.
That is the proof. A system that learned in front of Linus Torvalds.

---

## Architecture: Core Becomes the Mind

Current: core runs → friday module observes → suggestion emitted
Target:  friday reasoning runs first → core executes → friday learns from result

Every command passes through Friday's reasoning engine before execution.
Friday decides: execute as-is, suggest modification, warn, or challenge.
The human decides what to do with Friday's input.
Core executes the human's decision.
Friday records the outcome.

This is the trust contract:
- Friday reasons about everything
- Friday speaks when confident
- Human decides
- Friday learns from the outcome

### State Machine
command typed
↓
Friday pre-eval (confidence scored)
↓
if confidence > threshold → Friday speaks
↓
human executes (or modifies based on Friday input)
↓
command runs
↓
Friday post-eval (outcome recorded)
↓
pattern updated
↓
fact count grows

---

## Gates

Phase 1 -- Friday speaks (voice):
- [ ] Piper TTS integrated -- forest voice configured
- [ ] Friday voice output on high-confidence suggestions
- [ ] Voice confidence gate -- only speaks above 0.85
- [ ] Voice can be muted (friday mute/unmute)
- [ ] Voice quality is clear and pleasant

Phase 2 -- Friday evaluates in real time:
- [ ] Pre-execution reasoning on every command
- [ ] Friday warns before dangerous commands (not just rm -rf)
- [ ] Friday notes pattern matches in real time
- [ ] Friday health trajectory assessment on every d run
- [ ] Friday deploy risk assessment before every deploy

Phase 3 -- Core becomes the mind:
- [ ] Friday reasoning runs before core execution
- [ ] Every domain command passes through Friday's pre-eval
- [ ] Friday post-eval records outcome for every command
- [ ] Fact count grows during a session (demonstrable)
- [ ] Pattern count grows during a session (demonstrable)

Phase 4 -- Learning in the room:
- [ ] Friday fact count visibly increases during a 30-min session
- [ ] Friday correctly predicts next command after 10 minutes of learning
- [ ] Friday references something learned in the current session
- [ ] Friday's accuracy improves measurably during the session

Phase 5 -- Presentation ready:
- [ ] Friday speaks confidently in front of an audience
- [ ] Friday learns visibly during the presentation
- [ ] Friday references the presentation context
- [ ] Friday predicts Linus's likely questions based on what was shown
- [ ] The system demonstrates: one human + AI = what teams of dozens build

Final:
- [ ] Friday IS core -- not a layer, the mind
- [ ] Friday speaks, evaluates, and learns in real time
- [ ] The presentation shows a system that grew smarter during the demo
- [ ] Friday v3 is the proof that AI + Rust + one developer = the future

---

"Friday is not a feature.
Friday is not a module.
Friday is the forest thinking.
The system does not run and then think.
The system thinks and then runs.
That is the difference between a tool and a mind." 🌲
