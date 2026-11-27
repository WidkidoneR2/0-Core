---
id: 354
date: 2026-11-25
type: future
title: "the spine parser treats => in on EVENT => ACTION as a redirect"
status: planned
tags: [fsh, nsh, spine, parser, triggers]
depends_on: []
format: 3
---

## Vision
The documented trigger line
`on git_commit => notify "commit detected"` works in the
default shell. `=>` is one word, `on_cmd` finds it, the trigger
is added, and no file named notify is created. Ordinary redirects
that happen to involve `=` or `>`, including `echo a => b`,
`echo a=>b`, `echo test >= x` and `echo test > 0.5`, stay
redirects, the same way bash treats them. `spine migrate` then
reports the on-trigger history rows as Equivalent.

## The Problem
On 2026-11-25, a read-only scoping run against origin/main
50fdbf0f (nsh 6.1.1, after INT-353) confirmed a live regression
in the default shell. The same shape was the largest unexpected
group on the laptop audit (67 of 87 unexpected lines):

    nsh -c 'on git_commit => notify "commit detected"'

The router claims the line, then `on_cmd` prints
`Usage: on <trigger> => <action>` and exits 1. A file named
notify can also be created, because the spine parsed a stdout
redirect whose target is the word notify. With `NSH_SPINE=0`
the same line prints `Trigger added` and creates no such file.

The lexer is doing the POSIX thing. Unquoted `>` is always
RedirectOut (spine/lexer.rs:133), and `=>` is not a two-character
operator, so the stream is the word `=`, then `>`, then the next
word. `parse_command` builds that into a redirect in the
RedirectOut arm (spine/parser.rs, around line 165). The `on`
builtin never sees a `=>` token: `on_cmd`
(commands/mod.rs:13543) looks for a word that is exactly `=>`
(line 13591), misses it, and prints usage.

Legacy does not see a redirect here because `detect_redirect`
only matches a `>` written with spaces (`" > "` at
expand.rs:487). `=>` is `=` touching `>`, so the legacy plan
has no io and the audit labels the row
`io: spine redirects, legacy does not`.

This is a known-exception kind of bug, like `> 0.5`. The
parser already refuses a comparison on a query-shaped line
(ComparisonNotRedirect, parser.rs:36 and :242) so
`ps | where cpu > 0.5` works, while `echo test > 0.5` stays a
redirect, matching bash. The prior owner ruling is that `=>`
stays a known exception of that same kind: scoped, deliberate,
and not a lexer change.

Two nearby defects are real and are not this intent. On the
spine path, main.rs:2316 is a `continue` after a claimed line,
so `triggers::evaluate` never runs for a default-shell command
(the call sits later, after the legacy branch). Adding a
trigger would still not fire it. And the docs disagree with
`on_cmd`: the empty-list hint at triggers.rs:242 omits `=>`
(`on health_drop 90 notify "health degraded"`), and
cheatsheet_tui.rs:144 says `on <event> do <action>`.

## The Solution
Option 1, a carve-out in spine/parser.rs `parse_command`, in
the RedirectOut arm only. If the first word is `on` and the
previous word is `=` with its span ending where `>` starts,
pop the `=`, push the word `=>` (span covering both
characters), advance past `>`, and build no redirect. The
following word (`notify`, or the start of the action) is then
parsed as an ordinary word. Leave lexer.rs and expand.rs
untouched. Do the check before the fd-binding pop or any
target consume, so a successful carve-out never takes `notify`
as a redirect target.

G1 adds four unit tests next to the existing redirect tests
in spine/parser.rs. One is red today; the other three are
green today and must stay green:

- `on_arrow_is_words_not_a_redirect` (red).
  `on git_commit => notify "commit detected"` is Complete,
  the words are `on`, `git_commit`, `=>`, `notify`, and
  `commit detected` (the last one quoted), and
  `redirects` is empty. Today the line is Complete with a
  Write redirect to `notify`, and `=>` is not a word
  (the words are `on`, `git_commit`, `=`, plus the quoted
  string).
- `echo_spaced_fat_arrow_stays_a_redirect`.
  `echo a => b` is Complete with one Write redirect to
  unquoted `b`. The leftover `=` stays a word. A carve-out
  that fired for every `=>` would drop the redirect.
- `echo_stuck_fat_arrow_stays_a_redirect`.
  `echo a=>b` is Complete with one Write redirect to
  unquoted `b`. The lexer keeps `=` inside the previous
  word (`a=`), so the previous word is not `=` and the
  carve-out must not fire.
- `echo_ge_stays_a_redirect_to_equals`.
  `echo test >= x` is Complete with one Write redirect
  whose target is the unquoted word `=`, and `x` stays
  an argument. This matches bash (it writes a file named
  `=`). The nsh-test conformance suite already warns that
  a case like this can drop a file named `=` wherever
  the shell runs, which is why G4 uses a scratch dir.

G2 is the carve-out. Only `OperatorKind::RedirectOut`.
`echo test > 0.5` is already a redirect for a non-query
line and must stay one.

