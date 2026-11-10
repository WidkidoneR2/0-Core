#!/usr/bin/env bash
# INT-306: pre-push keeps the push's ref lines even when the gate reads stdin.
# cargo test -p core runs g5_pre_push_asks_seals_check_for_each_pushed_ref,
# which runs this. A temp repository holds a zero-gate that reads stdin and
# a core that records its arguments. The real hook is executed there.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
hook="$root/.githooks/pre-push"
if [[ ! -x "$hook" ]]; then
    echo "REFUSE: $hook is not an executable hook" >&2
    exit 1
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
git -C "$tmp" init -q
mkdir -p "$tmp/target/debug"
cat > "$tmp/target/debug/zero-gate" << 'EOF'
#!/bin/sh
cat > "$GATE_SEEN"
printf 'ran\n' > "$GATE_RAN"
exit 0
EOF
cat > "$tmp/target/debug/core" << 'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$CALLS"
exit 0
EOF
chmod 755 "$tmp/target/debug/zero-gate" "$tmp/target/debug/core"

zeros=0000000000000000000000000000000000000000
local_sha=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
remote_sha=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
newb=cccccccccccccccccccccccccccccccccccccccc
seen="$tmp/seen"
ran="$tmp/ran"
calls="$tmp/calls"
: > "$calls"
(
    cd "$tmp"
    printf '%s\n' \
        "refs/heads/main $local_sha refs/heads/main $remote_sha" \
        "refs/heads/topic $newb refs/heads/topic $zeros" \
        "refs/heads/old $zeros refs/heads/old $remote_sha" \
        | env GATE_SEEN="$seen" GATE_RAN="$ran" CALLS="$calls" \
            "$hook"
)
if [[ ! -f "$ran" ]]; then
    echo "REFUSE: the gate did not run" >&2
    exit 1
fi
got=$(cat "$seen" 2>/dev/null || true)
if [[ -n "$got" ]]; then
    echo "REFUSE: the gate read the ref lines:" >&2
    printf '%s\n' "$got" >&2
    exit 1
fi
want=$(printf '%s\n' \
    "intent seals check ${remote_sha}..${local_sha}" \
    "intent seals check ${newb} --not --remotes")
have=$(cat "$calls")
if [[ "$have" != "$want" ]]; then
    echo "REFUSE: the seal check did not see the saved refs" >&2
    printf 'got:\n%s\nwant:\n%s\n' "$have" "$want" >&2
    exit 1
fi
echo "ok refs survive a gate that reads stdin"
