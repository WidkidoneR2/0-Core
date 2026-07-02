---
id: 006
date: 2026-01-24
type: incidents
title: "GitHub token exposure in terminal - Immediate password reset and credential helper setup"
status: complete
tags: [GitHub token exposure, password reset]
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

---

## What Happened

While testing `faelight-git v3.0` sync command, accidentally entered GitHub Personal Access Token in the **username field** instead of the password field during git push authentication.

**Token exposed:** `ghp_[REDACTED]`

This exposed the token in terminal output and conversation history.

## Root Cause

**User error:** Misunderstanding of git authentication flow
- Username should be: `WidkidoneR2`
- Password should be: `[token here]`

**Systemic issue:** No credential helper configured, requiring manual entry every push

## Immediate Actions Taken

1. ✅ **Password reset** - Immediately reset GitHub password
2. ✅ **Token revocation** - Revoked exposed token
3. ✅ **Credential helper** - Configured `git config --global credential.helper cache`

## Prevention Measures

**Short-term:**
- Use credential.helper cache (expires after 15 min)
- Never paste tokens in username field
- Always double-check authentication prompts

**Long-term:**
- Consider SSH keys instead of HTTPS tokens
- Add git credential documentation to README
- Maybe add helper text in fg sync when it asks for credentials?

## Lessons Learned

1. **Security first:** Immediate action prevented unauthorized access
2. **Credential helpers:** Should have been set up from the start
3. **Testing tools:** Be careful when testing authentication flows
4. **Documentation:** Need better git auth setup docs

## Impact

**Severity:** High (token exposure)
**Duration:** ~5 minutes (quick response)
**Systems affected:** GitHub authentication
**Data compromised:** One personal access token (revoked)

## Status

✅ **RESOLVED** - All mitigation steps complete, no unauthorized access detected

