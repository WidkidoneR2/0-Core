# INT-112: the RISK.toml gate.
#
# For each staged path, walk UP to the nearest RISK.toml. Nearest wins -- a subdirectory can
# be more specific than its parent. If that file says risk = "critical", collect the checks
# named in its requires = [ ... ] and run them before the commit is allowed.
#
# Two dirs are critical today (nix/hosts/framework16, nix/modules/desktop). That rarity IS
# the design: a gate that fires on every commit is a gate people route around with
# --no-verify, which is how INT-113 and INT-119 both died.
#
# NO ASSOCIATIVE ARRAY, and that is not style. writeShellApplication runs this under
# `set -euo pipefail`, and under `set -u` bash treats a declared-but-EMPTY array as UNSET --
# so `${#needed[@]}` aborts the script before any logic runs. Found 2026-07-16 by running
# the gate instead of trusting it: BOTH the user-tier and critical-tier tests failed
# identically at line 31, which meant neither test told us anything about the walk-up. A
# plain string has no such trap.

needed=""

for f in "$@"; do
  dir=$(dirname "$f")
  while [ "$dir" != "." ] && [ "$dir" != "/" ]; do
    if [ -f "$dir/RISK.toml" ]; then
      if grep -q '^risk = "critical"' "$dir/RISK.toml"; then
        reqs=$(grep '^requires = ' "$dir/RISK.toml" | tr -d '[]",' | cut -d= -f2- || true)
        needed="$needed $reqs"
      fi
      break
    fi
    dir=$(dirname "$dir")
  done
done

needed=$(echo "$needed" | tr ' ' '\n' | grep -v '^$' | sort -u || true)

if [ -z "$needed" ]; then
  exit 0
fi

echo ""
echo "  RISK GATE (INT-112): critical-tier files staged."
echo "  These directories can leave a machine that will not boot or will not log in, so"
echo "  their checks run BEFORE the commit, not after the reboot."
echo ""

for c in $needed; do
  echo "  -> running check: $c   (a cold build takes minutes -- that is the deal)"
  if ! nix build ".#checks.x86_64-linux.$c" --no-link; then
    echo ""
    echo "  BLOCKED: check '$c' failed. This is the gate doing its job."
    echo "  The 2026-06-09 lockout cost 24 hours. This costs minutes."
    exit 1
  fi
  echo "  ok: $c passed"
done
echo ""
