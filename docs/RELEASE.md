# Faelight Forest Release Process

**Next release:** 1.0.0 (first public NixOS release -- versioning resets; the Arch-era
v8-v14 line is retired history.)
**Updated:** 2026-06-28
**Tooling:** faelight-release v2 (NixOS-native)

The release process is intentional. Nothing is automated end-to-end -- the human decides
when the forest is ready. faelight-release v2 handles the mechanical work; the judgement
stays with you.

---

## Versioning (post-NixOS reset)

NixOS Faelight Forest starts at **1.0.0**. Semantic versioning from here:

| Type  | Example | When |
|-------|---------|------|
| Major | 2.0.0   | Architectural leap (e.g. Friday becomes autonomous) |
| Minor | 1.1.0   | Significant features, batches of intents complete |
| Patch | 1.0.1   | Bug fixes, doc updates, polish |

---

## Pre-release gate

Before cutting any release, all of these must hold:

```
d                      # health dashboard -- aim for green
git status             # working tree clean, all pushed
nix develop ~/0-core#faelight-forest -c cargo build   # workspace builds
core integrity run     # integrity 100%
```

If any fail, fix first. A version number means nothing if the forest is not healthy.

---

## Cutting a release (faelight-release v2)

Plan first (dry-run -- shows exactly what will happen, writes nothing):

```
faelight-release plan 1.0.0
```

Preview the auto-generated changelog:

```
faelight-release preview 1.0.0 --theme "Theme Name"
```

Publish (interactive TUI -- confirms, then writes):

```
faelight-release publish 1.0.0 --theme "Theme Name"
```

Publish performs:
- Builds the changelog from completed intents since the last release.
- Updates the dynamic section of the root README.md (lines 1-37 -- faelight-release's
  domain; the static body below is owned by faelight-docs).
- Updates meta/VERSION.
- Records the **release_triad** in state.db: version + NixOS generation + commit_count +
  intent_range + theme + timestamp -- so any release can later be mapped to the exact
  generation that produced it.

---

## The release triad (INT-034)

faelight-release ties three things together per release:

- **Version** (e.g. 1.0.0)
- **NixOS generation** (the exact system generation booted at publish)
- **Commit count / intent range** (what work the release contains)

Query and protect them:

```
faelight-release status          # current generation
faelight-release history         # all release generations
faelight-release query 1.0.0     # which generation is release 1.0.0?
faelight-release gc-check        # warn if a release generation risks GC
faelight-release rollback 1.0.0  # roll back to a release generation
faelight-release diff 1.0.0      # changelog diff since a version
```

This means a release is not just a tag -- it is a recoverable system generation.

---

## README ownership boundary

The root README.md has two zones, owned by different tools -- they NEVER cross:

- **Lines 1-37 (dynamic):** owned by faelight-release. Version badge, title, live forest
  state. Updated automatically on publish.
- **Lines 38+ (static):** owned by faelight-docs. Identity, philosophy, tool catalog,
  links. Updated by hand / faelight-docs sync.

---

## What a release is (and is not)

- A release is not a deadline.
- A release is not a promise to anyone.
- A release does not happen until the work is genuinely done.
- A release does not require every planned intent to be complete.

A release captures what the forest IS at this moment. The next release captures what it
becomes.

---

## After publishing

- Tag and push.
- Run `d` once more -- confirm health.
- Update any external docs or presentations referencing the version.
- Note what the next release will contain in the active intents.

The forest does not rest after a release. It continues growing.

> "Every release is a checkpoint. The work continues. The version number is a name for
> what was. The forest is always becoming." 🌲
