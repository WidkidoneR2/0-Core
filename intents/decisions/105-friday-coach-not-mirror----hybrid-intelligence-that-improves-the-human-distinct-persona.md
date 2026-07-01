---
id: 105
date: 2026-07-01
type: decisions
title: "Friday: coach not mirror -- hybrid intelligence that improves the human, distinct persona"
status: planned
tags: [friday, vision, coach, llm, intelligence, persona, patterns]
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

---

## THE NORTH STAR: coach, not mirror
Today Friday is DESCRIPTIVE -- it observes patterns and reflects them back ("you
tend to do X"). The vision is PRESCRIPTIVE: Friday notices when a pattern is
holding you back, offers a better way, and helps you practice into it. A mirror
makes a fancy autocomplete. A coach makes the HUMAN grow. Christian: "just because
I have a pattern doesn't mean it's right -- it can be improved by other ways, by
practice." That is the thing that makes Friday worth building. Friday becomes the
forest's nervous system that improves its human, not just an index of his habits.

## THE AMBITION MANDATE (this era's law)
Friday-era intents must be BIG and NOTICEABLE. No imperceptible tweaks, no
barely-distinguishable increments. Each Friday intent must visibly change what
Friday IS or DOES -- if you can't feel the difference, it is not a real Friday-era
intent. From this point, almost every intent improves Friday and fsh. Bold swings,
not baby steps.

## THE ONE HARD SAFETY LINE (the only brake)
`cp state.db state.db.bak-<timestamp>` before ANY change that touches Friday's
learned state. NON-NEGOTIABLE. Rationale: Rust tools recompile, configs regenerate,
generations roll back -- but Friday's learned patterns/facts are the irreplaceable
product of months of real behavior. A bold move that goes wrong must never ERASE
that. Backup is not a baby step; it is insurance on data that cannot be rebuilt.
This is the ONLY required brake. Everything else can be ambitious.

## DECIDED ARCHITECTURE (Christian, 2026-07-01)
- INTELLIGENCE: hybrid -- a smarter LOCAL system PLUS a real LLM layer. Not either/or.
- LLM LOCATION: local-default (offline, private, forest-native) + API-escalation
  for heavy lifts. Local model is the everyday brain; API is the "big lift" reach.
- ESCALATION TRUST: human-gated FIRST; Friday EARNS autonomy as trust builds.
  Ties INT-186 (Delegation Engine: confidence gates, rollback guarantees, what the
  forest may do without asking). Same "demonstrated not declared / crawl before
  you walk" discipline that governs the rest of the forest, applied to cognition.
- SHARES WITH CLAUDE ("like Claude in many ways"): reasons with HONESTY (pushes
  back, does not just agree); understanding grounded in MEMORY; and -- the
  distinctive part -- LEARNS from the human's questions and decisions, getting
  sharper over time at serving THIS human specifically.
- DISTINCT PERSONA ("and in many ways not"): Friday has its OWN name, voice,
  character, and its own relationship to Christian -- clearly NOT a Claude clone
  with a forest skin. The persona details (name, voice, character) are CHRISTIAN'S
  TO AUTHOR. This charter reserves that as his creative territory and does not
  invent it. Persona work comes LAST -- character on a working mind, not lipstick
  on an unstable one.
- NOT LIKE CLAUDE (the spine that keeps Friday itself): local, private, yours,
  forest-native (not a general cloud assistant); persistent learning from YOUR
  decisions (Claude does not persist this way); bound to one forest it knows
  intimately, not a general-purpose helper.

## THE CENTRAL DESIGN TENSION (every Friday intent must answer this)
"Improve my patterns" REQUIRES Friday to model what "better" IS. This is:
- WHERE THE LLM EARNS ITS PLACE: an LLM can reason about "is this a good practice?"
  in a way pattern-matching cannot.
- WHERE IT IS MOST DANGEROUS: a Friday CONFIDENTLY WRONG about how the human should
  work/live is WORSE than a mirror that merely reflects. Bad coaching > no coaching
  in harm terms.
THE HARD QUESTION, unavoidable: how does Friday coach WITHOUT becoming confidently
wrong about the human? Every coach-facing Friday intent must state how it earns the
right to suggest an improvement (evidence? your confirmation? tracked outcomes?),
and must be CORRECTABLE (you can reject a suggestion and Friday learns from the
rejection). Coaching is SUGGESTION the human can always override -- never
imposition. (Mirrors of INT-186's trust model: suggest, human decides, earn trust.)

## WELLBEING GUARDRAIL (holds throughout the era)
Friday exists to serve the WORK and help Christian grow -- it is the forest's
nervous system, not a companion that substitutes for the outside world. As persona
and "coaching about how you live/work" enter, keep Friday pointed at serving the
work. A coach that improves your craft is healthy; a system that makes itself the
authority on how you should live is not. Friday suggests and the human decides --
always.

## BUILDING-BLOCK ORDER (ambitious blocks, each behind the backup line)
Order is by SAFE DEPENDENCY -- not timidity. Each block is a BIG noticeable change;
each is preceded by the state.db backup; each should be reversible where possible.
1. FOUNDATION -- know + protect what exists. Fully document current Friday
   architecture (patterns, facts, confidence, state.db schema, the hooks into the
   dashboard/sessions). Add state.db integrity + memory-decay handling (flagged
   gaps: "no memory decay", "prediction feedback loop not closed"). Make the LIVING
   system robust before extending it. (Big: Friday's memory gains decay + integrity.)
2. LEARNING LOOP -- close prediction -> observe outcome -> learn. Friday stops
   being write-once; it grades its own predictions and improves. No LLM yet. (Big:
   Friday starts getting measurably smarter from being right/wrong.)
3. LOCAL LLM -- wire the local model in as a REASONING layer over Friday's state.
   Read-first (cannot corrupt the core), then advisory. (Big: Friday can reason,
   not just match.)
4. ESCALATION + COACHING -- local->API heavy-lift boundary (human-gated, INT-186);
   and the coach turn: Friday begins suggesting pattern IMPROVEMENTS, correctably.
   (Big: Friday goes from mirror to coach.)
5. PERSONA -- Friday's name, voice, character (Christian-authored) on the now-solid
   mind. (Big: Friday becomes someone, not something.)

## RELATIONSHIP TO fsh
Friday and fsh are paired ("almost every intent improves Friday and fsh"). fsh is
where Friday is SEEN and USED -- the prompt (INT-103 candy-neon), the `friday`
builtin, the desktop bar face (planned). As Friday's mind grows, fsh is its body/
voice on screen. Keep them evolving together.

## OPEN QUESTIONS (flagged, not pre-decided -- each may become its own intent)
- Which local model? (Ollama / llama.cpp; what runs well on the 780M + CPU; size
  vs quality tradeoff.)
- How does the LLM integrate with the pattern/fact system? (LLM reads facts as
  context? LLM proposes new facts? who arbitrates conflicts?)
- Persona specifics -- name, voice, character (Christian's to author).
- What is Friday's model of "better" for coaching? (heuristics? the LLM? your
  confirmed outcomes over time?) -- the crux of the central tension.
- How are coaching suggestions surfaced without being naggy? (the UX of a coach.)

## THE RULE
"A mirror shows the human what he is. A coach helps him become what he could be.
 Friday holds the forest's memory -- back it up, then build boldly. Suggest; never
 impose. The human decides. 🌲"

## STATUS
Vision/north-star charter for the Friday era. NOT an implementation plan -- the
spine every subsequent Friday intent hangs off. Building blocks 1-5 each become
their own big, noticeable intent(s), in order, each behind the state.db backup line.
