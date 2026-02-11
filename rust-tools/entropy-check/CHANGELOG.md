# Changelog - entropy-check

## [2.0.0] - 2026-02-11

### 🎉 PRODUCTION READY

**Features:**
- Configuration drift detection
- Baseline snapshot creation
- Drift history tracking (30 days)
- JSON output support
- Health check integration

**Commands:**
- `--baseline` - Create/update baseline snapshot
- `--trends` - Show drift history
- `--json` - JSON output format
- `--health` - Run health check

**Detection:**
- Tracks configuration file changes
- Monitors system state drift
- Historical trend analysis
- Baseline comparison

**Code Quality:**
- Zero clippy warnings
- Safe unwrap usage (.unwrap_or_default)
- Clean error handling

**Usage:**
```bash
entropy-check --baseline    # Create baseline
entropy-check               # Check drift
entropy-check --trends      # Show history
entropy-check --json        # JSON output
```

---

## [1.0.0] - Earlier

Configuration drift detection tool.

---

**Version Format:** MAJOR.MINOR.PATCH
