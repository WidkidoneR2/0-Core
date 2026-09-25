---
id: 065
date: 2026-01-25
type: decisions
title: "Adopted source-first build strategy"
status: complete
tags: [architecture, build-system, git, rust, philosophy]
---

CONTEXT:
Previously committed compiled binaries (scripts/) to git repository.
This caused ~60MB repo size and binary diffs that couldn't be reviewed.

DECISION:
Gitignore scripts/ and backups/ directories.
Build Rust tools from source on local machine only.
Git now tracks only source code in rust-tools/.

RATIONALE:
- Single-machine use case = no deployment benefit from committed binaries
- Aligns with 'Understanding over convenience' philosophy
- Keeps git repo small (~10MB vs ~60MB)
- Enables meaningful code review through source diffs
- Forces building from source (learning opportunity)

IMPLEMENTATION:
- Added scripts/ and backups/ to .gitignore
- Removed 37 binaries from git tracking (kept local files)
- Created ARCHITECTURE.md documenting complete structure
- Created BUILD.md documenting build workflow
- Workflow unchanged: edit → build → cp → commit source

IMPACT:
- Git repo size: 60MB → 10MB
- All tools still work (binaries local, not tracked)
- Future machines must have Rust toolchain installed
- bump-system-version continues working (git add -A respects gitignore)

FILES CHANGED:
- .gitignore: Added build artifact sections
- docs/ARCHITECTURE.md: New comprehensive structure doc
- docs/BUILD.md: New build workflow guide

VERIFICATION:
- System health: 100% (14/14 checks passing)
- All tools functional: dot-doctor, faelight, bump-system-version
- Git operations clean: commits, pushes successful

---
