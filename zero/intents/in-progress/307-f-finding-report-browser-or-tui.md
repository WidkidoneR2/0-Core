---
id: 307
date: 2026-11-10
type: future
title: "F-Finding Report browser or TUI"
status: in-progress
tags: [bug report, nsh, novashell, findings]
---

## Vision
A person can look at the findings and see, for each one, how bad it is, whether
it is open or solved, who found it, who fixed it, and whether that fix's seal
still checks. The screen may be a TUI, a browser, or both. Nothing is built
before Christian rules the gates below. Color is allowed. Leaving the screen,
failing inside it, or piping its text must leave the shell usable.

## The Problem
Findings are files. zero/intents/findings/ holds 110 records named F-NNNN.md.
The highest number on disk is F-0117. F-0081 through F-0087 are gaps, and a gap
stays a gap. The one writer is findings::render
(zero/engine/src/domains/intent/findings.rs:104). It writes seven fields: id,
what, at, found_by, found_on, closed_without_fix, and severity. A field nobody
gave is the word unknown. parse_record (findings.rs:139) is the one reader. A
record with no severity line reads unrated, never a guessed low
(findings.rs:167). Measured 2026-11-10 against the 110 files: every record has
id, what, at, found_by, found_on and closed_without_fix; 21 have no severity
line (F-0010, F-0018, F-0020, F-0022, F-0024, F-0025, F-0026, F-0028, F-0029,
F-0030, F-0031, F-0032, F-0033, F-0034, F-0035, F-0042, F-0045, F-0049, F-0050,
F-0104, F-0109); severity on the rest is low 47, medium 36, high 6. None has
promoted_to. Three are closed without a fix, because closed_without_fix is not
the word no: F-0018, F-0031 and F-0107.

Whether a finding is solved is not a field. state_of (trace.rs:130) derives it
from Fixes: trailers on HEAD. A Fixes: value counts only when it is exactly F-
and four digits (is_finding_id, trace.rs:75). Fixed wins over a closure. Git
that cannot be read is Unread, which the report calls unknown, and is never
shown as open. A Finding: trailer does not close anything. trace_commit
(trace.rs:519) prints Fixes: as fixes and Finding: as touches. Measured
2026-11-10, git log on HEAD: 23 commits carry a Fixes: trailer, 18 carry a
Finding: trailer, and one carries both (e8679d19, Finding: F-0027, Fixes:
F-0026).

core intent trace F-0010 goes to trace_finding (trace.rs:304). It prints the
record, then open, closed without a fix, or each fixing commit with seal_line
(trace.rs:407): Intact, Broken, NoNote, Legacy, or NoSeal. core intent trace
INT-266 lists the findings that intent found (trace, trace.rs:166). The seal is
the first 16 hex of sha256 of the plan bytes (seal_digest, trace.rs:228). One
implementation. check_seal (trace.rs:254) is that verdict.

The list a person can already run is core bug, from INT-300. With no arguments
it is the dashboard (parser.rs:860, report at bug.rs:1148): counts, open by
severity, the oldest open bug, the median days from found to fixed, and fixes
in the last 7 days. core bug list (bug.rs:242) prints plain columns, from
render_list (bug.rs:153): id, state, severity, the day found, and what. The
default is the unresolved, worst first. --state, --severity and --all narrow
it. The row it prints (bug.rs:22) has no found_by and no seal. core bug show
F-NNNN (bug.rs:399) is one finding's story: where, severity, who found it, the
solved day, and the seal phrase (story, bug.rs:322). core bug check
(bug.rs:462) names a Fixes: value that is not F-NNNN, a Fixes: that names no
record, and a closure that a fix has overtaken. core bug welcome (bug.rs:1386)
prints one line for the shell banner: bugs: N open (H high). core bug rate and
core bug promote write only on apply SEAL. None of these is a screen.

