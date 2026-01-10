# 0-Core Workflows

Practical usage patterns for real-world scenarios.

---

## Daily Workflows

### Morning Posture Check

**Goal:** Know your system's state before starting work.

```bash
# 1. Open terminal
# 2. Check for changes
core-diff

# 3. Check system health
dot-doctor

# 4. Review and plan
```

**Why:** Sets intention, avoids surprises, primes the mind.

---

### Change Review Ritual

**Goal:** Make every config change intentional and observable.

```bash
# 1. Make changes in 0-core
unlock-core
# ... edit files ...

# 2. Review what changed
core-diff --verbose

# 3. Inspect critical packages carefully
core-diff wm-sway --open delta

# 4. Quick terminal review for others
core-diff --open delta

# 5. When satisfied, commit
git add <files>
git commit -m "descriptive message"
git push

# 6. Lock the core
lock-core
```

**Why:** Turns configuration edits into deliberate thinking sessions.

---

## Weekly Workflows

### Release Review

**Goal:** Understand all changes before tagging a release.

```bash
# 1. Check changes since last version
core-diff since v3.3.5

# 2. Review high-risk changes carefully
core-diff --high-risk

# 3. For each critical package
core-diff wm-sway --open delta
core-diff shell-zsh --open delta

# 4. Check overall health
dot-doctor

# 5. If clean, tag release
git tag v3.4.0
git push --tags
```

---

### Maintenance Loop

**Goal:** Prevent accumulation of technical debt.

```bash
# Pick a day (like Sunday) for maintenance

# 1. System updates
safe-update

# 2. Health check
dot-doctor

# 3. Review aging files
# (dot-doctor shows this automatically)

# 4. Check for uncommitted changes
core-diff

# 5. Small cleanup or refactoring
```

**Why:** Creates rhythm, enforces manual inspection, prevents entropy.

---

## Incident Response

### When Something Breaks

**Goal:** Quick diagnosis and recovery.

```bash
# 1. Check recent changes
core-diff since HEAD~5 --high-risk

# 2. Identify suspicious package
# Output shows which packages changed

# 3. Deep dive on suspect package
core-diff <package> --open delta

# 4. Review file-by-file
# Meld shows exact changes

# 5. Revert if needed
git checkout HEAD~1 <package>/

# 6. Or fix forward
# Edit files, test, commit

# 7. Document in INCIDENTS.md
```

---

## Integration Patterns

### With dot-doctor

```bash
# Post-change validation
core-diff --high-risk && dot-doctor
```

### With git workflow

```bash
# Pre-commit review
core-diff --verbose
git add <files>
git commit

# Verify clean
core-diff  # Should show nothing
```

### Morning routine

```bash
# All-in-one check
core-diff && dot-doctor
```

---

## Workflow Aliases

Quick shortcuts for common patterns:

```bash
# Morning check
cdiff

# Detailed review
cdv  # core-diff --verbose

# Focus mode
cdh  # core-diff --high-risk

# Quick stats
cds  # core-diff summary

# Visual inspection
cdm  # core-diff --open delta
cdd  # core-diff --open delta
```

See shell configuration for all aliases.

---

## Advanced Patterns

### Pre-Update Snapshot

```bash
# Before major updates, capture state
core-diff summary > /tmp/pre-update.txt

# After updates
core-diff summary > /tmp/post-update.txt

# Compare
diff /tmp/pre-update.txt /tmp/post-update.txt
```

### Package-Focused Development

```bash
# When working on specific package
cd ~/0-core

# Monitor just that package
watch -n 2 'core-diff wm-sway --verbose'

# Or use delta for live diff
core-diff wm-sway --open delta
```

### Release Checklist

```bash
# 1. Review changes
core-diff since $(git describe --tags --abbrev=0)

# 2. Verify health
dot-doctor  # Must be 100%

# 3. Update docs if needed
# 4. Bump version
# 5. Commit, tag, push
# 6. Lock
```

---

## Tips & Best Practices

### Use Summary Mode

When you just need quick stats:

```bash
core-diff summary
```

### Combine Flags

```bash
# Verbose high-risk only
core-diff --high-risk --verbose

# Package + tool
core-diff wm-sway --open delta
```

### Read the Output

Pay attention to:

- 🔴 **CRITICAL** always requires careful review
- File counts (1 file vs 20 files matters)
- Risk level in summary

### Don't Skip Meld

For critical packages, always use visual inspection:

```bash
core-diff wm-sway --open delta
```

Delta is fast, Meld is thorough.

---

## Common Scenarios

**Scenario:** "I changed something yesterday, what was it?"

```bash
core-diff since HEAD~1
```

**Scenario:** "Did I touch any critical configs?"

```bash
core-diff --high-risk
```

**Scenario:** "Show me exactly what changed in my shell config"

```bash
core-diff shell-zsh --open delta
```

**Scenario:** "Quick status before committing"

```bash
core-diff summary
```

---

## Philosophy in Practice

These workflows embody 0-Core principles:

- **Manual control** - You choose when to review
- **Intent over automation** - Explicit inspection steps
- **Recovery over perfection** - Focus on catching mistakes
- **Human comprehension** - Tools explain, don't hide

See [PHILOSOPHY.md](../PHILOSOPHY.md) for deeper context.
