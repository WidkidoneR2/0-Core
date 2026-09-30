The Project 0 Philosophy

Version: 1.0.0 Last Updated: 2026-09-30 System: Omarchy (Arch + Hyprland) · Project 0 · NovaShell (nsh)

A position on how computers should behave when used by real humans.

Part I: The Landscape

This is not "which is better." This is what problem each worldview believes it is solving.
1. Traditional Linux Distros (Arch, Debian, Fedora, Ubuntu)

Core Belief: "Give users tools and freedom; they'll figure it out."

Mental Model:

Files are mutable
State is implicit
History lives in your shell history (if anywhere)
Breakage is "part of the journey"

Strengths: Transparent. Flexible. Teaches fundamentals. Encourages exploration.

Hidden Costs: No guardrails. Drift is invisible until catastrophic. Recovery knowledge is tribal, not systemic.

Relationship to the User: Assumes competence. Does not protect against fatigue, stress, or error. Treats mistakes as personal failure.

Project 0 Divergence:

Freedom without structure is not empowerment — it's entropy.

You don't remove freedom. You shape it.

Traditional Linux: "You can do anything."
Project 0: "You can do anything — but you must understand the consequences."

Omarchy is the substrate, not the product. Arch gives the machine. Project 0 decides what is allowed to live on it.

2. systemd-Centric Linux (Modern Linux Reality)

This is not just systemd the tool — it's systemd the philosophy.

Core Belief: "Systems should manage themselves."

Mental Model: Declarative units. Implicit orchestration. Background automation. "If it's running, it's fine."

Strengths: Powerful. Consistent APIs. Handles complexity at scale.

Hidden Costs: Behavior is often non-obvious. Failures can be silent or deferred. User intent is inferred, not asked. Automation runs when you are not present.

Relationship to the User: Treats the user as an administrator of policies, not an active decision-maker. Optimizes for uptime, not understanding.

Project 0 Divergence:

Automation without consent is indistinguishable from loss of control.

systemd: "The system knows best when to act."
Project 0: "Nothing acts unless a human explicitly authorizes it."

Omarchy still runs systemd. That is accepted infrastructure. It is not a license for Project 0 to start work on its own.

3. Declarative Linux (The Neighbor We Left)

Declarative Linux is the closest philosophical neighbor — and the kind of system this machine used to be.

Core Belief: "If the system is purely declarative, it becomes safe."

Mental Model: State is derived, not modified. Rebuilds are atomic. Rollbacks are cheap. Configuration is code.

Strengths: Reproducibility. Explicit state. Rollbacks as a primitive.

Hidden Costs: Cognitive overhead is very high. Indirection hides real behavior. Users trust the model more than they understand the system. Debugging requires knowing the abstraction.

Relationship to the User: Treats the user as a programmer, not necessarily a system steward. Encourages correctness over comprehension.

Project 0 Divergence:

- A declarative system believes: "If the model is correct, the system is safe."
- Project 0 believes: "If the human understands the system, it is safe."

| Declarative Linux | Project 0 |
|---|---|
| Declarative purity | Intentional stewardship |
| Reproducibility | Recoverability |
| Abstraction | Explicitness |
| Trust the system | Trust the human |
| Correctness | Comprehension |

Declarative Linux eliminates classes of mistakes. Project 0 assumes mistakes are inevitable and designs around them.

The move to Omarchy was not a rejection of those strengths. It was an admission that understanding has a budget, and that a system which claims to understand itself has to be small enough for one person to keep honest.

4. The Core Insight

All systems choose who they trust:

Windows/macOS → Trust the vendor
systemd Linux → Trust the automation
Declarative Linux → Trust the model
Traditional Linux → Trust the user (without support)

Project 0 Makes a New Choice: Trust the user — and support them when they fail.

That's the gap it fills.

Part II: The Manifesto

Project 0 is not a distro. Project 0 is not a framework. Project 0 is not a set of dotfiles.

Project 0 is a position on how computers should behave when used by real humans.

The public name is Project 0. The repository stays 0-Core. The shell is NovaShell (nsh).

I. We Reject Invisible Complexity

If a system does something without the user understanding why, that behavior is a bug — even if it "works."

Magic is not a feature. Silence is not safety.

A tool that cannot answer must say so, rather than reporting an answer it never established.

II. We Value Intent Over Automation

Automation is not inherently good. Automation without consent is dangerous.

Nothing in a Project 0 system runs:

