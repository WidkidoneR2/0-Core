# 🌲 Faelight Forest Strategic Roadmap

## Current Status: v9.6.0
- 12/38 tools production-ready (32%)
- Health: 100%
- Path Resilience: 100%

---

## 🎯 Phase 1: Tool Audit Completion (CURRENT)
**Target:** v9.7.0-9.9.0
- Complete remaining 26 tools to production status
- Goal: 80%+ production coverage

---

## 🏛️ Phase 2: Tool Stability Tiers (POST-AUDIT)
**Target:** v10.0.0

### Tier System
```
Tier 0: CORE (Frozen APIs)
├── paths system
├── dot-doctor
├── faelight-stow
├── faelight-hooks
└── core-protect

Tier 1: STABLE INTERFACES
├── faelight-fm
├── faelight-bar
├── faelight-git
├── intent system
└── profile system

Tier 2: EXPERIMENTAL
├── New features
├── Active development
└── Breaking changes allowed

Tier 3: PLAYGROUND
├── Proof of concepts
├── Learning projects
└── No stability guarantees
```

### Benefits
- Prevents cascading refactors
- Clear upgrade paths
- Dependency confidence
- API stability guarantees

---

## 🔒 Phase 3: Security Hardening
**Target:** v10.1.0-10.2.0

### Integrated Security
1. **cargo audit** in git hooks
   - Pre-commit vulnerability scanning
   - Automated dependency updates
   - Security advisory tracking

2. **Dependency Minimalism**
   - Track dependency count per tool
   - Minimize attack surface
   - Prefer std library

3. **Reproducible Builds**
   - Lock file verification
   - Binary hash tracking
   - Build environment documentation

4. **Custom Secrets Scanner** (Advanced Project)
   - Beyond gitleaks
   - AST-aware scanning for Rust
   - Entropy scoring
   - Git history traversal
   - False positive management
   - Risk scoring system

---

## 🚨 Phase 4: Bus Factor Mitigation
**Target:** v10.3.0
**Purpose:** System survives 2-year absence

### Cold Boot Recovery Document
```
# From Zero to 0-Core
1. Fresh Arch Linux install
2. Clone repository
3. Run bootstrap script
4. Stow packages
5. Verify with doctor
6. Done in <30 minutes
```

### Philosophy Compression (5 pages max)
```
1. Why 0-Core exists
2. Core principles
   - Manual control over automation
   - Understanding over convenience
   - Complete ownership
3. Architecture overview
4. Tool communication patterns
5. Decision-making framework
```

### Architectural Dependency Map
- Visual graph of tool dependencies
- Critical path identification
- Bottleneck detection
- Refactor impact analysis

---

## 🏗️ Phase 5: Tool Communication Infrastructure
**Target:** v10.4.0-10.5.0

### Communication Patterns
1. **Shared Event Bus**
   - Tools publish/subscribe to events
   - Decoupled communication
   - `faelight-daemon` expansion

2. **Common State Directory**
```
   ~/.local/state/faelight/
   ├── health/
   ├── events/
   ├── locks/
   └── shared/
```

3. **IPC Enhancement**
   - Unix sockets for fast IPC
   - Message queue system
   - Tool discovery protocol

4. **Unified Logging**
   - Structured logs (JSON)
   - Central aggregation
   - Query interface

5. **Tool Dependency Graph**
   - Runtime dependency tracking
   - Visualization with graphviz
   - Impact analysis

---

## 🎓 Learning Projects (Ongoing)

### Security Focus
- [ ] Pattern engine design
- [ ] AST parsing for security
- [ ] Entropy analysis
- [ ] Risk scoring algorithms

### Systems Focus
- [ ] IPC mechanisms (sockets, queues)
- [ ] Event-driven architecture
- [ ] Distributed state management
- [ ] Process supervision

---

## 📊 Success Metrics

### Technical Excellence
- 100% production coverage
- Zero clippy warnings
- <100ms tool startup time
- 100% test coverage (future)

### Sustainability
- Cold boot recovery tested quarterly
- Documentation up-to-date
- Bus factor > 1 (others can maintain)
- Philosophy document complete

### Presentation Readiness
- Every decision justifiable
- Architecture clearly documented
- Live demos work flawlessly
- Security story compelling

---

## 🎯 Priority Order (Post-Audit)

1. **Tier System** (v10.0.0)
   - Prevents chaos during growth
   - Foundation for everything else

2. **Bus Factor Mitigation** (v10.3.0)
   - Documentation is urgent
   - Enables collaboration
   - Personal safety net

3. **Security Hardening** (v10.1.0)
   - cargo audit first (quick win)
   - Custom scanner (learning project)

4. **Tool Communication** (v10.4.0)
   - After tier system stabilizes
   - Requires stable APIs

---

## 💡 Guiding Principles

1. **Every change must serve the presentation**
   - Can you explain it to Linus?
   - Does it demonstrate mastery?

2. **Sustainability over features**
   - You must be able to maintain this
   - Others should understand it

3. **Security through comprehension**
   - Custom code = custom responsibility
   - Minimize dependencies
   - Understand everything

4. **Manual control over automation**
   - Intentional > convenient
   - Observable > magical

---

**This roadmap is a living document.**
**Update after each major milestone.**

