---
id: 145
date: 2026-08-17
type: decision
title: "what zero core owns -- the substrate boundary and the ownership overlay"
status: decided
tags: [decision]
---

## Context

Zero Core sits on NixOS. The question that decides whether it is a system or a wrapper is: **which
concepts does Core own, and which belong to the substrate?**

Get it wrong in one direction and Core becomes NixOS concepts renamed behind an API. Get it wrong
in the other and Core cannot describe its own system.

Three architecture documents proposed layer trees. None of them answered this question, because a
tree says where things live, not who owns them.

## Decision

### 1. The test

**Can this information be losslessly derived from the substrate?**

| Answer | Core's response |
| --- | --- |
| Yes | Core does not own it |
| No | Candidate for Core |
| **Partially** | **Core owns only the semantic delta, never the substrate wholesale** |

⚠️ The third row is the common case and it is where the wrapper problem lives. An earlier binary
version of this test ("would this concept survive a substrate change") was too permissive --
almost everything survives, so almost everything qualified.

### 2. The north star, and its companion

> **Zero Core is the minimum semantic layer required to preserve information that would otherwise
> disappear when intent becomes implementation.**

> **Core must not represent information that the substrate can already represent without loss.**

★ Together these shift the burden of proof. The question for any proposed Core concept is not
"is this abstraction useful" but **"what information would disappear without it?"**

If the answer is "a nicer API," do not add it. If the answer is "the reason, the decision, or the
intent," that is Core territory.

### 3. Structure and ownership are different axes

The eleven-layer tree (decision 143) answers **what exists and where does it live.**
This decision answers **who owns which kind of concern.**

Neither replaces the other. A component has a position in the tree *and* an ownership answer, and
they are independent facts.

| Attribute | Question it answers |
| --- | --- |
| Position in the eleven-layer tree | Where does it live? |
| Concern boundary (Core / Nix / substrate) | Whose concern is it? |
| First-party | Who implements it? |

⚠️ **First-party is an ownership attribute, not a layer and not a concern.** `faelight-vm` is
first-party software occupying substrate territory. `faelight-lock` is first-party software
implementing a substrate-facing mechanism. **Neither becomes Core by being ours.**

★ This is the rule that stops "first-party" becoming the next `Objects` -- a name that meant three
things and therefore nothing.

### 4. The concern boundaries

- **Core owns meaning** -- intent, decisions, rationale, semantic provenance
- **The Nix module system owns composition** -- options, merging, overrides, evaluation
- **The substrate owns execution** -- packages, services, derivations, processes, filesystem,
  systemd, compositor

And the sentence that keeps this honest:

> **Zero Core is not merely a configuration system. It is a system that contains first-party
> software, and a configuration system for that software and its substrate.**

### 5. Four words with four jobs

- **profile** = opinion -- what Zero Core believes a good system looks like
- **option** = authority -- where the user has legitimate power to disagree
- **implementation** = mechanism -- how it is actually done
- **generation** = deployed state -- what is currently true

### 6. The customization surface is never the update surface

> **Never make the user's customization surface the same artifact as the project's update surface
> when the underlying system supports declarative composition.**

⚠️ THE EVIDENCE FOR THIS RULE IS WHY WE LEFT OMARCHY -- stated precisely, because the crude version
is wrong. Omarchy does have an override boundary: its Hyprland config loads defaults from
`~/.local/share/omarchy/default/`, then user config from `~/.config/hypr/`, and its theming system
gives existing user files precedence.

The accurate claim is narrower: **Omarchy's customization boundary is partly file-based, and some
refresh and update paths still treat generated configuration as replaceable artifacts.** Issues
exist where updates replaced user Waybar and hypridle configuration, and DHH opened an issue
proposing a lock mechanism for exactly this.

★ A NixOS module has no such failure mode. The module owns the default definition; the user owns the
override; neither modifies the other's source artifact.

**Decidedness lives in profiles. Ownership lives in options.**

### 7. The brake on options

> **Expose an option when user intent is stable at the semantic level, not merely because an
> implementation has a configurable parameter.**

The decision procedure: **an option must correspond to a decision a user could defend in one
sentence.** If you cannot say why someone would reasonably want the other value, it is not an
option -- it is a constant.

Good: `zero.desktop.bar.enable`, `zero.desktop.bar.position`.
Not options: `bar.internal.cssSelector`, `widgetRendererPadding`.

⚠️ Without this brake Core becomes a giant GUI configuration API, which is the bloat this whole
architecture exists to avoid.

### 8. The ledger records decisions, never implementations

```
INTENT -> DECISION -> IMPLEMENTATION -> SUBSTRATE
```

Never the reverse. The substrate is evidence of what happened; the ledger explains why.

⚠️ An entry reading "mango.nix exists because we chose mango" is documentation, not a ledger entry.
The entry records the intent, the decision, the rationale, the alternatives rejected, and points
*at* the implementation.

★ This is what stops the ledger becoming a second source of configuration truth.

## Applied

### Provenance -- three kinds, and Core owns one

- **Artifact provenance** -- this package came from this derivation with these inputs. **Nix owns
  it.**
- **Configuration provenance** -- this option value was produced by these modules. **Nix owns it.**
- **Intent provenance** -- why does this exist, and which decision established that reason.
  **Core owns it.**

⚠️ Defining Core's provenance as "where did this come from" would duplicate the closure graph.
Defining it as "why" makes it genuinely orthogonal.

### The concepts, decided

| Concept | Owner | Note |
| --- | --- | --- |
| Intent, decisions, rationale | **Core** | No substrate counterpart exists at all |
| Generation | **Core** | ★ Already has THREE implementations here |
| Capability (authorization) | **Core** | Already implemented -- and implemented twice |
| Artifact | **Core** | Broader than a package |
| Secret -- the reason it exists | **Core, delta only** | Mechanism stays with sops |
| Identity -- principal | **Core** | Linux UID is one mapping, not the definition |
| Package | **Nix** | The store model is better than anything we would build |
| Process | **Substrate** | Core may observe, never redefine |
| Filesystem | **Substrate** | Core owns object, reference, artifact -- not VFS |
| Linux `CAP_*` | **Substrate** | Distinct from semantic capability |

### Measured, not assumed (2026-08-17)

- **Generation passes the hardest test with three existing implementations:** NixOS system
  generations, `faelight-vm` snapshot/rollback including EFI vars, and checkpoints
  (`auto-intent-222-start` recorded health, commit and tool count).
- **Capabilities are not hypothetical.**
  `ctx.capabilities.require("intent", &[Capability::FilesystemReadHome])` is live in `next_intent`.
  ⚠️ And the code exists **twice** -- `engine/src/capabilities/` and
  `engine/src/domains/capabilities/`. Two owners, unresolved, recorded here.
- **Secrets have no mechanism yet.** No sops-nix and no AppArmor anywhere in `nix/`. INT-163 lands
  first; the Core delta comes after. Defining the ontology before the mechanism would be the same
  ceremony decision 143 cut from the Model layer.

## Deliberately not decided here

- Activation guarantees and recovery. Different question, different decision.
- Directory structure. Decision 143 already ruled the tree is not the layout.
- Which of the duplicate `capabilities` owners survives.
- When any Faelight to Zero renaming happens. Decision 144 covers the registers; the migration is
  separate.

## Consequences

- Every proposed Core concept now faces one question with a checkable answer.
- Core cannot quietly grow, because "it would be a nicer API" is explicitly not a reason.
- First-party software has a home in the tree without dragging its concern into Core.
- The ledger has a defensible reason to exist that is independent of project management: it holds
  the one kind of information the substrate cannot produce.
