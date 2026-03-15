---
id: 127
date: 2026-03-15
type: future
title: "Schema Layer — Registry and Policy Validation"
status: planned
tags: [schema, validation, registry, integrity, structure, v10.9]
version: 10.9.0
priority: high
---

## Vision

Right now `tools.toml` and `sandbox-policies.toml` grow by convention.
No validation. A malformed entry is discovered at runtime — not at write time.

The schema layer catches errors at the source.

## Structure
```
0-core/
└── 04-schema/
    ├── registry.schema.json
    ├── policy.schema.json
    ├── tools.schema.json
    └── README.md
```

This becomes the 4th numbered gravity layer — between interfaces and runtime.

## What Gets Validated

### tools.toml entry
```toml
[[tool]]
name = "faelight-shell"        # required, unique
version = "0.2.0"              # required, semver
category = "shell"             # required, enum
description = "..."            # required, non-empty
expected_usage = "high"        # required, enum: high/medium/low/rare
```

### sandbox-policies.toml entry
```toml
[[policy]]
name = "untrusted"             # required, unique
allow_net = false              # required, bool
allow_fs_write = false         # required, bool
max_cpu_seconds = 60           # required, positive int
max_memory_mb = 256            # required, positive int
emit_events = true             # required, bool
description = "..."            # required, non-empty
```

## Integration

### doctor gains schema check
```
╭─ 🔒 Security
│  ✅ Schema Validation    All registry entries valid
│  ⚠️  Schema Validation    2 entries missing description
```

### core registry add validates before writing
```bash
core registry add faelight-newtoool   # validates against schema
                                       # rejects if invalid
```

### CI hook — validate on commit
Pre-commit hook runs schema validation.
Invalid registry entry cannot be committed.

## Success Criteria

- [ ] 04-schema/ directory created with JSON schemas
- [ ] doctor gains schema validation check
- [ ] tools.toml validated on every doctor run
- [ ] sandbox-policies.toml validated on every doctor run
- [ ] core registry add validates before writing
- [ ] Pre-commit hook validates registry files

---
*"The forest validates what it declares."* 🌲
