---
id: 353
date: 2026-11-24
type: future
title: "commands::tokenize drops an empty quoted argument and eats a backslash inside double quotes"
status: in-progress
tags: [fsh, nsh, tokenizer, quoting, spine, audit]
depends_on: []
format: 3
---

## Vision
The legacy tokenizer turns a line into the same words bash
would. An empty quoted word ("" or '') is one empty argument.
Inside double quotes a backslash is removed only before $, `,
", \ or a newline, and kept everywhere else. `spine migrate`
then stops reporting the empty-argument rows as unexplained,
because legacy and the spine agree.

## The Problem
On 2026-11-24, `spine migrate` on the laptop (nsh 6.1.0)
reported these as unexpected:

    argv: word count (2 vs 3)    printf "[%s]\n" ""
                                 printf '<%s>\n' ""
    argv: word count (6 vs 7)    python3 -c "..." "[%s]\n" ""
    argv: word count (29 vs 30)  printf '%s\n' '---' 'id: 399' ...  (2 rows)

The smaller count is legacy. The audit builds the legacy plan
from `ExecContext::from_line` (exec.rs:238), which calls
`commands::tokenize` (commands/mod.rs:538). Two rules there
differ from POSIX and bash:

1. An empty quoted word is dropped. A word is only pushed when
   `current` is non-empty (commands/mod.rs:570 and :578), so
   `""` never becomes a token. bash passes one empty argument.
2. Inside double quotes, a backslash escapes EVERY next
   character (commands/mod.rs:563, then :559). POSIX says it
   escapes only $ ` " \ and newline. So "[%s]\n" becomes
   `[%s]n`. bash keeps `[%s]\n`.

Reproduced on a box build of origin/main f0e63bfe:

    NSH_SPINE=0 nsh -c 'echo "[a\nb]" "" x'   prints  [anb] x
    nsh -c 'echo "[a\nb]" "" x'               prints  [a\nb]  x
    bash -c 'echo "[a\nb]" "" x'              prints  [a\nb]  x

The default spine path is already right. An external command
on the legacy path is also right, because legacy hands the raw
line to `sh -c` (commands/mod.rs:10380), and sh parses it.
So the wrong words reach only the places that use tokenize's
output directly: builtin argv on legacy routing (NSH_SPINE=0,
or a line the spine declines), the legacy background spawn
(engine.rs:849), `command_word` and `is_repl_state_command`
(main.rs:253-266), and the audit's model of legacy.

Why these rows read as unexpected and not as a known
difference: the QuotedWordParsing rule in
spine/compare.rs:310-312 refuses any source with a backslash,
and the rows have `\n` in them. An empty-quote row without a
backslash (for example `echo "" x`) is already counted as a
safe improvement. The 29 vs 30 rows are most likely the same
cause (a '' blank-line argument in a frontmatter printf). A
scratch corpus on the box with
`printf '%s\n' '---' 'id: 399' 'title: "x y"' '---' '' 'body'`
reproduces "word count (10 vs 11)". The full laptop lines have
not been read yet; G1 reads them.

INT-216 deferred adding spans to this tokenizer because it is
retiring. This intent adds no capability. It fixes two rules
that are wrong today and that live code still reads.

## The Solution
G1 reads the five laptop rows in full and adds unit tests next
to escape_tokenize_tests (commands/mod.rs:528). Red today
(measured on a copy of the function, f0e63bfe):
- tokenize(`printf "[%s]\n" ""`) is
  ["printf", "[%s]\\n", ""]   (today: ["printf", "[%s]n"])
- tokenize(`printf '<%s>\n' ""`) is ["printf", "<%s>\\n", ""]
  (today: ["printf", "<%s>\\n"])
- tokenize(`echo '' x`) is ["echo", "", "x"]
  (today: ["echo", "x"])
Guards, green today and must stay green:
- tokenize(`a""b`) is ["ab"]  (an empty quote inside a word is
  not a separate word)
- tokenize(`echo "a\"b"`) is ["echo", "a\"b"]
- tokenize(`echo "\$HOME"`) is ["echo", "$HOME"]
- tokenize(`echo "a\\b"`) is ["echo", "a\\b"]

