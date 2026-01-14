---
id: 049
date: 2026-01-14
type: future
title: "faelight-launcher v2.0 - Provider Architecture"
status: planned
tags: [architecture, v7.0, launcher, providers, risk, teaching]
---

## Vision

**A launcher that doesn't just find apps - it explains, teaches, and protects.**

**Philosophy:** "Walker does ranking. Faelight explains ranking."

## What Makes This Better Than Walker

### Walker's Strengths
✅ Async, non-blocking UI  
✅ Fast fuzzy matching  
✅ Plugin-based sources  
✅ Layer-shell integration  
✅ Low startup latency  

### Where We Beat Walker

**Technical:**
✅ Provider merging (one ranked list, not mode switching)  
✅ Single binary (no GTK dependency)  
✅ Deterministic performance (no regex, no hot path allocations)  
✅ Better indexing (mmap + inotify)  

**Philosophical (Our Competitive Advantage):**
✅ **Explainable ranking** - Press `?` to see WHY something is ranked #1  
✅ **Risk awareness** - intent-guard integration prevents dangerous commands  
✅ **Teaching mode** - Learn as you launch  
✅ **Context-aware** - Profile integration, smart scoring  
✅ **Frecency tracking** - Learns your patterns  

**Quote:** "Speed is table stakes. Intelligence is the differentiator."

---

## Architecture

### High-Level Design
```
┌─────────────┐
│  sway/wl    │
└─────┬───────┘
      │ Wayland
┌─────▼──────────────┐
│ UI (layer-shell)   │  ← faelight-core
│ input + render     │
└─────┬──────────────┘
      │ events
┌─────▼──────────────┐
│ Core Engine        │
│ query + scoring    │
└─────┬──────────────┘
      │ async
┌─────▼──────────────┐
│ Providers (plugins)│
│ apps / cmd / etc   │
└─────┬──────────────┘
      │
┌─────▼──────────────┐
│ Cache + Index      │
│ mmap + inotify     │
└────────────────────┘
```

### Module Structure
```
faelight-launcher/
├── src/
│   ├── main.rs              # Entry point
│   ├── ui/
│   │   ├── mod.rs           # UI state machine
│   │   ├── input.rs         # Keyboard handling
│   │   └── render.rs        # Drawing (uses faelight-core)
│   ├── engine/
│   │   ├── mod.rs           # Core engine
│   │   ├── scorer.rs        # Ranking algorithm
│   │   ├── query.rs         # Query parsing
│   │   └── explainer.rs     # Why this result?
│   ├── provider/
│   │   ├── mod.rs           # Provider trait
│   │   ├── apps.rs          # Desktop entries
│   │   ├── commands.rs      # Shell commands (risk-aware)
│   │   ├── intents.rs       # Intent ledger
│   │   └── profiles.rs      # Profile switching
│   ├── cache/
│   │   ├── mod.rs           # Index management
│   │   ├── apps.rs          # App cache + inotify
│   │   └── history.rs       # Frecency tracking
│   ├── config.rs            # TOML config
│   └── error.rs             # Error types
└── Cargo.toml
```

---

## Core Components

### 1. Provider Trait (The Foundation)
```rust
pub trait Provider: Send + Sync {
    /// Provider name (e.g., "apps", "commands")
    fn name(&self) -> &'static str;
    
    /// All available items
    fn items(&self) -> &[Item];
    
    /// Execute selected item
    fn execute(&self, item: &Item) -> Result<()>;
    
    /// Risk level (for risk-aware providers)
    fn risk_level(&self, item: &Item) -> RiskLevel {
        RiskLevel::Safe
    }
    
    /// Explain why this item matches
    fn explain(&self, item: &Item, query: &str) -> String {
        format!("Matches '{}'", query)
    }
    
    /// Provider weight in ranking
    fn weight(&self) -> f32 {
        1.0
    }
}

pub struct Item {
    pub label: String,        // What user sees
    pub exec: String,         // What gets executed
    pub category: String,     // For grouping
    pub metadata: Metadata,   // Provider-specific data
}

pub struct Metadata {
    pub provider: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum RiskLevel {
    Safe,       // Normal apps, safe commands
    Low,        // Minor system changes
    Medium,     // Package operations
    High,       // Privilege escalation
    Critical,   // Data loss, system damage
}
```

