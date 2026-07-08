---
id: 128
date: 2026-07-07
type: future
title: "Friday knowledge as native/foreign-aware data (db-backed, learnable languages)"
status: complete
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

## Scope (locked at cistart 2026-07-08, after 3 recon passes)
Recon rescoped this from the charter's "unify three stores" (wrong) to "structure the
FACTS table" (correct). Findings:
- friday_knowledge = 615 rows, but 469 are `session_summary` (a session log, NOT facts).
  The real knowledge is ~146 rows across build/nixos/rust/wayland/etc.
- Those fact rows ALREADY carry an implicit translation pattern: some rows have
  key = foreign phrasing ("pacman -Syu...") and fact = native answer ("nixos-rebuild...").
  128 makes that implicit structure EXPLICIT via columns.
- knowledge_entries (19 rows) is a SEPARATE situated-lessons engine (error_signature,
  success/failure, last_seen) -- LEAVE IT. friday_language (6 rows) is Friday's own coined
  vocabulary (health-check-loop, etc.) -- LEAVE IT. session_summary rows -- LEAVE (note:
  arguably mis-homed; hygiene for a later intent, not 128).
128 = add native/foreign/translation structure to friday_knowledge's FACT rows only.

## Success Criteria (locked)
- [x] friday_knowledge_meta companion table added (system/kind/translates_to, keyed on domain,key); base table untouched (615 rows intact), fresh-db-safe per INT-104 <!-- gate 1: proven live gen 322 -- companion table, not ALTER, matching INT-104 schema discipline -->
- [x] Existing fact rows backfilled: 146 meta rows (131 forest fact / 12 nixos native / 3 nixos translation); gap-check 0 unlabeled; base table untouched <!-- gate 2: labeled from evidence per INT-104 discipline; 3 pacman rows mapped to short native cmds in translates_to -->
- [x] sync_knowledge_meta() in ensure_tables auto-labels any fact row lacking meta (seed-agnostic, self-healing); PROVEN from empty: deploy reconstructed 131 forest / 12 nixos / 3 translation, 0 unlabeled <!-- gate 3: central labeler beats per-seed writes -- multiple seed paths + legacy db rows; INSERT OR IGNORE fills gaps only -->
- [x] Other stores untouched by 128: knowledge facts 146 (unchanged), knowledge_entries 19, friday_language 6; 128 touched ONLY the new companion table. (session_summary went 469->470 via the normal session roll -- external to 128, not our write.) <!-- gate 7: honest verify -- the +1 is an auto session log, not an INT-128 change; facts + other stores pristine -->

<!-- SCOPE CLOSE (2026-07-08): 128 delivers the native/foreign/translation DATA LAYER
     (gates 1-3, proven). The BEHAVIORAL gates below were moved to the follow-on intent
     because the metadata has no consumer yet -- surfacing/translating/teaching requires
     a teach mechanism that is itself feature-sized work (a "teaching path"). Removed here,
     not skipped -- rehomed to the follow-on:
       - Friday teaches only native facts as "this system"
       - Friday recognizes + translates a foreign command via a DATA row
       - A new fact/translation via db write is live with NO rebuild (the "learnable" proof)
     Follow-on: "Friday teaching path: bidirectional knowledge feedback loop" (INT-NNN,
     filled in once numbered). 128 is the foundation that intent builds on. -->

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
