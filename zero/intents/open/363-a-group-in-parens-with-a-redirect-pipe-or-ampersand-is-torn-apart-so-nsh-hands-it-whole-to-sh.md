---
id: 363
date: 2026-11-28
type: future
title: "a ( ) group with a redirect, pipe or & is torn apart, so nsh hands it whole to sh"
status: planned
tags: [nsh, splitter, subshell, sh]
depends_on: []
format: 3
---

## Vision
In bash, `(echo a; echo b) | wc -l`, `(echo hi) > f`,
`(sleep 5; echo done) &`, `echo x | (cat; echo y)` and
`time (...)` each run as one command. After this, nsh
does the same: a line piece that holds a ( ) group with
a redirect, a pipe or `&`, or a group after `time` or a
pipe, goes to `sh` whole, the way `for`, `if` and `case`
lines already do. When that piece names an nsh-only
alias or builtin, nsh says so in one short line before
it runs the piece, so the failure is not silent. A bare
group, `(a) && b`, quotes and `$( )` stay as they are.

## The Problem
Measured 2026-11-28 on a box checkout of origin/main
bdba6c91 (novashell 6.1.4, nsh-test 2.2.4), nsh built
with cargo +1.88.0. Each line was run as `nsh -c` and as
`bash -c` in its own scratch directory under /tmp, with a
scratch HOME and a scratch ZERO_STATE_DB, comparing exit
code, stdout and the files left behind (88 lines, script
and raw output in the notes). Laptop state was not
touched. `cargo +1.88.0 test -p novashell`: 313 passed;
`cargo +1.88.0 test -p core`: 397 passed.

How a group is cut today:
1. run_input (main.rs:1754) calls split_into_segments
   (main.rs:295), which calls split_semicolons_marked
   (main.rs:340). line_has_compound (main.rs:472) is
   true for any `(` outside quotes, a backslash and
   `$(`, so split_around_compounds (main.rs:527) cuts.
2. In it, a top-level `(` first pushes everything read
   so far as its own piece (main.rs:585), and the `)`
   that closes the group pushes the group alone as an
   atomic piece (main.rs:647-652). So whatever follows
   the `)` is a new piece, and `time` or `cmd |` in
   front of the `(` is a piece of its own.
3. Each piece goes to the engine. try_shell_construct
   (engine.rs:1757) runs a piece that is exactly one
   group through try_grouping (engine.rs:1784, with
   strip_one_group, engine.rs:2334, which returns None
   when text follows the `)`), and sends a piece whose
   first word is for, while, until, if or case to sh
   whole (engine.rs:1767).

Measured, bash first, then nsh on bdba6c91:
1. `(echo hi) > g.txt; cat g.txt` bash writes g.txt and
   prints `hi`. nsh prints `hi` on the screen, writes no
   file, says `empty plan: nothing to execute`, cat then
   fails, exit 1. The same with `>>`, `>g.txt` with no
   space, `2> e.txt`, `< i.txt` (`(cat) < i.txt` says
   `cat: missing filename`) and with a `for` loop inside.
2. `(echo a; echo b) | wc -l` bash prints `2`. nsh prints
   `a` and `b`, then `expected a command`, exit 1. The
   same for `|wc -l` with no space, a longer pipe, and
   `(echo oops 1>&2) 2>&1 | wc -c` (`empty stage in
   pipeline`).
3. `(echo hi) &` nsh runs the group in the foreground,
   then says `expected a command`, exit 1.
4. `echo x | (cat; echo y)` bash prints `x` and `y`.
   nsh prints `y` only, with `expected a command` and
   `cat: missing filename`.
5. `time (echo t1; echo t2)` the output matches, but
   nsh prints `time: missing command` and no timing,
   because `time` was cut off as its own piece
   (main.rs:585). Exit 0.
6. `(exit 3) > g.txt; echo rc=$?` bash: rc=3 and an empty
   g.txt. nsh: rc=1, no file.
