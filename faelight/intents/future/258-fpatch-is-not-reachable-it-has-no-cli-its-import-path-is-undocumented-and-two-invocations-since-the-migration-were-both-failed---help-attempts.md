---
id: 258
date: 2026-09-23
type: future
title: "fpatch is not reachable: it has no CLI, its import path is undocumented, and two invocations since the migration were both failed --help attempts"
status: planned
tags: [fpatch, tooling, dx, claude]
---

## Vision
The guarded patcher is used INSTEAD OF hand-rolled python, because it is already better than
what keeps replacing it.

## The Problem
`faelight/scripts/dev/fpatch.py` is 359 lines and it is CORRECT. Measured 2026-09-23:

```text
    patch(path, old, new, count=1)     exactly one match by default, or state the count
    patch_between(path, start, end, new_lines)
    _refuse   exit 1   RED      "nothing was written" -- a safe abort (INT-199)
    _internal exit 2   YELLOW   the file state is unknown -- fix fpatch, not your anchor (INT-215)
    the diff  removed AND added lines, red and green, since INT-215 G2
    a miss    shows the anchor with WHITESPACE VISIBLE and the nearest lines in the file
    FPATCH_COLOR  per-stream isatty, override both directions
```

⚠️ NONE OF THAT IS REACHABLE. It is an import from a relative path, `fpatch --help` is "command
not found", and the only TWO invocations since 2026-08-26 were both that failure, on 2026-09-23.

★ SO EVERY ASSISTANT EDIT IN THIS PROJECT RE-IMPLEMENTS IT, AND WORSE. A hand-rolled python block
asserts one match and aborts -- but prints no whitespace-visible anchor, no nearest lines, and
makes no distinction between "your anchor is wrong" and "the tool is wrong". fpatch learned both
of those from real failures; the replacement learns them again each time.

⭐ AND ITS DOCSTRING ALREADY THINKS ABOUT THIS EXACT READER: "output lands in three places -- a
terminal, a paste buffer, and an assistant context window. Colour helps the first and clutters the
other two." The tool was built for this use and is not used.

## The Solution
Make it callable WITHOUT READING ITS SOURCE. Either a CLI entry point, or the import form written
into AGENTS.md beside the generated-text-as-data rule, so the same lines are written every time.
`FPATCH_COLOR=0` belongs in that documented form for the same reason the docstring gives.

⚠️ NOT A REWRITE. INT-199 established what it SAYS and INT-215 how it is SEEN, and both were earned
from real failures -- six safe aborts read as a broken tool, two more read as crashes. This intent
changes HOW IT IS INVOKED, nothing else.

★ AND ONE RULE FROM INT-215 BELONGS WITH IT, because it is the rule that makes the tool safe:
AN ANCHOR MUST BE A COMPLETE LINE, NEVER A PREFIX OF ONE. Five gates were mangled when a helper
anchored on "- [ ] G1:" instead of the whole line; fpatch did exactly what it was asked.

## Success Criteria
- [ ] WATCH IT FAIL FIRST: run fpatch the way a new reader would, and record what happens.
      "command not found" is the finding this intent is filed on
- [ ] fpatch is invoked from a DOCUMENTED form that does not require reading fpatch.py
- [ ] that form lives in AGENTS.md beside the generated-text-as-data rule, and names FPATCH_COLOR=0
- [ ] a refusal still exits 1 and an internal error still exits 2, PROVEN after the change, not
      assumed -- INT-215 G5 is the precedent
- [ ] the count>1 display limitation is either recorded at the call site or fixed, and this gate
      says WHICH. Pre-existing: `first` comes from s.index(old), so with count>1 the other sites
      are replaced but not shown
- [ ] an edit in a real session uses it, and the session records whether it was easier than the
      hand-rolled block. If it was not, say so here rather than declaring victory
