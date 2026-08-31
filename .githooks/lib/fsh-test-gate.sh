# INT-202 gate 5: the suite runs without being typed.
#
# WHY PRE-PUSH AND NOT PRE-COMMIT, and risk-gate.sh already wrote the rule this follows:
# "a gate that fires on every commit is a gate people route around with --no-verify, which is
# how INT-113 and INT-119 both died." Nine commits in a day times fifty-five seconds is eight
# minutes spent waiting for an answer that rarely changes within a series. A push is the thing
# that leaves the machine, and a red push is what actually matters.
#
# THE SKIP IS THE FRAMEWORK'S, NOT THIS SCRIPT'S. The hook declares a `files` regex, so
# pre-commit does not invoke it at all unless the push touches shell or harness sources. No
# hand-rolled diff logic to get wrong.
#
# WHY IT BUILDS FIRST. NSH_BIN defaults to the DEPLOYED shell, so a hook that used the default
# would test the shell you are RUNNING rather than the code you are SENDING -- passing a broken
# change, and failing a good one pushed after a bad deploy. It builds and points NSH_BIN at the
# fresh debug binary instead.
#
# NO cargo IN runtimeInputs. writeShellApplication PREPENDS runtimeInputs to PATH without
# clearing it, so a toolchain pinned there could shadow the devshell's and silently test a
# different compiler than the one you develop with. The devshell provides cargo; this checks
# for it and says so plainly if it is missing.

root=$(git rev-parse --show-toplevel)
cd "$root"

if ! command -v cargo > /dev/null 2>&1; then
  echo ""
  echo "  FSH-TEST GATE (INT-202): cargo is not on PATH -- are you outside the devshell?"
  echo "  This gate builds what you are pushing. It will not silently test the deployed"
  echo "  shell instead, because that would answer a question you did not ask."
  exit 1
fi

echo ""
echo "  FSH-TEST GATE (INT-202): shell or harness sources are in this push."
echo "  Building and running the suite against the code being PUSHED (~1 minute)."
echo ""

if ! cargo build -p faelight-shell -p fsh-test --message-format=short; then
  echo ""
  echo "  BLOCKED: the workspace does not build. Nothing was tested."
  exit 1
fi

# THE PACKAGE IS faelight-shell; THE BINARY IS nsh. cargo build -p above takes the
# PACKAGE name and is unchanged. NSH_BIN takes a PATH, and cargo stopped producing
# target/debug/faelight-shell when the [[bin]] landed -- so this pointed at a stale
# artifact from before the rename, or at nothing in a clean tree.
if ! NSH_BIN="$root/target/debug/nsh" ./target/debug/fsh-test; then
  echo ""
  echo "  BLOCKED: fsh-test is red. This is the gate doing its job."
  echo "  Reproduce: NSH_BIN=\$PWD/target/debug/nsh ./target/debug/fsh-test"
  exit 1
fi

echo ""
echo "  ok: fsh-test passed against the code being pushed"
echo ""
