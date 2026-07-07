#!/usr/bin/env bash
# INT-043: prepare a deps-only source for crane's buildDepsOnly, kept hash-stable
# across our own changes so the Cachix push stays valid.
#
# We do NOT use crane's mkDummySrc: it sets `package.build = <store-path>` in each
# dummy Cargo.toml (crane issue #117), and this Nix (26.05) rejects a store path
# embedded in the TOML string it writes. So we hand-roll the equivalent here.
#
# Two jobs:
#  (1) normalize each crate's [package] version to 0.0.0 -- so cicomplete version
#      bumps do not perturb the deps hash. SECTION-AWARE: only the version line
#      inside a [package] table; [dependencies.*] specs are untouched.
#  (2) stub targets so cargo can check the workspace against a manifests-only src:
#      - every [package] crate gets src/main.rs (fn main) + src/lib.rs (empty)
#      - any explicit `path = "src/..."` target declared in a Cargo.toml is also
#        stubbed (covers extra [[bin]] like src/bin/auth.rs, src/bin/test-client.rs)
#      No proc-macros in this workspace, so a main() stub is always safe.
# Usage: normalize-deps-versions.sh <dir>   (mutates Cargo.toml + adds stubs in place)
set -euo pipefail
root="$1"

# (1) normalize [package] versions
while IFS= read -r -d "" f; do
  awk '
    /^\[/ { inpkg = ($0 == "[package]") }
    inpkg && /^version[[:space:]]*=/ { print "version = \"0.0.0\""; next }
    { print }
  ' "$f" > "$f.norm"
  mv "$f.norm" "$f"
done < <(find "$root" -name Cargo.toml -print0)

# (1b) normalize OUR workspace members' versions in Cargo.lock to match the 0.0.0
# we wrote into the Cargo.tomls -- otherwise cargo sees a manifest/lock mismatch
# and tries to update the lock, which --locked forbids. Our packages are the
# [[package]] entries with NO `source = ` line (path/workspace members); registry
# crates HAVE a source and must be left untouched.
if [ -f "$root/Cargo.lock" ]; then
  awk '
    /^\[\[package\]\]/ { inpkg=1; hassource=0; delete buf; n=0; buf[n++]=$0; next }
    inpkg {
      if (/^source[[:space:]]*=/) hassource=1
      if (/^$/) {
        # end of block: flush, zeroing version only if no source (our crate)
        for (i=0;i<n;i++) {
          if (!hassource && buf[i] ~ /^version[[:space:]]*=/) print "version = \"0.0.0\""
          else print buf[i]
        }
        print ""
        inpkg=0; next
      }
      buf[n++]=$0; next
    }
    { print }
    END {
      if (inpkg) for (i=0;i<n;i++) {
        if (!hassource && buf[i] ~ /^version[[:space:]]*=/) print "version = \"0.0.0\""
        else print buf[i]
      }
    }
  ' "$root/Cargo.lock" > "$root/Cargo.lock.norm"
  mv "$root/Cargo.lock.norm" "$root/Cargo.lock"
fi

# (2) stub targets for every crate that has a [package]
while IFS= read -r -d "" f; do
  if grep -q "^\[package\]" "$f"; then
    d="$(dirname "$f")"
    mkdir -p "$d/src"
    [ -e "$d/src/lib.rs" ]  || printf ""            > "$d/src/lib.rs"
    [ -e "$d/src/main.rs" ] || printf "fn main() {}\n" > "$d/src/main.rs"
    # honor any explicit target paths (e.g. extra bins under src/bin/)
    while IFS= read -r tp; do
      [ -z "$tp" ] && continue
      mkdir -p "$d/$(dirname "$tp")"
      [ -e "$d/$tp" ] || printf "fn main() {}\n" > "$d/$tp"
    done < <(awk -F'"' '/^[[:space:]]*path[[:space:]]*=[[:space:]]*"src\// {print $2}' "$f")
  fi
done < <(find "$root" -name Cargo.toml -print0)
