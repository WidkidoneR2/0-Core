---
id: 060
date: 2026-01-15
type: future
title: "Universal Search Foundation - Omniscient Launcher"
status: planned
tags: [v7.5, search, launcher, rust]
version: 7.5.0
depends:
  - 059  # Monorepo for shared code
relates:
  - 027  # Launcher fuzzy search
  - 017  # Original launcher
---

## Vision
**One search to find everything.** Apps, files, commands, intents, git commits, docs, history.

**Philosophy:** "If it exists in your system, you should be able to find it instantly."

## The Problem

**Current State:**
- App launcher only finds apps
- File search is separate (find, fd)
- Command history is separate (Ctrl+R)
- Intent search is separate (intent ls)
- Git search is separate (git log)
- No unified interface

**User Experience:**
```bash
# Where is that thing I worked on yesterday?
fd "file" ~/           # Check files
history | grep "cmd"   # Check commands  
git log --grep "msg"   # Check commits
intent ls              # Check intents
# TOO MANY TOOLS!
```

## The Solution

### One Keybind, Everything Searchable

**Super+Space** → Universal Search
```
🔍 Type: "wifi"
─────────────────────────────────
📱 NetworkManager (app)
📄 wifi-fix.sh (file, ~/scripts/)
💻 nmcli device wifi (command, used 5 times)
📝 Intent 003: Shell safety (mentions wifi)
🔀 Commit a1b2c3: Fix wifi on wake (3 days ago)
📚 Doc: network-troubleshooting.md
```

### Search Domains

**1. Applications** (existing)
- XDG desktop entries
- Frecency ranking

**2. Files**
- Recent files (last 7 days)
- Indexed directories (~/0-core, ~/Documents)
- Smart filtering (ignore .git, target/, node_modules)

**3. Commands**
- Shell history
- Frecency ranking
- Preview: shows last usage

**4. Intents**
- ID, title, tags, status
- Preview: shows description

**5. Git History**
- Commit messages
- File changes
- Preview: shows diff stats

**6. Documentation**
- README files
- Markdown docs
- Code comments

**7. Web Bookmarks** (future)
- Browser history
- Saved pages

## Architecture
```rust
// Universal search index
struct SearchIndex {
    apps: Vec<App>,
    files: Vec<File>,
    commands: Vec<Command>,
    intents: Vec<Intent>,
    commits: Vec<Commit>,
    docs: Vec<Doc>,
}

// Unified search
fn search(query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    results.extend(search_apps(query));
    results.extend(search_files(query));
    results.extend(search_commands(query));
    // ...
    results.sort_by_key(|r| -r.score);
    results
}
```

**Storage:** `~/.local/share/faelight-search/`
- `index.db` - SQLite database
- `cache/` - Indexed file content

## UI Mockup
```
┌─────────────────────────────────────────┐
│  🔍 Universal Search                    │
├─────────────────────────────────────────┤
│  > wifi_                                │
├─────────────────────────────────────────┤
│  📱 NetworkManager                      │
│     Application                          │
│                                          │
│● 📄 wifi-fix.sh                         │
│     ~/scripts/ • Modified 2h ago        │
│     Preview: #!/bin/bash                │
│              # Fix wifi on wake...       │
│                                          │
│  💻 nmcli device wifi                   │
│     Command • Used 5 times              │
│                                          │
│  📝 Intent 003: Shell Safety            │
│     Complete • Mentions: wifi, network  │
├─────────────────────────────────────────┤
│  ↑↓ Navigate  Enter Open  Esc Close    │
└─────────────────────────────────────────┘
```

## Indexing Strategy

**Incremental Updates:**
```bash
# File watcher (inotify)
# On file change: update index
# On git commit: index commit
# On command: update history
```

**Manual Refresh:**
```bash
faelight-search index
# Rebuilds entire index
```

## Success Criteria
- [ ] Search backend in Rust (faelight-search or in core)
- [ ] SQLite index database
- [ ] Search apps, files, commands, intents, git
- [ ] Fuzzy matching algorithm
- [ ] Frecency ranking
- [ ] Preview pane for results
- [ ] Incremental indexing
- [ ] Fast (<50ms for most queries)
- [ ] Integration with existing launcher
- [ ] Configurable search domains

## Performance Targets
- Index build: <10s for 100k files
- Search latency: <50ms
- Memory usage: <50MB
- Disk usage: <100MB index

## Timeline
**v7.5.0** - Foundation (apps, files, commands)
**v7.6.0** - Add intents, git, docs
**v7.7.0** - Preview pane, advanced filters
**v7.8.0** - ML ranking, personalization

## Dependencies
- Depends: Intent 059 (monorepo)
- Relates: Intent 027 (fuzzy search)
- Relates: Intent 017 (launcher)

## Progress Update (v7.5.0)
Phase 1 complete:
- Application search
- File search with fuzzy matching and recency scoring

Remaining scope:
- Command search
- Intent Ledger search
- Git commit history search

Future work targeted for v7.6.0+

---

_"Everything, instantly, everywhere."_ 🌲🔍