### 2. The Ranking Algorithm (Explainable)
```rust
pub struct ScoredItem<'a> {
    pub item: &'a Item,
    pub score: f32,
    pub components: ScoreComponents,  // For explanation
}

pub struct ScoreComponents {
    pub fuzzy_score: f32,      // Text match quality
    pub provider_weight: f32,  // Provider importance
    pub context_boost: f32,    // Profile relevance
    pub recency: f32,          // Recently used
    pub frequency: f32,        // Often used
    pub risk_penalty: f32,     // Safety consideration
}

impl ScoreComponents {
    pub fn total(&self) -> f32 {
        self.fuzzy_score * 0.45
            + self.provider_weight * 0.20
            + self.context_boost * 0.15
            + (self.recency + self.frequency) * 0.10
            - self.risk_penalty * 0.30
    }
    
    pub fn explain(&self, item: &Item) -> String {
        let mut parts = Vec::new();
        
        if self.fuzzy_score > 0.7 {
            parts.push(format!("✔ Strong text match ({:.0}%)", self.fuzzy_score * 100.0));
        }
        
        if self.recency > 0.5 {
            parts.push("✔ Recently used".into());
        }
        
        if self.frequency > 0.5 {
            parts.push("✔ Frequently used".into());
        }
        
        if self.context_boost > 0.0 {
            parts.push("✔ Matches active profile".into());
        }
        
        if self.risk_penalty > 0.0 {
            parts.push(format!("✖ {} risk", item.risk_level()));
        }
        
        parts.join("\n")
    }
}
```

### 3. Fuzzy Matching (Fast & Deterministic)
```rust
use nucleo_matcher::{Matcher, pattern::Pattern};

pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl FuzzyMatcher {
    pub fn score(&mut self, query: &str, target: &str) -> Option<f32> {
        // Prefix match gets massive boost
        if target.to_lowercase().starts_with(&query.to_lowercase()) {
            return Some(1.0);
        }
        
        // Fuzzy match with nucleo
        let pattern = Pattern::parse(query, nucleo_matcher::CaseMatching::Ignore);
        self.matcher.fuzzy_match(&target, &pattern)
            .map(|score| score as f32 / 1000.0)  // Normalize
    }
}
```

### 4. Teaching Mode (Press `?`)
```rust
pub enum LauncherMode {
    Normal,
    Teaching { item_index: usize },
}

impl LauncherState {
    fn handle_question_mark(&mut self) {
        if let Some(selected) = self.get_selected() {
            self.mode = LauncherMode::Teaching {
                item_index: self.selected_index,
            };
        }
    }
    
    fn draw_teaching_mode(&mut self, scored: &ScoredItem) {
        let item = scored.item;
        
        // Title
        self.canvas.draw_text(
            &mut self.glyph_cache,
            "Why is this ranked #1?",
            x, y,
            self.theme.font_size_large,
            self.theme.accent,
        );
        
        // Item info
        self.canvas.draw_text(
            &mut self.glyph_cache,
            &format!("Item: {}", item.label),
            x, y + 40,
            self.theme.font_size_normal,
            self.theme.text_primary,
        );
        
        // Score breakdown
        let explanation = scored.components.explain(item);
        self.canvas.draw_text(
            &mut self.glyph_cache,
            &explanation,
            x, y + 70,
            self.theme.font_size_normal,
            self.theme.text_secondary,
        );
        
        // Risk warning (if applicable)
        if scored.components.risk_penalty > 0.0 {
            self.canvas.draw_rect(x, y + 150, width, 60, self.theme.danger);
            self.canvas.draw_text(
                &mut self.glyph_cache,
                "⚠️  This command has elevated risk",
                x + 10, y + 165,
                self.theme.font_size_normal,
                self.theme.text_primary,
            );
        }
        
        // Hint
        self.canvas.draw_text(
            &mut self.glyph_cache,
            "Press Esc to return",
            x, height - 30,
            self.theme.font_size_small,
            self.theme.text_muted,
        );
    }
}
```

---

## Provider Implementations

### Provider 1: Apps (Desktop Entries)
```rust
pub struct AppsProvider {
    items: Vec<Item>,
    watcher: notify::RecommendedWatcher,
}

impl AppsProvider {
    pub fn new() -> Result<Self> {
        let items = Self::load_desktop_entries()?;
        let watcher = Self::watch_desktop_dirs()?;
        Ok(Self { items, watcher })
    }
    
    fn load_desktop_entries() -> Result<Vec<Item>> {
        use freedesktop_desktop_entry::DesktopEntry;
        
        let mut items = Vec::new();
        
        for dir in std::env::var("XDG_DATA_DIRS")
            .unwrap_or("/usr/share".into())
            .split(':')
        {
            let path = format!("{}/applications", dir);
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if let Ok(de) = DesktopEntry::from_path(entry.path()) {
                        if let Some(exec) = de.exec() {
                            items.push(Item {
                                label: de.name().unwrap_or_default().to_string(),
                                exec: exec.to_string(),
                                category: de.categories().unwrap_or_default().to_string(),
                                metadata: Metadata {
                                    provider: "apps".into(),
                                    icon: de.icon().map(String::from),
                                    description: de.comment().map(String::from),
                                    keywords: de.keywords().unwrap_or_default().to_vec(),
                                },
                            });
                        }
                    }
                }
            }
        }
        
        Ok(items)
    }
}

impl Provider for AppsProvider {
    fn name(&self) -> &'static str { "apps" }
    fn items(&self) -> &[Item] { &self.items }
    fn weight(&self) -> f32 { 1.0 }
    
    fn execute(&self, item: &Item) -> Result<()> {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&item.exec)
            .spawn()?;
        Ok(())
    }
}
```

