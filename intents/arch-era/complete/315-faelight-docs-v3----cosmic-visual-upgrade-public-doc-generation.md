---
id: 315
title: "faelight-docs v3 -- COSMIC visual upgrade, public doc generation, release sync"
status: complete
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
- [x] faelight-docs sync runs automatically on faelight-release publish -- already wired 2026-05-27
- [x] docs/public/ created -- 7 curated public docs, INT references stripped 2026-05-27
- [x] COMMAND-GUIDE always current -- faelight-docs sync keeps it live 2026-05-27

Phase 2 -- libcosmic UI:
- [x] libcosmic UI -- deferred to NixOS -- approved by: christian 2026-05-27
- [x] Browse -- deferred to NixOS -- approved by: christian 2026-05-27
- [x] Color palette -- deferred to NixOS -- approved by: christian 2026-05-27
- [x] Search -- deferred to NixOS -- approved by: christian 2026-05-27

Phase 3 -- Public generation:
- [x] docs/public/ generated with index.md -- 7 curated public docs 2026-05-27
- [x] Philosophy and Architecture included in public set 2026-05-27
- [x] faelight-docs public wired into sync -- runs on every release 2026-05-27

Final:
- [x] Demonstrated: docs/public/ generated, index clean, 7 docs curated 2026-05-27
- [x] Internal docs stay in docs/ -- public docs in docs/public/ only 2026-05-27

"The forest that documents itself
teaches others without revealing its secrets." 🌲
