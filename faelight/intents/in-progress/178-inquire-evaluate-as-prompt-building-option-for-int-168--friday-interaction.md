---
id: 178
date: 2026-07-19
type: study
title: "inquire: evaluate as prompt-building option for INT-168 / Friday interaction"
status: in-progress
tags: [fsh, inquire, reedline, input, int-168]
---

## Vision
Decide, with evidence, whether inquire belongs in fsh -- and if so, WHERE. Evaluation intent:
the deliverable is a documented verdict, not code. Like INT-177 (promkit), "not the line
editor" is a legitimate and useful outcome.

## The Problem
INT-168 swaps fsh's line editor (rustyline -> reedline, decided on craft grounds). inquire is
the third candidate that was floated (with promkit/177 and reedline). Before 168 commits, the
alternatives get an honest look so the choice is EARNED. The question 178 answers: is inquire
a real reedline alternative for fsh's line editor, or a different kind of tool?

## The Solution
Research inquire as it actually is, then categorize it against fsh's real 168 need. RESEARCHED
2026-07-20 (inquire v0.9.1, MIT, ~8.2M all-time downloads, actively maintained -- the de-facto
standard Rust prompt library):

inquire is a PROMPT LIBRARY, not a line editor. Its prompts are Text, Editor, DateSelect,
Select, MultiSelect, CustomType, Password -- discrete "ask the user for information via the
CLI" flows that return one value and exit. Same category as promkit. It has NO persistent
keystroke loop, no shell history, no always-on REPL input. (Its Text prompt has autocompletion,
but it is still one-shot ask-and-return, not the surface that owns every keystroke at a shell
prompt.)

Independent confirmation from the ecosystem: crates.io's "interactive" keyword separates
SHELL-builder crates (shellfish "run custom interactive shells", shrust) from PROMPT crates
(inquire, promkit). inquire is filed as a prompt library, not a shell-input library -- the
ecosystem itself draws the line 178 is drawing.

VERDICT: inquire is NOT the reedline alternative -- different category, same as promkit. It IS
a strong candidate for fsh's interactive-selection UI elsewhere, and arguably the STRONGER of
the two (far wider adoption, the de-facto standard, clean Select/MultiSelect/Confirm). So for
the picker/menu intents (INT-013 forest-start picker, INT-156 keybinding tester, launcher
menus), inquire is the leading candidate; promkit is the alternative.

## Success Criteria
- [ ] inquire researched as it actually is -- version, license, adoption, real prompt list --
      not from its tagline. (Done 2026-07-20: v0.9.1, MIT, ~8.2M downloads, active; prompt list
      from docs.rs.)
- [ ] inquire categorized against fsh's 168 need (persistent line-editor loop) vs what inquire
      provides (one-shot prompts). Verdict recorded with reasoning.
- [ ] The verdict feeds 168 explicitly: inquire is OUT as the line editor (168 stands on
      reedline -- the only one of the three candidates that owns a keystroke loop), noted as the
      LEADING candidate for interactive-selection UI (INT-013 / INT-156 / launcher).
- [ ] Each gate carries evidence per INT-158.

## The Rule
"Two evaluations, same finding: the 'alternatives' solve a different problem. That is not
wasted work -- it is what makes choosing reedline an EARNED decision instead of a guess." 🌲
