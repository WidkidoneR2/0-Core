---
id: 177
date: 2026-07-19
type: study
title: "promkit: evaluate as reedline alternative for INT-168 interactive input"
status: in-progress
tags: [fsh, promkit, reedline, input, int-168]
---

## Vision
Decide, with evidence, whether promkit belongs in fsh -- and if so, WHERE. This is an
evaluation intent: a documented verdict is the deliverable, not code. "Not a fit for the
line editor" is a legitimate and useful outcome.

## The Problem
INT-168 will swap fsh's line editor (rustyline -> reedline, decided on craft grounds).
Before committing, the alternatives deserve an honest look so the choice is EARNED, not
assumed. promkit is one candidate on the list. The question 177 answers: is promkit a real
reedline alternative for fsh's interactive input, or something else?

## The Solution
Research promkit as it actually is (not from its one-line description), then categorize it
against fsh's real need. RESEARCHED 2026-07-20 (promkit v0.12.1, MIT, 464 stars, active):

promkit is a PROMPT TOOLKIT, not a line editor. Its components are Readline, Confirm,
Password, Form, Listbox, QuerySelector, Checkbox, Tree, JSON, Text -- discrete interactive
prompts and selection widgets. Its model is one-shot: `.prompt().run()` returns a value and
exits. Projects built on it (jnv, sig, logu) are interactive TUI tools, not shells.

reedline, by contrast, owns a shell's PERSISTENT keystroke loop: history, completion,
hinting, multiline editing, the always-on REPL input surface. That is fsh's actual need for
168, and it is a different category from what promkit does.

VERDICT: promkit is NOT the reedline alternative -- different problem. It is, however, a
strong candidate for fsh's INTERACTIVE-SELECTION UI needs elsewhere: a `forest start
<context>` picker (INT-013), the keybinding tester (INT-156), launcher menus, any
"pick-from-a-list" or "confirm-this" moment. So 177 REMOVES promkit from the 168 line-editor
decision (narrowing it toward reedline) and FILES it as a candidate for picker/menu work.

## Success Criteria
- [ ] promkit researched as it actually is -- version, license, maintenance, real component
      list -- not from its tagline. (Done 2026-07-20: v0.12.1, MIT, 464 stars, active;
      README read directly.)
- [ ] promkit categorized against fsh's 168 need (persistent line-editor loop) vs what
      promkit provides (one-shot prompts + selection widgets). Verdict recorded with the
      reasoning.
- [ ] The verdict feeds the 168 decision explicitly: promkit is OUT as the line editor
      (168 narrows toward reedline), noted as a candidate for interactive-selection UI
      (INT-013 / INT-156 / launcher work) instead.
- [ ] Each gate carries evidence per INT-158.

## The Rule
"Evaluate the tool for what it IS, not what the intent title guessed. 'Wrong tool for this
job, right tool for that one' is a real answer -- and it narrows the next decision instead
of muddying it." 🌲
