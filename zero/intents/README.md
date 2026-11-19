# Intent ledger

Work in progress lives on one shelf. Finished work is a row. Record shelves
keep their own numbers.

## Folders

- `open/` is the lifecycle shelf. Status is the `status:` line in the file.
  There is no `in-progress/`, `complete/` or `future/` directory, and there
  is no `arch-era/`.
- `closed.tsv` is the index of finished intents. A row is the id, the status
  (`complete` or `cancelled`), the day it closed, the last commit, and the
  title.
- `decisions/`, `experiments/`, `incidents/` and `philosophy/` are record
  shelves. Each owns its own id sequence.
- `findings/` holds F-NNNN records. A finding number is not an intent id.

## Format 2

A file opts in with `format: 2`. It has four sections: Vision, The Problem,
The Solution, and Success Criteria. Under each gate sit indented `ask:`,
`ruling:` and `evidence:` fields. A wrapped line stays indented. A ticked
gate with an empty `evidence:` does not complete. A placeholder body, the
old INT-158 comment, a `[~]` gate, or a missing section is an issue.

A file with no `format:` line, or a number below 2, is format 1. The same
shape notes are listed. They are not issues, and completion does not require
`evidence:`.

## Format 3

`format: 3` keeps every format 2 rule and requires a `## Limits` section.
Each label below is a bullet at the start of the line, with text after the
colon. A wrapped line is indented.

- In scope:
- Out of scope:
- How deep:
- Stop and ask:
- Done means:

`- Must not change:` may be added. It is not required. A format 3 file with
no `## Limits`, or with one of the five labels missing or empty after the
colon, does not validate. A number above 3 is an issue (`unknown format 4`).
`core intent new` writes a format 3 file with these five labels between The
Solution and Success Criteria. An existing intent stays on its current format
until its own work moves it.

## Template

```text
---
id: NNN
date: YYYY-MM-DD
type: future
title: "..."
status: planned
tags: []
format: 3
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Limits
- In scope:
- Out of scope:
- How deep:
- Stop and ask:
- Done means:

## Success Criteria
- [ ] ...
      ask:
      ruling:
      evidence:
```

## Reading a closed intent

Closing removes the file from `open/` and appends its row to `closed.tsv`.
`core intent show <id>` prints that row and the command that reads the old
text. The text is the file as it stood in the last commit:

```text
git show <sha>:zero/intents/open/<file>
```

`<sha>` is the row's last commit. `<file>` is the name the file had in `open/`.
