# 🗺️ Faelight FM Development Roadmap

## Current Status: v0.1.0-beta
✅ Navigation working
✅ Zone detection basic
✅ Color system implemented
✅ Clean architecture

---

## Phase 3 — Attach Semantics (v0.2.0-v0.3.0)

**Intent Integration:**
```
📦 package.toml   [INT: 021] [Z: src] [✔ HEALTH]
📂 0-core/        [✘ Immutable]
```

**Integration Points:**
- `intent/` module → Read Intent Ledger
- `zones/` module → Use `faelight-zone` library
- `health/` module → Query `dot-doctor`
- `model/entry.rs` → Add intent_id, immutable flag

**Visual Indicators:**
- Intent tags `[INT: XXX]`
- Zone badges with colors
- Health status `✔ ⚠ ✘`
- Immutability warnings

---

## Phase 4 — Safety Guardrails (v0.4.0)

**Pre-Action Checks (actions/ module):**
```rust
pub fn delete(entry: &FaelightEntry) -> Result<ActionResult> {
    // 1. Check zone protection
    if entry.zone.is_immutable() {
        return Err(ZoneViolation);
    }
    
    // 2. Check intent attachment
    if let Some(intent_id) = entry.intent_id {
        warn!("File tagged with Intent {}", intent_id);
    }
    
    // 3. Check snapshot availability
    if !snapshot::is_available() {
        return Err(NoSnapshot);
    }
    
    // 4. Explicit confirmation
    prompt_confirmation(entry)?;
    
    // 5. Perform deletion
    fs::remove_file(&entry.path)?;
}
```

**Confirmation Prompts:**
```
⚠ You're about to delete a file tagged with Intent 042
Dispose safely via snapshot? (Y/n)
```

**Required Checks:**
- [ ] Zone immutability
- [ ] Intent attachment
- [ ] Health conflicts
- [ ] Snapshot availability
- [ ] User confirmation

---

## Phase 5 — Intent-Driven Actions (v0.5.0)

**Command Mode (command/ module):**

Instead of direct deletion:
```bash
delete foo.txt  # BLOCKED
```

Require semantic detachment:
```bash
faelight-fm> intent detach foo.txt
faelight-fm> delete foo.txt --force
```

**Commands to Implement:**
```rust
enum Command {
    IntentDetach { path: PathBuf },
    IntentShow,
    Delete { path: PathBuf, force: bool },
    Move { from: PathBuf, to: PathBuf },
    Copy { from: PathBuf, to: PathBuf },
    SnapshotCreate { message: String },
}
```

**Intent Commands:**
- `intent show` - Display intent for selected file
- `intent detach` - Remove intent association
- `intent attach <ID>` - Associate with intent
- `intent explain` - Show why file exists

---

## Phase 6 — Profile-Aware Behavior (v0.6.0)

**Profile Integration:**

| Profile    | Behavior                           |
|------------|------------------------------------|
| Work       | Hide Archives, show intents        |
| Gaming     | Show media first, hide work dirs   |
| Lowpower   | No previews, minimal colors        |
| Default    | Balanced view                      |

**Implementation in ui/:**
```rust
// ui/filelist.rs
pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    let profile = get_current_profile();
    
    let mut items: Vec<ListItem> = app.entries
        .iter()
        .filter(|e| should_show(e, profile))  // Profile filtering
        .map(|e| format_entry(e, profile))     // Profile styling
        .collect();
    
    if profile == Profile::Gaming {
        items.sort_by_key(|e| !e.is_media());  // Media first
    }
}
```

**Profile Settings:**
- **work**: `show_intents: true, hide_archive: true, dim_media: true`
- **gaming**: `media_first: true, hide_work: true, bright_colors: true`
- **lowpower**: `simple_icons: true, no_highlights: true, basic_colors: true`
- **default**: `balanced: true`

---

## Integration Timeline

**Week 1-2: Phase 3 (Semantics)**
- Intent Ledger integration
- Zone library integration  
- Health checks integration
- Visual indicators

**Week 3: Phase 4 (Safety)**
- actions/ module implementation
- Confirmation prompts
- Snapshot checks
- Immutability enforcement

**Week 4: Phase 5 (Intent Actions)**
- Command mode implementation
- Intent detachment workflow
- Semantic operations

**Week 5: Phase 6 (Profiles)**
- Profile detection
- Conditional rendering
- Profile-specific defaults

---

## Technical Notes

**Module Readiness:**
- `model/` - ✅ Ready (needs intent_id field)
- `zones/` - ✅ Ready (needs library integration)
- `intent/` - 📝 Placeholder (needs implementation)
- `health/` - 📝 Placeholder (needs implementation)
- `actions/` - 📝 Placeholder (needs implementation)
- `command/` - 📝 Placeholder (needs implementation)

**Dependencies to Add:**
- `faelight-zone` (already in 0-core)
- Intent Ledger parsing
- dot-doctor IPC or file parsing
- Profile system integration

---

**Philosophy Reminder:**
> "If a feature doesn't have a place to live, it doesn't get added."

Every phase respects the module boundaries. No feature creep.
