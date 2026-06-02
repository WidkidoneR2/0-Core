---
id: 274
title: "faelight-pick -- Fuzzy Selection Everywhere"
status: complete
date: 2026-05-06
tags: [skim, fuzzy, picker, fzf, rust, fsh, navigation, intent, history, files, vocabulary]
---
The forest has fzf. It works. It is written in Go.
That is the problem.
faelight-pick replaces fzf with skim -- pure Rust.
Not just a tool swap. An integration.
fzf is something you call from the shell.
skim is something you embed in Rust.
The difference: fzf sits outside the forest.
skim lives inside it.
---
THE PHILOSOPHY
fsh speaks human first.
Typing `intent show 239` is not human.
Typing `pick intent` and seeing a live fuzzy list is.
The forest knows 268 intents. 50 tools. 2477 commits.
Right now you navigate them by number or by memory.
faelight-pick means you navigate them by feel.
Type a few letters. The forest finds it.
This is not convenience for its own sake.
This is the forest becoming more fluent in how humans actually think.
---
WHAT CHANGES
fzf OUT:
  paru -R fzf
  One Go binary removed.
  The forest moves closer to 99% Rust.
skim IN:
  paru -S skim
  CLI tool: sk -- drop-in fzf replacement for any scripts
  Rust crate: skim -- embedded directly in fsh and faelight-fm
  One Rust binary. One crate. Zero Go.
---
FSH VOCABULARY: pick
pick intent              -- fuzzy search all intents, select to show
pick intent --active     -- only in-progress intents
pick file                -- fuzzy file search from current directory
pick file --core         -- fuzzy file search across 0-core
pick commit              -- fuzzy git log, select to show diff
pick tool                -- fuzzy tool list, select to show details
pick history             -- fuzzy command history, select to re-run
"pick" is the word.
You are picking something from a list.
Human. Direct. Obvious.
---
THREE INTEGRATION POINTS (priority order)
PRIORITY 1 -- Intent picker
  pick intent
  Shows all intents with fuzzy search.
  Preview panel: intent description on the right.
  Select: opens intent show for that intent.
  This replaces memorizing intent numbers.
  Graydon types: pick intent bar
  Instantly sees: INT-239 faelight-bar v2
PRIORITY 2 -- History search
  pick history
  Fuzzy search over all shell history in state.db.
  Select: re-runs that command.
  Replaces: linear arrow-up navigation.
  Replaces: search command (now interactive instead of list).
  This is the daily friction point that disappears.
PRIORITY 3 -- File navigation
  pick file
  Fuzzy search files from current directory using ripgrep.
  Select: opens file in $EDITOR or prints path.
  Replaces: fzf-based file picking.
  Integrates: rg for the search, skim for the selection.
---
FAELIGHT-FM INTEGRATION
faelight-fm already has file navigation.
skim embedded in faelight-fm means:
  Press / in fm -- opens skim picker for current directory
  Type letters -- live fuzzy filter
  Enter -- jumps to that file in fm
  This is how every modern file manager works.
  The forest will do it in pure Rust.
---
TECHNICAL ARCHITECTURE
Phase 1 -- Replace fzf with skim CLI:
  paru -R fzf
  paru -S skim
  Update any scripts calling fzf to call sk instead
  Verify: sk --version shows skim
Phase 2 -- Add skim crate to fsh:
  faelight-shell/Cargo.toml: skim = "0.10"
  Implement: pick_from_list(items: Vec<String>) -> Option<String>
  Core helper function used by all pick subcommands
Phase 3 -- pick intent:
  Query all intents from intents/ directories
  Format: "INT-NNN  status  title"
  Pass to skim with preview command
  Return selected intent ID
  Call intent show on selection
Phase 4 -- pick history:
  Query shell_history from state.db
  Format: "timestamp  command"
  Pass to skim
  Return selected command
  Option to re-run or just print
Phase 5 -- pick file:
  Use rg --files for file list (already in forest)
  Pass to skim
  Return selected path
  Open in $EDITOR or print
Phase 6 -- faelight-fm integration:
  Add skim crate to faelight-fm/Cargo.toml
  / keybind opens skim picker in current directory
  Selection jumps fm to that file
---
GATES
Phase 1 -- fzf removed, skim installed:
[ ] paru -R fzf -- confirmed removed
[ ] paru -S skim -- confirmed installed
[ ] sk --version shows skim version
[ ] No scripts in forest call fzf (verify with grep)
[ ] Any fzf references updated to sk
Phase 2 -- skim crate in fsh:
[ ] skim crate added to faelight-shell/Cargo.toml
[ ] pick_from_list() helper function implemented
[ ] cargo build clean -- zero warnings
[ ] pick with no args shows usage
Phase 3 -- pick intent works:
[ ] pick intent shows all 268 intents in skim
[ ] Fuzzy filter works -- type "bar" shows INT-239
[ ] Selection opens intent show correctly
[ ] pick intent --active filters to in-progress only
[ ] Preview panel shows intent description
Phase 4 -- pick history works:
[ ] pick history shows full command history
[ ] Fuzzy filter finds commands by keyword
[ ] Selection re-runs or prints command
[ ] Replaces linear arrow-up as primary history nav
Phase 5 -- pick file works:
[ ] pick file shows all files from current directory
[ ] pick file --core searches across 0-core
[ ] Selection opens in $EDITOR or prints path
[ ] Fast: results appear in under 200ms
Phase 6 -- faelight-fm integration:
[ ] / keybind opens skim in fm
[ ] Selection jumps to that file in fm
[ ] Escape cancels cleanly
Final Validation:
[ ] fzf is gone -- zero Go fuzzy finders in the forest
[ ] pick intent replaces memorizing intent numbers
[ ] pick history replaces linear arrow-up navigation
[ ] The forest navigates itself in pure Rust
[ ] Graydon types pick intent and nods
"You do not search the forest.
You ask the forest.
It shows you what you are looking for.
In Rust.
Always in Rust." 🌲
