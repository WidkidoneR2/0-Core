---
id: 359
date: 2026-11-27
type: future
title: "core intent park returns an open in-progress intent to planned"
status: in-progress
tags: [core, intent, ledger]
depends_on: []
format: 3
---

## Vision
`core intent park <id> --reason "..."` does what a hand
edit of the status line does today. It finds the open
intent, refuses an id that is not open and a file that is
not in-progress, rewrites only the frontmatter `status:`
line from in-progress to planned, keeps every other byte,
clears focus when that id is focused, and does not commit.
It prints a suggested commit message. After a park, `d`
no longer lists that intent as in progress.

## The Problem
Measured 2026-11-27 on a box checkout of origin/main
c2755efc (core 5.4.0), built with cargo +1.88.0. 386 of
386 `cargo +1.88.0 test -p core` tests passed. The
laptop's tree and state were not touched. All 35 files in
zero/intents/open/ are `status: planned`.

1. There is no park command. IntentCommands
   (cli/parser.rs:648-807) has start, complete, cancel,
   defer and override, and no Park variant. The same
   gap is in cli/commands.rs:371-378, the mapping at
   cli/mod.rs:262-264, and the dispatcher at
   app/dispatcher.rs:199-204. `core intent park` is an
   unknown subcommand. Returning an in-progress intent
   to planned is done by hand with sed.

2. `core intent defer` does not change status. Defer
   (parser.rs:793-798) takes a positional reason.
   defer_intent (intent/mod.rs:5844-5872) finds the file
   in folders::LIFECYCLE (open/), stamps a note under
   `## Gate Check` (`deferred: REASON -- approved by:
   christian DATE`), and writes the file. It does not
   call set_frontmatter_status. Status stays whatever it
   was.

3. `core intent cancel <id> --reason` closes the intent.
   Cancel (parser.rs:800-807) takes a mandatory `--reason`
   flag. cancel_intent (5724-5842) refuses an empty
   reason, finds the file in folders::ACTIVE (open/,
   folders.rs:15), rewrites status to cancelled with
   set_frontmatter_status from
   `["in-progress", "planned", "future", "deferred"]`
   (5746-5750), stamps a gate note, calls close_file to
   remove the file and stage closed.tsv, clears focus
   when that id is focused, and does not commit. It
   does not print a suggested commit message.
   cicomplete does (3546, completion_commit_message at
   2499).

4. The rewrite helper already exists and is exact.
   set_frontmatter_status (1298-1333) replaces one
   frontmatter status line and leaves a body quote of
   the same words alone. start uses it to go to
   in-progress (1411, from planned, future, deferred);
   complete uses it to go to complete (3467); cancel
   uses it to go to cancelled (5746). Nothing uses it
   to go back to planned.

5. start and `d` already do the right thing once the
   line is planned. start (1350) accepts planned and
   future, so a parked intent can be started again.
   in_progress_ids (166-171) keeps only
   `status == "in-progress"`, and doctor (mod.rs:501,
   1166) prints that list as "In progress". A planned
   status is not listed.

Related, found and left alone: start's guard (1350)
refuses status deferred even though the rewrite list
at 1411 would accept it, so a deferred-status file
cannot be started without a hand edit; defer does not
set status deferred; cancel does not suggest a commit
message. None of these is in this intent.

## The Solution
All work is in zero/engine/src/domains/intent/mod.rs,
plus the four files that carry a new Park variant.
The tests sit next to cancel_closes_like_complete_tests
(mod.rs:6949) and the INT-358 parse tests
(cli/parser.rs:1920), and reuse the scratch-repo
helpers cancel already uses (repo, write_open, ctx,
with_scratch_state). Focus is
`$XDG_STATE_HOME/0-core/intent/focus.toml`; the cancel
helper already points XDG_STATE_HOME at the scratch
repo's state dir.

Decision: park writes only the status line. The reason
is printed and included in the suggested commit
message, not stamped under `## Gate Check`. Why: the
job is the status transition, defer already means "add
a gate note", and set_frontmatter_status is built to
keep every other byte. A stamp would change the file
in a second place.

Decision: `--reason` is mandatory, as a `--reason`
flag like cancel, not a positional argument like
defer. An empty or whitespace-only reason is a domain
error, checked first, with the same shape as cancel
(5727-5734): "park requires a non-empty --reason".

Decision: the suggested message carries `Intent:
INT-{id}` and park prints a reminder to add `Seal:` of
that intent's plan (16 lowercase hex). The commit-msg
hook (.githooks/commit-msg:4-6, 75-90) refuses a
message that has Intent: and no Seal: unless it also
has Release:. Park does not invent a seal.

`core intent park <id> --reason "..."`:

1. Require FilesystemWriteHome, as cancel does.
2. Refuse an empty or whitespace-only reason.
3. Find the file with finder::find_intent_file on
   folders::ACTIVE (open/). If it is missing, a domain
   error: "Active intent {id} not found (already
   complete or cancelled?)".
