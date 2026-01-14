
## keyscan v0.1.0
**Sway Keybind Conflict Detection**

Parses Sway configuration to detect duplicate keybindings.
```bash
# Check current config
keyscan

# Check specific config
keyscan --config ~/.config/sway/config.backup

# JSON output
keyscan --json
```

**Features:**
- Normalizes modifiers (Mod4 → Super, $mod → Super)
- Handles bindsym and bindcode
- Filters flags (--no-repeat, --release, etc.)
- Reports line numbers for conflicts

**Output:**
```
🔍 keyscan - Sway Keybind Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ No conflicts detected
   99 unique keybindings
```

**Philosophy:** Prevention over correction. Catch conflicts before they cause confusion.

