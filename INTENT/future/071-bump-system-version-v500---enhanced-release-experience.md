---
id: 071
date: 2026-01-23
type: future
title: ""bump-system-version v5.0.0 - Enhanced Release Experience"
status: planned
tags: [bump-system]
---

CONTEXT:
v8.1.0 release revealed areas for improvement in bump-system-version.
Current v4.0.0 works well but needs more human checkpoints and review steps.

GOAL:
Transform bump-system-version from 'automation' to 'assisted release' -
the tool handles mechanics, human maintains control and understanding.

IMPROVEMENTS PLANNED:

1. Change Review Step (before commit)
   - Show full diff statistics
   - Option to review in editor
   - Explicit confirmation before commit

2. CHANGELOG Review/Edit
   - Allow editing generated CHANGELOG before insertion
   - Preview formatting
   - Confirm accuracy

3. Git Lock Detection
   - Check for .git/index.lock before operations
   - Offer to clean up and retry
   - Prevent silent failures

4. Rollback Helper Menu
   - Undo commit
   - Delete tag
   - Restore snapshot
   - Clear guidance if something goes wrong

5. Dry Run Mode
   - Full simulation without changes
   - Perfect for testing and verification

6. Better Error Messages
   - Clear explanations of what failed
   - Suggested fixes
   - Recovery options

TESTING STRATEGY:
- Use v8.1.1 through v8.1.9 for iterative testing
- Each test version validates one improvement
- Document learnings in this Intent

PHILOSOPHY:
'A garden requires attention at every step.'
More human checkpoints = better understanding = safer releases.

The tool should make releases easier, not automatic.
Manual control over automation. Always.

STATUS: future
RELATED: Intent 067 (Post-presentation evolution)
TARGET: v5.0.0 (after 8.2.0 release)"


---
