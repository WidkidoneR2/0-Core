---
id: 357
date: 2026-11-27
type: future
title: "the spine refuses compound commands instead of parsing them as a sequence"
status: in-progress
tags: [nsh, spine, parser, migrate]
depends_on: []
format: 3
---

## Vision
A line that opens with for, while, until, if or case, or a
( ) group, is refused by the spine parser as a compound
command. It is valid shell the spine has no grammar for, and
sh or the grouping handler already runs it. `spine migrate`
then lists these rows under Refused at parse, and the
"sequence" row under Declined at lowering is gone, so the
report stops saying THE 38 ARE THE BLOCKER for lines the
shell already runs the same way bash does.

## The Problem
Measured 2026-11-27 on a box checkout of origin/main
0a7807a4 (nsh 6.1.2, nsh-test 2.2.2), built with cargo
+1.88.0. Laptop state was not touched.

Christian's 2026-11-24 `spine migrate` said: Declined at
lowering: 95 bare assignment, 70 value pipeline, 38
sequence. Legacy still owns 203 lines: 165 ruled kinds, 38
not yet ruled. THE 38 ARE THE BLOCKER. These 38 are in
Feature gaps, not in the Unexpected count.

How a row becomes "sequence":
1. The audit splits each history row with the live
   splitter, crate::split_into_segments
   (commands/mod.rs:1057). Ordinary `;`, `&&` and `||` are
   split away there.
2. What keeps a `;` or `&&` inside is what the splitter
   keeps atomic on purpose: for, while and until loops,
   if ... fi, piped while loops, and ( ) groups
   (split_semicolons_marked, main.rs:340; line_has_compound,
   main.rs:459; split_around_compounds, main.rs:510).
3. The spine parser has no compound grammar. parse_command
   pushes `for` (or `(cd`) as an ordinary first word
   (spine/parser.rs:422), and parse_sequence reads the
   inner `;` and `&&` as connectors and builds a Sequence
   (spine/parser.rs:553). So
   `for x in a b; do echo $x; done` parses Complete as a
   command named for, then `do echo $x`, then `done`.
4. lower_pipeline declines a Sequence as "sequence"
   (spine/plan.rs:552; the same in lower_with_io at
   :816). observe() counts that as a feature gap with key
   `unlowerable: sequence` (spine/migrate_audit.rs:247).
   It is not in RULED_DELIBERATE (migrate_audit.rs:556),
   so it is counted as not yet ruled.

Scratch reproduction: a one-row db per line, run through
`spine migrate`. Each of these gave `1 sequence` under
Declined at lowering:
`for x in a b; do echo $x; done`,
`while true; do sleep 1; done`,
`until false; do echo x; done`,
`if true; then echo y; fi`,
`if [ -f x ]; then echo y; else echo n; fi`,
`ls | while read f; do echo $f; done`,
`(cd /tmp && ls)`, `(echo b; echo c)`,
`(cargo build; cargo test)`, the group in
`echo a && (echo b; echo c)` and in
`time (cargo build; cargo test)`, and
`for i in 1 2 3; do echo $i; done && echo ok`.
The laptop's 38 rows were not listed one by one; the
report prints only examples.

Live behaviour is not the bug. try_shell_construct
(engine.rs:1757, called at main.rs:2080, before the spine
router) sends a line whose first word is for, while, until,
if or case to sh whole, and try_grouping (engine.rs:1784)
runs a ( ) group. A piped while reaches the router, the
spine declines it, and legacy runs it. Measured against
bash -c on the box: for, if, piped while and the groups
above print the same output.

So the spine is claiming a parse it does not have. The
parser already refuses valid shell it does not own
(Refusal, spine/parser.rs:30), and the router already
treats a refusal as legacy fallback
(exec.rs:1365, SpineAttemptError::Refused => Declined).
A compound command belongs there.

## The Solution
G1 adds four unit tests next to the existing refusal tests
in spine/parser.rs, all named int357_ so one filter runs
them. They assert through the Debug text of the result, so
they compile before the new variant exists. Three are red
today and one is a green guard:

