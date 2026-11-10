#!/usr/bin/env bash
# INT-306 G5. The tests were ignored while they were red (test-g5-red.sh
# ran them with --ignored and they failed). They are not ignored now.
# cargo test -p core runs them. This runs the same filter.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
if [[ ! -f "$root/Cargo.toml" ]]; then
    echo "REFUSE: cannot find the repo root from $0" >&2
    exit 2
fi
cd "$root"
exec cargo test -p core g5_ -- --test-threads=1
