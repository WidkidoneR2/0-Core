---
id: 268
date: 2026-05-03
type: feature
title: "fsh natural language -- Friday interrupt levels and ? prefix"
status: complete
tags: [shell, fsh, friday, intelligence, natural-language]
version: TBD
---
INT-245 Pillar 5. The shell that understands intent, not just syntax.
    ? build and deploy core
    ? show me what changed today
    ? find the rust file with E0597
Friday translates to real commands. You confirm or reject.
Also: interrupt levels (CHALLENGE / RECOMMEND / SUGGEST / SILENT).
- [ ] ? prefix sends input to Friday for translation
- [ ] Friday returns a real fsh command with confidence score
- [ ] User confirms (y) or rejects (n) before execution
- [ ] CHALLENGE level stops dangerous commands before execution
- [ ] RECOMMEND level surfaces suggestion before you finish typing
- [ ] SUGGEST level mentions after execution
- [ ] Demonstrated: ? translates 5 natural language queries correctly
Ships as fsh v2.1.0.
