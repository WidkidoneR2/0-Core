# The 0-Core Philosophy

**Version:** 8.4.0  
**Last Updated:** 2026-01-27  
**System:** Arch Linux + Sway + 0-Core

*A position on how computers should behave when used by real humans.*

---

## Part I: The Landscape

This is not "which is better." This is what problem each worldview believes it is solving.

### 1. Traditional Linux Distros (Arch, Debian, Fedora, Ubuntu)

**Core Belief:** "Give users tools and freedom; they'll figure it out."

**Mental Model:**
- Files are mutable
- State is implicit
- History lives in your shell history (if anywhere)
- Breakage is "part of the journey"

**Strengths:**
- Transparent
- Flexible
- Teaches fundamentals (eventually)
- Encourages exploration

**Hidden Costs:**
- No guardrails
- Drift is invisible until catastrophic
- Users confuse freedom with lack of structure
- Recovery knowledge is tribal, not systemic

**Relationship to the User:** Assumes competence. Does not protect against fatigue, stress, or error. Treats mistakes as personal failure.

**0-Core Divergence:**

*0-Core says: Freedom without structure is not empowerment — it's entropy.*

You don't remove freedom. You shape it.

**Key difference:**
- **Traditional Linux:** "You can do anything."
- **0-Core:** "You can do anything — but you must understand the consequences."

---

### 2. systemd-Centric Linux (Modern Linux Reality)

This is not just systemd the tool — it's systemd the philosophy.

**Core Belief:** "Systems should manage themselves."

**Mental Model:**
- Declarative units
- Implicit orchestration
- Background automation
- "If it's running, it's fine"

**Strengths:**
- Powerful
- Consistent APIs
- Handles complexity at scale
- Enables modern desktop expectations

**Hidden Costs:**
- Behavior is often non-obvious
- Failures can be silent or deferred
- User intent is inferred, not asked
- Automation runs when you are not present

**Relationship to the User:** Treats the user as an administrator of policies, not an active decision-maker. Optimizes for uptime, not understanding.

**0-Core Divergence:**

*0-Core says: Automation without consent is indistinguishable from loss of control.*

**Key difference:**
- **systemd:** "The system knows best when to act."
- **0-Core:** "Nothing acts unless a human explicitly authorizes it."

This is where the **12-hour password incident** becomes foundational, not anecdotal. systemd optimizes for machines. 0-Core optimizes for humans under real conditions.

*See: `docs/INCIDENTS/2025-12-14-password-sudo-failure.md`*

---

### 3. NixOS (The Most Interesting Contrast)

NixOS is the closest philosophical neighbor — and the most important comparison.

**Core Belief:** "If the system is purely declarative, it becomes safe."

**Mental Model:**
- State is derived, not modified
- Rebuilds are atomic
- Rollbacks are cheap
- Configuration is code

**Strengths:**
- Reproducibility
- Explicit state
- Rollbacks as a primitive
- Extremely powerful at scale

**Hidden Costs:**
- Cognitive overhead is very high
- Indirection hides real behavior
- Users trust the model more than they understand the system
- Debugging requires knowing the abstraction

**Relationship to the User:** Treats the user as a programmer, not necessarily a system steward. Encourages correctness over comprehension.

**0-Core Divergence:**

This is the critical difference:
- **NixOS believes:** "If the model is correct, the system is safe."
- **0-Core believes:** "If the human understands the system, it is safe."

**Key contrast:**

| NixOS | 0-Core |
|-------|--------|
| Declarative purity | Intentional stewardship |
| Reproducibility | Recoverability |
| Abstraction | Explicitness |
| Trust the system | Trust the human |
| Correctness | Comprehension |

NixOS eliminates classes of mistakes. **0-Core assumes mistakes are inevitable and designs around them.**

---

### 4. The Core Insight

All systems choose who they trust:

- **Windows/macOS** → Trust the vendor
- **systemd Linux** → Trust the automation
- **NixOS** → Trust the model
- **Traditional Linux** → Trust the user (without support)

**0-Core Makes a New Choice:** Trust the user — and support them when they fail.

That's the gap 0-Core fills.

---

## Part II: The 0-Core Manifesto

0-Core is not a distro.  
0-Core is not a framework.  
0-Core is not a set of dotfiles.

**0-Core is a position on how computers should behave when used by real humans.**

