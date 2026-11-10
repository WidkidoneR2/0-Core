#!/usr/bin/env bash
# INT-306: the commit-msg hook. Not a zero-gate stage. cargo test -p core runs
# g5_commit_msg_hook, which runs this. A message with no Intent: is left alone.
# A message with Intent: INT-xxx and no Seal: of 16 lowercase hex is refused,
# unless it carries Release:, and that exemption is printed.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
hook="$root/.githooks/commit-msg"
if [[ ! -x "$hook" ]]; then
    echo "REFUSE: $hook is not an executable hook" >&2
    exit 1
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

one() {
    local name=$1 expect=$2 needle=$3 body=$4
    local f="$tmp/$name"
    printf '%s' "$body" > "$f"
    local before after err rc
    before=$(cat "$f")
    set +e
    err=$("$hook" "$f" 2>&1)
    rc=$?
    set -e
    after=$(cat "$f")
    if [[ "$before" != "$after" ]]; then
        echo "REFUSE: $name: the hook rewrote the message" >&2
        exit 1
    fi
    if [[ "$rc" != "$expect" ]]; then
        echo "REFUSE: $name: exit $rc, wanted $expect" >&2
        printf '%s\n' "$err" >&2
        exit 1
    fi
    if [[ -n "$needle" && "$err" != *"$needle"* ]]; then
        echo "REFUSE: $name: output did not contain: $needle" >&2
        printf '%s\n' "$err" >&2
        exit 1
    fi
    if [[ -z "$needle" && -n "$err" ]]; then
        echo "REFUSE: $name: expected no output, got:" >&2
        printf '%s\n' "$err" >&2
        exit 1
    fi
    echo "ok $name"
}

one no-intent 0 "" "just a subject
"
one prose 0 "" "mentioned

The prose names Intent: INT-9 but not as a trailer line.
"
one subject-only 0 "" "Intent: INT-306
"
one not-an-id 0 "" "fix

Intent: not-an-id
"
one no-seal 1 "refused:" "fix

Intent: INT-306
"
one uppercase 1 "refused:" "fix

Intent: INT-306
Seal: 0123456789ABCDEF
"
one short 1 "refused:" "fix

Intent: INT-306
Seal: 0123456789abcde
"
one sealed 0 "" "fix

Intent: INT-306
Seal: 0123456789abcdef
"
one release 0 "release exemption used" "INT-295: complete; nsh 5.3.12

Intent: INT-295
Release: nsh 5.3.12, nsh-test 2.0.15
"
one release-and-seal 0 "release exemption used" "bump

Intent: INT-9
Seal: 0123456789abcdef
Release: nsh 1.2.3
"

# The refusal says what to add.
f="$tmp/says-what"
printf '%s' "fix

Intent: INT-306
" > "$f"
set +e
err=$("$hook" "$f" 2>&1)
rc=$?
set -e
if [[ "$rc" != 1 || "$err" != *"Seal:"* || "$err" != *"Release:"* ]]; then
    echo "REFUSE: the refusal does not say to add Seal: or Release:" >&2
    printf '%s\n' "$err" >&2
    exit 1
fi
echo "ok says-what"
echo "commit-msg ok"