G2 keeps an empty quoted word. Track whether the current word
has started (a quote opened, or any character pushed). Push on
a space or at the end when the word has started, even if it is
empty.

G3 makes a backslash inside double quotes follow POSIX. It
escapes only $ ` " \ and newline. Before any other character
both the backslash and the character are kept. Expected side
effect: rows the audit files as BackslashPreservation
(spine/compare.rs:329-336) converge to Equivalent, so the Safe
improvements count falls and Equivalent rises by about the same
number. That is the audit agreeing with itself, not a loss.

G4 is the audit proof on the box, with no laptop state:
a scratch state.db (ZERO_STATE_DB) holding the rows from G1
and `echo "" x`. `spine migrate` reports each one as
Equivalent, and no `argv: word count` shape remains.

G5 is the laptop proof.

## Limits
- In scope: `commands::tokenize` in
  zero/shell/novashell/src/commands/mod.rs and its unit tests;
  one conformance case in zero/shell/nsh-test/src/main.rs
  (CONFORMANCE_CASES, line 7480) for printf with an empty
  argument; a novashell patch bump at close.
- Out of scope: the spine lexer and parser; the comparator
  rules in spine/compare.rs; detect_redirect; the `;` splitter
  in main.rs (find -exec \; and brace groups, a separate
  intent); a backslash OUTSIDE quotes (legacy does not remove
  it, bash does; separate intent); spans for tokenize
  (INT-216); deleting legacy routing.
- How deep: five gates, about an evening. One production
  function changes. No new crate, no new dependency.
- Stop and ask: an existing unit test, nsh-test case or devbox
  case relied on the old behavior (for example a line starting
  with `""` now has an empty command word, which matches bash
  and the spine but changes legacy alias lookup); the five
  laptop rows turn out to have a cause other than an empty
  quoted word or a backslash; a fix seems to need a change
  outside In scope.
- Done means: G1 to G5 ticked with evidence.
- Must not change: live state.db, the spine path, the
  comparator's policy table, any file outside the repo, and
  any secret or confidential value.

## Success Criteria
- [x] G1 red first: the five laptop rows are read in full
      (state.db shell_history), and the G1 tokenize tests are
      in the tree and red on the code as it stands.
      ask: tokenize drops an empty quoted word and eats a backslash before n
      ruling:
      evidence: laptop state.db read in full, one stored row matched (id 254243, the INT-320 rehearsal printf); d9d5e8c4 three tokenize tests red on the old code
- [x] G2 an empty quoted word is one empty argument. The
      empty-word tests are green; `a""b` is still one word.
      ask: "" and '' each produce one empty token
      ruling:
      evidence: 0c92a776 empty-word tests green, a""b still one word; laptop novashell 298/298
- [x] G3 a backslash inside double quotes escapes only
      $ ` " \ and newline. The backslash tests are green,
      including the existing escaped-quote test.
      ask: "[%s]\n" stays [%s]\n and "a\"b" stays a"b
      ruling:
      evidence: ebf3c58a tokenize 17/17 incl. the escaped-quote test; results matched bash case by case on the Cursor VM
- [x] G4 a scratch state.db holding the G1 rows and
      `echo "" x` gives Equivalent for every row in
      `spine migrate`, and no `argv: word count` shape.
      ask: the audit no longer reports the empty-argument rows
      ruling:
      evidence: Cursor VM scratch ZERO_STATE_DB, 6 rows incl. both printf shapes and echo "" x: int-353 6/6 Equivalent, main f0e63bfe 2 argv word count rows
- [x] G5 laptop: `spine migrate` from the working tree build
      shows no `argv: word count` rows; `NSH_SPINE=0 nsh -c
      'echo "[a\nb]" "" x'` prints the same as bash; nsh-test
      (with the new conformance case), doors and d are green.
      ask: the laptop agrees with bash, and doors and d are green
      ruling:
      evidence: laptop on 5877d86c: spine migrate 98.7 percent, no argv word count shape; NSH_SPINE=0 echo equals bash byte for byte; a8e1272f doors 4/4 with nsh-test 306/306; d 96 percent after merge, only warning the uncommitted 353 file