---

### I. We Reject Invisible Complexity

If a system does something without the user understanding why, that behavior is a bug — even if it "works."

**Magic is not a feature.**  
**Silence is not safety.**

---

### II. We Value Intent Over Automation

Automation is not inherently good. Automation without consent is dangerous.

**Nothing in a 0-Core system runs:**
- Without being explicitly triggered
- Without declaring its intent
- Without exposing its failure modes

**Convenience must never outrank clarity.**

---

### III. We Treat the User as a Steward, Not a Consumer

A 0-Core user is not protected from the system. They are **supported** by it.

**The system assumes:**
- You will make mistakes
- You will forget things
- You will work while tired, stressed, or distracted

Design that ignores this reality is negligent.

---

### IV. We Design for Recovery, Not Perfection

Breakage is not shameful. Recovery is not optional.

**Every action must have:**
- A visible blast radius
- A documented rollback path
- A clear failure signature

A system that cannot be recovered under pressure is not robust.

---

### V. We Prefer Explicit Structure Over Implicit Freedom

Freedom without structure decays into entropy.

**0-Core uses:**
- Semantic naming (`wm-sway`, not `hypr`)
- Numbered priority (`~/0-core`, `~/1-src`, `~/2-projects`)
- Immutable boundaries (`lock-core`, `unlock-core`)

Not to restrict freedom — **but to preserve it over time.**

---

### VI. We Expose Assumptions

Every configuration makes assumptions. Hidden assumptions are technical debt.

**A 0-Core system declares:**
- What it assumes
- What it cannot guarantee
- What it will not attempt to do

**Honesty is a feature.**

---

### VII. We Optimize for Human Comprehension

Performance is secondary.  
Aesthetics are optional.  
**Understanding is mandatory.**

If a system cannot be explained simply, it is too complex to be trusted.

---

### VIII. We Reject "Set and Forget"

Systems drift. Humans change. Contexts evolve.

A healthy system requires **stewardship**. A neglected system will fail — eventually.

0-Core embraces this reality instead of hiding it.

---

### IX. We Do Not Seek Mass Adoption

0-Core is not for everyone.

**It is for those who want:**
- Agency over convenience
- Clarity over magic
- Responsibility over illusion

**Depth matters more than scale.**

---

### X. We Believe Computers Should Be Respectful

**A respectful system:**
- Asks before acting
- Speaks when uncertain
- Fails loudly
- Recovers gracefully
- Never lies about what it is doing

**This is not nostalgia. This is maturity.**

---

## Part III: Philosophy in Practice

These principles are not theoretical. They manifest in concrete design decisions:

### Intent Ledger
Every major decision is documented in `INTENT/`. Not just *what* changed, but *why*.

**Example:** Intent 035 documents the Sway migration after the Omarchy catastrophe. Not to shame the past, but to learn from it.

### Health Monitoring
`doctor` doesn't just check — it **explains**. 14 checks with detailed rationale, dependency graphs, and fix instructions.

**Principle:** Transparency over magic.

### Git Hooks
`faelight-hooks` scans for secrets, merge conflicts, and validates commit messages *before* damage occurs.

**Principle:** Recovery over perfection.

### Core Protection
`lock-core` makes `~/0-core` immutable at the filesystem level. You must **explicitly** choose to edit.

**Principle:** Explicit structure over implicit freedom.

### Update Management
`faelight-update` never runs automatically. It shows impact analysis, version changes, and requires confirmation.

**Principle:** Intent over automation.

### Manual Triggers
No cron jobs. No systemd timers at boot. Everything runs **when you decide**.

**Principle:** Manual control over automation.

---

## Closing Statement

0-Core is a refusal to accept:
- Forced automation
- Hidden state
- Infantilized users
- Opaque systems

**It is a reminder that:**

*A personal computer should be understandable in its entirety.*

Not because it must be.  
But because it can be.

---

## Related Documentation

- **Theory of Operation:** `docs/THEORY_OF_OPERATION.md`
- **Policies:** `docs/POLICIES.md`
- **Intent Ledger:** `INTENT/`
- **Architecture:** `docs/ARCHITECTURE.md`

---

*Manual control over automation.*  
*Understanding over convenience.*  
*Intent over convention.*  
*Recovery over perfection.* 🌲