7. `(echo a) > g.txt && echo after` bash prints `after`.
   nsh prints `a`, fails with `empty plan`, and `after`
   never runs. `(false) | cat || echo fb` and
   `true && (echo a) > g.txt` are cut the same way.
8. `X=1; (X=2; echo in=$X) > g.txt; echo out=$X` and
   `mkdir sub; (cd sub; pwd) > g.txt` print what the
   group should have written to the file on the screen
   and leave no file.
   A session variable set on the same line (`ZZ=1;
   (echo zz=$ZZ) > g.txt`) is seen by sh once the line
   goes there; measured with the prototype.

Already the same as bash, so not part of this:
`(echo a; echo b)`, `(echo a) && echo b`,
`(false) || echo b`, `(echo a); echo b`, `echo a; (echo b)`,
a redirect or pipe inside the parens (`(echo a > f)`,
`(echo a | cat)`), a paren inside quotes
(`echo "(x) > f"`, `echo '(a) | b'`, `echo "time (x)"`),
`echo \(x\) > f`, `find d \( ... \)`, `echo $(echo hi) > f`,
`echo "$(echo a)" | cat` (with a `;` inside the `$( )` it
is not, see below) and a for loop with a redirect
(`for x in 1 2; do echo $x; done > f`).

Different from bash and not in this intent (see Limits):
- Brace groups `{ a; b; }` (the later intent).
- Process substitution: `diff <(echo a) <(echo a)` and
  `echo hi > >(cat)` fail with `redirect missing target
  file`. The `(` there is not a group, and this intent
  leaves it alone on purpose.
- `echo $(echo a; echo b) | wc -l` prints an extra `a`;
  `echo $(echo hi) & wait` prints `hi & wait`;
  `echo $((1+2)) | cat` is refused with `legacy routing
  does not implement pipelines`.
- `time echo t1` (no group): `command not found: time`.
- `wait` is not an nsh command (`command not found:
  wait`), so `(a) & wait` cannot wait for it.
- A function definition `f() { echo fn; }; f`, and
  `if true; then echo y; fi | cat` (`expected a
  command`).
- `(case x in x) echo X;; esac)` with nothing after it:
  strip_one_group counts the `)` of the case pattern, so
  nsh says `subshells are not supported yet: (case`
  (error.rs:47). With a pipe or redirect after it the
  new rule hands it to sh and it works; bare, it does not.
- The destructive-command guard does not look inside a
  ( ) group or a for loop. Measured with a path that does
  not exist, `rm -rf /etc/zz_nonexistent_363` is REFUSED
  under `nsh -c`, and so is the same rm after a `;`,
  but `(rm -rf /etc/zz_nonexistent_363)`, the same with
  `> file`, and the same inside a `for` loop are not
  refused. This is true on bdba6c91 and is not changed
  by this intent in either direction. It is a safety
  gap that needs its own intent.

## The Solution
G1 adds three tests in a new `mod int363_split` at the
very end of main.rs. All are named int363_ so one filter
runs them. They call split_into_segments only. Two are
red today and one is a green guard:
- `int363_group_with_redirect_pipe_or_amp_is_one_piece`
  (red). Each of `(echo hi) > g.txt`, `(echo hi)>g.txt`,
  `(echo a; echo b) >> g.txt`, `(cat) < i.txt`,
  `(echo oops 1>&2) 2> e.txt`,
  `(echo oops 1>&2) 2>&1 | wc -c`,
  `(echo a; echo b) | wc -l`, `(echo a; echo b)|wc -l`,
  `(echo a; echo b) | sort -r | head -1`, `(echo hi) &`,
  `((echo deep)) > g.txt` and
  `(case x in x) echo X;; esac) | cat` is one piece.
  `(echo a) > g.txt; echo b` is two pieces;
  `(echo a) | cat && echo b` is `(echo a) | cat` and
  `echo b`; `true && (echo a) > g.txt` is `true` and
  `(echo a) > g.txt`; in `(false) | cat || echo fb` the
  first piece carries the `||` operator.
