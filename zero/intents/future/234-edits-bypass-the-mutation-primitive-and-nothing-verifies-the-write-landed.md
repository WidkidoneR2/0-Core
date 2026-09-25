---
id: 234
date: 2026-08-27
type: arch
title: "edits bypass the mutation primitive and nothing verifies the write landed"
status: planned
tags: [architecture, rust, design]
---

## The Problem

fpatch is the mutation primitive. AGENTS.md says edits go through it. On
2026-08-27 roughly fifteen edits went through hand-rolled python instead, and
nothing noticed.

The cost was measurable that day. Two edits died three times each on anchors
carrying an em dash or a box-drawing rule -- exactly the case fpatch refuses
outright while naming the offending characters, and exactly the case
patch_between exists for. Every hand-rolled script ended with a write and a
print, so none of them verified that the file on disk held the replacement.

THAT VERIFICATION IS THE WHOLE REASON THE CLASS EXISTS. On 2026-08-06 three
patches reported "17 replaced" truthfully while the file contained none: the
shell had eaten braces out of the replacement before python ever saw it, so the
in-memory replace was correct and the result was wrong. Reading the file back
is the only check that spans the gap between what was sent and what was
written, and a hand-rolled script does not do it.

## The class boundary, defined before the work starts

This is NOT "all writes must use fpatch". Some writes are inherently different
and forcing them through a patch primitive would be worse than not having one.

INSIDE THE CLASS -- editing a file that already exists, whose current content
matters, where the edit is expressed as a CHANGE rather than a replacement:
- source files
- AGENTS.md, docs, conventions
- intent bodies

OUTSIDE THE CLASS:
- creating a file that did not exist (ship/main.rs, intent scaffolds)
- whole-file generation where prior content is irrelevant
- build artifacts
- binary installation -- ship writes binaries by atomic rename and must NOT go
  through a text-patch primitive

THE EDGE THAT PROVES THE BOUNDARY IS REAL: the INT-145 body rewrite replaced an
entire body but KEPT the frontmatter, so current content mattered. Inside the
class, despite looking like whole-file generation.

## Acceptance test

CAN A NEW MUTATION MECHANISM BE INTRODUCED WITHOUT INVENTING ANOTHER BESPOKE
RULE?

If the class boundary above answers that question for a mechanism nobody has
thought of yet, the abstraction is right. If each new mechanism needs its own
clause, it is not.

## The hard part, stated up front

ENFORCEMENT CANNOT LIVE INSIDE FPATCH. It cannot detect calls that never
happen. A guard against its own bypass is impossible from inside.

Nor is deadwood an obvious home: the fifteen bypasses were scripts written to
/tmp and executed once. They never entered the repository, so no repository
scan could have seen them. Whatever checks this has to observe the ACT, not the
tree.

That may mean the honest conclusion is that this class is unenforceable and the
right output is documentation plus discipline. Reaching that conclusion by
investigation is a valid outcome; reaching it after building a detector that
cannot work is not.

## Success Criteria
- [ ] The class boundary above survives contact with a mechanism not listed in it
- [ ] Whether the act is observable at all is ANSWERED, not assumed
- [ ] If enforceable: reports, never rewrites -- same contract as deadwood
- [ ] If not enforceable: say so, and record why, rather than building anyway

## Related

- INT-233 -- ownership discipline. Deliberately separate. That invariant is
  about WHERE KNOWLEDGE LIVES; this one is about HOW STATE CHANGES. Collapsing
  them into one detector would produce something that does neither well.
- INT-199 -- established that a safe abort and a crash must not look the same,
  and fpatch _refuse is the reference implementation.
- INT-203 -- fsh brace-expands heredoc bodies under a quoted delimiter, which is
  the transport corruption the read-back catches. fpatch.py is the wrong layer
  to fix that; the corruption happens before python receives the argument.
- INT-231 -- nine code comments already cite INT-234 for a DIFFERENT 234 that
  was never filed (commands.rs:77, parser.rs:476/478/480/482/488/493/495,
  planning.rs:1). This number is now taken by something else.