Two nsh screens already take the terminal, and neither is a findings browser.
cheat is an interactive REPL arm (novashell main.rs:3122). The -c path runs
the line and exits (main.rs:1088) before that arm, so nsh -c cheat does not
open it. cheatsheet_tui.rs is ratatui and crossterm: enable_raw_mode,
EnterAlternateScreen, then LeaveAlternateScreen and disable_raw_mode after the
loop (cheatsheet_tui.rs:347). The comment on line 2 calls it fuzzy-searchable.
The filter (filter_entries, cheatsheet_tui.rs:733) is a substring contains on
name, description, category and expansion. It is not fzf, and it is not a
fuzzy scorer. Tab cycles kind: all, builtins, commands, keybinds, aliases. The
rows come from command_registry in state.db. it (main.rs:3127, INT-254) opens
intent_tui.rs, the same ratatui and crossterm pattern, two panes, a status
filter, a substring search, and e for $EDITOR. It loads future, in-progress
and complete only (intent_tui.rs:468). It does not read findings. An absent
0-Core is refused before the alternate screen (main.rs:3132). history_tui,
health_tui, git_tui, db-browse, friday-chat and zero-ade are ratatui as well,
and none of them is a findings view.

The short names a person types live in ~/.config/nsh/config.nsh, which is not
in this tree. docs/COMMAND-GUIDE.md records ints as core intent show, inta as
core intent add, cistart as core intent start, and cicomplete as core intent
complete (lines 21-25). inta needs a terminal (intent/mod.rs:2757). None of
those names opens a findings screen.

## The Solution
No surface and no home are chosen here. The behaviour under discussion is a
view of the findings that shows severity, state, who found one and who fixed
it, and the seal verdict of that fix, with filters, in color, and that never
leaves the shell broken. Each gate is a question for Christian to rule. A
ruling is a RULED line in this file, dated, in his words. Declining a gate,
with the reason, closes it. No code is written before G1 through G6 have those
lines.

## Success Criteria
- [ ] G0 The recon in The Problem is read against the tree and either stands or
      is corrected here before any other gate. Re-read: 110 finding files,
      highest F-0117, gaps F-0081 through F-0087; render writes seven fields
      and parse_record is the one reader; 21 records have no severity and read
      unrated; state comes from Fixes: through state_of, and Finding: only
      touches; core bug list is plain and has no found_by and no seal; core
      bug show and core intent trace F-x carry the seal verdict; cheat and it
      are ratatui and crossterm, cheat's filter is substring and not fzf, and
      it does not load findings. Proof: a dated line in this file saying the
      recon stands, or the correction.
- [x] G1 The surface. Christian rules a TUI, a browser, or both, and why. A
      browser is a page a person opens. A TUI is a screen inside the terminal.
      Both means each is a view of the same rows, not two reports. No code
      before this line exists. Proof: the RULED line.
RULED 2026-11-10 (Christian): a TUI now. A browser page is possible future work, and it is out of scope for this intent.
- [x] G2 The home. Christian rules one place, or a split, and why. The options
      left open: a new tool; a core subcommand beside core bug, which already
      owns the rows; an nsh builtin in the same REPL door as cheat and it. A
      browser is not an nsh builtin. The ruling says what that home can see,
      and what it must not start on its own. No code before this line exists.
      Proof: the RULED line.
RULED 2026-11-10 (Christian): it should look and work like cheat. The ruling is cheat-style, same framework. The exact binary or subcommand name is left for the red step.
- [x] G3 The framework, if the surface has a TUI. Christian rules whether it
      reuses cheatsheet_tui (ratatui, crossterm, alternate screen), reuses the
      two-pane layout of intent_tui, or is a new screen. fzf is not what cheat
      uses. If the surface is only a browser, this gate says which program
      serves the page and that the shell does not depend on it. Proof: the
      RULED line.
RULED 2026-11-10 (Christian): reuse the cheatsheet TUI framework (ratatui and crossterm), the one cheat uses. The ruling is cheat-style, same framework. The exact command name is left for the red step.
- [x] G4 What a row shows. Christian rules the columns. The facts already
      measured, to accept or cut: severity (missing reads unrated), state
      (open, solved, closed, unknown, from state_of, never a stored fixed
      field), found_by, found_on, and for a fix the commit and the seal verdict
      from check_seal (Intact, Broken, NoNote, Legacy, NoSeal). Whether a
      Finding: touch is shown is part of the ruling. core bug's Row (bug.rs:22)
      does not carry found_by or the seal, so a wider row is a ruling, not an
      assumption. Proof: the RULED line.