- `int363_time_and_pipe_into_a_group_are_one_piece` (red).
  `time (echo t1; echo t2)`, `time (echo t1) > t.txt` and
  `echo x | (cat; echo y)` are one piece each;
  `echo pre; time (echo t1)` is two;
  `true && echo x | (cat; echo y)` is `true` and the rest;
  `(echo a) > g.txt; (echo b) | cat; (echo c) &` is three.
- `int363_plain_groups_quotes_and_substitutions_are_
  unchanged` (green). `(echo a; echo b)` is one piece;
  `(echo a) && echo b`, `(false) || echo b`,
  `(echo a); echo b`, `echo a; (echo b)` and
  `(echo a > g.txt); echo b` split as they do today;
  `echo "(x) > q.txt"`, `echo '(a) | b'`,
  `echo "(a) &"`, `echo "time (x)"`, `echo \(x\) > e.txt`,
  `echo $(echo hi) > c.txt`, `echo $(echo a) | cat`,
  `echo time | cat` and `find . \( -name a -o -name b \)
  -print` are one piece each.

G2 is the splitter. The rule, in one place:
- A piece is handed to sh when, outside quotes, a
  backslash and `$( )`, `<( )`, `>( )`, it holds a
  top-level `( )` group and either (a) after the `)`
  and any blanks comes `>`, `<`, `|` that is not `||`,
  `&` that is not `&&`, or digits followed by `>` or `<`
  (as in `2>&1`); or (b) the text in front of the `(`,
  since the last `;`, `&&` or `||`, is `time` alone or
  ends in a single `|`.
- The piece starts after the last `&&` or `||` in front
  of it, and ends at the next `;`, `&&`, `||` or a
  single `&` (kept in the piece) outside quotes, parens
  and for/while/until/if/case regions. The text before and
  after is split as today, so the `&&` and `||` chain and
  the other parts of the line keep their own route.
- It goes in split_around_compounds (renamed inside as
  scan_compounds, which also reports whether it kept such
  a piece; split_around_compounds keeps its signature).
  Three small helpers (after_last_chain_operator,
  group_head_needs_sh, group_tail_needs_sh) hold the
  rule. `line_has_sh_only_group(line)` asks the same scan,
  so the splitter and the engine cannot disagree, which is
  the INT-361 lesson (one owner).
- A `(` right after `<` or `>` is process substitution,
  not a group, and is left as today.
G2 adds one test, `int363_which_lines_need_sh_for_a_group`,
a table of 10 lines that need sh and 19 that do not.

G3 is the handoff, in try_shell_construct (engine.rs:1757)
only: a piece for which line_has_sh_only_group is true
runs with `sh -c <piece>`, the same call as for a
for/if/case piece, and its exit status is recorded the
same way. Everything else in that function stays. Two
engine tests: a group with a redirect or pipe is run by
sh (file content, exit statuses 0, 3, 0, 0), and a plain
command is not taken.

G4 is the notice. command_words_in (main.rs) reads the
first word of every simple command in the piece: it cuts
at `;`, `&`, `|`, parens and newlines outside quotes (but
not at the `&` of `2>&1` or `&>f`), then skips
`NAME=value`, `time`, `!`, a shell keyword and a
redirect-only part. sh_handoff_notice (engine.rs) takes
each such word that nsh runs itself (an entry in
commands::builtin_names::BUILTINS, or a row in
shell_aliases) and asks sh, with `sh -c 'command -v
"$1"'`, whether sh knows it. A word sh knows (cat, cd, pwd,
env, echo) is not named. If any word is left, one line
goes to stderr before the piece runs:
`nsh: note: this line runs in sh, which does not know the
nsh word: intents` (or `words: gs (alias for git status),
ll (alias for ls)`). The piece still runs; sh prints its
own `not found` and the status is sh's (127). Measured
with the prototype: `(gs; ll; intents) > /dev/null` prints
the note, then sh's three errors, exit 127. Four tests: one
for command_words_in, three for the notice (an alias, a
builtin, and no notice for words sh knows or for quoted
text).

