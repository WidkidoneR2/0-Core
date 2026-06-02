---
id: 267
date: 2026-05-03
type: feature
title: "fsh parallel execution -- parallel { } block and ||| operator"
status: complete
tags: [shell, fsh, parallel, performance, innovation]
version: TBD
---
INT-245 Pillar 1. The core innovation. What no other shell does natively.
    parallel {
        deploy core
        deploy faelight-shell
        deploy faelight-term
    }
All three build simultaneously. Output labeled by tool. Waits for all.
Also: `deploy core ||| deploy faelight-shell` inline parallel syntax.
- [ ] parallel { } block syntax parses and executes
- [ ] ||| operator runs both sides simultaneously
- [ ] Labeled output streams -- no interleaving
- [ ] One failure does not cancel others
- [ ] jobs / wait / cancel commands live
- [ ] Demonstrated: 3+ simultaneous deploys complete faster than serial
Ships as fsh v2.0.0. This is the version number that earns the major bump.