4. Read the file. If the frontmatter status is not
   in-progress (already planned, deferred, or any
   other value), a domain error naming the status, and
   the file is not written.
5. Rewrite with set_frontmatter_status, from =
   `["in-progress"]` only, to = `planned`. Write that
   text. Every other byte stays, including a body
   quote of "status: planned" and any `## Gate Check`
   block.
6. If read_focus() is this id, remove focus_file(),
   as cancel (5786-5790) and complete (3504-3508) do.
   A focus on another id is left alone.
7. Do not call close_file, do not move the file, do
   not touch closed.tsv, do not git add, do not
   commit.
8. Return a ParkReport and print: the id, the reason,
   in-progress -> planned, the relative path, whether
   focus was cleared, then "park does not commit.
   Suggested commit message:" and the message:

   INT-{id}: park; {reason}

   Intent: INT-{id}

   plus the reminder to add Seal: of that intent's
   plan.

G1 adds the red tests and the seams they need, no
behaviour change. Seams in intent/mod.rs, public to
the crate:

`struct ParkReport { rel_path: String,
suggested_commit: String, cleared_focus: bool }`
(Debug and Clone);

`fn park_intent(ctx: &AppContext, id: &str,
reason: &str) -> CoreResult<ParkReport>` as a stub
that returns an empty Ok report and writes nothing;

`fn park_commit_message(id: &str, reason: &str)
-> String` returning the two-line subject-and-trailer
shape above.

Tests are named int359_ so one filter runs them:
`cargo +1.88.0 test -p core int359_`. All seven are red
today and fail on an assertion, none errors. The
seventh also holds two parse guards that already pass.

- `int359_park_rewrites_only_the_status_line` (red).
  An open file with status in-progress, a body quote
  of `status: planned`, and a `## Gate Check` block.
  park_intent(&ctx, id, "taking a break") is Ok. The
  file bytes equal set_frontmatter_status(src,
  &["in-progress"], "planned"). The quote and the
  gate block are unchanged. rel_path is
  `zero/intents/open/{file}`.
- `int359_park_refuses_an_id_not_open` (red). No file
  in open/, and also a file only in decisions/. Both
  are Err; nothing is written; the decisions file is
  byte-identical.
- `int359_park_refuses_already_planned_and_any_other_status`
  (red). A planned file in open/ is Err and names
  planned; a deferred-status file in open/ is Err and
  names deferred. Both files stay byte-identical.
- `int359_park_refuses_an_empty_reason` (red). An
  in-progress file, reason `""` or `"   "`, is Err
  and names a non-empty --reason; the file stays
  byte-identical.
- `int359_park_clears_focus_when_focused_and_leaves_other_focus`
  (red). With focus.toml for this id, park removes
  that file and cleared_focus is true. With focus.toml
  for another id, park leaves that file and
  cleared_focus is false. In both cases the status
  line is planned.
- `int359_park_does_not_commit_or_stage_and_suggests_a_message`
  (red, on a git repo with the in-progress file
  committed). After park, HEAD is unchanged, the
  index is clean, closed.tsv is absent, the file is
  still in open/, and only the status line is dirty.
  suggested_commit equals park_commit_message.
- `int359_park_parses_with_reason_and_park_alone_does_not`
  (red, in cli/parser.rs next to the INT-358 parse
  tests).
  `Cli::try_parse_from(["core", "intent", "park",
  "312", "--reason", "taking a break"])` is Ok,
  `["core", "intent", "park", "312"]` is Err, and
  `["core", "intent", "cancel", "181", "--reason",
  "x"]` and `["core", "intent", "defer", "181",
  "a note"]` are Ok. Only the two park assertions
  are red.

G2 is the write, the refusals, and the returned
message. park_intent does steps 1-5 and 7 above (not
focus yet, not the print) and fills suggested_commit
from park_commit_message. The four write and refusal
tests and the no-commit message test go green. The
focus test stays red.

G3 is focus. park_intent does step 6. The focus test
goes green. A park of an unfocused intent does not
create or clear a focus file.

G4 wires the command and prints the report. IntentCommands
gains `Park { id: String, #[arg(long)] reason: String }`
with the doc line "Return an in-progress intent in
open/ to planned (rewrites only the status line, does
not commit)" (next to Cancel in cli/parser.rs); the
same fields on IntentCommand::Park (commands.rs), the
mapping (cli/mod.rs) and the dispatcher
(app/dispatcher.rs) calling park_intent. The command
prints the report in the shape above and returns Ok
on success, or the domain error on a refusal. The
parse test and the suggested-message test go green.
All seven int359_ tests pass. Proof on the box, on a
scratch HOME copy of the tree: make one open file
in-progress, run `core intent park ID --reason
"taking a break"` (exit 0, only the status line
changes, no commit), run it again (refused, already
planned), run `core intent park 999 --reason "x"`
(refused, not found). Paste the output.

