---
id: 302
title: "tailspin evaluation -- log colorizer forest integration"
status: complete
date: 2026-05-14
type: eval
tags: [tailspin, logs, colorizer, fsh, vocabulary, tools]
depends_on: [300]
---
## What Is tailspin
tailspin (tspin) is a Rust log colorizer.
It reads log output and highlights:
  - Timestamps
  - Log levels (INFO, WARN, ERROR, DEBUG)
  - UUIDs, IPs, URLs, paths, numbers
  - HTTP methods and status codes
  - Key=value pairs
Source: https://github.com/bensadeh/tailspin

## Why This Matters for the Forest
fsh already has `show` for reading files and `search` for finding content.
Logs are a gap -- raw log output is unreadable without color.
tailspin fills this gap cleanly, stays in Rust, and pipes naturally.

Use cases:
  journal -f | tspin          -- live system logs, colorized
  show --log /var/log/syslog  -- vocabulary-integrated log reading
  deploy core | tspin         -- colorized build output
  fsh -c "core doctor" | tspin -- colorized health output

## Evaluation Criteria
1. Does it work in fsh pipes without issues?
2. Does it handle fsh/forest output formats well?
3. Can it be integrated as a fsh vocabulary enhancement?
4. Is the configuration format reasonable (TOML)?
5. Does it conflict with bat (which handles file display)?

## Integration Plan (if evaluation passes)
- Add `tspin` alias in fsh
- Add `show --log` vocabulary flag that pipes through tspin
- Add `journal` vocabulary word: `journal` = `journalctl -f | tspin`
- Register in command registry (INT-259)
- Add to fsh_audit.sh tests

## Gates
- [ ] tailspin installed and working in fsh pipes
- [ ] journal -f | tspin renders correctly in foot and faelight-term
- [ ] show --log flag implemented in fsh vocabulary
- [ ] journal vocabulary word added
- [ ] registered in command registry
- [ ] no conflicts with bat display
- [ ] 3 days daily use confirms value

---
"The forest should be able to read its own signals.
Logs are signals. They deserve to look like signals." 🌲
