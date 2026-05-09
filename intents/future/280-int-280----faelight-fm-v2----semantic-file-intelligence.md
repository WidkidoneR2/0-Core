---
id: 280
title: "faelight-fm v2 -- Semantic File Intelligence"
status: planned
date: 2026-05-07
tags: [faelight-fm, file-manager, semantic, intelligence, friday, evolution]
---
faelight-fm already surpasses Yazi in some ways.
It knows the forest. It speaks Rust. It integrates with Friday.
INT-280 does not replace faelight-fm.
It evolves it -- one capability at a time.
If evolution fails to deliver what the forest needs,
INT-281 (The Forest Explorer) begins.
But we evolve first.
---
WHAT faelight-fm ALREADY HAS
Forest-aware file display.
Friday integration (press f for Friday context).
Forest colors throughout.
Integration with state.db.
This is already ahead of Yazi.
---
WHAT IT NEEDS
Semantic awareness:
  fm where language = rust and touched < 7d
  fm where intent = 243 (show files from INT-243 build)
  fm where changed and not committed
  fm risk (show files with recent changes near health events)
Speed improvements:
  Sub-50ms directory open
  Instant search with skim integration (already have sk)
  Cached metadata for repeated directories
Keyboard efficiency (ranger-level):
  hjkl navigation (already have j/k)
  Space to mark files
  Bulk operations on marked files
Friday integration depth:
  When you open a file, Friday shows:
    Last intent that touched it
    Health state when it was last changed
    Whether this file has caused incidents
---
SEMANTIC METADATA APPROACH
No heavy database. No new dependencies.
Use what the forest already has:
git log --follow [file] -- intent context from commit messages
state.db shell_history -- what commands were run on/near this file
state.db forest_events_v2 -- what events fired when file was changed
.git blame -- who changed what and when
File extension + path -- language and domain inference
Sidecar files (.faelight-meta) only if needed.
Git context first -- it is already there.
---
THE fm QUERY SYNTAX
fm where language = rust      -- all .rs files in current tree
fm where touched < 7d         -- files modified in last 7 days
fm where intent = 243         -- files touched during INT-243
fm where changed              -- uncommitted changes
fm where risk                 -- files changed near health events
fm recent                     -- most recently touched files
fm intent                     -- files grouped by intent that touched them
---
GATES
Phase 1 -- Speed:
[ ] Directory open under 50ms for 0-core (measured)
[ ] skim integration -- / opens fuzzy search in current directory
[ ] File preview renders without lag
Phase 2 -- Keyboard efficiency:
[ ] h moves up to parent directory
[ ] Space marks/unmarks files
[ ] Marked files: d delete, c copy path, Enter open all
Phase 3 -- Semantic queries:
[ ] fm where language = rust works
[ ] fm where touched < 7d works
[ ] fm where changed shows uncommitted files
[ ] fm recent shows 20 most recently touched files
Phase 4 -- Friday depth:
[ ] Opening a file shows: last intent, health at change time
[ ] fm risk shows files changed near health drops
[ ] fm intent groups files by the intent that created them
Final:
[ ] fm where intent = 243 returns files from that build session
[ ] faelight-fm feels faster than Yazi for forest navigation
[ ] Friday panel shows real context for every file opened
"The file manager that knows
not just where files are
but what they mean
and what the forest was doing
when they were last touched." 🌲


ARCHITECTURE NOTE (2026-05-08)
UI layer: ratatui -- TUI running inside faelight-term
Not a standalone Wayland surface.
faelight-fm v2 is a terminal application, not a compositor client.
ratatui gives: keyboard-driven UI, forest colors, file operations,
semantic search display -- all inside the terminal.
Libcosmic considered for future graphical layer only.