G5 is the laptop proof.

## Limits
- In scope: intent/mod.rs (ParkReport, park_intent,
  park_commit_message, the int359_ tests next to
  cancel_closes_like_complete_tests); one Park
  variant through cli/parser.rs, cli/commands.rs,
  cli/mod.rs and app/dispatcher.rs; one parse test in
  cli/parser.rs; a core bump at close.
- Out of scope: a gate stamp or any write other than
  the status line; changing defer, cancel, start or
  complete (including teaching start to accept status
  deferred, and adding a suggested commit message to
  cancel); a cipark alias or nsh completion for one;
  closed.tsv and close_file; moving the intent file;
  a confirmation prompt; the integrity domain; zero-core
  path helpers; the commit-msg hook.
- How deep: five gates, a short sitting. One new
  function (park_intent) and one helper
  (park_commit_message); one Park variant through four
  CLI files. No new crate or dependency. Reuses
  set_frontmatter_status, finder::find_intent_file,
  folders::ACTIVE, and the focus-clear pattern cancel
  and complete already use.
- Stop and ask: an existing intent or CLI test fails
  or needs its assertion changed; park would write any
  byte other than the frontmatter status line; park
  would stage, commit, or touch closed.tsv; a case
  seems to need a change outside In scope.
- Done means: G1 to G5 ticked with evidence.
- Must not change: live state.db and the laptop's tree
  (agents use scratch HOME and scratch repos only),
  the behaviour and printed text of defer, cancel,
  start and complete, set_frontmatter_status itself
  (reuse it, do not edit it), closed.tsv, any file
  outside the repo, and any secret or confidential
  value.

## Success Criteria
- [x] G1 red first: the seven int359_ tests and the
      seams (ParkReport, the empty park_intent, and
      park_commit_message) are in intent/mod.rs and
      cli/parser.rs. All seven fail on their
      assertions, none errors, and the cancel and
      defer parses inside the seventh already pass.
      The 386 existing tests stay green.
      ask: today there is no park command, and a hand edit is the only way back to planned
      ruling: confirmed on int-359 299a79c4
      evidence: 299a79c4 -- seams ParkReport, park_intent stub and park_commit_message plus seven int359_ tests; all seven failed on their assertions, none errored; 386 existing passed
- [x] G2 park_intent rewrites only the status line,
      refuses an id not in open/, a non-in-progress
      status (already planned included) and an empty
      reason, neither stages nor commits, and returns
      the suggested message. The five write, refusal
      and no-commit tests pass.
      ask: only the status line changes, a bad id or status is a no-op, and the message is ready
      ruling: confirmed on int-359 297f3dde
      evidence: 297f3dde -- status read from the frontmatter only; empty reason, id not in open, and any status other than in-progress are refused and write nothing; only the status line changes; no commit or stage; five int359_ tests pass, 391 total
- [x] G3 park_intent clears focus only when this id is
      the focused intent. The focus test passes.
      ask: parking the focused intent drops focus, and another intent's focus stays
      ruling: confirmed on int-359 0bafbcce
      evidence: 0bafbcce -- focus removed only when read_focus is this id, the same way cancel and complete do; another focus is left alone; int359 focus test passes, 392 total
- [x] G4 `core intent park <id> --reason` is wired and
      prints the report and the suggested commit
      message in the shape above. All seven int359_
      tests pass. Box scratch proof pasted.
      ask: the command reads the way it should, does not commit, and suggests the message
      ruling: confirmed on int-359 9d1c7c29
      evidence: 9d1c7c29 -- Park wired through parser, commands, cli mod and dispatcher; --reason required; prints the report and suggested message with a Seal reminder; box proof park, second park refused, 999 refused; 393 of 393 pass
- [x] G5 laptop: after merge and ship, `core intent
      park` on a scratch in-progress intent in a copy
      of the tree rewrites only that status line, exits
      0, and a second park is refused; `d` does not
      list that id as in progress; the laptop's real
      open intents and focus are untouched; d is green.
      ask: the shipped binary parks a scratch intent, and the laptop's ledger is untouched
      ruling: confirmed on laptop
      evidence: Block T 7 int359_ passed, 393 of 393, park help ok, park 999 refused, tree clean, doors 4 of 4; merge, 5 seals intact, ship core, d 96 pct with only cistart uncommitted, Integrity 100 pct; scratch clone park 359 printed Intent parked, clone status empty, second park refused as planned

## Job order
Jobs run on branch int-359 only. Agents never push to
main. Job I files the intent. G1 first, alone. Then
G2, G3, G4 in that order (each builds on park_intent).
Each job pulls with rebase before it pushes. G5 is
the laptop. The seal is the first 16 hex of sha256 of
int359-plan.txt: dcd49185d67fd512.

## Versions
- core minor (`core intent park` is a new user-facing
  command; core is 5.4.0, so the close bumps to 5.5.0)
