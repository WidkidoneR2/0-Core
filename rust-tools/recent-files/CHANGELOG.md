# Changelog - recent-files

## [2.0.0] - 2026-02-11

### 🎉 PRODUCTION READY

**Features:**
- Find files by modification time
- Time ranges: hour, today, week, month
- Category grouping (Documents, Code, Media, etc.)
- Interactive selection with fzf
- Quick open most recent file
- Full or relative paths
- Configurable result limits

**Commands:**
```bash
recent-files                  # Today's files
recent-files week             # Last week
recent-files -l 20            # Show 20 per category
recent-files --open           # Interactive fzf
recent-files --open-first     # Open most recent
```

**Improvements:**
- Better error handling
- Helpful error hints
- Check for missing dependencies
- Clear messages

**Dependencies:**
- fzf (optional, for --open)
- $EDITOR (for opening files)

**Code Quality:**
- Clean 399+ lines
- Zero clippy warnings
- Proper error handling
- Helpful hints

---

## [0.3.0] - Earlier

File activity tracker.

---

**Version Format:** MAJOR.MINOR.PATCH
