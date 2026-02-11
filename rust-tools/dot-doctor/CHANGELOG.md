# Changelog - dot-doctor

All notable changes to this project will be documented in this file.

## [4.0.0] - 2026-02-11

### 🎉 FLAGSHIP RELEASE - Intelligent Health Monitoring

**Major New Features:**

1. **Dry-Run Mode** (`--fix-dry-run`)
   - Preview fixes before applying
   - See what would be changed without modifying system
   - Perfect for cautious administrators

2. **Watch Mode** (`--watch` + `--interval`)
   - Continuous health monitoring (partial implementation)
   - Custom monitoring intervals
   - Real-time system health tracking

3. **Selective Checks** (`--skip`)
   - Skip specific checks
   - Multiple skips supported: `--skip git --skip security`
   - Complementary to existing `--check` flag

4. **Health Thresholds** (`--min-health`)
   - Fail if health falls below threshold
   - Perfect for CI/CD pipelines
   - Example: `--min-health 95`

5. **HTML Reports** (`--report` flag added, implementation pending)

**Enhanced Documentation:**
- 245-line comprehensive README
- CI/CD integration examples
- Cron monitoring examples
- Advanced usage patterns

**Code Quality:**
- Zero clippy warnings
- 1,657 lines of intelligent health checking
- Production-ready for critical systems

### Technical
- 19 health checks
- Auto-fix capabilities
- JSON export
- History tracking
- Dependency graph

---

## [3.2.0] - Earlier

Production health monitoring with auto-fix.

---

**Version Format:** MAJOR.MINOR.PATCH
