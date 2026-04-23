---
id: 234
title: "Core v21 -- Friday Planning Layer: From Reaction to Anticipation"
status: in-progress
date: 2026-04-16
tags: [core, v21, friday, planning, anticipation, conversation, context, reasoning]
---
v18 gave Friday one voice.
v19 gave Friday a mouth.
v20 gave Friday pattern synthesis.
v21 gives Friday a mind that plans ahead -- and remembers what it thought.
The line between v20 and v21:
v20 predicts across the forest -- temporal models, plan history, trust.
v21 predicts within the session -- context buffer, forward-chaining, anticipation.
v20 speaks when asked. v21 speaks when it has earned the right.
The difference between reaction and anticipation:
Reaction:     "you ran deploy -- I predict fg commit next"
Anticipation: "you are 3 steps into INT-234 -- based on your pattern,
               you will need to deploy core, then verify, then commit.
               The build will take 15 seconds. Health is 100%. Continue."
Friday stops waiting to be asked.
Friday starts knowing what comes next.
ARCHITECTURE
Schema (matches v20 convention -- see phase2.rs):
CREATE TABLE IF NOT EXISTS friday_session_context (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id      TEXT NOT NULL,
    timestamp       INTEGER NOT NULL,
    exchange_kind   TEXT NOT NULL,
    content         TEXT NOT NULL,
    references_id   INTEGER,
    facts_cited     TEXT NOT NULL DEFAULT '',
    confidence      REAL NOT NULL DEFAULT 0.0,
    approved        INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_friday_session_ctx_session
    ON friday_session_context(session_id, timestamp);
exchange_kind values: ask, observation, anticipation, conclusion, signal.
references_id lets Friday cite a prior exchange in the same session.
facts_cited is a comma list of friday_knowledge.id values -- the audit trail for inference.
approved mirrors friday_plan_history -- the feedback loop so accuracy can be measured later.
SESSION LIFECYCLE
Session ID format: YYYYMMDD-HHMMSS-pid (sortable, human-readable, unique per fsh launch).
Start: fsh launch writes current_session_id to friday_state.
       First command writes an exchange_kind=signal, content=session_start row.
End:   fsh clean exit, OR 30 minutes idle since last shell_history entry.
       On end, Friday writes a session summary to friday_knowledge
       (domain=session_summary) containing top 3 exchanges by confidence.
Idle detection: lazy -- every core friday command checks last exchange timestamp.
                If > 30 min, roll previous session forward, emit new session_start.
                Friday Daemon v2 (INT-235) replaces this with active monitoring later.
Ownership:  fsh triggers session-start and session-end.
            core does the writes. fsh is interface; core is policy. (DEC-005)
FORWARD-CHAINING INFERENCE
Given 2+ facts from friday_knowledge, derive a conclusion:
Fact: "deploy takes 15s for core"
Fact: "deploy took 2s this session"
Conclusion: "build did not recompile -- check if file was saved"
Conclusion stored in friday_session_context with facts_cited populated.
This is not retrieval. This is inference.
Simple forward chaining. No ML. No loops. Explainable every time.
RATE LIMITING
v19 established last_spoken_ts (30 min) for unprompted speech. v21 respects it.
Anticipation has its own cooldown: max one anticipation per active-intent context switch,
detected by the active intent changing in the session.
Confidence gate for unprompted speech: >= 0.85 AND rate limit respected.
COMMANDS
core friday plan           -- session-aware evolution of v20 plan; cites prior exchanges
core friday context        -- show current session buffer (last 10 exchanges)
core friday reason <q>     -- chain facts to answer a question
core friday anticipate     -- surface what Friday expects you to need next
core friday session-start  -- fsh launch hook (writes current_session_id)
core friday session-end    -- fsh exit hook (writes session summary)
All six documented in COMMAND-GUIDE as they ship.
// Hard dependency: Core v20 (INT-219) complete -- satisfied 2026-04-19
IMPLEMENTATION GATES
⬜ friday_session_context table created with schema and index
⬜ session lifecycle: session-start / session-end commands wired in fsh
⬜ idle timeout (30 min) detected lazily on every core friday call
⬜ session summary written to friday_knowledge on session end (top 3 by confidence)
⬜ last 10 exchanges stored and queryable via core friday context
⬜ references_id populates when Friday cites a prior exchange
⬜ forward-chaining inference: 2 facts derive 1 conclusion, stored with facts_cited
⬜ core friday plan now cites prior session exchanges when relevant
⬜ core friday context displays current session buffer
⬜ core friday reason chains facts to answer a question
⬜ core friday anticipate predicts next action using session + temporal models
⬜ unprompted speech gated by confidence >= 0.85 AND anticipation enabled AND rate limit
⬜ COMMAND-GUIDE updated with all six new commands
DEMONSTRATION GATES
⬜ Friday used inference in a real session and the conclusion was correct
    (audited via approved=1 on a conclusion row with non-empty facts_cited)
⬜ Friday anticipated the next action and was right
    (audited via approved=1 on an anticipation row)
⬜ Friday referenced a prior exchange in the same session and it helped
    (references_id non-null, human confirms usefulness)
⬜ Session summary preserved useful context across sessions
    (a friday_knowledge entry from a prior session surfaced in a later session)
INTEGRATION
⬜ Planning layer feeds Friday Daemon v2 (INT-235)
⬜ friday panel in faelight-term v12 (INT-232) can read session context
"The forest has been watching.
 v21 is the moment it starts thinking ahead --
 and remembers what it thought." 🌲
