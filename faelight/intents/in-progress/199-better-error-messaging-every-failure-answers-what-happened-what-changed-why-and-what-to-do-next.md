---
id: 199
date: 2026-07-29
type: future
title: "better error messaging: every failure answers what happened, what changed, why, and what to do next"
status: in-progress
tags: [fsh, errors, diagnostics, ux, tooling, conventions]
---

## Vision
Christian's design. Every Faelight failure answers the same four questions in the same order,
before any internal detail: WHAT HAPPENED, DID ANYTHING CHANGE, WHY, and WHAT DO I DO NEXT. A
traceback answers a fifth question -- what went wrong inside the program -- which matters to
whoever is fixing the tool and almost never to whoever is using it. Today the fifth answer is
the only one printed.

## The Problem -- MEASURED, not imagined
2026-07-29, during the INT-169 spine work, `fpatch` aborted six times. Every abort printed:

    Traceback (most recent call last):
      File "<stdin>", line 4, in <module>
      File "faelight/scripts/dev/fpatch.py", line 54, in patch
        assert n == count, f"{path}: expected {count} match(es), found {n}"
    AssertionError: ...: expected 1 match(es), found 0

The tool behaved CORRECTLY every time -- it refused a patch whose anchor no longer matched, and
wrote nothing. But the single fact that mattered, NO FILES WERE MODIFIED, appeared nowhere. It
had to be inferred from knowing how the tool works. Twice that session the output was read as
"the patch broke something" when it meant "the patch safely refused", and the wrong recovery
was attempted as a result.

★ THAT IS THE WHOLE INTENT IN ONE OBSERVATION: a safe abort and a crash produced identical-looking
output, so the reader could not tell a working guard from a broken tool.

## The design (Christian's, preserved in shape)
Five sections, same order, every tool:

    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    x  PATCH FAILED
    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    Status
      Safe abort.

    Result
      No files were modified.

    Reason
      Expected exactly one OpenSSH configuration block.
      Found 0 matches.

    Possible causes
      - The file has already been modified.
      - The patch was created against an older version.
      - The search pattern is no longer valid.

    Recovery
      - Verify you are patching the intended file.
      - Compare the current file against the patch.
      - Regenerate the patch if necessary.

    Debug
      Error: PATCH-0003

    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Four principles
1. ASSERTIONS ARE FOR BUGS, NOT FOR EXPECTED FAILURES. An assertion means the program reached a
   state that should never happen. A missing search pattern means the requested operation cannot
   be completed safely. Those are different events and must not share a presentation. Raise a
   descriptive error and let the command render it; keep the non-zero exit.
2. TELL THE USER WHAT DID NOT HAPPEN. The absence of side effects is often the most reassuring
   fact available, and it is the one currently hardest to extract.
3. DIAGNOSTICS ARE OPT-IN. Structured output by default; the traceback and the error code behind
   a debug mode. Normal use stays approachable without losing anything a maintainer needs.
4. RECOVERY IS PART OF THE INTERFACE. An error should begin the debugging workflow, not end it --
   numbered, runnable next steps, so external documentation is rarely needed.

## Severity taxonomy -- the distinction that carries the design
    Info          operation completed
    Warning       completed, but something deserves attention
    Safe Abort    the tool intentionally stopped to avoid an unsafe change
    Internal      the tool itself hit a defect

★ "There was not enough information to apply this patch safely" and "the patch tool crashed" must
never look the same. Today they do.

## Scope guardrails
- ⚠️ NOT a rewrite of every tool at once. Define the standard first; adopt it where failures are
  actually being read.
- ⚠️ NOT prettier errors. The objective is failures that are immediately understandable, safe, and
  actionable. Formatting is the means.
- ⚠️ ERROR CODES ARE NOT THE GOAL AND MAY NEVER BE NEEDED. Christian, 2026-07-29: the
  message should carry the diagnostic, so nobody has to look a code up. A code needs a catalogue
  behind it -- a second artifact to maintain and a second one to go stale. fpatch proved the point
  the same day: showing the anchor in repr plus the nearest lines made every failure
  self-explaining with no identifier at all.
- THE PURPOSE, in Christian's words: to simplify the terminal he lives in, and to build for
  beyond tomorrow. Not prettier errors -- fewer moments spent reconstructing what a tool meant.
- ★ START WITH `fpatch`, because that is where the problem was measured and where the next
  occurrence is certain.

## Success Criteria
- [x] The five-section structure is written down as a convention (docs/CONVENTIONS.md) before any
<!-- evidence: commit e5db4d33, 2026-08-07. docs/CONVENTIONS.md 65 -> 136 lines, section
     'Failure output (INT-199)' in the same shape as the INT-158 section: rule, example, three
     limits, why it exists, the tell, exemplars. NOTE the ordering this gate asked for was NOT
     honoured -- fpatch adopted the structure first and the convention is back-derived from it.
     The section says so in its own text rather than implying otherwise. -->
      tool adopts it
- [ ] The severity taxonomy exists in code, not only in prose -- a safe abort and an internal
      error are DIFFERENT TYPES, so a tool cannot accidentally present one as the other
- [x] `fpatch` raises a descriptive error instead of asserting, and its failure output states
<!-- evidence: faelight/scripts/dev/fpatch.py, _refuse() at line 32. Prints 'Result / No changes
     written to {path}' as the FIRST section, then Reason, What was compared, Likely cause,
     Recovery. No assertion reaches the user; the non-zero exit is kept via sys.exit(1). -->
      that nothing was written
- [x] A traceback appears only in debug mode, and the debug path still produces everything a
<!-- evidence: fpatch.py line 25, DEBUG = 'FPATCH_DEBUG' in os.environ. Without it _refuse ends
     in sys.exit(1) and no traceback is printed; with it set, _refuse raises PatchRefused so a
     maintainer gets the full Python traceback plus the structured message already printed. -->
      maintainer needs
- [x] At least one failure is demonstrated end to end: the message alone was enough to choose the
<!-- evidence: demonstrated 2026-08-07 during this intent's own work. patch_between refused with
     'The start marker matched 3 lines. It must match exactly 1.', listed all three with their
     line numbers and contents, stated no changes were written, and recommended lengthening the
     marker. The fix was one edit, chosen from the message alone, with no source reading. -->
      correct recovery, with no source reading
- [ ] Each gate carries evidence per INT-158

## Relationship
- Origin: Christian's design, 2026-07-29, written the same day the problem was measured six times
  in one session.
- INT-192 is the sibling: forest tools cannot express an UNDETERMINED outcome, so failed checks
  report clean. Same family -- a tool that cannot say what actually happened -- from the opposite
  direction: 192 is about silence, 199 is about noise.
- INT-167 (DevBox) owns the structured-event substrate. If error codes become queryable rather
  than printed, they belong on that spine, not in a second store.
- SEQUENCING (Christian): AFTER the remaining fsh intents. The shell comes first; this improves
  how the shell reports failure, which is worth more once the shell is stable.

## The Rule
"A safe abort and a crash printed the same thing, so a working guard could not be told from a
broken tool. An error message is the start of the debugging process, not the end of it." 🌲
