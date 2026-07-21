---
id: 178
date: 2026-07-19
type: study
title: "inquire: evaluate as prompt-building option for INT-168 / Friday interaction"
status: complete
tags: [fsh, inquire, reedline, input, int-168]
---

## Vision
Decide, with evidence, whether inquire belongs in fsh -- and if so, WHERE. Evaluation intent:
the deliverable is a documented verdict, not code. The verdict came back YES, with a clear and
motivated home -- see below.

## The Problem
inquire was floated as one of three candidates for INT-168's line-editor swap (with promkit/177
and reedline). 178 asks: is inquire a reedline alternative, or a different tool with a different
job in fsh?

## The Solution
Researched 2026-07-20 (inquire v0.9.1, MIT, ~8.2M all-time downloads -- the de-facto standard
Rust prompt library, actively maintained). Prompts: Text (with autocomplete), Editor, DateSelect,
Select, MultiSelect, CustomType, Password. Each is a one-shot "ask, get a value, return" flow.

VERDICT PART 1 -- NOT the line editor. inquire is a PROMPT LIBRARY, not a persistent keystroke
loop. No shell history, no always-on REPL input. reedline is the only one of the three candidates
that owns a keystroke loop, so 168 stands on reedline -- now an EARNED choice, not a guess.
(crates.io itself separates shell-builders like shellfish/shrust from prompt libs like inquire/
promkit -- the ecosystem draws the same line.)

VERDICT PART 2 -- YES, and the home is the INTENT LEDGER (Christian's finding, 2026-07-20). This
is the real payoff and it is bigger than "a menu library for someday." inquire's Select /
MultiSelect / Confirm / Text / Editor map directly onto the ledger workflow Christian runs
constantly:
- `inta` / new-intent as GUIDED CREATION: Select the type from the valid set (no more inventing
  `type: fsh` -- it bit us twice this session), Select the status, MultiSelect tags from existing
  ones (consistent tagging, no typo one-offs), Text/Editor for title and vision.
- Confirm before delete / archive / cancel -- no accidental loss.
- Select-to-act: `cistart` / `dc` / prioritize by picking from a shown list instead of recalling
  the number.
- Edit existing records through prompts instead of hand-editing markdown.
- The same capability also serves Friday interaction ("did you mean X or Y?" as a Select) and
  future pickers/menus (forest-start selector, keybinding tester).
inquire's design goal in its own words -- polished prompts are very easy to add -- means each of
these is a small clean addition, not a hand-rolled TUI.

promkit (177) is the alternative for this same role; inquire is preferred (far wider adoption,
de-facto standard, cleaner Select/MultiSelect/Confirm).

THE WORK ITSELF gets its own intent -- see the follow-up filed for the inquire-powered
interactive ledger. 178's job was the evaluation, and the evaluation says: adopt inquire, for the
ledger first.

## Success Criteria
- [x] inquire researched as it actually is -- version, license, adoption, real prompt list.
      <!-- DONE 2026-07-20. inquire v0.9.1, MIT, ~8.2M all-time downloads (de-facto standard Rust
prompt lib), active. Prompts from docs.rs: Text, Editor, DateSelect, Select, MultiSelect,
CustomType, Password. -->
      (Done 2026-07-20: v0.9.1, MIT, ~8.2M downloads, active; prompts from docs.rs.)
- [x] Categorized against the 168 line-editor need: NOT a line editor (no keystroke loop). 168
      <!-- DONE 2026-07-20. inquire is a one-shot prompt library, no persistent keystroke loop / no
shell history. reedline is the only one of the three candidates that owns a keystroke loop, so 168
stands on reedline -- an earned choice. crates.io separates shell-builders (shellfish/shrust) from
prompt libs (inquire/promkit). -->
      stands on reedline as the only real line-editor candidate. Recorded.
- [x] A concrete, motivated home identified for inquire in fsh, not a vague "someday": the INTENT
      <!-- DONE 2026-07-20. Home = the INTENT LEDGER (Christian's finding): guided inta, MultiSelect
tags, Confirm-before-delete, edit/prioritize by prompt; plus Friday interaction and menus. Verdict:
ADOPT. -->
      LEDGER (guided creation, MultiSelect tags, Confirm-before-delete, edit/prioritize by prompt),
      plus Friday interaction and menus. Verdict: adopt.
- [x] The build work is captured as its own intent so this finding becomes real work, not a note.
      <!-- DONE 2026-07-20, commit bd6efdc5. Filed as INT-181 "faelight-prompt: the forest's
interactive layer" -- a survivor-tier pillar in faelight-core, first surface the guided ledger,
with Friday + git as sequenced siblings. Includes verify-first recon findings. -->
- [x] Each gate carries evidence per INT-158.
      <!-- DONE 2026-07-20. Research + verdict + follow-up intent all recorded with evidence. -->

## The Rule
"An evaluation earns its keep when it finds the RIGHT home, not just a yes/no. inquire is not the
line editor -- it is the interactive skin for the ledger you already live in." 🌲
