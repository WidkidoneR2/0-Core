---
id: 044
date: 2026-01-13
type: future
title: "Keybind Conflict Detection"
status: complete
tags: [safety, sway, keybinds, validation, v6.9]
---

## The Problem

Sway config has 50+ keybinds across:
- Window management
- Workspace switching
- Application launching
- System operations
- Profile switching

**Current risk:** Adding new keybind might silently override existing one.

## The Vision

Validate Sway config for keybind conflicts before applying changes.

## Implementation

### Tool: `keyscan` or `sway-keybind-check`

**What it does:**
```bash
keyscan ~/.config/sway/config

# Output:
✅ No conflicts detected (54 unique bindings)

# OR

⚠️  Conflict detected:
   Mod+d → faelight-launcher (line 45)
   Mod+d → exec fuzzel (line 89)
   
   Resolution needed before applying config
```

**How it works:**
1. Parse Sway config
2. Extract all `bindsym` and `bindcode` entries
3. Build keybind map
4. Detect duplicates
5. Show conflicts with line numbers

### Integration Points

**dot-doctor check:**
```rust
fn check_sway_keybinds() -> CheckResult {
    // Run keyscan
    // Warn if conflicts found
}
```

**core-diff enhancement:**
```bash
cdiff wm-sway
# Shows: "2 new keybinds, 1 potential conflict"
```

**Pre-commit hook:**
```bash
# In .git/hooks/pre-commit
if sway config changed:
    run keyscan
    block commit if conflicts
```

## Technical Design

### Keybind Model
```rust
struct Keybind {
    modifiers: Vec<Modifier>,  // Super, Shift, Ctrl, Alt
    key: String,                // d, Return, Left, etc
    command: String,            // What it executes
    location: Location,         // File + line number
}

enum Modifier {
    Super,
    Shift,
    Ctrl,
    Alt,
}
```

### Parsing Strategy

**Simple regex approach:**
```rust
// Match: bindsym Mod+Shift+d exec kitty
let pattern = r"bindsym\s+([^\s]+)\s+(.+)";
```

**Proper parser (better):**
- Handle `bindsym --no-repeat`
- Handle `bindcode`
- Handle multiline commands
- Handle `mode "resize"` blocks

### Output Formats

**Human-readable (default):**
```
🔍 keyscan - Sway Keybind Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ 54 unique keybinds
⚠️  1 conflict detected

Conflict:
  Mod+d
    Line 45: exec faelight-launcher
    Line 89: exec fuzzel
```

**JSON (automation):**
```json
{
  "total_binds": 54,
  "conflicts": [
    {
      "keybind": "Mod+d",
      "bindings": [
        {"line": 45, "command": "exec faelight-launcher"},
        {"line": 89, "command": "exec fuzzel"}
      ]
    }
  ]
}
```

## Phases

### Phase 1: Basic Detection (v6.9.0)
- Parse Sway config
- Detect duplicate keybinds
- Report conflicts
- CLI tool only

### Phase 2: Integration (v6.9.0 or v7.0.0)
- Add to dot-doctor
- Add to core-diff
- Optional pre-commit hook

### Phase 3: Intelligence (v7.0.0+)
- Suggest available keybinds
- Show keybind "heat map"
- Detect near-conflicts (Mod+d vs Mod+Shift+d)
- Profile-aware checking

## Success Criteria

**v6.9.0:**
- [x] keyscan tool works
- [x] Detects conflicts accurately
- [x] Shows file + line numbers
- [x] JSON output available

**Nice to have:**
- [x] dot-doctor integration
- [ ] core-diff integration
- [ ] Pre-commit hook option

## Alternatives Considered

**Manual auditing:** Rejected - error-prone, doesn't scale  
**swaymsg -t get_binding_state:** Limited - only shows active binds, not config source  
**External tool (sway-check):** Could work, but custom tool fits 0-Core philosophy

## Philosophy Alignment

> "The system should tell you when you're about to break something."

This is **policy enforcement without automation**:
- Detects the problem (conflict)
- Shows you the issue (with context)
- Lets you decide (manual fix)
- Never auto-fixes

## Estimated Effort

- Parser: 2-3 hours
- Conflict detection: 1 hour
- CLI output: 1 hour
- Testing: 1 hour
- Integration: 1-2 hours

**Total: ~6-8 hours (1 day)**

---

_"Keybinds are muscle memory. Conflicts are chaos."_ 🌲
