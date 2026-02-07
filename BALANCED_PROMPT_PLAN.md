# 🎯 BALANCED PROMPT + ENHANCED BAR - Implementation Plan

## GOAL: Clean, Informative, Zero Redundancy

### BEFORE (Current - messy):
```
🌲 0-CORE 0-core📦 root⚠ 0 open  christian🔓  git:⚠︎ risk=MED 3±  main [!3] 󱘗 v1.93.0
```

### AFTER (Balanced - perfect!):
```
🟢 🌲 0-CORE 📦 root christian🔓  git:MED  main [!3] 󱘗 v1.93.0
```

### BAR (Enhanced):
```
[🎯 3 in-progress] [📦 23 updates] [18:45] [🔋 85%] [📶 WiFi]
```

---

## IMPLEMENTATION STEPS

### STEP 1: Prompt Cleanup (Starship)
**File:** `03-interfaces/stow/prompt-starship/.config/starship.toml`

**Changes to make:**

1. **ADD: Health Dot Module** (at the beginning of format)
```toml
[custom.health_dot]
command = '''
if command -v doctor >/dev/null 2>&1; then
  health=$(doctor 2>/dev/null | grep "Health:" | awk "{print \$2}" | tr -d "%")
  if [ -n "$health" ]; then
    if [ "$health" -eq 100 ]; then
      echo "🟢"
    elif [ "$health" -ge 90 ]; then
      echo "🟡"
    else
      echo "🔴"
    fi
  fi
fi
'''
when = 'true'
format = "[$output]($style) "
style = "bold white"
description = "Visual health indicator"
```

2. **FIX: Smart Path** (remove duplicate)
```toml
[custom.smart_path]
# Remove this entire module - it's duplicating zone
# Zone already shows "🌲 0-CORE"
```

3. **REMOVE: Git Diff Stats** (duplicate info)
```toml
[custom.git_diff_stats]
# Remove this entire module
# git_status already shows [!3] which is better
```

4. **REMOVE: Intent Incidents** (moving to bar)
```toml
[custom.intent_incidents]
# Remove this entire module
# Moving to bar where it's more useful
```

5. **SIMPLIFY: Git Risk** (shorter format)
```toml
[custom.git_risk]
# Change the format line in the command:
# FROM: echo "${msg}${details}"
# TO:   echo "git:${risk}"  # Just show the risk level
```

6. **UPDATE: Format String** (new order)
```toml
format = """
[┌─](bold green) \
${custom.health_dot}\
${custom.faelight_zone}\
${custom.cargo_root}\
$os\
$username\
$hostname\
${custom.core_lock} \
${custom.git_risk}\
$git_branch \
$git_status\
$rust\
$cmd_duration
[└─](bold green)$character
"""
```

---

### STEP 2: Bar Enhancement (Rust)
**File:** `rust-tools/faelight-bar/src/main.rs`

**Changes to make:**

1. **REMOVE: Health display code**
   - Find the health percentage code
   - Delete or comment out

2. **ADD: Intent counter function**
```rust
fn get_intent_count() -> String {
    let output = Command::new("intent")
        .arg("list")
        .arg("--active")
        .output();
    
    if let Ok(out) = output {
        let content = String::from_utf8_lossy(&out.stdout);
        let count = content.lines()
            .filter(|l| l.contains("[in-progress]"))
            .count();
        
        if count > 0 {
            format!("🎯 {} in-progress", count)
        } else {
            "✅ All done!".to_string()
        }
    } else {
        "".to_string()
    }
}
```

3. **ADD: Update counter function**
```rust
fn get_update_count() -> String {
    let output = Command::new("checkupdates")
        .output();
    
    if let Ok(out) = output {
        let count = String::from_utf8_lossy(&out.stdout)
            .lines()
            .count();
        
        if count > 0 {
            format!("📦 {} updates", count)
        } else {
            "✅ Up to date".to_string()
        }
    } else {
        "".to_string()
    }
}
```

4. **UPDATE: Bar display** (in main loop)
```rust
// Replace health display with:
let intents = get_intent_count();
let updates = get_update_count();

// Add to bar output
format!("[{}] [{}] [{}] [{}] [{}]", 
    intents, updates, time, battery, network)
```

---

### STEP 3: Testing Checklist

After implementation:

- [ ] Prompt shows: `🟢 🌲 0-CORE 📦 root christian🔓 git:MED main [!3] 󱘗 v1.93.0`
- [ ] Health dot changes: 100% = 🟢, 90-99% = 🟡, <90% = 🔴
- [ ] Bar shows: `[🎯 X in-progress] [📦 X updates] [time] [battery] [network]`
- [ ] No duplicate "0-core" in prompt
- [ ] No git diff stats (3±) duplication
- [ ] Intent count accurate in bar
- [ ] Update count accurate in bar

---

### STEP 4: Build & Deploy
```bash
# Build bar with new features
cd ~/0-core
cargo build --release -p faelight-bar

# Deploy
cp target/release/faelight-bar scripts/

# Restart bar
pkill faelight-bar
nohup ~/0-core/scripts/faelight-bar &

# Reload prompt
source ~/.zshrc

# Test
doctor  # Check health dot changes
```

---

### STEP 5: Commit Strategy
```bash
# Commit 1: Prompt cleanup
git add 03-interfaces/stow/prompt-starship/.config/starship.toml
git commit -m "feat(prompt): Balanced prompt with health dot! 🟢

REMOVED REDUNDANCY:
- Duplicate path (0-core shown twice)
- Git diff stats (3±) - redundant with [!3]
- Intent incidents (moving to bar)

ADDED:
- Health dot: 🟢 (100%) 🟡 (90-99%) 🔴 (<90%)
- Simplified git risk: git:MED

RESULT: Clean, balanced, informative!
Before: 🌲 0-CORE 0-core📦 root⚠ 0 open christian🔓 git:⚠︎ risk=MED 3± main [!3]
After:  🟢 🌲 0-CORE 📦 root christian🔓 git:MED main [!3] 󱘗 v1.93.0"

# Commit 2: Bar enhancement
git add rust-tools/faelight-bar/
git commit -m "feat(bar): Intent tracking + update counter! 🎯📦

REMOVED:
- Health percentage (now in prompt as dot)

ADDED:
- Intent counter: 🎯 3 in-progress
- Update counter: 📦 23 updates

Bar now shows ACTIONABLE info:
[🎯 3 in-progress] [📦 23 updates] [18:45] [🔋 85%] [📶 WiFi]

Prompt shows STATUS, Bar shows WORK!"
```

---

## TIME ESTIMATE: 45 minutes
- Prompt changes: 20 mins
- Bar changes: 20 mins  
- Testing: 5 mins

Perfect for tonight's session! 🎯

---

## BONUS: Intent Workflow Integration

With intents in the bar, your workflow becomes:
1. See "🎯 3 in-progress" in bar
2. Run `intent list --active` to see what they are
3. Work on one
4. Run `intent complete 076`
5. Bar updates to "🎯 2 in-progress"

Visual feedback loop! 🔄
