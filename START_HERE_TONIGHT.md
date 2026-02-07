# 🌲 START HERE TONIGHT - Session Roadmap

## 🎯 CHOSEN DESIGN: EVEN CLEANER PROMPT
```
🟢 🌲 0-CORE 🏠🔓  git:MED  main [!3] 󱘗 v1.93.0
```

**What this shows:**
- 🟢 Health dot (100% = green, 90-99% = amber, <90% = red)
- 🌲 0-CORE zone (where you are)
- 🏠🔓 Home unlocked (lock status, not username)
- git:MED risk level
- main branch
- [!3] git status (3 modified)
- 󱘗 v1.93.0 Rust version

**What we removed:**
- ❌ Duplicate "0-core" path
- ❌ "christian" username (you know who you are!)
- ❌ 📦 root indicator (you know you're in cargo)
- ❌ Intent incidents (moving to bar)
- ❌ Git diff stats 3± (duplicate of [!3])

---

## SESSION PLAN

### PHASE 1: PROMPT (Start Here!) ⏱️ 20 mins
**File:** `03-interfaces/stow/prompt-starship/.config/starship.toml`

**Steps:**
1. Add health_dot module (🟢🟡🔴)
2. Remove username, show 🏠 instead
3. Remove cargo_root (📦 root)
4. Remove smart_path (duplicate)
5. Remove git_diff_stats (3±)
6. Remove intent_incidents
7. Simplify git_risk format
8. Update format string
9. Test with: `source ~/.zshrc`

### PHASE 2: BAR ⏱️ 20 mins
**File:** `rust-tools/faelight-bar/src/main.rs`

**Steps:**
1. Remove health percentage display
2. Add intent counter (🎯 3 in-progress)
3. Add update counter (📦 23 updates)
4. Build: `cargo build --release -p faelight-bar`
5. Deploy: `cp target/release/faelight-bar scripts/`
6. Restart: `pkill faelight-bar && nohup ~/0-core/scripts/faelight-bar &`

**Result:**
```
[🎯 3 in-progress] [📦 23 updates] [18:45] [🔋 85%] [📶 WiFi]
```

### PHASE 3: REST AS WE GO
- Code quality sprint (if time)
- Other improvements
- Whatever flows!

---

## QUICK START COMMANDS

When you return:
```bash
# 1. See the plan
cat START_HERE_TONIGHT.md

# 2. See detailed implementation
cat BALANCED_PROMPT_PLAN.md

# 3. Start working on prompt
nvim 03-interfaces/stow/prompt-starship/.config/starship.toml

# 4. Or just tell me "let's start!" and I'll guide you step by step
```

---

## EXPECTED RESULT

**PROMPT:**
```
🟢 🌲 0-CORE 🏠🔓  git:MED  main [!3] 󱘗 v1.93.0
```

**BAR:**
```
[🎯 3 in-progress] [📦 23 updates] [18:45] [🔋 85%] [📶 WiFi]
```

Clean. Minimal. Informative. Perfect. 🎯

---

See you tonight! Rest well! 💎🌲