### Provider 2: Commands (Risk-Aware)
```rust
pub struct CommandsProvider {
    items: Vec<Item>,
}

impl CommandsProvider {
    pub fn new() -> Self {
        // Generate from $PATH binaries
        let items = Self::scan_path();
        Self { items }
    }
    
    fn scan_path() -> Vec<Item> {
        // Scan $PATH for executables
        // Filter out GUI apps (already in AppsProvider)
        // TODO: Implement
        vec![]
    }
}

impl Provider for CommandsProvider {
    fn name(&self) -> &'static str { "commands" }
    fn items(&self) -> &[Item] { &self.items }
    fn weight(&self) -> f32 { 0.8 }  // Slightly lower than apps
    
    fn risk_level(&self, item: &Item) -> RiskLevel {
        // Use intent-guard patterns!
        use intent_guard::check_risk;
        check_risk(&item.exec)
    }
    
    fn execute(&self, item: &Item) -> Result<()> {
        // Check risk before execution
        let risk = self.risk_level(item);
        
        if risk >= RiskLevel::High {
            // Show warning (integrate with intent-guard)
            eprintln!("⚠️  High risk command: {}", item.exec);
            // TODO: Confirmation dialog
        }
        
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&item.exec)
            .spawn()?;
        Ok(())
    }
    
    fn explain(&self, item: &Item, query: &str) -> String {
        let risk = self.risk_level(item);
        if risk > RiskLevel::Safe {
            format!("Matches '{}'\n⚠️  Risk level: {:?}", query, risk)
        } else {
            format!("Matches '{}'", query)
        }
    }
}
```

### Provider 3: Intents (Ledger Integration)
```rust
pub struct IntentsProvider {
    items: Vec<Item>,
}

impl IntentsProvider {
    pub fn new() -> Result<Self> {
        let items = Self::load_intents()?;
        Ok(Self { items })
    }
    
    fn load_intents() -> Result<Vec<Item>> {
        // Parse INTENT/ directory
        // Create items for "Open intent XXX"
        // TODO: Implement
        Ok(vec![])
    }
}

impl Provider for IntentsProvider {
    fn name(&self) -> &'static str { "intents" }
    fn items(&self) -> &[Item] { &self.items }
    fn weight(&self) -> f32 { 0.5 }  // Lower priority
    
    fn execute(&self, item: &Item) -> Result<()> {
        // Open intent in $EDITOR
        std::process::Command::new(std::env::var("EDITOR").unwrap_or("nvim".into()))
            .arg(&item.exec)
            .spawn()?;
        Ok(())
    }
}
```

### Provider 4: Profiles (Quick Switching)
```rust
pub struct ProfilesProvider {
    items: Vec<Item>,
}

impl ProfilesProvider {
    pub fn new() -> Result<Self> {
        let items = vec![
            Item {
                label: "Switch to work profile".into(),
                exec: "profile switch work".into(),
                category: "profile".into(),
                metadata: Metadata::default(),
            },
            Item {
                label: "Switch to personal profile".into(),
                exec: "profile switch personal".into(),
                category: "profile".into(),
                metadata: Metadata::default(),
            },
        ];
        Ok(Self { items })
    }
}

impl Provider for ProfilesProvider {
    fn name(&self) -> &'static str { "profiles" }
    fn items(&self) -> &[Item] { &self.items }
    fn weight(&self) -> f32 { 0.7 }
    
    fn execute(&self, item: &Item) -> Result<()> {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&item.exec)
            .spawn()?;
        Ok(())
    }
}
```

---

