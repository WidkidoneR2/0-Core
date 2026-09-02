---
id: 238
date: 2026-09-01
type: fix
title: "process substitution is claimed by the spine as a stdin redirect so diff angle-paren echo a runs against a file named open-paren-echo"
status: planned
tags: [fix, bugfix]
---

## Vision

A construct nsh does not model is refused, not mis-modelled.

## The Problem

MEASURED 2026-09-02 on the deployed shell:

    diff <(echo a) <(echo b)     nsh:  cannot read (echo: No such file or directory
                                 bash: 1c1 / < a / --- / > b
    sh -c same string            produces the diff -- sh here DOES support it

The lexer operator table ends with a wildcard:

    (open-angle, _) => (OperatorKind::RedirectIn, 1)

So open-angle followed by open-paren lexes as a one-character stdin redirect and
the paren starts the next word. The parser then sees a valid command with a
stdin redirect, parses cleanly, and the SPINE CLAIMS THE LINE -- confirmed with
NSH_OBSERVE=router, which reports claimed then spine ran.

⭐ THE PRINCIPLE IS ALREADY WRITTEN IN THIS FILE, on OperatorKind::Background:
an incomplete refusal is a mis-execution. Omitting an operator is not neutral --
without the Background arm, sleep 5 ampersand would lex as three words, parse
cleanly, and run sleep with an ampersand as a literal argument. That is exactly
what open-angle-paren does today.

## The Solution

A ProcessSub operator kind, two-character match, refused at parse. Refused means
legacy takes the line and hands it to sh, which handles it. The spine does not
need to model process substitution; it needs to SAY it does not.

⚠️ AND THE PLACEMENT IS THE WHOLE DIFFICULTY. Attempted 2026-09-02 and reverted:

  FIRST ATTEMPT: added ProcessSub to the connector break list in parse_command
  beside Background. Wrong -- breaking ENDS the command, so the line ran with no
  arguments at all and difft printed its help. The connectors break because they
  are VALID there and bind looser; process substitution is not valid there in
  any sense, which is a different fact needing a different exit.

  SECOND ATTEMPT: returned Err(UnsupportedOperator) from inside parse_command
  loop. The diff WORKED -- correct output through legacy -- but every such line
  emitted:

      failed to open command_execution record: UNIQUE constraint failed:
      command_execution.session_id, command_execution.execution_id

  A DUPLICATE LIFECYCLE RECORD. Neither existing refusal does this: Background
  and ComparisonNotRedirect were both tested and are silent. The difference is
  WHERE they refuse -- both raise in parse_line, after a node exists, while this
  raised mid-loop after the execution record had already opened. Legacy then
  opened it again.

So the next attempt starts with the lifecycle question rather than the parser:
when does command_execution open relative to routing, and what does a refusal
after that point owe it. That is INT-191 territory and it is the actual work.

## Success Criteria

- [ ] Watch it fail first: diff angle-paren echo a angle-paren echo b reports
      cannot read (echo on the current binary, recorded before any change
- [ ] The spine REFUSES rather than claims -- NSH_OBSERVE=router shows the line
      going to legacy
- [ ] The command produces the same output bash does
- [ ] NO duplicate command_execution record -- the second attempt did, and a
      warning on every such line is a worse trade than the bug it fixes
- [ ] An ordinary stdin redirect is untouched: cat angle-bracket file still works
- [ ] nsh-test carries a case, and the two rows move from unexpected to
      Refused at parse in spine migrate

## Notes

- Found by splitting the migrate audit io bucket into shapes (9f572f98). Of 59
  unexpected differences, 43 were history debris -- trigger definitions and a
  pasted prompt line -- and 4 more were artefacts of comparing unexpanded input.
  These two were the only real language question in the set.
- The compiler enforces the decision: parser.rs has NO catch-all on operator
  kinds, deliberately, so adding a variant makes the match non-exhaustive. That
  guard fired correctly on the first attempt and named the one site.
- sh on this machine supports process substitution, so legacy delegation is a
  real fix rather than a deferral to something equally broken.
