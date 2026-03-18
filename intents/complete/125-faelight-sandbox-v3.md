---
id: 125
date: 2026-03-12
type: future
title: "faelight-sandbox v3 — Full Policy Engine & Deep Isolation"
status: complete
tags: [sandbox, isolation, security, policy, rust, v11, architecture]
version: 11.0.0
priority: medium
depends_on: [124]
---

## Vision

v2 adds observability. v3 adds control.

A full policy engine with declarative TOML policies,
deep OS-level isolation, and memory/CPU profiling.
The sandbox becomes a first-class security boundary.

## The Three Pillars

### Pillar 1 — Declarative Policy Engine
```toml
# registry/sandbox-policies.toml

[[sandbox.policy]]
name = "untrusted-script"
allow_net = false
allow_fs_write = false
allow_fs_read = ["~/0-core/runtime"]
allow_env = ["PATH", "HOME"]
max_cpu_seconds = 30
max_memory_mb = 256
emit_events = true

[[sandbox.policy]]
name = "network-tool"
allow_net = true
allow_fs_write = false
allow_env = ["PATH", "HOME", "WAYLAND_DISPLAY"]
max_cpu_seconds = 60
emit_events = true
```

Apply a policy:
```bash
faelight-sandbox run --policy untrusted-script -- ./unknown-script.sh
```

### Pillar 2 — Deep OS Isolation

Beyond `unshare --net`:
- Filesystem namespace (mount namespace)
- PID namespace isolation
- Seccomp syscall filtering
- Read-only bind mounts for sensitive paths
- Tmpfs overlay for write isolation
```bash
faelight-sandbox run --isolate full -- command
# Full namespace isolation:
#   Network: isolated
#   Filesystem: overlay tmpfs
#   PIDs: isolated namespace
#   Syscalls: seccomp filtered
```

### Pillar 3 — Deep Resource Profiling

OS-level resource measurement:
- Peak memory (RSS, VSZ)
- CPU time (user + system)
- Disk I/O bytes read/written
- Network bytes (if allowed)
- Syscall count
```bash
faelight-sandbox profile -- cargo build
# Output:
#   Duration:  47.3s
#   CPU user:  43.1s  system: 2.8s
#   Memory peak: 1.2GB
#   Disk read:  892MB  write: 156MB
#   Syscalls:  2,847,291
```

## core advise Integration (v3)

With enough data, `core advise` surfaces anomalies:
```
→ faelight-browser sandbox exceeded memory limit 3 times
  Policy: network-tool (256MB limit)
  Suggest: increase limit or investigate memory usage

→ Unknown script attempted 47 blocked syscalls
  Risk: elevated — review before running unsandboxed
```

## Success Criteria

- ✅ TOML policy engine — 5 policies (default, untrusted, network-tool, build, strict)
- ✅ --policy flag on run command
- ✅ Real network isolation via unshare (tested — curl blocked under strict policy)
- ✅ Policy restrictions logged to state.db with every run
- ✅ Policy info displayed in session header
- ✅ Mount namespace isolation — --isolate full flag
- ✅ PID namespace isolation — --isolate full flag
- ⬜ Seccomp syscall filtering
- ✅ Peak memory tracking via /proc/self/status
- ✅ Duration tracking via Instant::now()
- ✅ Disk I/O tracking — /proc/self/io read/write bytes
- ✅ `core advise` surfaces sandbox activity and policy violations
- ✅ doctor monitors sandbox health — 24th check added

---
*"Control is not restriction. It is understanding."* 🌲
