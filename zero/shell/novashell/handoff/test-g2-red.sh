#!/usr/bin/env bash
# INT-295 G2 red. Not run by cargo test -p novashell, nsh-test, d, or
# core intent validate. The reinstall case is an ignored test in zero-doctor.
# G3 made the setup-command case a normal test. G5 turns the reinstall case green.
# The tests use a temp HOME. They never read or write the real HOME.
set -euo pipefail
root=$(cd "$(dirname "$(readlink -f "$0")")/../../../.." && pwd)
if [[ ! -f "$root/Cargo.toml" ]]; then
    echo "REFUSE: cannot find the repo root from $0" >&2
    exit 2
fi
cd "$root"
exec cargo test -p zero-doctor --lib g2_ -- --ignored --nocapture --test-threads=1