G3 is the audit proof on a scratch ZERO_STATE_DB, with no
laptop state. Two `shell_history` rows only: the on line
above, and `echo a => b`. Open the schema through
`StateDb::open` by running `nsh -c 'true'` (or `spine
migrate`) against that db first. Insert the rows with
sqlite via `bash -c`, because nsh does not yet expand
`${VAR:-default}`. Before the fix, the on row is
`io: spine redirects, legacy does not`. After it, the on
row is Equivalent and the echo row is unchanged (still
`io: spine redirects, legacy does not`, because legacy
never sees `" > "` in `=>` and the carve-out is only for
`on`).

G4 is one ordinary nsh-test case in
zero/shell/nsh-test/src/main.rs, not a CONFORMANCE_CASE.
Conform cases compare against bash (which has no `on`
builtin) and the harness drops output lines that start
with `[`, `x `, or the cross mark, so a usage or status
line can vanish from the comparison. The case runs
`on git_commit => notify "commit detected"` in a scratch
directory, with its own ZERO_STATE_DB. It expects exit 0,
stdout containing the ASCII substring `Trigger added`,
and no file named `notify` in that directory.

G5 is the laptop proof: those behaviours, plus doors
and d.

## Limits
- In scope: the RedirectOut carve-out and its unit tests in
  zero/shell/novashell/src/spine/parser.rs; one ordinary
  nsh-test case in zero/shell/nsh-test/src/main.rs; a
  novashell patch bump at close, and an nsh-test patch bump
  if that case lands.
- Out of scope: lexer.rs and expand.rs; the comparator
  rules in spine/compare.rs; trigger firing on the spine
  path (main.rs:2316 skips `triggers::evaluate`; a
  follow-up); the doc mismatches at triggers.rs:242 and
  cheatsheet_tui.rs:144 (a separate follow-up); the stuck
  form `on EVENT=>ACTION` with no spaces around `=>`
  (the previous word is then `EVENT=`, not `=`; the
  documented syntax has spaces); the `;` splitter, brace
  groups, `${VAR:-default}` expansion, and the other items
  in the approved weekend order.
- How deep: five gates, about an evening. One production
  function changes. One new nsh-test case. No new crate,
  no new dependency. Four Cursor jobs on branch int-354
  (G1 and G2 sequential in parser.rs; G3 is a proof with
  no repo commit; G4 edits nsh-test after G2).
- Stop and ask: an existing unit test or nsh-test case
  relied on `on ... =>` being a redirect; the scratch
  migrate before-shape is not
  `io: spine redirects, legacy does not`; a fix seems to
  need a change outside In scope; a laptop row uses the
  stuck form `on EVENT=>ACTION` and would stay unexpected
  after G2.
- Done means: G1 to G5 ticked with evidence.
- Must not change: live state.db, lexer.rs, expand.rs,
  spine/compare.rs, triggers.rs, cheatsheet_tui.rs,
  main.rs, any file outside the repo, and any secret or
  confidential value.

## Success Criteria
- [ ] G1 red first: the four parser tests are in the tree.
      `on_arrow_is_words_not_a_redirect` is red on the
      code as it stands; the three echo guards are green.
      ask: on EVENT => ACTION is parsed as a redirect today, and the echo guards already pass
      ruling:
      evidence:
- [ ] G2 the carve-out in the RedirectOut arm. The four
      G1 tests are green. `echo test > 0.5` and the
      existing ComparisonNotRedirect tests stay green.
      ask: on EVENT => ACTION keeps => as a word, and echo redirects are unchanged
      ruling:
      evidence:
- [ ] G3 a scratch state.db holding the on line and
      `echo a => b` gives Equivalent for the on row after
      the fix; before the fix that row is
      `io: spine redirects, legacy does not`; the echo
      row is unchanged.
      ask: the audit agrees on the on row, and the echo row did not move
      ruling:
      evidence:
- [ ] G4 an ordinary nsh-test case, not a CONFORMANCE_CASE,
      runs the on line in a scratch dir, exits 0, prints
      `Trigger added`, and creates no `notify` file.
      ask: the default shell adds the trigger and does not redirect to notify
      ruling:
      evidence:
- [ ] G5 laptop: in a scratch dir,
      `nsh -c 'on git_commit => notify "commit detected"'`
      prints `Trigger added`, exits 0, and creates no
      `notify` file; `echo a => b`, `echo a=>b`,
      `echo test >= x` and `echo test > 0.5` stay
      redirects and match bash; `NSH_SPINE=0` still adds
      the trigger; nsh-test, doors and d are green.
      ask: the laptop agrees, and doors and d are green
      ruling:
      evidence:

## Job order
Jobs run on branch int-354 only. Agents never push to main.
G1 then G2 in parser.rs. G3 and G4 after G2. G5 is the
laptop. Prompts are in cursor-jobs.md. The seal is the
first 16 hex of sha256 of int354-plan.txt:
227128e30f653200.

## Versions
- novashell patch (the default shell accepts on EVENT => ACTION)
- nsh-test patch if the G4 case lands
