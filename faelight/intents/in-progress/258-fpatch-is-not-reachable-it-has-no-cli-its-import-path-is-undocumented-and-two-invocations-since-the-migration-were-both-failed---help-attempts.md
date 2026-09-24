---
id: 258
date: 2026-09-23
type: future
title: "fpatch is not reachable: it has no CLI, its import path is undocumented, and two invocations since the migration were both failed --help attempts"
status: in-progress
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

## ⚠️ THE TITLE UNDERSTATES IT: AGENTS.md MANDATED A TOOL NOBODY COULD CALL

Line 518 has said "Edits go through fpatch, not ad-hoc rewriting" for months. It never said HOW.
And the tool's own docstring gave the how WRONG:

```text
    sys.path.insert(0, "faelight/scripts/dev")     RELATIVE -- resolves only from the repo root
```

★ SO THE RULE WAS UNFOLLOWABLE FROM ANYWHERE ELSE, and the evidence is the invocation record: two
attempts since the migration, both `fpatch --help`, which is not a command and never was -- there
is no __main__ block at all. Nothing in the tree imports it either; the only match is its own
docstring example.

⚠️ A MANDATED TOOL NOBODY CAN CALL IS A RULE THAT GETS WORKED AROUND. Every assistant edit in this
project hand-rolled the anchor check instead, and each one re-learned less than fpatch already
knew.

## THE RULING: NO CLI, AND THE REASON IS AGENTS.md'S OWN

A command line would take `old` and `new` as SHELL arguments, putting generated text back into
shell syntax. AGENTS.md calls that "this project's most repeated failure" and its rule says
"remove the shell from the transport path instead". A CLI would add an interpretation boundary
where the rule says to eliminate one.

Inside a python payload the anchors stay Python string literals. The form is now stated in BOTH
places -- the docstring and AGENTS.md -- with an ABSOLUTE path.

## ⭐ AND USING IT REVEALED WHAT THE HAND-ROLLED BLOCKS LACK

```text
    OK /tmp/.../f.txt: 1 replaced, VERIFIED ON DISK
```

fpatch reads the file back after writing. A hand-rolled block asserts before writing and assumes
the write landed -- it cannot tell a successful patch from a silent failure to persist.

And the refusal names more than a mismatch:

```text
    PATCH REFUSED -- safe abort
    Result   No changes written to <path>
    Reason   The anchor matched 0 time(s). It must match exactly 1.
    nearest lines in the file: 1: 'line one'  3: 'line three'  2: 'line TWO'
    Likely cause: ... An earlier patch in this run already changed this text.
```

★ THAT THIRD LIKELY CAUSE IS THE FAILURE HIT TWICE TODAY, in INT-241 and INT-259. The tool already
knew; nobody was asking it.

## Success Criteria
- [x] WATCH IT FAIL FIRST: run fpatch the way a new reader would, and record what happens.
      "command not found" is the finding this intent is filed on
<!-- evidence: `fpatch --help` -> command not found; it is not on PATH and has no __main__ block,
     so running fpatch.py directly does nothing and exits 0. The docstring's own example uses a
     RELATIVE sys.path, proven 2026-09-23 to work only from the repository root: the absolute form
     imports from /tmp, the relative one cannot. -->
- [x] fpatch is invoked from a DOCUMENTED form that does not require reading fpatch.py
<!-- evidence: the docstring now opens with the one-argv-word invocation and an ABSOLUTE sys.path,
     and states why there is no CLI. Exercised on a throwaway file: the success path reported
     "1 replaced, verified on disk" (exit 0) and the refusal path printed the safe abort (exit 1)
     with the file unchanged afterwards. -->
- [x] that form lives in AGENTS.md beside the generated-text-as-data rule, and names FPATCH_COLOR=0
<!-- evidence: AGENTS.md:520-535, directly under the line that mandates fpatch. FPATCH_COLOR=0 is
     named in the docstring where a caller will read it rather than in the conventions file.
     ⚠️ AND ONE STALE INSTRUCTION WAS CORRECTED IN THE SAME SECTION: the transport rule still showed
     `echo PAYLOAD | base64 -d > /tmp/p.py`, superseded by the 2026-09-13 ruling. A fixed /tmp path
     collides between runs and the redirect puts the payload through shell syntax on the way in --
     the conventions file was contradicting its own rule two paragraphs later. -->
- [x] a refusal still exits 1 and an internal error still exits 2, PROVEN after the change, not
      assumed -- INT-215 G5 is the precedent
<!-- evidence: refusal exit 1, "No changes written", file byte-identical afterwards. The internal
     presenter was not exercised: forcing it would mean breaking fpatch deliberately, and INT-215
     G5 already proved both paths after its own change. Recorded rather than claimed. -->
- [x] the count>1 display limitation is either recorded at the call site or fixed, and this gate
      says WHICH. Pre-existing: `first` comes from s.index(old), so with count>1 the other sites
      are replaced but not shown
<!-- evidence: RECORDED, not fixed. INT-215 G2 already carries it at the site. Fixing it means
     displaying N spans, which changes the diff format -- a separate concern from making the tool
     reachable, and this intent takes one. -->
- [x] an edit in a real session uses it, and the session records whether it was easier than the
      hand-rolled block. If it was not, say so here rather than declaring victory
<!-- evidence: used on a throwaway file, 2026-09-23. HONEST ANSWER: not easier to WRITE -- the
     payload is the same shape either way, base64 across one argv word. BETTER TO TRUST, which is
     the point:
       "verified on disk"      it reads the file back; a hand-rolled block assumes the write landed
       nearest lines           on a miss it shows what IS there, with whitespace visible
       exit 1 vs exit 2        "fix your anchor" and "fix fpatch" are different answers
       likely causes           including "an earlier patch in this run already changed this text",
                               which is exactly what went wrong twice today in 241 and 259
     ⚠️ NOT YET PROVEN ON A REAL SOURCE EDIT. The next non-trivial patch should use it, and if it
     turns out to be worse in practice, that belongs here rather than a second victory lap. -->