RULED 2026-11-10 (Christian): columns, in order: F-#### id, description (the short title), date found, date fixed, severity, state. State is open in red or fixed in green. Date found is the date in the finding file (found_on). Date fixed is the date of the commit carrying Fixes: F-####, through state_of. When fixed is not applicable, or a date is missing, show an em dash. Never a guessed date. Enter opens the full finding, including the fixing commit and whether its seal checks out. Colors and sort: fixed is green, open or needs attention is red, no severity set is yellow (needs triage). Open findings come first, ordered high, medium, low, and high gets a bold red marker. Fixed findings sink to the bottom in green.
- [x] G5 Filters. core bug list already takes --state and --severity and
      defaults to the unresolved, worst first. Christian rules whether the
      screen uses those same words, and whether search is the substring filter
      cheat and it already use. Proof: the RULED line.
RULED 2026-11-10 (Christian): type-to-filter like cheat (substring), plus single keys: o open only, h high only, a show all. The filters can be tested to see if they work. The red step includes a test for each: the substring filter, o, h, and a.
- [x] G6 Color that must never break the shell. Christian rules what is colored
      and what stays plain. A TUI returns the terminal to the state it found:
      raw mode off, alternate screen left, including when it fails. A colored
      line on a pipe, or under NO_COLOR, stays readable. commit-msg's color()
      already checks NO_COLOR and whether stderr is a terminal. core bug list
      is plain today. INT-256 is the class this gate refuses to repeat: a tool
      that dies when its output is piped. Proof: the RULED line.
RULED 2026-11-10 (Christian): colors are off when NO_COLOR is set, or when output is not a terminal, including when it is piped. The terminal is always restored on exit, including a panic or a crash. It must never break the shell.
- [x] G7 Red, then the viewer, only after G1 through G6 are ruled. Shown
      failing first, under its own plan Seal: the row, each filter (the
      substring filter, o, h, and a), and the restore. The command is
      findings. Red failed first, under its own plan Seal. The screen ruling
      is cheat-style, same framework. Then the same tests pass. The suite is
      the one the ruled home already uses: cargo test -p core, cargo test -p
      novashell, or an nsh-test row. Proof: the red output, then the green
      output, each under its plan Seal.
RULED 2026-11-10 (Christian): the command is findings.
<!-- evidence: 2026-11-10. Red commit ee6a4846, Seal baa4438095ea0cff. cargo test -p core g7_ -- --ignored --test-threads=1: 1 passed, 10 failed. The pass is g7_key_a_shows_all, because the stub returns every row it is given and assemble returns none. cargo test -p core on that commit: 266 passed, 0 failed, 11 ignored. Green: cargo test -p core 277 passed, 0 failed, 0 ignored. cargo test -p novashell 258 passed. cargo test -p nsh-test 6 passed. cargo doc -p core --no-deps and cargo doc -p novashell --no-deps: 0 warnings. cargo fmt --check clean. -->
- [ ] G8 Doors, after any ship. cargo test -p core passes. cargo test -p
      novashell passes. nsh-test passes after cargo build -p novashell -p
      nsh-test. core intent validate reports every intent valid. d reports 0
      failed. ship, then reload, and the deployed artifact is what was watched.
      Proof: the door output.

## Out of scope
- Rewriting a finding so a missing severity becomes a stored line. A missing
  field reads unrated. Old records stay as they are.
- A second reader of a finding file, or a second seal hash. parse_record and
  seal_digest stay the ones.
- Storing open or solved on the record. state_of stays the derivation.
- Changing cheat or it, unless a ruling says the new screen lives inside one
  of them.
- Filing, rating, or closing findings. core bug rate and core bug promote
  already own those writes.
- A browser page. G1 leaves it as possible future work, outside this intent.
- A program that starts at login, or a listener the shell needs in order to
  start.

## Open for Christian
- G0, whether the recon stands.
- G8, the doors.
