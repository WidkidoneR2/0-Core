# Faelight Forest Release Process
**Version:** 13.1.0
**Updated:** 2026-05-08
The release process is intentional. Nothing is automated.
Every release is a checkpoint -- the forest at a moment in time.
---
| Type | Example | When |
|------|---------|------|
| Major | 14.0.0 | Architectural leap, Friday becomes central |
| Minor | 13.1.0 | Significant features, new intents complete |
| Patch | 13.0.1 | Bug fixes, doc updates, minor polish |
---
Before bumping any version number, all of these must be true:
    d
    git status
    cargo build --workspace
    core integrity run
If any of these fail, fix them first. The version number means nothing if the forest is not healthy.
---
    d
    core integrity run
    core intent list --active
    cicomplete NNN
    deploy core
    deploy faelight-shell
    deploy faelight-bar
    release 13.1.0
    Or manually:
    bump-system-version 13.1.0
    This updates:
    - VERSION file
    - README.md (version badge, title)
    - fsh welcome screen (reads VERSION at runtime)
Add entry at the top (after the header).
Write what changed in plain language -- no intent numbers.
Format:
    What shipped (2-3 sentences)
    What shipped:
    - Tool name -- what it does now
    - Feature -- why it matters
    Forest state:
    Health: 100% x Commits: NNNN x Tools: 51 x Intents: NNN complete
    fg done "release: Faelight Forest 13.1.0 -- The Forest That Knows Itself"
    lock-core
---
- A release is not a deadline.
- A release is not a promise to anyone.
- A release does not happen until the work is genuinely done.
- A release does not require every planned intent to be complete.
A release captures what the forest is at this moment.
The next release captures what it becomes.
---
After the release is tagged and pushed:
- Update any external docs or presentations that reference the version
- Run d one more time to confirm 100% health
- Note what the next release will contain in the active intents
The forest does not rest after a release. It continues growing.
"Every release is a checkpoint.
The work continues.
The version number is a name for what was.
The forest is always becoming." 🌲
