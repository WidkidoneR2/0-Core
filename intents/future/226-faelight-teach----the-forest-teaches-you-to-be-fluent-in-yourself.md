---
id: 226
date: 2026-04-13
type: feature
title: \"faelight-teach -- The Forest Teaches You to Be Fluent in Yourself\"
status: planned
tags: [feature, rust, faelight]
version: TBD
requires: [208,223]
unlocks: []
strategic_value: leaf
---
Building a system and being fluent in it are different skills.
You built fsh. But do you reach for query by reflex?
You built fsearch. But do you still type grep first?
You built core partner consult. But do you use it before big decisions?
You built core why. But do you ask it when something feels wrong?
Creation is not internalization.
Mastery is not ownership.
faelight-teach exists because the builder deserves to become the master.
Not through documentation. Not through memory.
Through practice. Every session. Naturally.
The teach system observes without interrupting.
Every session it tracks:
- Commands you used the slow way when a faster way exists
- Shell builtins you forgot to use (query, fsearch, patch, edit, run)
- Core commands you have never touched
- Patterns you repeat that could be automated
- Gaps between what the system can do and what you actually ask of it
It does not judge. It notices.
And when the moment is right, it teaches.
At session start, after the welcome message, one small thing:
  🌿  Today you can practice:
  You used grep 4 times this week where fsearch would be faster.
  Try this now:
    fsearch "fn expand" --type rs
  Then try:
    fsearch "pattern" --file main.rs
  30 seconds. Then we build.
Not a quiz. Not a test.
A moment of intentional practice before the day begins.
Dismissable with any keystroke.
Completable in under a minute.
faelight-teach reads your shell_history and commit_patterns.
It knows what you did. It knows what you could have done instead.
Gap categories:
- Shell gaps: using external tools when builtins exist
- Core gaps: commands you have never run in 30+ days
- Pattern gaps: sequences you repeat that a single command could replace
- Velocity gaps: things that take you 3 steps that should take 1
Gap examples it detects:
  head -n 50 file.rs            → query file.rs :50
  grep -n pattern file.rs       → query file.rs pattern
  cat file | grep pattern       → cat file | grep pattern (now native)
  python3 /tmp/fix.py           → run fix.py
  git diff file.rs              → diff file.rs (INT-224)
teach practice launches an interactive session:
  teach practice shell     — practice fsh builtins
  teach practice core      — practice core commands
  teach practice pipes     — practice native pipe patterns
  teach practice query     — practice query/fsearch/patch
  teach practice gaps      — work on your personal detected gaps
Each exercise:
1. Shows you a scenario from your real work history
2. Asks you to solve it the forest way
3. Compares your answer to the optimal command
4. Explains why the forest way is better
5. Records your result to learning_history
teach progress shows your fluency journey:
  Shell builtins:   ████████░░  82% fluent
  Core commands:    ████░░░░░░  43% fluent
  Pipe patterns:    ██████░░░░  61% fluent
  Query/search:     █████████░  90% fluent
Fluency = (times used correctly) / (times the opportunity arose)
This is not a score to optimize.
It is a mirror.
Not badges. Not gamification.
Milestones that mean something:
  First time you used fsearch instead of grep: recorded
  First session with zero /tmp files: recorded
  First time you used core partner consult before a decision: recorded
  First week with 100% native pipe usage: recorded
The forest remembers your firsts.
When Friday arrives, teach and Friday become partners.
Friday observes your work patterns.
Teach turns those observations into lessons.
Friday says: "You have been avoiding core why lately."
Teach says: "Let us practice it right now."
The teacher knows what the student needs
because the student and the teacher share the same memory.
  teach                    — show today lesson (same as session start)
  teach practice <topic>   — interactive practice session
  teach progress           — fluency dashboard
  teach gaps               — show current detected gaps
  teach history            — learning history and milestones
  teach skip               — skip today lesson (recorded, not judged)
  teach config             — configure lesson frequency and topics
fsh v4/v5 — shell builtins complete (INT-223 done)
state.db shell_history — session data flowing
commit_patterns — session patterns available (INT-208 done)
faelight-teach v1 — existing tool (extend, do not replace)
Phase 1 — gap detector (reads shell_history, identifies missed opportunities)
Phase 2 — morning lesson at session start (one gap, one practice, dismissable)
Phase 3 — interactive practice mode (teach practice shell/core/pipes)
Phase 4 — fluency tracking (teach progress dashboard)
Phase 5 — milestone recording (firsts and achievements)
Phase 6 — Friday integration hooks (observations feed teach)
⬜ gap detector reads shell_history and identifies missed builtins
⬜ gap detector identifies fsearch vs grep opportunities
⬜ gap detector identifies query vs head/tail opportunities
⬜ morning lesson fires at session start (dismissable)
⬜ morning lesson shows one gap with two practice examples
⬜ teach practice shell — interactive exercises from real work history
⬜ teach practice core — exercises for unused core commands
⬜ teach progress — fluency dashboard with per-topic percentages
⬜ teach gaps — shows current personal gap list
⬜ teach history — learning history and milestone log
⬜ milestone: first fsearch use recorded
⬜ milestone: first zero-/tmp session recorded
⬜ Friday integration hooks prepared
⬜ d passes 100% after full implementation
"You built this forest.
Every tree. Every root. Every signal.
But a gardener who never walks their own garden
grows blind to what it has become.
faelight-teach is your morning walk.
One observation. One practice. One step toward fluency.
Not because you are behind.
Because mastery is not a destination.
It is a daily choice.
The forest wants to teach you
everything it already knows." 🌲
