
---

## Resolution (INT-106 close)
- #1 rename rules_dir -> policy_dir: DONE (4 refs in paths.rs; builds clean; policy_dir test assert passes).
- #2 font path fix: DONE (lib.rs test now runtime-read-or-skip; `cargo test -p faelight-core` runs clean, 11 passed).
- #3 route 40+ hardcoded path strings: SPLIT OUT to INT-115 (per this intent's own guidance -- multi-session, per-tool, not a 1.0.0 blocker).