G5 is the box proof, with no repo commit. The same 88
probe lines are run as `nsh -c` and `bash -c` on a main
build and on an int-363 build, once with `sh` as dash (the
box default) and once with `sh` as bash (put first on PATH,
as on an Arch laptop). Measured with the prototype, sh=bash:
52 lines that differ from bash on main match on the
prototype, 20 that matched still match, none went from
matching to differing; 16 still differ and each is a case
in the list above, the notice line, or an nsh display
line (the job line of `(sleep 0.1 &)`, the export echo). With sh=dash: 46
fixed, 20 same, none regressed, 22 still differ (the same
16, plus the four `time (` lines, `<<<` and `((`, which
dash does not run as bash does).

G6 adds four CONFORMANCE_CASES to zero/shell/nsh-test/
src/main.rs after the INT-361 cases, compared against bash
in /tmp: a group with a redirect (the file is emptied
first and read back in capitals, so only a real redirect
gives the answer), a group with a pipe to `wc -l`,
`echo pipe-in | (cat; echo pipe-after)`, and
`(echo grp-d) > /tmp/nsh_conform_h.txt && echo grp-after`.
They are red on main with `nsh -c` (measured: each of the
four differs from bash, and the first, only after the
capitals trick) and match bash on the prototype. The full
nsh-test suite drives a pty REPL and hung on the box
(--category=repl produced no result in 10 minutes), so
it was not run; the laptop runs it through doors.

G7 is the laptop proof.

A box prototype of G1 to G4 adds about 260 non-test lines
and removes 12 in main.rs and engine.rs, plus 231 test
lines; G6 adds 8 lines. `cargo +1.88.0 test -p novashell`
went from 313 to 323 passed (G1 alone: 314 passed, 2
failed; G2 317; G3 319; G4 323); `cargo +1.88.0 test -p
core` is 397 passed; nsh and nsh-test build with `cargo
+1.88.0 build -p novashell -p nsh-test`; rustfmt reports
nothing on the changed lines.

## Limits
- In scope: split_around_compounds (as scan_compounds),
  three small helpers, line_has_sh_only_group and
  command_words_in in zero/shell/novashell/src/main.rs;
  try_shell_construct and one new function
  (sh_handoff_notice) in engine.rs; ten int363_ tests;
  four CONFORMANCE_CASES in nsh-test; a novashell patch
  bump and an nsh-test patch bump at close.
- Out of scope: brace groups (own intent); process
  substitution; `time` without a group; `wait` and job
  control for a group sent to sh; function definitions;
  `if ...; fi | cmd`; the bare `(case ...)` group;
  `$(a; b)` and `$((..))` with a pipe or `&`;
  `${VAR:-x}`; the destructive-command guard inside
  groups and for loops (finding above, its own intent);
  try_grouping, strip_one_group, the spine, exec.rs,
  expand.rs (split_logical), contains_outside_quotes.
- What changes for the user, plainly:
  - A line piece that holds such a group now runs under
    sh. nsh's own words (aliases, builtins like intents,
    health, d) are not available inside it, and nsh's
    cat, which draws with bat, is the real cat there.
    That is what the note says. Before, `(intents) |
    head -1` ran the builtin and broke the pipe; now sh
    fails with `intents: not found` after the note.
  - `( ) &` is started by sh in the background and
    returns at once. nsh does not list it as a job, and
    `jobs` and `fg` do not see it.
  - `time (...)` needs an sh that has `time`. bash as sh
    has it. dash does not: on dash, `time (echo a)` was
    `a` plus `time: missing command` with exit 0 and now
    is `Syntax error: word unexpected (expecting ")")`,
    exit 2, and nothing runs. The laptop's /bin/sh is
    checked in Block T (`ls -l /bin/sh`); if it is dash
    the time case is recorded as a limit, not a gate.
