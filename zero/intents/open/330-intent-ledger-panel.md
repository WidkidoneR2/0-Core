---
id: 330
date: 2026-11-19
type: future
title: "The intent ledger panel: core intent list --json and a Quickshell panel that reads it"
status: in-progress
tags: [ledger, intents, panel, quickshell, json]
depends_on: [321]
format: 3
---

## Vision
Christian sees the ledger on his desktop without opening a terminal. A dark
Quickshell panel on Hyprland shows the totals, every open intent with its
status and gates, the last close, and whether the tree is clean and pushed.
The panel reads `core intent list --json` and never writes anything. Later
panels for the NovaShell Map and Friday reuse its look.

## The Problem
Read against main b4f5fa36.
1. `core intent list` and `core intent show` print colored text only. There
   is no `--json` on either (cli/parser.rs IntentCommands::List and Show), so
   a panel would have to scrape terminal output.
2. The numbers a panel needs live in three places. Planned, in progress and
   gates come from read_open() and gates::read_gates in intent/mod.rs. Done
   comes from zero_core::ledger::completed_ids(), which adds closed.tsv and
   the git archive (246 today). Clean and pushed is worked out only in the
   nsh banner (novashell main.rs, git status and rev-list origin/main..HEAD).
3. Blocked is counted two ways. The banner uses the narrow rule (a planned
   intent with any dependency not complete). core uses dep_state, where a
   cancelled dependency clears. Today they differ on 041, which depends on
   cancelled 039.
4. Nothing extracts the first sentence of ## Vision. The title is all a row
   has today.
5. Quickshell is not installed by any repo step, there is no QML in the repo,
   and docs/TREE.md has no entry for where a panel would live.

## The Solution
One new pure file, intent/ledger_json.rs, builds a serde struct from what
core already reads: the open rows, their gates, closed.tsv, completed_ids,
dep_state, and two git answers. `core intent list --json` and `core intent
show <id> --json` print it to stdout and nothing else; warnings stay on
stderr. serde and serde_json are already dependencies of core
(zero/engine/Cargo.toml), so nothing new is added.

The panel is one QML file, zero/panels/ledger/shell.qml, beside a
fixture.json. It runs `~/.local/bin/core intent list --json` through a
Quickshell Process with a StdioCollector, JSON.parse's the text, and polls
every 10 seconds, or on `r`. With LEDGER_FIXTURE set it reads that file
instead. Christian opens it with `core intent panel`: a new file,
intent/panel.rs, finds quickshell on PATH and starts
`quickshell -p <repo>/zero/panels/ledger` detached, as a floating window he
opens and closes. The repo path comes from zero_core::paths, never a
literal. If quickshell is missing it stops safely (green, nothing changed)
and prints the install line.

Choices Christian ruled on 2026-11-19: the Pending card counts planned
only; the rows show every open intent and then the last five closed.
Starting points to check together while testing (G4 and G6), and change
then if a better way shows up: a cancelled status is grey; a floating
window, not pinned to an edge; opened with `core intent panel`; and
`intent list` keeps the capability it asks for today.

The list schema, version 1:

```json
{
  "schema": 1,
  "counts": { "total": 285, "done": 246, "in_progress": 0,
              "planned": 39, "blocked": 2, "cancelled": 0 },
  "intents": [
    { "id": "330", "title": "The intent ledger panel: ...",
      "status": "planned", "blocked": true,
      "vision": "Christian sees the ledger on his desktop without opening a terminal.",
      "gates": { "done": 0, "total": 6 }, "format": 3,
      "depends_on": ["321"], "file": "open/330-intent-ledger-panel.md" }
  ],
  "recent_closed": [
    { "id": "329", "status": "complete", "closed": "2026-11-19",
      "title": "Intent format 3: ..." }
  ],
  "last_close": { "id": "329", "status": "complete", "closed": "2026-11-19",
                  "last_commit": "a2ef02c2ca9b5be97e0cc20a0568a7f7b93c633c",
                  "title": "Intent format 3: ..." },
  "tree": { "clean": true, "pushed": true }
}
```

