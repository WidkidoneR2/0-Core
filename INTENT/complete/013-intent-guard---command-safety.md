---
id: 013
date: 2026-01-13
type: future
title: "intent-guard - Command Safety Guard"
status: complete
tags: [rust, safety, shell, v6.9]
---

## The Vision

Prevent dangerous command execution through awareness, not automation.

**Philosophy:** Never block. Always warn. You decide.

## Implementation Complete

**v0.2.1 shipped** - Context-aware command safety system

### Features
- ✅ Pattern-based risk detection (11 patterns)
- ✅ 4 risk levels (Critical/High/Medium/Low)
- ✅ User confirmation (exact word for critical)
- ✅ Shell integration (zsh preexec hook)
- ✅ Context-aware matching (no false positives)

### Pattern Database (11 total)

**Critical (5):**
- rm -rf on critical paths
- rm -rf ~/0-core
- mkfs.* operations
- mv ~/0-core
- Shell config overwrite (> ~/.zshrc)

**High (4):**
- chmod 777
- dd device operations
- curl/wget | bash
- find -delete

**Medium (2):**
- pacman -R
- systemctl disable/stop

### Architecture
```rust
// Risk levels with color coding
enum RiskLevel { Low, Medium, High, Critical }

// Pattern with function-based checker
struct Pattern {
    name: &'static str,
    description: &'static str,
    check: fn(&str) -> bool,
    risk: RiskLevel,
}

// Context-aware checkers
fn check_rm_rf_root(cmd: &str) -> bool {
    // Only triggers on actual rm commands
    // Checks command start, not just content
}
```

### Shell Integration
```zsh
preexec() {
    intent-guard check-command "$1" 2>&1
    if [ $? -ne 0 ]; then
        kill -INT $$  # Cancel execution
    fi
}
```

### Testing Results

✅ **No false positives:**
- Git commits with dangerous strings: pass
- Echo commands with redirects: pass
- Cat/printf to configs: pass

✅ **Correct detection:**
- Actual rm -rf commands: trigger
- Actual curl | bash: trigger  
- Actual config overwrites: trigger

## Success Criteria

- [x] Pattern detection working
- [x] Risk assessment accurate
- [x] User confirmation required
- [x] Shell integration functional
- [x] No false positives on safe commands
- [x] Context-aware matching
- [x] Tested and verified

## Files

- Binary: `~/0-core/scripts/intent-guard`
- Source: `~/0-core/rust-tools/intent-guard/src/main.rs`
- Hook: `~/0-core/shell-zsh/.config/zsh/.zshrc` (preexec)

## Versions

- v0.1.0 - Initial implementation (5 patterns)
- v0.1.1 - Context-aware matching
- v0.2.0 - Expanded database (11 patterns)
- v0.2.1 - Fixed false positives

---

_"The guard doesn't prevent mistakes. It prevents accidents."_ 🌲
