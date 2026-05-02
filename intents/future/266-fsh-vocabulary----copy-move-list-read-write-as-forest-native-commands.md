---
id: 266
date: 2026-05-02
type: feature
title: \"fsh vocabulary -- copy move list read write as forest-native commands\"
status: planned
tags: [feature, rust, faelight]
version: TBD
---

## Vision
INT-261 established the principle and shipped the first two words:
delete and find. This intent ships the next five: copy, move, list,
read, write.
These five words complete the core file system vocabulary. Together
with delete and find, the forest can express every common file operation
in plain English:
  copy file.txt to backup/file.txt
  move old/config.rs to archive/config.rs
  list files in rust-tools
  read main.rs
  write "hello forest" to notes.txt
  delete temp.log
  find "*.rs" @rust
A human who has never used Linux reads this and understands it. That
is the test. That is the goal.
Each word follows the same pattern established by delete:
- Forest-aware behavior that earns the rename (not just a synonym)
- Safe-by-default (no silent destruction)
- Friday-observable (emits events to state.db)
- UNIX command still works unchanged (cp, mv, ls, cat, echo all work)
1. Human words first. The canonical name is the English word.
2. UNIX is fallback, never broken. cp, mv, ls, cat still work.
3. Short aliases for fingers. copy -> cp alias preserved.
4. Forest-aware behavior earns the rename.
5. No vocabulary by completionism. Each word must earn its place.
6. Vocabulary grows through daily-driving.
7. Friday participates in vocabulary decisions.
Forest-aware behaviors:
- Refuses to overwrite by default. copy a.txt to b.txt fails if b.txt
  exists, with a clear message: "b.txt already exists. Use copy a.txt
  to b.txt overwrite to replace it."
- overwrite modifier bypasses the protection.
- Source-tree warning: if destination is in rust-tools/, intents/,
  scripts/, or docs/, prompts for confirmation.
- Emits file_copied event to Friday signal stream.
- Alias: cp still works unchanged.
Forest-aware behaviors:
- Same overwrite protection as copy.
- Source-tree warning on both source and destination.
- If source is a file referenced in recent commits or open intents,
  Friday surfaces a warning: "this file was referenced in INT-XXX."
- Emits file_moved event with source, destination, intent context.
- Alias: mv still works unchanged.
Forest-aware behaviors:
- Default: list files in current directory.
- Output is Value::Table (integrates with INT-265 pipelines).
- Each row: name, size, type, modified, git-status badge.
- Git tracking badge: tracked, untracked, modified, ignored.
- Alias: ls still works unchanged.
- list files produces different output than list directories.
- list files in @rust searches rust-tools/ (forest path shortcuts).
Forest-aware behaviors:
- Syntax-aware rendering for known file types (.rs, .toml, .md, .json).
- Shows file metadata header: size, modified, git status.
- For large files (>1000 lines), shows first 50 with prompt to continue.
- Emits file_read event (Friday learns which files are read often).
- Alias: cat still works unchanged.
Forest-aware behaviors:
- Refuses to overwrite by default (same as copy).
- overwrite modifier bypasses.
- Source-tree warning for protected paths.
- For Rust files: validates UTF-8, warns if content looks like a
  binary write (protects against the corruption bugs documented in
  COMMAND-GUIDE).
- append modifier: write "line" to file append adds to end.
- Emits file_written event.
- Alias: echo and > still work unchanged.
The vocabulary safety model is consistent across all five words:
1. Destructive actions (overwrite, move) require explicit confirmation
   or the overwrite modifier.
2. Source-tree paths (rust-tools/, intents/, scripts/, docs/, engine/)
   always prompt for confirmation regardless of modifier.
3. Trash: delete sends to forest-trash. copy, move, write do NOT use
   trash -- they are not deletion operations.
4. Every action emits an event to state.db. Nothing happens silently.
These five words, combined with delete and find, establish the
verb-object grammar pattern from the broader vocabulary vision:
  <verb> <object> [target] [modifiers]
  copy   file.txt    to backup/     overwrite
  move   config.rs   to archive/
  list   files       in @rust
  read   main.rs
  write  "content"   to notes.txt   append
  delete temp.log
  find   "pattern"   @intents
This grammar is learnable in 5 minutes. It is predictable. It composes
with the pipeline syntax (INT-265). It is Friday-observable. It works
the same way every time.
Each word follows the delete pattern exactly:
- New match arm in fsh commands/mod.rs
- Args parsing for source, target, modifiers
- Safety checks (overwrite protection, source-tree warning)
- Action execution
- Event emission to state.db
Copy, move, write need the overwrite protection pattern.
List needs Value::Table output (integrates with INT-265).
Read needs syntax-aware rendering (reuses fsh existing colorize_line).
Five words, five match arms, same structure each time. Each one takes
about 30 minutes to build and test correctly.
- INT-261 (vocabulary principle and delete/find as template)
- INT-265 (pipeline integration for list output)
- ForestDb event emission (already in all builtins)
- fsh commands/mod.rs builtin infrastructure
- [ ] copy file to destination works, refuses overwrite by default
- [ ] copy with overwrite modifier bypasses protection
- [ ] copy emits file_copied event to state.db
- [ ] move file to destination works, refuses overwrite by default
- [ ] move warns if source referenced in recent intents
- [ ] move emits file_moved event to state.db
- [ ] list files produces Value::Table output
- [ ] list shows git tracking badge per file
- [ ] list supports @rust @intents @scripts shortcuts
- [ ] read file renders with syntax highlighting
- [ ] read shows file metadata header
- [ ] write content to file refuses overwrite by default
- [ ] write with append modifier adds to end of file
- [ ] write emits file_written event
- [ ] All five words warn on source-tree paths
- [ ] cp, mv, ls, cat, echo all continue working unchanged
- [ ] All five integrate with INT-265 pipeline syntax
- [ ] All five appear in INT-260 cheatsheet TUI
- Five words: copy, move, list, read, write
- Safety model: overwrite protection, source-tree warnings
- Value::Table output for list (pipeline integration)
- Event emission for all five
- Syntax rendering for read
- Forest path shortcuts for list
- Recursive operations (copy directory recursively is INT-267 if needed)
- Network operations (copy to remote, download)
- Permission management (change permissions is a separate vocabulary word)
- Archive operations (archive, extract are separate vocabulary words)
- set permissions (chmod) as forest vocabulary -- needs its own intent
- create directory -- needs its own intent with template awareness
- rename as distinct from move -- evaluate after move is daily-driven
- open (launch in application) -- needs app registry integration
⬜ Not started
---
*"A child learns copy before cp.
They learn move before mv.
They learn list before ls.
The human word comes first because it is the natural name.
UNIX is the historical abbreviation.
The forest can do better.
The forest will do better." 🌲*
