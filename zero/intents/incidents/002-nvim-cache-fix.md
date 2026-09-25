---
id: 002
date: 2024-12-19
type: incidents
title: "Neovim Cache Corruption"
status: resolved
severity: low
duration: "15 minutes"
tags: [neovim, treesitter, cache]
---

## Summary

Treesitter error when pressing `:` in markdown files. Neovim became unusable for markdown editing.

## Root Cause

Corrupted Neovim cache files (likely after plugin updates).

## Resolution

```bash
rm -rf ~/.local/share/nvim
rm -rf ~/.cache/nvim
```

Restarted Neovim. Plugins reinstalled automatically.

## Lesson

When Neovim behaves strangely after updates, clear caches first before debugging.

---

_Part of the 0-Core Intent Ledger_ 🌲
