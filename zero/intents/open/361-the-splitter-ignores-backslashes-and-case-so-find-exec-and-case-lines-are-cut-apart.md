---
id: 361
date: 2026-11-27
type: future
title: "the splitter ignores backslashes and case, so find -exec and case lines are cut apart"
status: planned
tags: [nsh, splitter, parser]
depends_on: []
format: 3
---

## Vision
nsh cuts a line at `;`, `&&` and `||` before it runs
anything. After this, that cut treats a backslash the
way bash does: outside single quotes, the next character
is taken literally. So `find ... -exec cmd {} \;`,
`find . \( ... \)`, `echo a\;b` and `echo "a\"b; c"`
reach the command whole. A `case ... esac` line is kept
whole the way for, while, until and if already are, so
its `;;` is not cut and sh runs it. Nothing else about
how lines are split or run changes.

## The Problem
Measured 2026-11-27 on a box checkout of origin/main
50f22621 (novashell 6.1.3, nsh-test 2.2.3), nsh built
with cargo +1.88.0. Each line was run as `nsh -c` and
as `bash -c` in a scratch directory under /tmp, with a
scratch HOME and a scratch ZERO_STATE_DB. Laptop state
was not touched. `cargo +1.88.0 test -p novashell`: 308
passed.

How a line is cut today:
1. run_input (main.rs:1725) calls split_into_segments
   (main.rs:1733; the function is at main.rs:295).
2. That calls split_semicolons_marked (main.rs:340). It
   has a plain loop that cuts at `;` (main.rs:384-404,
   the `;` arm at 395). When line_has_compound
   (main.rs:459) sees for, while, until or if at a
   command position, or any `(`, it uses
   split_around_compounds (main.rs:510) instead.
3. Each piece then goes through split_logical
   (expand.rs:565), which cuts at `&&` and `||`.
All four scanners track ' and " quotes. None of them
knows the backslash. The word list in word_at
(main.rs:423) and the opener matches (main.rs:490 and
569) know for, while, until and if, but not case.

