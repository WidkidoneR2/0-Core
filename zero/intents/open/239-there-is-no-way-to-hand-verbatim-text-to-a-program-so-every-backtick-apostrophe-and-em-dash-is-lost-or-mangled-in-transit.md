---
id: 239
date: 2026-09-01
type: fix
title: "there is no way to hand verbatim text to a program so every backtick apostrophe and em dash is lost or mangled in transit"
status: planned
tags: [fix, bugfix]
---

## Vision

Text reaches a program as it was written.

## The Problem

There is no way to hand verbatim text to a program through nsh. Every route
interprets something:

    heredoc with a quoted delimiter   breaks on an apostrophe in the body
    -c with a double-quoted string    backticks become command substitution
    printf                            needs escaping, which is the same problem
    python3 -c with nested quotes     three layers of quoting, one shell

MEASURED, SEVEN TIMES IN ONE WEEK, and the loss has three distinct shapes:

LOUD -- the command fails or hangs. A heredoc whose body contains an apostrophe
leaves the shell waiting for a delimiter that will never arrive. Costs a
Ctrl+C and a retry.

MANGLED -- the shell runs part of the text. Backticks in a comment became
command substitution and printed sh: io:: command not found before the file
was even written.

⚠️ SILENT -- and this is the one that matters. The text is written with words
MISSING and the result still parses. Three Rust comments lost their backticked
words and compiled cleanly. A pointer in AGENTS.md shipped as
"Read  before choosing a level" with no filename, and was committed, pushed,
and only noticed on a re-read. Nothing failed. Nothing warned.

⭐ THE THREE FAILURE MODES ARE NOT ONE PROBLEM.

1. TRANSIT LOSS -- the shell consuming characters before the program sees them.
   That is what this intent is about, and it is the one nsh can fix.
2. ASSERTION MESSAGING -- when a line-index edit misses, the error says what
   WAS there and nothing about where the right line is, costing a round trip
   to print the neighbourhood. A convention, not a shell feature.
3. DISCIPLINE -- constructing patterns from characters that do not survive.
   Also a convention.

Only the first is nsh's to solve.

## The Solution

A verb that takes text VERBATIM and delivers it somewhere, with no interpretation
between the two. Shape to be decided, but the requirement is exact:

    text in, bytes out, nothing consumed

The obvious candidates, none yet chosen:

- a builtin taking a terminator that cannot appear in the body, reading raw
  until it appears, writing to a file or a command's stdin
- an editor-style capture, where nsh opens a buffer and hands the result on
- something that reads from a file already on disk and never passes it through
  a shell string at all

⚠️ THE LAST ONE IS WHAT ALREADY WORKS, and it is worth saying: writing the
content to a file with an editor, then having a program read that file, has
never lost a character. Every failure this week came from trying to construct
text INSIDE a shell command. So the feature may be ergonomic rather than
missing capability -- which changes what it is worth.

## Success Criteria

- [ ] Watch it fail first: a body containing a backtick, an apostrophe and an
      em dash, delivered by each existing route, with what each one loses
      recorded
- [ ] One route exists where all three survive, demonstrated on that same body
- [ ] The route is usable for the case that keeps failing: multi-line text
      going to a file or to a program's stdin
- [ ] nsh-test carries a case with all three characters in it, because a
      transit fix that is not tested will regress the moment quoting changes

## Notes

- Seven instances this week, on 2026-09-01 and 09-02. Four cost a retry, two
  shipped mangled text that had to be corrected afterwards, and one was
  committed and pushed before anyone noticed.
- INT-236 is the sibling and it is about the opposite direction: quoting that
  nsh HONOURS on one line and stops honouring on two. Both are the same subject
  seen from different sides -- what the shell does to text it was handed.
- ⚠️ Do not solve this by adding an escaping helper. This tree spent INT-193,
  195, 196 and 209 reducing the number of places that own quoting, and a
  quote-the-string-for-me function would be a new one. The fix is a route that
  does not interpret, not a better interpreter.
