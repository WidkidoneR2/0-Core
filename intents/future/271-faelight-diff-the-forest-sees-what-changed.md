---
id: 271
title: "faelight-diff -- The Forest Sees What Changed"
status: in-progress
date: 2026-05-05
tags: [faelight-diff, diff, compare, ratatui, tui, rust, friday, intelligence, files, directories, git]
---
The forest has difftastic. It sees line changes.
faelight-diff sees meaning.
Not just what changed -- but what it means.
Not just two files -- but two worlds, side by side.
The diff tool the forest deserves.
Built in Rust. Built with ratatui. Built for the presentation.
---
VOCABULARY
fsh command: compare
  compare file1 file2          -- side by side file diff
  compare dir1 dir2            -- directory tree diff
  compare --git                -- diff against last commit
  compare --git HEAD~3         -- diff against any commit
  compare --intent             -- diff since cistart of active intent
  compare --staged             -- what is about to be committed
Keybind: Ctrl+Alt+C -- opens faelight-diff in current context
If cursor is on a file: opens that file vs git HEAD automatically.
If in a git repo: opens git diff view automatically.
The forest knows where you are. compare just works.
---
THREE MODES
MODE 1 -- FILE COMPARE
Two files, side by side. Forest colors. Syntax aware.
Left panel: file A with line numbers
Right panel: file B with line numbers
Changed lines: amber highlight
Added lines: green highlight
Removed lines: red highlight (dimmed)
Unchanged context: dim, readable
Navigate: j/k line by line, J/K section by section
Jump: ] next change, [ previous change
Open: o -- open file at current line in $EDITOR
Copy: y -- copy changed block to clipboard
Friday: f -- show Friday context for this change
MODE 2 -- DIRECTORY COMPARE
Two directory trees, side by side.
Left: directory A file tree
Right: directory B file tree
Color coding:
  Green: file exists only in B (new)
  Red: file exists only in A (removed)
  Amber: file exists in both but differs
  Dim: file identical
Navigate: j/k through files, Enter to open file diff
Filter: / to search filenames, Tab to switch panels
Summary line: X new, Y removed, Z modified
MODE 3 -- GIT MODE
The living history of the forest.
compare --git opens the full git diff view:
  Left panel: commit browser (log --oneline)
  Right panel: diff for selected commit
Navigate commits: j/k
Expand: Enter -- full diff for commit
Branch: b -- compare two branches
Blame: B -- show git blame for current file
Friday: f -- what was Friday doing during this commit
---
FRIDAY INTEGRATION
Every diff has context. Friday knows it.
When you open a file diff:
  Friday checks: was this file changed during an active intent?
  Friday checks: did this change cause a health event?
  Friday checks: is this a pattern it has seen before?
Friday panel (press f):
  "This file was last modified during INT-232 (faelight-term v11).
   The change on line 47 fixed the borrow conflict in render().
   Similar changes in 3 previous intents -- always in the render loop.
   Confidence: pattern is structural, not accidental."
Friday contradiction detection:
  If you are comparing two approaches and Friday sees a conflict:
  "Warning: approach B contradicts the decision made in INT-186.
   DEC-003 chose subprocess over in-process for Wayland clipboard.
   This diff reverses that decision. Intended?"
---
FOREST VOCABULARY THROUGHOUT
No Unix noise. No --porcelain flags. No cryptic output.
The tool speaks forest.
compare tells you:
  "3 files changed. 2 new. 1 removed. forest is growing."
  not: "3 files changed, 47 insertions(+), 12 deletions(-)"
compare --git tells you:
  "Last commit: INT-239 Gate 4. 5 files. All intentional changes."
  not: "commit b4e25047 Author: christian Date: Sun May 3"
Error messages are human:
  "Cannot compare: file1 does not exist."
  not: "diff: file1: No such file or directory"
---
VISUAL DESIGN
Forest colors throughout:
  Background:
  Changed: amber
  Added: green
  Removed: red
  Unchanged: dim
  Active line: subtle highlight, not jarring
Layout:
  Header bar: shows what is being compared + mode
  Two main panels: left and right, equal width
  Status bar: navigation hints + Friday indicator
  Friday panel: slides in from right (press f, dismiss with Escape)
Nerd font icons throughout:
  Modified file:  (pencil)
  New file:  (plus)
  Removed file:  (minus)
  Directory:  (folder)
  Git commit:  (branch)
  Friday signal:  (tree)
---
TECHNICAL ARCHITECTURE
Language: Rust (100% -- no Python, no shell glue)
TUI: ratatui (proven in faelight-fm)
Diff engine: similar-rs or own implementation
  (difftastic is CLI only -- faelight-diff owns its own diff logic)
Git: git2-rs (libgit2 bindings -- pure Rust)
Syntax: no dependency -- forest color palette is enough
State: no persistent state -- stateless tool, reads on open
Binary: faelight-diff
Deployed to: scripts/faelight-diff
fsh vocabulary: compare (INT-261 pattern -- human first)
Keybind: Ctrl+Alt+C in Niri config
Integration points:
  faelight-term: open compare from terminal keybind
  faelight-bar: Friday panel shows active diff context
  fsh: compare command with full vocabulary
  state.db: Friday reads diff history for pattern detection
---
GATES
Phase 1 -- Foundation:
[ ] Cargo.toml: ratatui, crossterm, git2 dependencies
[ ] Basic two-panel layout renders in terminal
[ ] File diff logic: reads two files, computes line diff
[ ] Forest color palette applied to diff output
[ ] j/k navigation between changed lines works
[ ] ] and [ jump between change sections
Phase 2 -- File Mode complete:
[ ] Side by side rendering correct at all terminal widths
[ ] Line numbers accurate on both sides
[ ] Added/removed/changed lines colored correctly
[ ] Unchanged context dimmed correctly (3 lines above/below)
[ ] o opens file at current line in $EDITOR
[ ] Header shows filenames and change summary
Phase 3 -- Directory Mode:
[ ] Directory tree renders in left panel
[ ] File status (new/removed/modified/identical) colored correctly
[ ] Enter on a file opens file diff for that file
[ ] / search filters filenames live
[ ] Summary line shows X new Y removed Z modified
Phase 4 -- Git Mode:
[ ] compare --git opens git diff against HEAD
[ ] compare --git REF diffs against any ref
[ ] Commit browser shows log --oneline with forest formatting
[ ] Selecting commit shows its diff in right panel
[ ] compare --staged shows what is about to be committed
[ ] compare --intent diffs since cistart timestamp
Phase 5 -- Friday Integration:
[ ] Press f opens Friday panel for current diff
[ ] Friday reads intent history for changed files
[ ] Friday detects if change reverses a previous decision
[ ] Friday panel dismisses cleanly with Escape
[ ] Friday indicator in status bar when signal available
Phase 6 -- fsh vocabulary + keybind:
[ ] compare command registered in fsh vocabulary
[ ] compare with no args opens git diff if in repo
[ ] compare file1 file2 opens file diff
[ ] compare dir1 dir2 opens directory diff
[ ] Ctrl+Alt+C keybind added to Niri config
[ ] Keybind opens compare in context of focused window
Final Validation:
[ ] compare file1 file2 opens and renders correctly
[ ] compare --git shows last commit diff
[ ] Directory diff correctly identifies new/removed/modified
[ ] Friday panel surfaces a real insight about a real change
[ ] Christian says: "I understand this diff better than with any other tool"
[ ] Graydon opens it and nods
"The forest does not just record what changed.
The forest understands why." 🌲
