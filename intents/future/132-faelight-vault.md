---
id: 132
date: 2026-03-16
type: future
title: "faelight-vault — Forest-Native Credential Manager"
status: planned
tags: [security, vault, credentials, encryption, rust, v11]
version: 11.0.0
priority: low
depends_on: [130]
---

## Vision

A forest-native credential manager built on faelight-gen.
Every credential has a health score.
Old passwords surface in core audit.
The vault is a forest participant — not a standalone app.

## Commands
```bash
faelight-vault add github       # store credential
faelight-vault get github       # retrieve
faelight-vault list             # all entries as table
faelight-vault rotate github    # regenerate and update
faelight-vault audit            # find weak or old credentials
faelight-vault generate github  # generate and store in one step
```

## Credential Health Scores

Every credential scored 0-100:

| Factor | Weight |
|--------|--------|
| Age (days since last rotation) | 30% |
| Strength (entropy bits) | 40% |
| Type (random vs weak pattern) | 30% |
```
╭─ 🔐 Vault Audit ───────────────────────────────
│  github        score: 92  🟢  rotated: 3d ago
│  twitter       score: 45  🟡  rotated: 187d ago
│  old-server    score: 12  🔴  rotated: 412d ago
╰────────────────────────────────────────────────
```

## faelight-shell Integration
```
vault list | where score < 50 | sort score
vault list | where age > 90
```

## core advise Integration
```
→ 2 credentials older than 90 days
  Consider rotating: twitter, old-server
  Run: faelight-vault audit
```

## Security Architecture

- Encrypted at rest using age encryption
- Master key stored in system keyring
- Every access logged to state.db
- Secure memory clearing with zeroize crate
- No plaintext on disk ever

## Depends On

- faelight-gen (INT-130) — for credential generation
- Core v7 schema layer — for vault schema validation
- state.db — for audit trail and health scores

## Success Criteria

- [ ] faelight-gen integrated for generation
- [ ] age encryption for storage
- [ ] Credential health scores
- [ ] core audit integration
- [ ] core advise surfaces weak credentials
- [ ] faelight-shell pipeline support
- [ ] vault list | where score < 50

---
*"Trust, but verify. Store, but protect."* 🌲