Without being explicitly triggered
Without declaring its intent
Without exposing its failure modes

Convenience must never outrank clarity.

III. We Treat the User as a Steward, Not a Consumer

A Project 0 user is not protected from the system. They are supported by it.

The system assumes:

You will make mistakes
You will forget things
You will work while tired, stressed, or distracted

Design that ignores this reality is negligent.

IV. We Design for Recovery, Not Perfection

Breakage is not shameful. Recovery is not optional.

Every action must have:

A visible blast radius
A documented rollback path
A clear failure signature

A system that cannot be recovered under pressure is not robust.

A sandbox that cannot keep its laws refuses to start. It never starts anyway and hopes. The host must survive the world being deleted.

V. We Prefer Explicit Structure Over Implicit Freedom

Freedom without structure decays into entropy.

Project 0 uses:

Honest names (nsh, not a colliding abbreviation; zero-* crates, not a digit the toolchain refuses)
Real directories under ~/.config/zero, ~/.local/state/zero, ~/.cache/zero, ~/.local/share/zero and ~/.config/nsh — one name each, with no second path to drift
Structural protection (LUKS, git history, written sandbox laws) instead of a store that pretends mutation cannot happen

Not to restrict freedom — but to preserve it over time.

VI. We Expose Assumptions

Every configuration makes assumptions. Hidden assumptions are technical debt.

A Project 0 system declares:

What it assumes
What it cannot guarantee
What it will not attempt to do

Honesty is a feature. Absent capabilities degrade visibly. Runtime proof, not a feature flag.

VII. We Optimize for Human Comprehension

Performance is secondary. Aesthetics are optional. Understanding is mandatory.

If a system cannot be explained simply, it is too complex to be trusted.

The surface stays small enough to maintain with AI assistance. That is a constraint, not an apology.

VIII. We Reject "Set and Forget"

Systems drift. Humans change. Contexts evolve.

A healthy system requires stewardship. A neglected system will fail — eventually.

Project 0 embraces this reality instead of hiding it.

IX. We Do Not Seek Mass Adoption

Project 0 is not for everyone.

It is for those who want:

Agency over convenience
Clarity over magic
Responsibility over illusion

Depth matters more than scale.

X. We Believe Computers Should Be Respectful

A respectful system:

Asks before acting
Speaks when uncertain
Fails loudly
Recovers gracefully
Never lies about what it is doing

This is not nostalgia. This is maturity.

Part III: Philosophy in Practice

These principles are not theoretical. They show up as design decisions:
Intent Ledger

Every major decision is documented in intents/. Not just what changed, but why. Gates are checked only when the thing they describe has been demonstrated on real data.
Health / Doctor

doctor does not just check — it explains. PASS, FAIL, and UNDETERMINED are different states. A check that cannot tell the difference is not a check.
Safety Guard

Destructive work is challenged on every execution door, including -c and every executing segment of a chain. Two doors that disagree about the language are a bug.
Isolation

devshell / bwrap is the place to break things. Live 0-Core is inside it. Credentials are not. Leaving undoes everything that was not promoted by name.
Updates

Nothing updates itself. Impact is shown. Confirmation is required. The shell executes; the human decides.
Friday

Friday earns trust through accuracy. Never acts without approval. Confidence is explicit. Trust decays on wrong predictions. Silence when uncertain.
Human Vocabulary

nsh speaks human first. UNIX is the fallback. delete before rm. The human should not have to translate intent into machine syntax before the machine will listen.

XI. The Shell Speaks Human First

Human language is the primary interface. UNIX is the fallback. The shell learns the human. Not the other way around.
XII. Trust Is Earned, Not Granted

Intelligence without accountability is noise.

Friday speaks when it:

Has something worth saying
Has confidence to stand behind it
Has earned the right through demonstrated accuracy

One signal per context — never a stream.

A partner that earns its voice is more valuable than one that always speaks.

Closing Statement

Project 0 is a refusal to accept:

Forced automation
Hidden state
Infantilized users
Opaque systems

It is a reminder that:

A personal computer should be understandable in its entirety.

Not because it must be. But because it can be.

Related Documentation

Theory of Operation: docs/THEORY_OF_OPERATION.md
Policies: docs/POLICIES.md
Intent Ledger: intents/
Architecture: docs/ARCHITECTURE.md
Shell Philosophy: docs/NSH-PHILOSOPHY.md


Manual control over automation. Understanding over convenience. Intent over convention. Recovery over perfection.