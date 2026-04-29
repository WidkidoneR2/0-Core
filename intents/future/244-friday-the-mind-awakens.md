---
id: 244
title: "Core v22 -- Friday: The Useful Partner"
status: planned
date: 2026-04-19
last_revised: 2026-04-28
tags: [core, v22, friday, documentation, cartographer, memory, voice, partner]
---

v18: Friday finds one voice.
v19: Friday finds its mouth.
v20: Friday thinks ahead.
v21: Friday plans and anticipates.
v22: Friday becomes useful.

Not "Friday wakes up and converses in plain English."
Not "Friday writes intents from scratch."

Those framings reach for capabilities that require generative language
models -- explicitly out of scope for this project ("not a connection
to the internet"). The original v22 draft promised them anyway. This
revision is honest about what Friday can be without an LLM, and what
that limited Friday can do that genuinely changes daily work.

v22 is the version where Friday earns its place by doing real work:
keeping documentation in step with development, mapping the system
back to its builder, remembering what was decided and why, and
reflecting on its own performance. None of these require Friday to
"converse." They require Friday to be honest, observant, and tireless
in service of a single human builder maintaining a 50-tool system alone.

This is what v22 needs to be for the NY presentation: not a chatbot,
but a partner that demonstrably reduces the cognitive load of building
something this large with one mind.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 1: DOCUMENTATION STEWARD
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

faelight-docs syncs files. Friday keeps narrative coherence.

The problem this solves:
50 tools. 199 intents complete. 2400+ commits. Documentation lags
constantly. README.md, TOOLS.md, COMMAND-GUIDE.md, CHANGELOG entries,
intent gate updates, architecture diagrams -- each shipped commit
should ripple through these but the ripple is manual today.

What Friday does:
- After every commit, reads the diff and proposes which docs need updating
- After every intent close, drafts the corresponding CHANGELOG entry
  and TOOLS.md edit if a tool was affected
- After every architectural decision (PTY exec, lock state file,
  chattr scope), proposes the doc text that captures the why
- Surfaces stale docs: "TOOLS.md last updated 18 days ago, you've
  shipped 23 commits since"
- All proposals require approval. Nothing auto-writes to source-controlled docs.

Tone of these proposals:
"Three doc updates suggested: TOOLS.md (faelight-shell description),
COMMAND-GUIDE.md (new $? expansion section), CHANGELOG.md (v11.9.1).
Want to review?"

Not "I have made the following changes." Not "Shall I update?" Just
the observation and the offer.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 2: DUAL PRESENCE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Friday lives in two places: inline (fsh) and dedicated (faelight-term pane).

INLINE (fsh):
After every command, Friday MAY speak. Only when it has something
worth saying. Interrupt levels from v20 govern when:
  CHALLENGE: Friday will stop you. "That approach has failed 3 times."
  RECOMMEND: Friday will suggest. "TOOLS.md should be updated after this."
  SUGGEST:   Friday will mention. "INT-234 has 11 open gates."
  SILENT:    Friday watches.

Inline speech is single-line, no fanfare. Style proven by the prediction
arrows already shipped ("-> deploy core (99%)").

DEDICATED PANE (faelight-term Ctrl+Shift+F):
Persistent context view. Shows:
- Current session arc (commits, deploys, intents touched, health trend)
- Open documentation suggestions
- Active contradictions Friday detected
- Recent knowledge entries relevant to what you're working on

This pane is read-mostly. Not a conversation interface. A constantly-
updated dashboard that reflects Friday's awareness of the session.

Both channels share session context. What Friday flagged inline at
14:00 shows in the pane at 16:00 with status "still open" or "resolved
in commit X."

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 3: PERSISTENT MEMORY ACROSS SESSIONS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Friday remembers. Not preferences -- decisions and their reasons.

friday_decisions table:
  - decision_id, timestamp
  - what was decided
  - what alternatives were considered
  - why this path was chosen
  - what intent / commit ties to it
  - what to revisit if the assumptions change

Examples of recorded decisions:
- "Skip runtime/ from chattr +i because daemons need write access"
- "Path 3 narrow PTY introduction (not full refactor) because deadline"
- "grep | not a fsh bug -- documented, no code change"
- "Defer 10 tool README intents -- shell focus during stabilization week"

When you ask "why did we exclude runtime/?", Friday answers from this
record. When future you (or anyone reading the repo) asks, the
record is there in plain language.

Working-memory recovery:
On session start, Friday emits a 3-line brief:
  - Where you left off (last open thread)
  - What's still in flight
  - The most recent decision that might affect today's work

This is the "where was I?" 5-second answer instead of the 5-minute
reconstruction.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 4: SYSTEM CARTOGRAPHER
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Friday holds a live, accurate map of what exists.

The state friday_map maintains:
  - tools: name, version, role, status, last commit, last deploy
  - intents: id, status, owner-of-the-moment (which tool/area)
  - dependencies: tool A depends on tool B, change to A affects B
  - patterns: which tools tend to be deployed together, in what order
  - health: per-tool score and trend

Live updates:
- Tool builds -> map refreshes that tool's metadata
- Intent state changes -> map updates linked tools
- Dependencies change -> map propagates the new edges

What the cartographer enables:
- "Show me everything affected by changing faelight-core's API"
  Friday traces the dependency edges and lists impacted tools/intents.
- "What are the candidates for retirement?"
  Friday surfaces tools matching the audit-stale signature with
  context (no events, missing README, intent already mentions
  consolidation).
- Pre-deploy: "Deploying faelight-shell will trigger downstream
  rebuild signals for faelight-term. OK?"

This is the externalized version of the mental model you currently
carry around. The map survives sleep, suspend, reboot, and
distraction.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 5: SELF-REVIEW
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

After every session, Friday writes a debrief to friday_knowledge.

Per-session debrief contents:
- Predictions made / right / wrong
- Suggestions offered / accepted / dismissed
- Documentation proposals approved / rejected
- Decisions Friday recorded
- New patterns observed
- Old patterns that did not hold

The debrief is not philosophical -- it's accounting. Numbers Friday
can use to calibrate next session's confidence levels.

Visible to you:
  core friday self-review        -- last session's debrief
  core friday self-review --week -- weekly aggregate
  core friday lessons            -- patterns Friday updated this period

The debrief feeds back into Friday's confidence scoring. Predictions
that have been right 9 of 10 times get higher confidence. Patterns
that broke last session get demoted until they prove themselves again.

This is the only "self-improvement" Friday does in v22. Honest, narrow,
and based on data Friday already records. No claims of "learning" --
just bookkeeping that lets confidence stay calibrated.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 6: VOICE (TONE CALIBRATION)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Friday speaks in templated patterns -- but the templates themselves
have been calibrated by 2000+ commits of real work.

What this means concretely:
- Friday does NOT generate prose. No LLM, no internet, no chat.
- Friday DOES select among carefully-tuned phrasings.
- The phrasings reflect a deliberate voice: direct, evidence-based,
  willing to push back, willing to say "I don't know yet."

Examples of the voice (not new in v22 -- already partially shipped):
  "deploy completes -> fg commit (99%)"  -- short, confident, factual
  "Friday knows this (99% confidence): ..." -- pointer, not lecture
  "1 contradiction active. CONTRADICTION: ..." -- name the issue, no apology

What v22 adds:
- Confidence-gated phrasing. Below 0.85, Friday says "I don't know yet."
  Above 0.85, Friday states the observation directly without hedging.
- Contradiction phrasing. When Friday's data conflicts (e.g. "you
  usually deploy core first, but you haven't this session"), Friday
  surfaces the contradiction explicitly, not as a guess.
- Push-back phrasing. When confidence is high AND the user's planned
  action contradicts past data, Friday says so once, then drops it.

Friday does not pretend to converse. It speaks in fixed patterns
with calibrated content. That is enough.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
WHAT v22 IS NOT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

v22 is NOT:
  - Natural language conversation (would require an LLM)
  - Friday writing intent files from scratch (same)
  - A connection to the internet
  - A copy of any other AI system
  - A replacement for human judgment
  - An autonomous agent that acts without permission

v22 IS:
  - A documentation steward that proposes -- never auto-writes
  - A persistent map of the system, kept current
  - A persistent record of decisions and their reasons
  - A bookkeeping system for Friday's own predictions
  - A calibrated voice that has earned the right to push back
  - A partner that reduces cognitive load on a single human builder

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
WHY THIS MATTERS FOR THE NY PRESENTATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

The presentation is not about Friday-the-product. It is about:

1. The collaborative experience of building a full operating system
   with an AI partner over months -- what that takes, what it
   produces, where it succeeds and fails.

2. Friday demonstrating real work -- catching a build error and
   surfacing the right knowledge, predicting the next step,
   updating documentation in real time, recovering session context
   after a long break.

3. A 99.9% Rust system as proof that a single human + AI can build
   what teams of dozens normally build, in months instead of years,
   with more architectural coherence than committee-built distros
   precisely because there is one mind behind the decisions.

4. fsh as the long-term thesis -- a Rust shell that humanizes Linux,
   built to be understood, not memorized. v22 is one step on that
   path; full vision is years.

5. The implication: Rust is the future of kernel work, of distros,
   of system tooling. This forest is one prototype of what that
   future looks like. Post-Linus is not a slogan -- it is the
   honest question this work is starting to ask.

v22 must demonstrate Friday doing useful work in real conditions.
Not Friday philosophizing about consciousness. Not Friday
writing prose. Friday saving Christian time by being a competent
partner in the specific, demoable ways listed above.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
HARD DEPENDENCIES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Core v20 -- Friday Phase 2 (INT-219) complete (2026-04-19)
✅ Core v21 -- Friday Planning Layer (INT-234) complete (2026-04-28)
✅ friday_session_context table from v21 (live)
✅ Confidence scoring infrastructure from v19 (live)
⬜ faelight-term v2 dedicated pane (INT-232 ships before Pillar 2)
⬜ friday_decisions table created (Pillar 3)
⬜ friday_map table created (Pillar 4)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
GATES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Pillar 1 -- Documentation Steward:
⬜ Friday parses commit diffs and proposes affected docs
⬜ Doc proposals presented inline after commit; user approves to apply
⬜ Stale-doc detection: TOOLS.md / README / CHANGELOG age vs commit count
⬜ Demonstrated: a commit triggers a real doc proposal that lands in source

Pillar 2 -- Dual Presence:
⬜ Inline speech respects v20 interrupt levels
⬜ faelight-term Ctrl+Shift+F pane shows live session arc
⬜ Pane shows open Friday observations with status (open/resolved)
⬜ Both channels share friday_session_context state

Pillar 3 -- Persistent Memory:
⬜ friday_decisions table created with schema (id, timestamp, what,
   alternatives, why, ties_to, revisit_when)
⬜ Decisions recorded automatically on intent close, after major commits
⬜ core friday why <topic> queries the decision record
⬜ Session-start brief (3 lines) shipped on first friday call of new session

Pillar 4 -- System Cartographer:
⬜ friday_map table tracking tools, intents, dependencies, patterns, health
⬜ Map updates on every build / deploy / intent transition
⬜ core friday impact <change> traces dependency edges
⬜ Pre-deploy check surfaces affected downstream tools

Pillar 5 -- Self-Review:
⬜ Per-session debrief written to friday_knowledge automatically
⬜ Debrief includes prediction accuracy, suggestion accept rate, decisions recorded
⬜ core friday self-review queries last session's debrief
⬜ Confidence scoring uses debrief feedback to calibrate next session

Pillar 6 -- Voice:
⬜ Confidence-gated phrasing live (>=0.85 direct, <0.85 "I don't know yet")
⬜ Contradiction surfacer phrases conflicts explicitly
⬜ Push-back phrasing demonstrated in real session (Friday said "X has
   failed N times" and was right)

Final / Demonstration:
⬜ NY presentation rehearsal: Friday demonstrates 4 pillars live in <10 min
⬜ Session recovery demonstrated: 24h gap, Friday brings session back in 5s
⬜ Documentation proposal accepted in a real workflow (not staged demo)
⬜ Cartographer answers "what does X depend on" in real working session
⬜ Self-review aggregate shows calibrated confidence (predictions right
   X% of time matches stated confidence band)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
WHAT WAS REMOVED FROM ORIGINAL v22
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

For honesty -- the original draft had two pillars that required an LLM:

- Original Pillar 1: Natural Language Conversation
  ("friday alone opens conversation mode... You: what should I work on next?
   Friday: INT-234. The planning layer is the natural continuation...")

  Removed because: this requires generative language. Without an LLM, the
  best Friday could do is templated responses that fall over fast. Promising
  this would require either an internet connection (violates project
  principles) or shipping something that demos well once and embarrasses
  on the second run.

- Original Pillar 4: Friday Co-Builds
  ("friday draft intent ... Friday writes a full intent file from scratch")

  Removed because: same reason. Generating coherent intent files requires
  language modeling Friday does not have. Christian writes better intents
  than templated generation could produce. Co-build was reaching for
  capability the architecture cannot support.

These two pillars are not abandoned -- they are deferred to a future
intent ("Friday with Voice -- Local Language Model Integration") that
honestly scopes the LLM dependency.

The four pillars that remain are the ones a non-LLM Friday CAN ship,
that genuinely change the work, and that demonstrate well in the NY
presentation context.

"v18 through v21 built the foundation.
v22 is what the foundation can honestly support today.

Not a mind that wakes up.
A partner that shows up.

The forest already has a voice -- it speaks in patterns,
in confidence scores, in the predictions arrow that knows
what comes next. v22 makes that voice useful at scale:
documentation that keeps up, a map that stays current,
decisions that stay legible, predictions that stay
calibrated.

This is Friday earning its place, one demonstrable
capability at a time. The bigger leap waits for v23." 🌲
