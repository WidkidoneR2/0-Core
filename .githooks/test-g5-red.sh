#!/usr/bin/env bash
# INT-306 G5 red. Not run by a bare cargo test -p core: the g5_ tests are
# ignored until the guard exists, so main stays green. This runs them.
# They fail while the stubs answer untouched and the hook is absent.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
if [[ ! -f "$root/Cargo.toml" ]]; then
    echo "REFUSE: cannot find the repo root from $0" >&2
    exit 2
fi
cd "$root"
exec cargo test -p core g5_ -- --ignored --nocapture --test-threads=1
