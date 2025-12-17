# 🚨 Incident Journal

> Learning repository: Every failure is a lesson, every fix is knowledge.

## Purpose

This directory contains detailed incident reports documenting:
- What broke
- Why it broke
- How we fixed it
- What we learned
- How we prevent recurrence

## Philosophy

**Pain → Knowledge → Architecture**

We don't hide failures. We document them, learn from them, and turn them into system improvements.

## Incident Index

### 2025-12-14: Password/Sudo Failure (Critical)
**File:** `2025-12-14-password-sudo-failure.md`  
**Duration:** 12 hours  
**Impact:** System authentication broken  
**Lesson:** Never automate sudo at boot  
**Policy Added:** Authentication Policy, Automation Policy

---

## Creating New Incidents

1. Copy `TEMPLATE.md`
2. Name: `YYYY-MM-DD-brief-description.md`
3. Fill in all sections
4. Link from this README
5. Reference in POLICIES.md if applicable

## Incident Severity

- **Critical:** System unusable
- **High:** Major functionality lost
- **Medium:** Important features broken
- **Low:** Minor issues

## Review Schedule

- Critical incidents: Review in 1 month
- High incidents: Review in 3 months
- Medium/Low: Review in 6 months

---

**Remember:** If you're not documenting failures, you're doomed to repeat them.

---

## Current Incidents

| Date | Title | Severity | Duration | Status |
|------|-------|----------|----------|--------|
| 2025-12-14 | [Password/Sudo Failure](2025-12-14-password-sudo-failure.md) | Critical | 12 hours | Resolved |