total is done + in_progress + planned. recent_closed is the last five rows
of closed.tsv, newest first. A value that could not be read is
null, never 0 or false: tree.clean is null when git cannot answer, and
last_close is null when closed.tsv cannot be read. `show <id> --json` prints
one intent object plus `"gates_list": [{ "label": "G1", "state": "open",
"text": "..." }]`, or for a closed id the closed.tsv row with `"closed": true`
and the `git show` command that reads its text. An unknown id prints nothing
to stdout and exits 1.

Panel colors, sampled from ref-ledger.jpg and ref-register.jpg: background
#111622, cards #20293a, divider #384050, heading #90d0f0, muted #98a0b0,
done #78d888, planned and pending #f0c048, in progress #70a0e8, blocked and
failed #e07870, cancelled grey #98a0b0, live badge #58a860 text on #103020.

## Limits
- In scope: a new intent/ledger_json.rs; `--json` on intent list and intent
  show (the flag in cli/parser.rs, the pass-through in cli/mod.rs,
  cli/commands.rs and app/dispatcher.rs, the call in intent/mod.rs); a new
  intent/panel.rs and the `intent panel` subcommand wired the same way; a new
  zero/panels/ledger/ with shell.qml and fixture.json; one docs/TREE.md row
  for zero/panels/.
- Out of scope: the NovaShell Map and Friday panels (ref-friday.jpg); any
  write from the panel; a Hyprland keybind or autostart; changing the banner
  or its blocked rule; the nsh `it` screen; shipping QML through ship; a
  panel pinned to a screen edge; changing the capability `intent list`
  asks for.
- How deep: one production file per job. The builder and its tests are one
  job; the CLI wiring is one; panel.rs is one; shell.qml is one. Red tests first. No new Rust
  dependencies. Tests build their ledger in a temp dir and never open the
  live state.db.
- Stop and ask: a JSON field or the panel command needing a Rust file
  beyond ledger_json.rs and panel.rs; a wiring
  change bigger than passing `json` through; the JSON blocked count
  disagreeing with the banner after 321; quickshell missing from the Arch
  extra repo, or a QML type the installed version does not have; anything
  in the panel that writes; zero/panels/ failing the tree map, the crate
  checks, deadwood or the docs-path check; installing anything.
- Done means: G1 to G7 ticked with evidence, and Christian has seen the
  panel on his laptop.
