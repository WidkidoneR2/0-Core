# archaeology-0-core v1.0.0

🏺 **System-wide history explorer**

Dig through 0-Core's evolution. Connect commits to intents. Understand how you got here.

---

## Concept

**The problem:** Git history is chronological, but you think in **packages** and **intents**.

**archaeology-0-core solves this** by making history queryable:
- "What changed in wm-sway?"
- "What did I do this week?"
- "What commits implemented intent #036?"
- "What changed since v7.0.0?"

**Git archaeology made useful.**

---

## Usage

### Package-specific history
```bash
archaeology-0-core wm-sway
```
Shows all commits that touched the Sway package.

### System timeline
```bash
archaeology-0-core --timeline
```
Chronological view of all changes across all packages.

### Recent work
```bash
archaeology-0-core --this-week
```
Last 7 days of commits.

### Changes since version
```bash
archaeology-0-core --since v7.0.0
```
Everything that changed since a specific release.

### Intent-based archaeology
```bash
archaeology-0-core --by-intent 036
```
All commits related to Intent #036.

---

## Why This Matters

### Before archaeology-0-core:
```bash
git log --all -- stow/wm-sway/  # Verbose, hard to parse
git log --since="1 week ago"     # Just dates, no context
git log --grep="036"             # Missed related commits
```

### With archaeology-0-core:
```bash
archaeology-0-core wm-sway       # Clean package history
archaeology-0-core --this-week    # Contextualized recent work
archaeology-0-core --by-intent 036  # Intent-to-commit mapping
```

**Connects git history to your mental model.**

---

## Output Format

Shows:
- **Date** - When it happened
- **Changes** - What files were touched
- **Intents** - Which intent (if any) this implements
- **Packages** - Which stow packages affected

**Example output:**
```
📅 2026-01-24  Intent #036: Sway keybind improvements
   📦 wm-sway
   📝 Added workspace switching shortcuts
   🔗 Implements intent 036 (Sway migration)

📅 2026-01-23  Performance improvements
   📦 faelight-bar
   📝 Cached model architecture
   🔗 No linked intent

📅 2026-01-22  Security hardening
   📦 vcs-git
   📝 Added gitleaks config
   🔗 Implements intent 040 (Security audit)
```

---

## Use Cases

### 1. Package Evolution
**Question:** "How has wm-sway evolved?"
```bash
archaeology-0-core wm-sway
```
Shows complete history of Sway configuration changes.

### 2. Weekly Review
**Question:** "What did I accomplish this week?"
```bash
archaeology-0-core --this-week
```
Perfect for weekly planning/retrospectives.

### 3. Release Notes
**Question:** "What changed since last release?"
```bash
archaeology-0-core --since v8.0.0
```
Auto-generate release notes from git history.

### 4. Intent Verification
**Question:** "Did I fully implement intent #036?"
```bash
archaeology-0-core --by-intent 036
```
See all commits that claim to implement it.

### 5. Incident Investigation
**Question:** "When did the Sway config break?"
```bash
archaeology-0-core wm-sway | grep -A 2 "keybind"
```
Find when specific changes were made.

---

## Philosophy

**"Git history is archaeology, not just version control."**

Your commits tell a story:
- What you tried
- What worked
- What you learned
- Why you made choices

**archaeology-0-core** makes that story readable.

---

## Integration

### With Intent Ledger
Automatically detects intent references in commit messages:
```bash
git commit -m "feat: Add keybinds (Intent #036)"
# Later: archaeology-0-core --by-intent 036
```

### With Stow Packages
Understands package boundaries:
```bash
archaeology-0-core shell-zsh  # Only shell config history
```

### With bump-system-version
Use for release notes:
```bash
archaeology-0-core --since v8.0.0 > release-notes.txt
```

---

## Technical Details

**Queries git log** with smart filtering:
- Package filter: `git log --all -- stow/<package>/`
- Timeline: `git log --all --pretty=format:...`
- Time-based: `git log --since="7 days ago"`
- Intent-based: `git log --all --grep="Intent #<id>"`

**Parses commit messages** to extract:
- Intent references (Intent #XXX)
- Package names (from file paths)
- Change descriptions

**Formats output** for human reading:
- Chronological order
- Clear structure
- Context from commit messages

---

## Examples

### Before big changes
```bash
# Review recent history to avoid conflicts
archaeology-0-core --this-week
```

### After learning something new
```bash
# See how you implemented similar features before
archaeology-0-core faelight-bar --since v6.0.0
```

### For presentations
```bash
# Show your work evolution
archaeology-0-core --timeline | head -50
```

---

## Part of 0-Core

One of 30+ Rust tools in the Faelight Forest ecosystem.

Complements the Intent Ledger by connecting intentions to actual implementation.

**Philosophy:** Understanding over convenience. Know how you got here.

See: https://github.com/WidkidoneR2/0-Core
