import glob, hashlib, os, shutil, subprocess, sys, tempfile, time, unittest
from pathlib import Path

LIMIT = 2
TAIL_MARK = "AGENT-TAIL-MARK"

FAKE_BWRAP = """#!/bin/sh
case "$FAKE_MODE" in
  ok) echo "fake answer"; exit 0 ;;
  stall) echo "AGENT-TAIL-MARK" >&2; exec sleep 30 ;;
esac
exit 3
"""
FAKE_CURL = "#!/bin/sh\nprintf 200\n"


def source_path():
    env = os.environ.get("HERMES_EVAL_SRC")
    if env:
        return env
    return str(Path(__file__).resolve().with_name("hermes-eval.py"))


class TimeoutPath(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="hermes-eval-test-")
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.home = os.path.join(self.tmp, "home")
        ev = os.path.join(self.home, ".cache", "zero", "hermes-eval")
        os.makedirs(os.path.join(ev, "home"))
        os.makedirs(os.path.join(ev, "work", "repo"))
        subprocess.run(["git", "init", "-q", os.path.join(ev, "work", "repo")], check=True)
        self.bindir = os.path.join(self.tmp, "bin")
        os.makedirs(self.bindir)
        for name, body in (("bwrap", FAKE_BWRAP), ("curl", FAKE_CURL)):
            p = os.path.join(self.bindir, name)
            with open(p, "w") as f:
                f.write(body)
            os.chmod(p, 0o755)
        self.runs = os.path.join(ev, "runs")

    def launch(self, mode, limit=None):
        with open(source_path(), encoding="utf-8") as f:
            text = f.read()
        if limit is not None:
            n = text.count("TIMEOUT=240\n")
            if n != 1:
                self.fail("HARNESS: expected exactly one line 'TIMEOUT=240' in the launcher, found " + str(n) + ". Update this test together with the code.")
            text = text.replace("TIMEOUT=240\n", "TIMEOUT=" + str(limit) + "\n")
        # The launcher refuses to run unless its file name carries the hash of its own text.
        digest = hashlib.sha256(text.encode()).hexdigest()[:12]
        copy = os.path.join(self.tmp, "hermes-eval-" + digest + ".py")
        with open(copy, "w", encoding="utf-8") as f:
            f.write(text)
        env = {"HOME": self.home, "PATH": self.bindir + ":" + os.environ.get("PATH", ""), "FAKE_MODE": mode, "LANG": "C.UTF-8"}
        t0 = time.time()
        p = subprocess.run([sys.executable, copy, "a task"], env=env, capture_output=True, text=True, timeout=60)
        took = time.time() - t0
        logs = glob.glob(os.path.join(self.runs, "*.txt"))
        log = None
        if len(logs) == 1:
            with open(logs[0], encoding="utf-8") as f:
                log = f.read()
        return p, took, log

    def test_control_normal_run_is_logged(self):
        p, took, log = self.launch("ok")
        self.assertIsNotNone(log, "HARNESS: no single run log was written. exit " + str(p.returncode) + " stdout: " + p.stdout[-300:] + " stderr: " + p.stderr[-300:])
        self.assertIn("rc: 0", log, "HARNESS: the stand-in did not give a normal run, so the timeout tests mean nothing.")
        self.assertIn("fake answer", log)

    def test_timeout_writes_a_log_with_rc_minus_one(self):
        p, took, log = self.launch("stall", LIMIT)
        self.assertLess(took, 20, "the fake limit of " + str(LIMIT) + " seconds did not end the run")
        self.assertIsNotNone(log, "a timed-out run wrote no log. exit " + str(p.returncode) + " stdout: " + p.stdout[-300:] + " stderr: " + p.stderr[-300:])
        self.assertIn("rc: -1", log)
        self.assertIn("TIMEOUT after", log)

    def test_timeout_message_names_the_limit_that_was_used(self):
        p, took, log = self.launch("stall", LIMIT)
        self.assertIsNotNone(log)
        self.assertIn("TIMEOUT after " + str(LIMIT) + " seconds", log, "the log does not name the limit that ended the run")

    def test_timeout_keeps_the_tail_of_what_the_agent_wrote(self):
        # Christian ruled on 2026-11-06 that the tail is kept: the log shows the last 600 characters, as on a normal run.
        p, took, log = self.launch("stall", LIMIT)
        self.assertIsNotNone(log)
        self.assertIn(TAIL_MARK, log, "what the agent wrote before the timeout is not in the log")


if __name__ == "__main__":
    unittest.main()