- Must not change: the text output of `core intent list` and `show` without
  `--json`; closed.tsv and every intent file; the banner; live state.db;
  ~/.config and Hyprland config (Christian's step); Cargo.toml members;
  root.

## Success Criteria
- [x] G1 list --json is a stable schema (intent/ledger_json.rs). Red tests
      first in a temp ledger: the top-level keys and their types; counts
      equal the rows; a blocked row; a format 1 file with no Vision gives
      "vision": ""; an unreadable file still leaves stdout valid JSON with
      the warning on stderr; git unknown gives tree nulls.
      ask: list --json has the schema above and the tests pass
      ruling:
      evidence: cargo test -p core 375 passed, including the list --json temp-ledger tests: schema 1 keys and types, counts equal the rows, a blocked row, empty vision on format 1, unreadable file keeps stdout JSON with the warning on stderr, git unknown gives tree nulls; laptop list --json after ship: listed 37, total 284, done 247, in_progress 4, planned 33, blocked 1, cancelled 5
- [x] G2 show --json for an open id, a closed id and an unknown id. The open
      id has gates_list; the closed id has "closed": true and the git show
      command; the unknown id prints nothing and exits 1.
      ask: show --json answers all three cases
      ruling:
      evidence: laptop core intent show 330 --json has gates_list and status planned; show 321 --json has closed true, status complete and a git_show command; show 999 --json prints nothing and exits 1
- [x] G3 the panel renders from the fixture. zero/panels/ledger/shell.qml
      reads fixture.json when LEDGER_FIXTURE is set, and a Rust test checks
      that fixture.json has the same keys list --json produces. The only
      Process in the QML is the core or fixture read.
      ask: the panel draws the fixture and the fixture matches the schema
      ruling:
      evidence: LEDGER_FIXTURE=/home/christian/0-core/zero/panels/ledger/fixture.json core intent panel drew Total 3, Done 5, Pending 2, blocked row 005 and cancelled 006; cargo test fixture_json_has_the_keys_list_json_produces; the only Process in shell.qml is the core or fixture read
- [x] G4 the look matches the screenshots. Dark panel, live badge, the
      total, done and pending cards (pending is planned only), rows with
      number, title, one-sentence vision, status word and gates (5/7), then
      the last five closed with cancelled in grey, and a footer with the last
      close and clean and pushed. A floating window. Colors as in The
      Solution.
      ask: Christian compares a screenshot with ref-ledger.jpg and
      ref-register.jpg
      ruling:
      evidence: laptop screenshot compared with the ledger look: dark panel, live badge, total done and pending cards, open rows with number title vision status and gates, last five closed with cancelled in grey, footer with last close and clean and pushed, a floating window with no Hyprland border, rounding 18 and the card edge to edge (rule in ~/.config/hypr/looknfeel.lua, margin 0 in 2e58b7ea)
- [x] G5 the panel matches the ledger on the laptop. After Christian
      installs quickshell, the panel's planned plus in progress equals
      `core intent list` Total, and done, planned and blocked equal the
      banner.
      ask: the panel and the terminal give the same numbers
      ruling:
      evidence: live panel Done 247 and Pending 33 match the banner; planned 33 plus in progress 4 equals core intent list Total 37; one BLOCKED row, 167, matches the banner 1 blocked; the Total card showed 37 from counts.listed, matching the text Total as ruled
- [x] G6 core intent panel opens it (intent/panel.rs). Red tests first with
      a fake PATH: quickshell missing gives a green safe stop that names
      `sudo pacman -S quickshell` and starts nothing; quickshell present
      starts it detached with -p and the panels path from zero_core::paths.
      ask: core intent panel opens the window, and refuses safely without
      quickshell
      ruling:
      evidence: cargo test -p core: quickshell missing is a green safe stop naming sudo pacman -S quickshell and starts nothing; quickshell present starts detached with -p and zero/panels/ledger from paths.rs; laptop core intent panel opened the window
- [x] G7 doors and d green on the laptop, with zero/panels/ in docs/TREE.md.
      ask: doors and d are green
      ruling:
      evidence: doors 4/4 before the push; on the branch doors 4/4 and d 96%; after the merge the nsh banner shows health 96% with the one uncommitted file, 330; docs/TREE.md has the zero/panels/ row

## Job order
- Wave 1, side by side: the G1 red tests, fixture.json, the TREE.md row.
- Wave 2: ledger_json.rs until G1 is green; the shell.qml draft against the
  fixture.
- Wave 3, side by side: the CLI wiring and the G2 tests; panel.rs and the
  G6 tests.
- Wave 4, on the laptop: Christian installs quickshell, then G4, G5, G6
  and G7.

## Open for Christian
- Install is yours: `sudo pacman -S quickshell` (Arch extra, 0.3.2 on
  2026-11-19). Check first with `pacman -Q quickshell`; a 2026-09-02 note in
  notify/mod.rs says Quickshell owned the notification bus then.
- Should quickshell go in zero/registry/packages.txt.
- While testing: grey for cancelled, the floating window, `core intent
  panel`, and the write capability `intent list` asks for. Change any of
  them then if a better way shows up.

## Versions
- core minor (list and show --json, intent panel)
