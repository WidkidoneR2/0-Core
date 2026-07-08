---
id: 128
date: 2026-07-07
type: future
title: "Friday knowledge as native/foreign-aware data (db-backed, learnable languages)"
status: planned
tags: [Friday, faelight-shell, fsh, foreign aware, learnable]
---

## Why (the problem INT-117 exposed)
INT-117 (de-Arch Friday's language) revealed the real issue is STRUCTURAL, not textual.
Friday's knowledge lives as a flat hardcoded Rust literal:
    let knowledge = vec![
        ("arch", "pacman -Syu updates all packages...", 0.95),
        ("wayland", "...", 0.90),
        ...
    ];
(engine/src/domains/friday/mod.rs ~L961). Facts are (category, text, confidence) tuples
keyed by a bare string. This structure has three long-term costs:
1. Every language/OS shift = editing Rust source + rebuilding core (117 was manual grind).
2. No notion of NATIVE vs FOREIGN. "pacman is Arch" and "update is nix" sit at the same
   level -- Friday can't express "this is my system's way" vs "this is another system's
   way I recognize." The bilingual model (recognize foreign, translate to native) has to
   be hand-wired per string instead of being a property of the data.
3. Knowledge can't be LEARNED. New facts require a code change + deploy, not a db write.
   This blocks the "Friday learns different languages over time" vision.

## The Core Idea
Knowledge facts become DATA (in state.db), not a code literal, with a native/foreign
dimension and explicit translation pairs. Friday's bilingualism becomes structural:
Nix is native; Arch (and future systems) are foreign-but-recognized, each foreign term
optionally carrying a translation to the native equivalent.

## Proposed shape (design sketch -- refine at implementation)
A knowledge schema roughly like:
    fact(id, domain, text, confidence, system, kind, native_of, translates_to)
where:
  - system        = "nixos" (native) | "arch" | "debian" | ... (foreign-known)
  - kind          = "concept" | "command" | "translation"
  - native_of     = which system this is THE way for (nixos facts are native_of nixos)
  - translates_to = for a foreign command, the native fact/command it maps to
                    (e.g. arch:"pacman -Syu"  translates_to  nixos:"update")
This lets Friday: (a) teach only native facts as "your system", (b) RECOGNIZE foreign
terms, (c) TRANSLATE a foreign term to the native way when you type it ("you typed
pacman -- on this system that's `update` then `deploy`"), (d) LEARN new systems by
inserting rows, no rebuild.

## Migration path (incremental, non-breaking)
1. Add a knowledge table to state.db (schema above). Keep the existing vec! as a SEED
   loader -- migrate the hardcoded facts into rows on first run (idempotent upsert).
2. Tag each seeded fact with system/kind (the 117 work already sorts arch vs nixos vs
   systemd -- reuse that judgment as the seed data).
3. Add translation rows for the foreign commands 117 identified (pacman->update, etc).
4. Point Friday's fact lookup at the db table instead of the vec!.
5. Retire the vec! literal once the table is the source of truth.
Each step build-gated; the vec! stays as fallback until the table proves out.

## Connection to open architectural gaps
- Ties into INT-118 (Friday engine resumption) -- knowledge-as-data is foundational to
  a resumable/learning Friday.
- Aligns with the noted gaps: no memory decay on state.db, prediction feedback loop not
  closed. A db-backed fact table with confidence + last-seen is the natural place decay
  and reinforcement would live.
- ENABLES "Friday learns different languages" -- the long-term vision stated 2026-07-07.

## Success Criteria (draft -- refine at cistart)
- [ ] Knowledge fact schema in state.db with system/kind/translation fields
- [ ] Existing hardcoded facts migrated to rows (idempotent seed), vec! retired or fallback-only
- [ ] Friday teaches only native (nixos) facts as "this system"
- [ ] Friday recognizes a foreign command and translates it to the native way
- [ ] A new fact can be added via db write (no core rebuild) -- demonstrated
- [ ] Adding a new "language" (system) is data-only -- demonstrated with a minimal example

## Relationship
- FOLLOWS INT-117 (which does the textual de-Arch within the current structure -- ships
  the bilingual BEHAVIOR by hand; 128 makes it STRUCTURAL so it scales).
- Related INT-118 (Friday engine resumption), and the memory-decay / prediction-feedback
  gaps. NOT a 1.0.0 blocker: current flat structure works; this is the scalable foundation.

## The Rule
"A forest that only speaks one language forgets it was ever planted elsewhere. Teach the
 native tongue -- but remember how to translate. And keep the words as data, so learning
 a new language is a seed, not a rebuild." 🌲

friday_knowledge (state.db) is a SEPARATE, richer knowledge store from both the
friday/mod.rs vec! seed AND the knowledge_entries table (core knowledge search).
It ALREADY does foreign->native translation: rows where key = Arch phrasing
("pacman -Syu updates...") and fact = nix answer ("nixos-rebuild switch upgrades...").
So 128's native/foreign/translation model PARTLY EXISTS in the db already -- 128 should
unify these three stores and formalize the translation pattern that friday_knowledge
already demonstrates. Also: re-keying a fact (arch->nixos) in the vec! seed leaves the
old-domain rows ORPHANED in the db (insert-not-upsert). 128 must handle re-key cleanup
so knowledge edits reach the runtime, not just the source.
