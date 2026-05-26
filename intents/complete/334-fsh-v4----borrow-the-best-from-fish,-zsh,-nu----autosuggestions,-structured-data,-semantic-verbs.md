---
id: 334
title: "fsh v4 -- borrow the best from Fish, Zsh, Nu -- autosuggestions, structured data, semantic verbs"
status: complete
date: 2026-05-25
tags: [fsh, shell, fish, zsh, nu, autosuggestions, structured-data, semantic]
---

## Why This Intent Exists

fsh v3 is the daily driver. Everyone who has seen it wants it as a default shell.
That feedback is the most important signal from the demo reviews.
The score gap between fsh and faelight-term is evidence that the shell is
already the strongest part of the forest.

v4 is not a rewrite. It is a selective borrowing from three of the best shells
in existence, applied with forest principles.

Fish has 22,000+ commits. Zsh has decades of production use. Nu invented
structured data as a first-class shell concept. We study all three and take
only what earns its place in the forest.

## What We Borrow

### From Fish
- **Autosuggestions**: Grey ghost text showing the most likely command completion
  based on shell history. No configuration needed. Just works.
  Implementation: after every keypress, query shell_history for the most recent
  matching prefix. Display in dimmed color. Tab or right arrow accepts.
- **Syntax highlighting as you type**: Commands that exist highlight green.
  Unknown commands highlight red. Arguments highlight based on type.
- **Web-based config UI** -- skip this. Forest does config via state.db and fsh config files.

### From Zsh
- **Powerful glob patterns**: `**/*.rs` recursive glob, `^pattern` negation
- **zmv-style mass rename**: `fsh rename "*.txt" "*.md"` -- rename by pattern
- **History substring search**: Ctrl+R with live filtering as you type
- **Completion system**: Tab completion that understands command context

### From Nu
- **Structured data pipeline**: Commands that return tables, not text.
  `fsh ps | where cpu > 10 | sort-by cpu`
  `fsh ls | where size > 1mb | sort-by modified`
- **Typed values**: Numbers stay numbers through pipes, not text
- **Built-in data commands**: `where`, `sort-by`, `select`, `get`, `each`

## Forest Principles That Override

- No configuration bloat. Every feature works with zero config.
- Forest vocabulary first. `fsh list processes` not `ps aux`.
- Friday watches every command. Autosuggestions informed by Friday patterns.
- state.db is the memory. History is queryable, not just sequential.

## Gates

✅ Autosuggestions implemented -- ghost text from shell_history after every keypress 2026-05-26
✅ Autosuggestions respect Friday's known patterns -- confidence>=0.7 fallback from friday_patterns 2026-05-26
✅ Syntax highlighting as you type -- green for known commands, red for unknown 2026-05-26
✅ Ctrl+R history search with live filtering -- beautiful TUI already present 2026-05-26
✅ Recursive glob ** works in fsh 2026-05-26
✅ Structured data pipeline: ps returns queryable table, full usernames 2026-05-26
✅ where/sort-by/select work on structured output -- ps | where user = christian | first 5 2026-05-26
✅ fsh rename pattern-based mass rename -- *.txt -> *.md demonstrated 2026-05-26
✅ All features work with zero configuration -- no config files required 2026-05-26
✅ Demonstrated: all v4 features active in daily use -- gates 1-9,11 met in single session 2026-05-26
✅ Friday integration: autosuggestions use friday_patterns confidence>=0.7 as fallback 2026-05-26
✅ fsh v4 is daily driver -- 82/82 regression tests pass, zsh/nu removal planned pre-NixOS migration 2026-05-26
