---
id: 228
title: "faelight-docs v2 -- Deploy Intelligence for All Documents"
status: planned
date: 2026-04-13
tags: faelight-docs, intelligence, github, documentation
requires: []
unlocks: []
strategic_value: leaf
---
faelight-docs v1 syncs the README.
It updates version numbers, tool counts, intent counts.
It is a find-and-replace engine with intent awareness.
That is not enough.
Every document in the forest has a source, an owner, an intent, and a freshness.
faelight-docs v2 tracks all of them.
Like deploy tracks binary versions and records every deploy to state.db,
faelight-docs v2 tracks every document update and records it.
Like deploy warns before a risky deploy,
faelight-docs v2 warns when a document is stale.
Like deploy emits signals so the forest knows what changed,
faelight-docs v2 emits signals when docs drift from their source.
Every managed document has a manifest entry:
  name: COMMAND-GUIDE.md
  owner: INT-202
  source: generated from domains + intent completions
  freshness: 2026-04-13
  auto_update: partial (version/dates only)
  manual_required: section additions, new command docs
  fdocs status         — all documents, freshness, drift indicators
  fdocs sync           — sync all auto-updatable fields
  fdocs check          — identify stale documents
  fdocs record <doc>   — record a manual update to state.db
  fdocs log            — history of all document updates
  fdocs why <doc>      — which intent owns this document
The README is the first thing Linus and Graydon see.
It must be accurate, beautiful, and purposeful.
v2 manages the README as a living document with sections:
  DYNAMIC: version, health, intents, tools (auto-updated by bump)
  SEMI-DYNAMIC: latest release, recent capability additions (updated by fdocs sync)
  MANUAL: philosophy, architecture description, the human voice
fdocs sync never touches MANUAL sections.
fdocs check warns when DYNAMIC sections are stale.
fdocs diff shows what changed since last sync.
Every fdocs sync emits to engine_signals:
  source: faelight-docs, signal_type: doc_synced
  payload: document name, sections updated, freshness
Friday uses this to know: documentation is current.
Friday can say: "COMMAND-GUIDE has not been updated since INT-221 completed."
⬜ document registry defined (all managed docs have manifest entry)
⬜ fdocs status shows all docs with freshness and drift
⬜ fdocs sync updates all auto-updatable fields correctly
⬜ fdocs check identifies stale documents
⬜ fdocs record logs manual updates to state.db
⬜ fdocs log shows full document update history
⬜ fdocs why <doc> shows owning intent
⬜ README sections correctly categorized (dynamic/semi/manual)
⬜ fdocs sync never touches MANUAL sections
⬜ engine_signals emission on every sync
⬜ GitHub README accurate and presentable for Linus/Graydon
⬜ d passes 100% after full implementation
"A system that cannot document itself clearly
cannot explain itself to others.
faelight-docs v2 is the forest finding its public voice.
Every document current. Every section owned.
Nothing stale. Nothing forgotten." 🌲
