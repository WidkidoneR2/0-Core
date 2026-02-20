# faelight-cleanup

Faelight Forest hygiene tool. Scans for and removes stale files, caches, and orphans.

## Usage
```bash
faelight-cleanup          # scan only (safe)
faelight-cleanup --clean  # remove with confirmation
faelight-cleanup --clean --yes  # remove without prompts
faelight-cleanup --verbose      # show orphaned scripts too
```

## What it scans

- Backup dirs (`*-backup-*`, `*-migration-*`)
- Session notes (TONIGHT.md, TODO-TOMORROW.md etc.)
- Temp logs (`/tmp/faelight-*.log`)
- Yazi cache (`~/.cache/yazi`)
- Cargo registry cache (when >50MB)
- Duplicate intent numbers
- Pacman cache (when >500MB)