- `int357_compound_keywords_are_refused` (red).
  `for x in a b; do echo $x; done`,
  `while true; do echo x; done`,
  `until false; do echo x; done`,
  `if true; then echo y; fi` and
  `case x in a) echo a;; esac` are Refused, and the Debug
  text names CompoundCommand with keyword for, while,
  until, if and case.
- `int357_piped_while_is_refused` (red).
  `ls | while read f; do echo $f; done` is Refused with
  keyword while.
- `int357_paren_group_is_refused` (red).
  `(cd /tmp && ls)`, `(echo b; echo c)` and `(echo hi)`
  are Refused with keyword `(`.
- `int357_non_opener_keywords_still_parse` (green).
  `"for" x`, `fortune` and `echo if then fi` are
  Complete, and `echo for; echo if` and `echo a; echo b`
  are still a Sequence.

G2 is the refusal. In spine/parser.rs add
`Refusal::CompoundCommand { keyword: &'static str, span }`
and the same ParseError variant, map one to the other in
parse(), and in parse_command, just before the first word
is pushed (words and assignments both empty), return it
when that word starts with an unquoted literal that is
exactly for, while, until, if or case, or that starts with
`(`. Quoted words do not count, as in bash. Add the
matching arm to diagnose_parse in spine/mod.rs (the match
is exhaustive, so the build needs it) and to
spine_parse_error in spine/migrate_audit.rs with the key
`refused: compound command (for, while, until, if, case,
( )) -- sh owns it`. A box prototype of this change is 99
added lines across those three files including the G1
tests, and cargo test -p novashell went 302 to 306 passed.

The refusal also fires inside `$( )`, because the parser
parses a substitution body. On the box
`x=$(for i in 1 2; do echo $i; done)` moved from
`bare assignment` to `compound command`. Both route to
legacy, so the shell runs it the same way.

G3 is the audit proof on a scratch ZERO_STATE_DB with no
laptop state. Ten rows: the eight compound rows from G1's
shapes (for, while, until, if, piped while, two groups, and
`echo a && (echo b; echo c)`) plus `echo a; echo b` and
`echo hi`. Measured on the box, before (main 0a7807a4):
Equivalent 4, Feature gaps 8, `8 sequence` under Declined
at lowering, and "8 not yet ruled. THE 8 ARE THE BLOCKER".
After (prototype): Equivalent 4, Feature gaps 0, no
sequence row, `8 compound command` under Refused at parse,
"0 not yet ruled", and "Every remaining kind is ruled
deliberate."

G4 adds four ordinary CONFORMANCE_CASES to
zero/shell/nsh-test/src/main.rs, compared against bash:
`for x in a b; do echo item-$x; done`,
`if true; then echo if-taken; fi`,
`printf 'p\nq\n' | while read f; do echo got-$f; done`
(written with doubled backslashes in the Rust literal, as
the existing printf case is), and
`(echo grp-a; echo grp-b)`. No output line starts with `[`,
`x ` or the cross mark, so the harness filter drops none.
These are regression guards: the shell's behaviour does
not change, so they are green before and after G2.

G5 is the laptop proof.

## Limits
- In scope: the refusal and its four unit tests in
  zero/shell/novashell/src/spine/parser.rs; one
  diagnostic arm in spine/mod.rs; one key in
  spine/migrate_audit.rs spine_parse_error; four
  CONFORMANCE_CASES in zero/shell/nsh-test/src/main.rs;
  a novashell patch bump and an nsh-test patch bump at
  close.
