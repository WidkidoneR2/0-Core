---
id: 083
date: 2026-06-23
type: future
title: "registry alias-hygiene: fix collapsed [[alias]] blocks in aliases.toml"
tags: [registry, aliases, toml, hygiene, cleanup]
version: TBD
---
## Why
Found during INT-025: registry/aliases.toml has 4 collapsed [[alias]] blocks (41 headers
vs 45 command= lines). Each merges two entries under one header with duplicate
aliases/category keys, missing the second entry's [[alias]] header + command. Strict
tomllib rejects the file ("cannot overwrite a value"); the Rust parser tolerates it, so
tooling works -- but the file is malformed and should be repaired for correctness + so
future strict-TOML tooling (and INT-061 canonical structure) is not blocked.
## What (from the original finding)
 -- lines ~227-232:
  [[alias]]
  command = "faelight-niri-bridge"
  aliases = ["niri-bridge", "nb"]
  category = "compositor"
  aliases = ["forecast-plain", "ffp"]   <- should be its own [[alias]] + command="..."
  category = "intelligence"

Impact: strict tomllib rejects the file ("cannot overwrite a value"); the Rust registry
parser tolerates it, so tooling currently works. Pre-existing (in pre-edit backups).

Fix (own intent): locate all 4 collapsed blocks, restore each second entry's [[alias]]
header + its command= line, verify `python3 -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" registry/aliases.toml` parses clean, and that header count == command count.

## Gates
- [x] locate all 4 collapsed [[alias]] blocks (header_count != command_count)
- [x] restore each second entry's [[alias]] header + command= line
- [x] `python3 -c "import tomllib; tomllib.load(open('registry/aliases.toml','rb'))"` parses clean
- [x] [[alias]] header count == command= line count; tooling (core registry list) still rc=0
## The Rule
"A registry that cannot be parsed is not a registry -- it is a hope." 🌲

## Gate Check
✅ DEMONSTRATED (2026-06-23) -- registry/aliases.toml now strict-TOML valid.
Found TWO bugs (charter assumed "4 collapsed blocks" -- reality was different):
- Bug 1 (the parse blocker): the faelight-niri-bridge [[alias]] block had orphaned
  debris (aliases=["forecast-plain","ffp"] + category="intelligence") with no header
  and no command -- a real collapse that lost forecast-plain/ffp's command. tomllib
  rejected at line 216 ("cannot overwrite a value"). Removed the orphaned debris.
- Bug 2 (stale cruft): 4 entries (lla/llt/cw/cwc) used single-bracket [x] table syntax
  with a wrong schema (command/description/tags) and STALE commands -- toml [cw] said
  "cargo watch" but the real cw alias (config.fsh) is "core why summary". Not parsed by
  the registry, conflicting with reality. Deleted all 4 (16 lines).
VERIFIED: tomllib parses clean; core registry list rc=0 (28 active); [[alias]] headers
== command= lines (38==38); real cw/cwc aliases in config.fsh untouched and working.
