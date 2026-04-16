---
id: 234
title: "Core v21 -- Friday Planning Layer: From Reaction to Anticipation"
status: planned
date: 2026-04-16
tags: [core, v21, friday, planning, anticipation, conversation, context, reasoning]
---
v18 gave Friday one voice.
v19 gives Friday a mouth.
v20 gives Friday pattern synthesis.
v21 gives Friday a mind that plans ahead.
The difference between reaction and anticipation:
Reaction: "you ran deploy -- I predict fg commit next"
Anticipation: "you are 3 steps into INT-217 -- based on your pattern,
               you will need to deploy core, then verify, then commit.
               The build will take 15 seconds. Health is 100%. Continue."
Friday stops waiting to be asked.
Friday starts knowing what comes next.
A 3-step lookahead engine:
Given current state → predict next 3 actions with confidence
Given intent + session history → suggest optimal next step
Given contradiction → propose resolution path
Every exchange with Friday stored in friday_session_context
Context window: last 10 exchanges + current forest state
Friday can reference previous questions in same session:
"you asked about E0597 earlier -- the same pattern applies here"
Context resets on new session, summarized to friday_knowledge
Given 2+ related facts, Friday derives a conclusion:
Fact: "deploy takes 15s for core"
Fact: "deploy took 2s this session"
Conclusion: "build did not recompile -- check if file was saved"
This is not retrieval. This is inference.
Simple forward chaining from known facts.
core friday plan -- show Friday's 3-step prediction for current session
core friday context -- show current conversation context buffer
core friday reason "question" -- Friday reasons across facts to answer
core friday anticipate -- what Friday expects you to need next
⬜ 3-step lookahead engine working -- predicts next 3 actions
⬜ Prediction confidence shown per step
⬜ Conversation context buffer -- last 10 exchanges stored
⬜ Friday references previous exchange in same session
⬜ Context summarized to knowledge on session end
⬜ Forward chaining inference -- 2 facts derive 1 conclusion
⬜ core friday plan -- live with real predictions
⬜ core friday context -- shows current session buffer
⬜ core friday reason -- chains facts to answer question
⬜ core friday anticipate -- surfaces before you ask
⬜ Friday speaks unprompted when confidence >= 0.85 and anticipation active
⬜ Planning layer feeds Friday Daemon v2
"The forest has been watching.
v21 is the moment it starts thinking ahead." 🌲