- Out of scope: a compound-command grammar in the spine
  (for, while, until, if, case, subshell, brace group;
  plan.rs:810-815 explains why one plan cannot run a
  sequence); main.rs split_into_segments and the
  splitter; brace groups (`{ echo a; echo b; }` fails
  today, command not found: {); find -exec with an
  escaped `;`; `case` lines, which the splitter cuts at
  `;;`; a ( ) group followed by a redirect, a pipe or `&`
  (measured: `(echo hi) > f`, `(echo a; echo b) | wc -l`
  and `(echo hi) &` differ from bash today); `time ( )`;
  `"for" x` reaching sh through command_word; lexer.rs;
  RULED_DELIBERATE; the labels in spine/audit.rs
  (`spine audit` counts any non-operator refusal as
  ParseFailure); a deeper `{` or `[[` or `!` refusal.
- How deep: five gates, about an evening. One production
  function gains one check. No new crate, no new
  dependency. Four Cursor jobs on branch int-357 (G1 then
  G2 in parser.rs; G3 is a proof with no repo commit; G4
  edits nsh-test).
- Stop and ask: an existing unit test, golden case or
  proptest relied on a compound line parsing Complete; a
  live line changes route or output; the G3 before-shape
  is not `8 sequence`; a fix seems to need a change
  outside In scope; the laptop still shows a sequence row
  after G2.
- Done means: G1 to G5 ticked with evidence.
- Must not change: live state.db, lexer.rs, main.rs,
  engine.rs, exec.rs, plan.rs, the existing
  RULED_DELIBERATE list, any file outside the repo, and
  any secret or confidential value.

## Success Criteria
- [x] G1 red first: the four int357_ tests are in
      spine/parser.rs. The three refusal tests fail on the
      code as it stands, none errors, and
      int357_non_opener_keywords_still_parse passes. The
      existing tests stay green.
      ask: compound lines parse Complete as a Sequence today, and the guard already passes
      ruling: confirmed on int-357 6df0ae9c
      evidence: 6df0ae9c -- parser.rs tests only; 303 passed, the 3 refusal tests failed as expected, none errored, non_opener passed
- [x] G2 the CompoundCommand refusal in parse_command, the
      diagnose_parse arm in spine/mod.rs and the
      spine_parse_error key in spine/migrate_audit.rs.
      All four int357_ tests pass and cargo test -p
      novashell is green.
      ask: compound commands are refused, and plain sequences and every other test are unchanged
      ruling: confirmed on int-357 cffdc020
      evidence: cffdc020 -- parser.rs, mod.rs, migrate_audit.rs; 306 of 306 pass, int357_ 4 passed
- [x] G3 a scratch state.db with the ten G3 rows gives
      `8 sequence` and "8 not yet ruled" before the fix,
      and after it no sequence row, `8 compound command`
      under Refused at parse, "0 not yet ruled", and
      Equivalent still 4.
      ask: the audit moves the compound rows from sequence to refused, and Equivalent does not move
      ruling: confirmed on a scratch db, no repo commit
      evidence: main gave 8 sequence and 8 not yet ruled; int-357 gave no sequence row, 8 compound command under Refused at parse, 0 not yet ruled, Equivalent 4
- [x] G4 four CONFORMANCE_CASES for for, if, piped while
      and a ( ) group are in nsh-test and match bash.
      ask: the default shell still runs compound commands the way bash does
      ruling: confirmed on int-357 c82aa8d0
      evidence: c82aa8d0 -- 4 cases match bash -c output; nsh-test door pass on laptop
- [x] G5 laptop: cargo test -p novashell int357_ passes;
      `./target/debug/nsh -c 'spine migrate'` shows no
      sequence row under Declined at lowering and
      "0 not yet ruled" unless a new unruled kind appears;
      nsh-test, doors and d are green.
      ask: the laptop agrees, and nsh-test, doors and d are green
      ruling: confirmed on laptop, with a scope addition Christian approved
      evidence: after G2 the laptop still showed 2 sequence (one plain grep then echo line, typed twice), a Stop and ask case; Christian ruled plain sequence deliberate; 5a0527f4 prints examples per stage kind, 09d6dcb9 adds unlowerable sequence to RULED_DELIBERATE; final audit 213 compound command refused, 2 sequence ruled, 0 not yet ruled; int357_ 6 passed, doors 4 of 4, d 96 pct with only cistart uncommitted

## Job order
Jobs run on branch int-357 only. Agents never push to main.
G1 then G2 in parser.rs. G3 and G4 after G2. G5 is the
laptop. The seal is the first 16 hex of sha256 of
int357-plan.txt: cde6a01c17fe8bbb.

## Versions
- novashell patch (the spine refuses compound commands)
- nsh-test patch (four conformance cases)