Measured, bash first, then nsh:
1. `find . -maxdepth 1 -name f1.txt -exec echo found {} \;`
   bash prints `found ./f1.txt`. nsh prints
   ``find: missing argument to `-exec'`` and exits 1.
   The `;` arm (main.rs:395) cut the line after the
   backslash, so find never got its `;`. The same with
   `-print` after it, and after `echo x &&`.
2. `find . -maxdepth 1 \( -name f1.txt -o -name zz \) -print`
   bash prints `./f1.txt`. nsh runs three pieces:
   `find . -maxdepth 1 \`, `( -name f1.txt -o -name zz \)`
   and `-print`. line_has_compound (main.rs:485) took
   the escaped `(` for a group.
3. `echo a\;b` bash prints `a;b`. nsh prints `a\` and
   then `command not found: b`.
4. `echo "a\"b; echo c"` bash prints `a"b; echo c`.
   nsh prints `a"b` and then runs `echo c` as a second
   command. The escaped quote closed the quote
   (main.rs:391) and the `;` inside the string cut the
   line (main.rs:395). The same with `&&` inside the
   quotes: expand.rs:580 closes the quote and 584 cuts.
   This runs text the user typed inside quotes as a
   command.
5. `case x in a) echo A;; x) echo X;; esac` bash prints
   `X`. nsh cuts it at each `;;` (main.rs:395), sh gets
   `case x in a) echo A` and says `Syntax error: end of
   file unexpected (expecting ";;")`, then
   `command not found: x)` and `esac`. Also wrong after
   `echo pre;`, before `&& echo after`, and with a
   `(x)` pattern.

Already the same as bash, so not part of this: `-exec
... ';'` and `-exec ... {} +`; `(echo a) && echo b`;
for, if, piped while and plain ( ) groups; `echo 'a\';
echo b` (a backslash inside single quotes is literal in
bash too); `env V=val sh -c 'echo ${V:-dflt}'`.

Different from bash, but not in this intent (see
Limits):
- Brace groups. `{ echo a; echo b; }` is cut at
  main.rs:395 and gives `command not found: {` and `}`.
  The same with `> file`, `| wc -l` and after `&&`.
- A ( ) group followed by a redirect, a pipe or `&`.
  `(echo hi) > g.txt` writes nothing and says
  `empty plan: nothing to execute`;
  `(echo a; echo b) | wc -l` prints a and b, then
  `expected a command`; `(echo hi) &` the same.
  split_around_compounds closes the group as its own
  piece (main.rs:618-625), so the redirect, pipe or `&`
  is left in a piece with no command.
- `time (echo t1; echo t2)`. The output lines match,
  but nothing is timed: nsh prints
  `time: missing command`, because main.rs:560-561
  pushes `time` as its own piece before the group.
- `${VAR:-default}` and the other `${...}` forms (`:=`,
  `:+`, `-`, `#`, `%`). nsh prints the text as written,
  for example `echo ${HOME:-dflt}` prints
  `${HOME:-dflt}`. This is not the splitter. The spine
  keeps any `${...}` that is not a plain name as literal
  text on purpose (spine/parser.rs:694 and 713-716; the
  reason is in the comment just above 713). With
  NSH_SPINE=0, legacy expand_vars (main.rs:917-932)
  reads the whole body as a name and prints nothing.
  Plain `${HOME}` works.

## The Solution
G1 adds five unit tests in a new `mod int361_split` at
the end of main.rs, after `mod int328_split`
(main.rs:4485). All are named int361_ so one filter runs
them. They call split_into_segments only. Four are red
today and one is a green guard:
- `int361_escaped_semicolon_is_not_a_split` (red).
  `find . -name x -exec echo found {} \;` and
  `echo a\;b` are one piece each;
  `echo x && find . -name x -exec echo {} \;` is two.
- `int361_escaped_quote_does_not_end_the_quote` (red).
  `echo "a\"b; c"` and `echo "a\"b && c"` are one piece.
- `int361_escaped_paren_is_not_a_group` (red).
  `find . \( -name a -o -name b \) -print` is one piece.
- `int361_case_is_one_segment` (red).
  `case x in a) echo A;; x) echo X;; esac` is one piece;
  `echo pre; case x in x) echo X;; esac && echo after`
  is three: `echo pre`, the case, `echo after`.
- `int361_plain_splits_are_unchanged` (green).
  `echo a; echo b`, `echo 'a\'; echo b`,
  `echo "a\\"; echo b`, `echo case; echo esac` and
  `echo a && echo b` are each two pieces, as today.

G2 is the fix, in those four scanners only:
- A backslash outside single quotes keeps itself and the
  next character in the piece and skips past both. This
  goes in the plain loop of split_semicolons_marked, in
  line_has_compound, in split_around_compounds, and in
  split_logical. Inside single quotes nothing changes.
  The backslash stays in the text; the next stage
  removes it, as it already does for a line with no
  `;` (measured: `echo a\;b` prints `a;b` after the fix).
- case and esac join word_at's list, case joins the two
  opener matches, RegionCloser gets an Esac variant, and
  esac pops it: the same shape as if and fi. The case
  piece reaches try_shell_construct (engine.rs:1757),
  which already sends a line whose first word is case to
  sh whole (engine.rs:1764).
A box prototype of G1 and G2 adds 114 lines and removes
7 in main.rs and expand.rs; 69 of the added lines are
the tests. Before the fix `cargo +1.88.0 test -p novashell
int361_` gave 1 passed and 4 failed on assertions;
after it, 5 passed, and the full suite went from 308 to
313 passed.

G3 is the box proof, with no repo commit. The same 41
probe lines are run as `nsh -c` and `bash -c` on a main
build and on an int-361 build. Measured with the
prototype: the 13 lines in the five groups above differ
from bash on main and match on the branch; the 12 lines
that already matched still match; the 16 follow-up lines
(brace groups, group plus redirect, pipe or `&`,
`time ( )`, `${...}`) differ on both, unchanged.

G4 adds four CONFORMANCE_CASES to
zero/shell/nsh-test/src/main.rs, after the INT-357
cases, compared against bash in /tmp:
`echo esc-a\;b`, `echo "q\"x; echo leak"`,
`case k in k) echo case-k;; esac`, and
`echo f > /tmp/nsh_conform_f.txt; find
/tmp/nsh_conform_f.txt -exec echo found {} \;` (one
line in the file; written with doubled backslashes in
the Rust literal, as the printf cases are). No output
line starts with `[`, `x ` or the cross mark. Unlike
INT-357's cases these are red before G2: measured with
`nsh -c` on the box, main gives `esc-a\` and
`command not found: b`, `q"x` then `leak`, the sh
syntax error, and find's missing argument; the
prototype gives the same output as bash for all four.
The full nsh-test suite was not run on the box (it
hangs on the VM); the laptop runs it through doors.

G5 is the laptop proof.

## Limits
- In scope: split_semicolons_marked, word_at,
  RegionCloser, line_has_compound and
  split_around_compounds in
  zero/shell/novashell/src/main.rs; split_logical in
  zero/shell/novashell/src/expand.rs; five int361_
  tests in main.rs; four CONFORMANCE_CASES in
  zero/shell/nsh-test/src/main.rs; a novashell patch
  bump and an nsh-test patch bump at close.
- Out of scope: brace groups; a ( ) group followed by a
  redirect, a pipe or `&`; `time ( )`; `${VAR:-default}`
  and the other `${...}` forms (spine variable
  expansion); legacy NSH_SPINE=0 output (there,
  `echo a\;b` prints `a\;b`, because the legacy echo
  keeps the backslash); a backslash before `$(` or
  inside it; contains_outside_quotes (main.rs:642, used
  only for `|||`); try_shell_construct, try_grouping
  and the spine; merging the splitters into one
  (foundation item 8).
- How deep: five gates, about an evening. Four
  scanners each gain one escape check, and case gets
  the same handling as if. No new crate, no new
  dependency, no new function outside the tests.
- Stop and ask: an existing unit test, golden case or
  conformance case changes result; a line that matched
  bash on main stops matching; the fix seems to need a
  change outside In scope; `spine migrate` on the
  laptop changes in a way the note below does not
  explain.
- Note: the migrate audit splits history rows with the
  same split_into_segments (commands/mod.rs:1057), so
  `spine migrate` may count a few rows differently
  (for example a case row now arrives whole and is
  refused as a compound command, per INT-357). Record
  the numbers as evidence; they are not a gate.
- Done means: G1 to G5 ticked with evidence.
- Must not change: live state.db, engine.rs, exec.rs,
  the spine, lexer.rs, what an unescaped quote does,
  any file outside the repo, and any secret or
  confidential value.

## Success Criteria
- [ ] G1 red first: the five int361_ tests are in a new
      mod int361_split in main.rs. The four escape and
      case tests fail on their assertions (none errors)
      and int361_plain_splits_are_unchanged passes. The
      308 existing tests stay green.
      ask: today the splitter cuts at an escaped character and inside case, and nothing tests it
      ruling:
      evidence:
- [ ] G2 the backslash check in the four scanners and
      case ... esac handled like if ... fi. All five
      int361_ tests pass and cargo test -p novashell is
      313 passed.
      ask: an escaped character and a case line stay whole, and plain splits are unchanged
      ruling:
      evidence:
- [ ] G3 box proof: of the 41 probe lines, the 13 that
      differ from bash on main match bash on int-361,
      the 12 that matched still match, and the 16
      follow-up lines are unchanged.
      ask: the fixed lines now match bash and nothing that matched before moved
      ruling:
      evidence:
- [ ] G4 four CONFORMANCE_CASES for an escaped `;`, an
      escaped quote, a case line and find -exec with
      `\;` are in nsh-test.
      ask: the laptop suite holds these four lines to bash from now on
      ruling:
      evidence:
- [ ] G5 laptop: cargo test -p novashell int361_
      passes; the four new conform_ cases pass in
      doors' nsh-test run; doors and d are green.
      ask: the laptop agrees, and nsh-test, doors and d are green
      ruling:
      evidence:

## Job order
Jobs run on branch int-361 only. Agents never push to
main. Job I files the intent. G1 first, alone, then G2
(same file). G3 (a proof, no commit) and G4 after G2.
G5 is the laptop. The seal is the first 16 hex of sha256
of int361-plan.txt: b73d76095a4904c8.

## Versions
- novashell patch (a splitter bug fix, no new command;
  6.1.3 to 6.1.4)
- nsh-test patch (four conformance cases; 2.2.3 to
  2.2.4)
