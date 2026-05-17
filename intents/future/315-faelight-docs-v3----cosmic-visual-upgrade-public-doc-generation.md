---
id: 315
title: "faelight-docs v3 -- COSMIC visual upgrade, public doc generation, release sync"
status: planned
date: 2026-05-17
tags: faelight-docs, libcosmic, documentation, public, release
depends_on: [314]
blocks: []
---

## Why This Intent Exists

faelight-docs currently generates COMMAND-GUIDE from core domains.
It works but it is invisible -- a background tool with no face.
The forest's documentation layer should be as beautiful as its tools.

---

## Vision

faelight-docs v3 is the forest's public voice:
- Generates COMMAND-GUIDE (already works)
- Generates public-facing docs in sync with every release
- libcosmic UI for browsing docs locally
- Separates internal docs (philosophy, intents, workflows) from public docs

---

## Gates

Phase 1 -- Release sync:
- [ ] faelight-docs sync runs automatically on faelight-release publish
- [ ] Public doc set separate from internal doc set
- [ ] COMMAND-GUIDE always current, never stale

Phase 2 -- libcosmic UI:
- [ ] faelight-docs opens a libcosmic window
- [ ] Browse all commands, domains, aliases visually
- [ ] Forest color palette, as polished as faelight-fm
- [ ] Search across all docs

Phase 3 -- Public generation:
- [ ] Generates a public docs site structure (markdown → HTML)
- [ ] Credits and philosophy pages auto-generated
- [ ] Every release publishes updated docs

Final:
- [ ] faelight-docs is the forest's library -- beautiful, complete, always current
- [ ] Internal and public docs never mixed

"The forest that documents itself
teaches others without revealing its secrets." 🌲
