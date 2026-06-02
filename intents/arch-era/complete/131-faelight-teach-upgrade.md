---
id: 131
date: 2026-03-16
type: future
title: "faelight-teach upgrade — Interactive faelight-shell Tutorial"
status: complete
tags: [teach, shell, tutorial, learning, interactive, v10.9]
version: 10.9.0
priority: medium
---

## Vision

faelight-teach currently teaches core commands.
The upgrade adds an interactive faelight-shell tutorial module —
real learning by doing, not just reading.

Lessons run inside faelight-shell itself.
The student types real commands and sees real forest data.

## Commands
```bash
teach shell                 # start faelight-shell tutorial
teach shell list            # list all available lessons
teach shell pipeline        # specific lesson: pipelines
teach shell aliases         # specific lesson: aliases
teach shell data            # specific lesson: structured data
teach shell git             # specific lesson: git commands
teach shell watch           # specific lesson: watch mode
teach shell progress        # show completion progress
```

## Interactive Lesson Format

Each lesson waits for the student to type the correct command:
```
📚 Lesson 3 — The Pipeline
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
In faelight-shell, data flows through pipes.
Everything is structured — tables, rows, values.

Try this:
  tt | where score < 75 | sort score

The | operator passes structured data
between commands.

🌲 forest> _
```

On correct input:
```
✅ Perfect! The pipeline filtered 12 tools below 75.
   Next: adding more filters with count

Press Enter to continue...
```

On wrong input:
```
  Not quite — try: tt | where score < 75 | sort score
  Hint: 'tt' is the tools table command
```

## Lesson Structure — TOML Files

Lessons are data, not code. Easy to add without recompiling:
```toml
# lessons/shell/03-pipeline.toml
id = 3
title = "The Pipeline"
description = "Learn how data flows between commands"

[[steps]]
instruction = "List all tools as a table"
expected = "tt"
hint = "tt stands for tools-table"
success = "The tools table has name, version, score, deployed columns"

[[steps]]
instruction = "Filter tools with score below 75"
expected = "tt | where score < 75"
hint = "Use: tt | where score < 75"
success = "The where operator filters rows by condition"

[[steps]]
instruction = "Sort by score ascending"
expected = "tt | where score < 75 | sort score"
hint = "Add | sort score to the pipeline"
success = "Perfect pipeline! filter → sort"
```

## Lesson Modules
```
lessons/shell/
  01-basics.toml        — health, events, version, help
  02-tables.toml        — tt, et, at, dt
  03-pipeline.toml      — where, select, sort, first, last
  04-aliases.toml       — alias, unalias, persistent queries
  05-git.toml           — gc, gf, git structured data
  06-watch.toml         — watch health, watch events
  07-histogram.toml     — histogram, domains
  08-advanced.toml      — multi-stage pipelines
```

## Progress Tracking

Progress stored in state.db:
```sql
CREATE TABLE teach_progress (
    module   TEXT,
    lesson   INTEGER,
    completed INTEGER,
    timestamp INTEGER
);
```
```bash
teach shell progress
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  faelight-shell tutorial
  Progress: 5/8 lessons complete
  ██████████░░░░░░  62%
  
  ✅ basics      ✅ tables    ✅ pipeline
  ✅ aliases     ✅ git       ⬜ watch
  ⬜ histogram   ⬜ advanced
```

## Success Criteria

- [ ] teach shell launches tutorial module
- [ ] 8 lesson files in TOML format
- [ ] Interactive command validation
- [ ] Hint system on wrong input
- [ ] Progress tracking in state.db
- [ ] teach shell progress command
- [ ] Lessons use real forest data

---
*"The forest teaches those who ask."* 🌲
