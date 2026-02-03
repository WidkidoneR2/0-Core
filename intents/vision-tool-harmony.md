# Tool Harmony Vision - Captured 2026-02-02

## The Insight

"I want my tools to work with one another - checking each other so everything is running in harmony"

This is the natural evolution from:
- Individual tools → Hardened tools → **Integrated ecosystem**

## Current State

Tools work independently:
- faelight-bar checks health
- dot-doctor runs checks
- faelight-stow manages symlinks
- Each tool has its own logic

## Vision: Harmonic System

### Tools That Check Each Other

**Example 1: faelight-bar + dot-doctor**
- ✅ DONE! Bar now calls doctor for health
- Shows accurate system state
- Both tools in sync

**Example 2: bump-system-version + dot-doctor**
- Could: Refuse to bump if health < 100%
- Could: Auto-run health check before version bump
- Integration: Version releases only when system is healthy

**Example 3: faelight-git + faelight-hooks**
- Could: Hooks validate before commit
- Could: Git checks hook health
- Integration: Commits only when validation passes

**Example 4: faelight-update + multiple tools**
- Could: Check with doctor before updating
- Could: Verify stow health before applying updates
- Could: Consult faelight-git for uncommitted changes
- Integration: Safe updates that respect system state

### The Architecture Pattern
```
faelight-core/
  ├── health.rs    - Shared health checking framework
  ├── paths.rs     - Centralized paths (Intent 076)
  ├── config.rs    - Shared configuration
  └── ipc.rs       - Inter-tool communication?
```

### Priority Order

1. **NOW (v8.9.0-v9.0.0):** Harden remaining 15 tools
   - Add paths.rs modules
   - Ensure CLI standards
   - Individual excellence first

2. **PHASE 2 (v9.x):** Tool Integration
   - faelight-term + faelight-fm (your stated priority!)
   - Tools can call each other's APIs
   - Shared health framework
   - Doctor becomes the "orchestrator"

3. **PHASE 3 (v10.0):** Full Ecosystem
   - Tools self-monitor
   - Automatic coordination
   - System-wide harmony checks
   - One unified health model

## Immediate Next Steps

Tomorrow:
1. Continue tool hardening (faelight-fetch, faelight-notify, etc.)
2. Get to 20-25/40 tools hardened
3. Document any integration ideas that arise

LATER (After v9.0.0):
1. Focus on faelight-term completion
2. Focus on faelight-fm completion
3. Then explore tool integration patterns
4. Then consider editor experiments

## The Big Picture

This aligns PERFECTLY with Intent 076 "Tool Ecosystem Evolution":
- Layer 1: Core Infrastructure (paths, config, errors)
- Layer 2: Domain Services (tools as libraries)
- Layer 3: Integration (tools call each other)
- Layer 4: Harmony (system self-manages)

You're thinking in SYSTEMS, not just tools!
