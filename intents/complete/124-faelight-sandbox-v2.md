---
id: 124
date: 2026-03-12
type: future
title: "faelight-sandbox v2 — Forest-Aware Isolation Environment"
status: complete
tags: [sandbox, isolation, security, rust, v10.8]
version: 10.8.0
priority: medium
---

## Vision

faelight-sandbox currently provides basic process isolation.
v2 makes it a forest-aware isolation environment — every
sandboxed run is tracked, audited, and logged to the ledger.

## What v2 Adds

### Ledger Integration
Every sandbox run emits events:
```
sandbox.start   — what ran, with what args, at what time
sandbox.end     — exit code, duration, resource usage
sandbox.blocked — what was attempted but blocked
```

### Policy Engine
Declarative sandbox policies in registry:
```toml
[[sandbox.policy]]
name = "untrusted-script"
allow_net = false
allow_fs_write = false
allow_env = ["PATH", "HOME"]
emit_events = true
```

### Resource Tracking
- CPU time used
- Memory peak
- Files touched
- Network connections attempted

### `core advise` Integration
If a sandboxed tool keeps hitting resource limits or
generating blocked events — surfaces in advisory:
```
→ faelight-browser sandbox blocked network 3 times today
  Consider: review policy or restrict further
```

### Audit Trail
```bash
faelight-sandbox history          # all sandbox runs
faelight-sandbox history --today  # today's runs
faelight-sandbox audit <name>     # what did this tool do?
```

## Success Criteria

- [ ] Events emitted for every sandbox run
- [ ] Policy engine with TOML declarations
- [ ] Resource usage tracking
- [ ] History and audit commands
- [ ] `core advise` surfaces sandbox anomalies
- [ ] doctor monitors sandbox health

---
*"Trust, but verify. Isolate, but observe."* 🌲
