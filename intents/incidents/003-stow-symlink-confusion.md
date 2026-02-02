---
id: 003
date: 2026-01-12
type: incident
title: "Stow Symlink Confusion - Shell Alias Loss"
severity: high
status: resolved
tags: [stow, shell, recovery, v6.8]
---

## What Happened

During entropy-check v0.1 development, attempted to add aliases to shell-zsh package but confused two .zshrc files:
- `shell-zsh/.zshrc` (WRONG - should not exist)
- `shell-zsh/.config/zsh/.zshrc` (CORRECT - actual stowed file)

Edited wrong file, then deleted it, breaking the symlink chain. User lost access to shell and all aliases temporarily.

## Impact

- Shell stopped loading (~2 minutes)
- All aliases inaccessible
- User panic 😱
- No actual data loss (git saved us)

## Root Cause

**Misunderstanding of stow directory structure:**
- Stow packages mirror HOME structure: `package/.config/foo/bar` → `~/.config/foo/bar`
- But there was a duplicate `shell-zsh/.zshrc` (flat file) that shouldn't exist
- Instructions edited the wrong file
- `stow --adopt` merged wrong file, then deletion broke symlink

## How We Recovered

1. Verified actual config still intact: `shell-zsh/.config/zsh/.zshrc` ✓
2. Recreated symlink manually: `ln -s ~/0-core/shell-zsh/.config/zsh/.zshrc ~/.zshrc`
3. Reloaded shell: `source ~/.zshrc`
4. All aliases restored

## Lessons Learned

1. **Always verify stow structure before editing**
   - Use `stow --no-act` to see what would happen
   - Check actual symlink targets: `ls -la ~/.zshrc`

2. **Safety nets worked:**
   - Git protected source files
   - Snapshot #23 was available (snapper rollback ready)
   - Manual symlink recreation was straightforward

3. **Better workflow for adding aliases:**
```bash
   # CORRECT way:
   cd ~/0-core
   echo "alias foo='bar'" >> shell-zsh/.config/zsh/.zshrc
   stow -R shell-zsh
   source ~/.zshrc
   
   # WRONG way (what we did):
   echo "alias foo='bar'" >> shell-zsh/.zshrc  # Wrong file!
   stow --adopt shell-zsh  # Merged wrong content
```

4. **Verify symlinks in faelight-stow:**
   - Should detect mismatched symlink targets
   - Should warn about files in stow packages that aren't symlinked

## Prevention

- [ ] Update faelight-stow to detect unexpected files in stow packages
- [ ] Add check for symlink target correctness
- [ ] Document stow structure in docs/STOW.md
- [x] Always `ls -la` target before editing
- [x] Keep git clean before risky operations

## Related

- Tool: faelight-stow v0.1
- Intent: 015 (entropy-check development)
- Snapshot: #23 (before v6.8.0 work)

---

_"Recovery over perfection. We designed for the 2am mistake."_ 🌲

The safety nets worked. No data lost. System recovered in 5 minutes.
