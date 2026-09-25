# 04-schema — Registry Validation Layer

JSON schemas for all 01-registry/ files.

## Files

| Schema | Validates |
|--------|-----------|
| tools.schema.json | 01-registry/tools.toml |
| zones.schema.json | 01-registry/zones.toml |
| profiles.schema.json | 01-registry/profiles.toml |
| policies.schema.json | 01-registry/sandbox-policies.toml |

## Philosophy

Errors are caught at write time, not runtime.
The forest validates what it declares.

## Validation
```bash
core doctor run     # includes schema validation check
core registry validate  # validate all registry files manually
```

## Adding New Registry Files

1. Create the TOML file in 01-registry/
2. Create matching .schema.json in 04-schema/
3. Add validation to engine/src/domains/doctor/mod.rs

*"Structure is not constraint. Structure is the forest knowing where its roots are."*
