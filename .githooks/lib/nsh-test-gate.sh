# INT-202 gate 5: the suite runs without being typed.
#
# WHY PRE-PUSH AND NOT PRE-COMMIT, and risk-gate.sh already wrote the rule this follows:
# "a gate that fires on every commit is a gate people route around with --no-verify, which is
# how INT-113 and INT-119 both died." Nine commits in a day times fifty-five seconds is eight
# minutes spent waiting for an answer that rarely changes within a series. A push is the thing
# that leaves the machine, and a red push is what actually matters.
#
# ⚠️ THERE IS NO SKIP. This described a `files` regex declared by the Nix pre-commit
# framework, which died with nix/. zero-gate runs this script on EVERY pre-push regardless of
# what the push touches, so every push pays the suite. The comment survived its framework by a
# week and was still promising a filter that no code implements.
#
# Implement a path filter in zero-gate as another Gate, or accept the cost. Do not describe one
# that is not there.
#
# WHY IT BUILDS FIRST. NSH_BIN defaults to the DEPLOYED shell, so a hook that used the default
# would test the shell you are RUNNING rather than the code you are SENDING -- passing a broken
# change, and failing a good one pushed after a bad deploy. It builds and points NSH_BIN at the
# fresh debug binary instead.
#
# WHY IT DOES NOT PIN A TOOLCHAIN. This was written about writeShellApplication prepending
# runtimeInputs to PATH and shadowing the devshell's compiler. The devshell is gone with nix/,
# but the reasoning outlived it: a gate that supplies its own cargo can silently test a
# different compiler than the one you build with. It checks for cargo and says so plainly if
# it is missing, rather than providing one.

root=$(git rev-parse --show-toplevel)
cd "$root"

if ! command -v cargo > /dev/null 2>&1; then
  echo ""
  echo "  NSH-TEST GATE (INT-202): cargo is not on PATH."
  echo "  This gate builds what you are pushing. It will not silently test the deployed"
  echo "  shell instead, because that would answer a question you did not ask."
  exit 1
fi

echo ""
echo "  NSH-TEST GATE (INT-202): every push runs the suite -- there is no path filter."
echo "  Building and running the suite against the code being PUSHED (~1 minute)."
echo ""

if ! cargo build -p novashell -p nsh-test --message-format=short; then
  echo ""
  echo "  BLOCKED: the workspace does not build. Nothing was tested."
  exit 1
fi

# THE PACKAGE IS faelight-shell; THE BINARY IS nsh. cargo build -p above takes the
# PACKAGE name and is unchanged. NSH_BIN takes a PATH, and cargo stopped producing
# target/debug/faelight-shell when the [[bin]] landed -- so this pointed at a stale
# artifact from before the rename, or at nothing in a clean tree.
if ! NSH_BIN="$root/target/debug/nsh" ./target/debug/nsh-test; then
  echo ""
  echo "  BLOCKED: nsh-test is red. This is the gate doing its job."
  echo "  Reproduce: NSH_BIN=\$PWD/target/debug/nsh ./target/debug/nsh-test"
  exit 1
fi

echo ""
echo "  ok: nsh-test passed against the code being pushed"
echo ""
