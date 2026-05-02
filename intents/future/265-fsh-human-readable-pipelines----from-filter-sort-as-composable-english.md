---
id: 265
date: 2026-05-02
type: arch
title: \"fsh human-readable pipelines -- from filter sort as composable English\"
status: planned
tags: [architecture, rust, design]
version: TBD
---

## Vision
Unix pipes are one of the great ideas in computing. The problem is not
the concept -- it is the syntax. This:
  cat file.txt | grep error | sort | uniq
requires a human to mentally parse four separate commands, understand
the implicit text stream connecting them, know that grep takes a pattern
not a keyword, and remember that uniq only deduplicates adjacent lines.
The forest vocabulary principle (INT-261) says: human words first, UNIX
as fallback. That principle applies to pipelines too.
This intent introduces human-readable pipeline syntax to fsh:
  from file.txt
  | filter contains "error"
  | sort
  | unique
Or more concisely on one line:
  from file.txt | filter contains "error" | sort | unique
Same power. No mental translation required. A 16-year-old reads it and
knows exactly what it does.
The deeper claim: when pipelines read like English, they become
composable in a way that UNIX pipes are not -- because you can reason
about them without decoding the syntax first. Friday can read them.
Documentation can include them without explanation. They compose with
the forest vocabulary commands:
  find "*.log" @rust | filter contains "error" | sort | take 10
That sentence is English. It is also a valid fsh pipeline.
Unix pipes operate on raw text streams. Every command in a pipeline
must know and handle the text format of the previous command. This is
why awk and sed exist -- to reshape text between commands.
The forest pipeline is different. It operates on Value::Table -- the
structured data format fsh already uses internally. When a pipeline
stage produces Value::Table, the next stage receives structured rows,
not raw text. This means:
  list files | filter size > 1MB | sort by size descending
Works because list files produces rows with name, size, type columns.
filter size > 1MB operates on the size column directly. sort by size
descending sorts on that column. No awk. No cut. No column parsing.
This is the pipeline idea that changes what a shell can do. Not just
prettier syntax -- a fundamentally more composable data model.
  from <file>           -- read file as lines or structured data
  list files [in path]  -- directory listing as rows
  find <pattern>        -- file search results as rows (INT-266)
  db <table>            -- state.db query results as rows (INT-263)
  filter contains "text"       -- rows where any field contains text
  filter <column> > <value>    -- rows where column matches condition
  filter <column> = <value>    -- exact match
  filter <column> != <value>   -- exclusion
  sort                         -- sort by first column
  sort by <column>             -- sort by named column
  sort by <column> descending  -- reverse sort
  unique                       -- deduplicate rows
  take <n>                     -- first N rows
  skip <n>                     -- skip first N rows
  select <column> [column...]  -- keep only named columns
  count                        -- print row count
  show                         -- formatted table output (default)
  as json                      -- JSON output
  to file <path>               -- write to file
Finding recent errors:
  from /var/log/syslog | filter contains "error" | take 20
Large files in project:
  list files in rust-tools | filter size > 100KB | sort by size descending
Recent git commits:
  db events --domain git | filter action = commit | take 10
Searching source code:
  find "*.rs" @rust | filter contains "TODO" | sort
Pipeline chaining with forest vocabulary:
  find "main.rs" @rust | filter contains "fn " | sort | take 5
fsh already handles pipes via the sh fallback. The new pipeline syntax
needs to be detected before the sh fallback fires. Detection rule:
if the line contains | and the first token is a known pipeline source
(from, list, find, db), route to the forest pipeline evaluator.
A pipeline evaluator in fsh that:
1. Evaluates the source to get initial Value::Table
2. Applies each stage in sequence
3. Renders the final output via the sink (default: show)
Ensure all forest vocabulary commands (find, list, db, delete) produce
and consume Value::Table consistently. This is the data contract that
makes pipelines composable.
Pipeline patterns recorded in state.db so Friday can:
- Surface "you run this pipeline often -- want to alias it?"
- Detect when a pipeline produces 0 rows (possible error)
- Suggest pipeline completions from history
- Value::Table already in fsh (proven in fsearch, query, db builtins)
- INT-263 db builtin (pipeline source)
- INT-266 vocabulary words (find, list as pipeline sources)
- fsh command parser (extends existing pipe detection)
- [ ] from file.txt | filter contains "x" | sort works end-to-end
- [ ] list files | filter size > 1MB | sort by size descending works
- [ ] db events | filter action = commit | take 10 works
- [ ] find "*.rs" @rust | filter contains "TODO" works
- [ ] Pipeline stages chain correctly via Value::Table
- [ ] UNIX pipes still work unchanged (fsh fallback unaffected)
- [ ] take N and skip N work correctly
- [ ] sort by column works on structured data
- [ ] count produces correct row count
- [ ] as json produces valid JSON
- [ ] No regression in existing fsh pipe handling
- Parser extension to detect forest pipeline syntax
- Pipeline evaluator (source → filter → transform → sink)
- Value::Table as the pipeline data contract
- Sources: from, list, find, db
- Filters: contains, column comparisons
- Transformers: sort, unique, take, skip, select
- Sinks: show, count, as json, to file
- Replacing UNIX pipes (they still work)
- Parallel pipeline execution (sequential only in v1)
- Custom pipeline stages from user scripts
- Cross-process pipelines
- Named pipelines / pipeline aliases (save pipeline as name)
- Pipeline debugging (step-through evaluation)
- Type inference (column types inferred from data)
⬜ Not started
---
*"Unix pipes are one of the great ideas in computing.
The syntax is not.
from file.txt | filter contains error | sort | unique
is the same idea.
Written for humans.
The concept survives. The syntax improves." 🌲*