## Frecency Tracking
```rust
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct History {
    launches: HashMap<String, LaunchHistory>,
}

pub struct LaunchHistory {
    count: u32,
    last_used: u64,  // Unix timestamp
}

impl History {
    pub fn record_launch(&mut self, item_id: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.launches
            .entry(item_id.into())
            .and_modify(|h| {
                h.count += 1;
                h.last_used = now;
            })
            .or_insert(LaunchHistory {
                count: 1,
                last_used: now,
            });
    }
    
    pub fn frecency_score(&self, item_id: &str) -> f32 {
        if let Some(history) = self.launches.get(item_id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            let age_hours = (now - history.last_used) / 3600;
            
            // Recency decay
            let recency = if age_hours < 1 {
                1.0
            } else if age_hours < 24 {
                0.8
            } else if age_hours < 168 {  // 1 week
                0.5
            } else {
                0.2
            };
            
            // Frequency boost
            let frequency = (history.count as f32).ln() / 10.0;
            
            recency * 0.6 + frequency * 0.4
        } else {
            0.0
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let path = std::env::var("HOME")? + "/.local/state/faelight/launcher-history.json";
        let json = serde_json::to_string_pretty(&self.launches)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
```

---

## Configuration
```toml
# ~/.config/faelight/launcher.toml

[ui]
width = 600
height = 400
font_size = 12.0
max_results = 10

[providers]
apps = true
commands = true
intents = true
profiles = true

# Provider weights (optional overrides)
[weights]
apps = 1.0
commands = 0.8
intents = 0.5
profiles = 0.7

[behavior]
auto_launch_delay = 0  # 0 = disabled, >0 = ms to auto-launch if single result
fuzzy_threshold = 0.3  # Minimum score to show
risk_warnings = true   # Show warnings for high-risk commands
```

---

## Implementation Phases

### Phase 1: Foundation (Week 3, Days 1-2)

**Build the skeleton:**
- Project structure
- Provider trait
- Basic UI (faelight-core)
- Apps provider only
- No fancy features yet

**Goal:** Launch apps, basic fuzzy search

### Phase 2: Ranking (Week 3, Days 3-4)

**Implement scoring:**
- Fuzzy matcher (nucleo)
- Scoring algorithm
- Provider merging
- Result sorting

**Goal:** Accurate, fast ranking

### Phase 3: Intelligence (Week 3, Day 5)

**Add smart features:**
- Frecency tracking
- History persistence
- Context-aware scoring

**Goal:** Launcher learns

### Phase 4: Safety (Week 4, Days 1-2)

**Risk integration:**
- Commands provider
- intent-guard integration
- Risk-aware scoring

**Goal:** Launcher protects

### Phase 5: Teaching (Week 4, Days 3-4)

**Explainability:**
- Teaching mode (`?` key)
- Score explanation
- Risk warnings

**Goal:** Launcher teaches

### Phase 6: Polish (Week 4, Day 5)

**Final touches:**
- Config hot-reload
- Error handling
- Performance tuning
- Documentation

**Goal:** Production-ready

---

## Performance Targets

**Startup (cold):** <30ms  
**Query latency:** <1ms  
**Memory:** <15MB  
**Index rebuild:** Async, never blocks UI

**Measurement:**
```bash
# Cold start
time faelight-launcher --dry-run

# Query latency
# Type in launcher, observe frame time

# Memory
ps aux | grep faelight-launcher
```

---

## Success Criteria

**v2.0 Must Have:**
- [ ] Provider trait working
- [ ] Apps provider functional
- [ ] Commands provider functional
- [ ] Fuzzy matching fast (<1ms)
- [ ] Ranking algorithm accurate
- [ ] Frecency tracking persists
- [ ] Risk-aware scoring
- [ ] Teaching mode (`?` key)
- [ ] Config file support
- [ ] <30ms cold start
- [ ] Uses faelight-core
- [ ] Zero regressions from v1.0

**Nice to Have (defer if needed):**
- Intents provider
- Profiles provider
- Auto-launch on single result
- Keyboard shortcut hints
- Preview pane

---

## Why This Beats Walker

**Walker's approach:**
- Mode switching (apps mode, calc mode, etc.)
- Fast but opaque ranking
- No risk awareness
- No learning explanation

**Our approach:**
- Unified ranking (all providers merged)
- Explainable ("Press ? to see why")
- Risk-aware (intent-guard integration)
- Teaching mode (learn as you launch)
- Profile-aware (context matters)

**Result:** Not just faster. Smarter. Safer. More intentional.

---

## Timeline

**Week 3-4 of v7.0.0:**
- Week 3: Foundation + Ranking + Intelligence (5 days)
- Week 4: Safety + Teaching + Polish (5 days)

**Total: 10 days focused work**

---

_"Walker finds. Faelight understands."_ 🌲