- How deep: seven gates, about an evening. No new crate,
  dependency or file. Nothing in nsh is run by anything
  but sh in the new path.
- Stop and ask: an existing test, golden case or
  conformance case changes result; a probe line that
  matched bash on main stops matching; the fix seems to
  need a change outside In scope; the destructive guard
  looks like it has to be part of this; `spine migrate`
  on the laptop changes in a way the note below does not
  explain.
- Note: the migrate audit splits history rows with the
  same split_into_segments (commands/mod.rs:1057), so
  `spine migrate` may count a few rows differently (a
  row with a group and a redirect now arrives whole).
  Record the numbers as evidence; they are not a gate.
- Done means: G1 to G7 ticked with evidence.
- Must not change: live state.db, exec.rs, the spine,
  expand.rs, what an unescaped quote does, how a bare
  group, `(a) && b` or a for/if/case line is run, any file
  outside the repo, and any secret or confidential value.

## Success Criteria
- [ ] G1 red first: three int363_ tests are in a new
      mod int363_split at the end of main.rs. The two
      piece tests fail on their assertions (none errors)
      and int363_plain_groups_quotes_and_substitutions_
      are_unchanged passes. The 313 existing tests stay
      green.
      ask: today the splitter cuts a group from its redirect, pipe, & and time, and nothing tests it
      ruling:
      evidence:
- [ ] G2 the splitter rule in split_around_compounds with
      line_has_sh_only_group. The three G1 tests and
      int363_which_lines_need_sh_for_a_group pass and
      cargo test -p novashell is 317 passed.
      ask: a group with a redirect, pipe, & or time stays one piece, and everything else splits as before
      ruling:
      evidence:
- [ ] G3 try_shell_construct hands such a piece to sh. The
      two engine tests pass, `nsh -c '(echo hi) > f'`
      writes f with hi, and cargo test -p novashell is 319
      passed.
      ask: nsh runs the piece the way bash does, with sh's exit status
      ruling:
      evidence:
- [ ] G4 the notice: command_words_in, sh_handoff_notice
      and the one-line note. The four notice tests pass,
      cargo test -p novashell is 323 passed, and
      `(gs; ll; intents) > /dev/null` prints the note
      once, then sh's errors.
      ask: an nsh-only word inside such a piece is named instead of failing silently
      ruling:
      evidence:
- [ ] G5 box proof: the 88 probe lines run as nsh -c and
      bash -c on a main build and on the int-363 build,
      with sh as dash and with sh as bash. No line that
      matched bash on main stops matching. With sh as
      bash, the 52 lines that differed now match; with sh
      as dash, 46 do. Every line that still differs is
      explained in the intent or the notes.
      ask: the fixed lines match bash and nothing that matched before moved
      ruling:
      evidence:
- [ ] G6 four CONFORMANCE_CASES for a group with a
      redirect, a pipe, a pipe into a group and a group
      with a redirect then `&&` are in nsh-test.
      ask: the laptop suite holds these four lines to bash from now on
      ruling:
      evidence:
- [ ] G7 laptop: cargo test -p novashell int363_ passes
      (10 tests); the four new conform_ cases pass in
      doors' nsh-test run; `ls -l /bin/sh` is recorded;
      doors and d are green.
      ask: the laptop agrees, and nsh-test, doors and d are green
      ruling:
      evidence:

## Job order
Jobs run on branch int-363 only. Agents never push to
main. Job I files the intent and G1 (same job). Then G2,
G3 and G4, each alone, in that order (all in main.rs and
engine.rs). G5 is a proof with no commit. G6 is the
nsh-test cases. G7 is the laptop. The seal is the first
16 hex of sha256 of int363-plan.txt: be8245e46410cc74.

## Versions
- novashell patch (a splitter fix and a one-line notice,
  no new command; 6.1.4 to 6.1.5)
- nsh-test patch (four conformance cases; 2.2.4 to 2.2.5)
