#!/usr/bin/env python3
"""INT-311. doors runs the ruled set, and it never calls ship or reload.

The script is copied into a temp tree and run against fakes, so this does not
run cargo, nsh-test, or core for real.
"""

import os
import pty
import shutil
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
SCRIPT = REPO / "zero" / "scripts" / "doors"

RULED = (
    "cargo test -p core",
    "cargo test -p novashell -p nsh-test",
    "core intent validate",
)


def assert_order(order, nsh_bin):
    lines = [line for line in order.splitlines() if line.strip()]
    banned = [line for line in lines if line.startswith("ship") or line.startswith("reload")]
    if banned:
        raise AssertionError("the run reached ship or reload: " + ", ".join(banned))
    expect = [
        "cargo test -p core",
        "cargo test -p novashell -p nsh-test",
        "nsh-test NSH_BIN=" + nsh_bin,
        "core intent validate",
    ]
    if lines != expect:
        raise AssertionError("ruled set mismatch:\n got: " + repr(lines) + "\n want: " + repr(expect))


def _fake(path, body):
    path.write_text(body)
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


class Doors(unittest.TestCase):
    def test_a_ship_call_is_rejected(self):
        order = (
            "cargo test -p core\n"
            "cargo test -p novashell -p nsh-test\n"
            "nsh-test NSH_BIN=/tmp/nsh\n"
            "core intent validate\n"
            "ship\n"
        )
        with self.assertRaises(AssertionError):
            assert_order(order, "/tmp/nsh")

    def test_a_short_set_is_rejected(self):
        with self.assertRaises(AssertionError):
            assert_order("cargo test -p core\n", "/tmp/nsh")

    def test_repo_script_runs_the_ruled_set(self):
        code, out, order = self._run()
        self.assertEqual(code, 0, out.decode("utf-8", "replace"))
        root = order["root"]
        assert_order(order["text"], str(root / "target" / "debug" / "nsh"))
        text = out.decode("utf-8", "replace")
        self.assertIn("doors", text)
        for cmd in RULED:
            self.assertIn(cmd, text)
        self.assertIn("pass", text)
        self.assertIn("passed", text)
        self.assertIn("d is its own door", text)
        self.assertIn("separately", text)
        self.assertNotIn("\x1b", text)

    def test_a_failed_door_is_nonzero_and_the_rest_still_run(self):
        code, out, order = self._run(cargo_rc="1")
        self.assertNotEqual(code, 0)
        self.assertIn("fail", out.decode("utf-8", "replace"))
        assert_order(order["text"], str(order["root"] / "target" / "debug" / "nsh"))

    def test_no_color_on_a_terminal_drops_the_escapes(self):
        code, out, _order = self._run(tty=True, no_color="1")
        self.assertEqual(code, 0)
        self.assertNotIn("\x1b", out.decode("utf-8", "replace"))

    def test_a_terminal_is_colored(self):
        code, out, _order = self._run(tty=True)
        self.assertEqual(code, 0)
        self.assertIn("\x1b", out.decode("utf-8", "replace"))

    def _run(self, cargo_rc="0", tty=False, no_color=None):
        if not SCRIPT.is_file():
            self.fail("zero/scripts/doors does not exist")
        if not os.access(SCRIPT, os.X_OK):
            self.fail("zero/scripts/doors is not executable")
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        root = Path(tmp.name)
        scripts = root / "zero" / "scripts"
        scripts.mkdir(parents=True)
        dest = scripts / "doors"
        shutil.copy(SCRIPT, dest)
        dest.chmod(dest.stat().st_mode | stat.S_IEXEC)
        debug = root / "target" / "debug"
        debug.mkdir(parents=True)
        log = root / "log"
        log.mkdir()
        bindir = root / "bin"
        bindir.mkdir()
        _fake(
            bindir / "cargo",
            "#!/bin/sh\nprintf '%s\\n' \"cargo $*\" >> \"$DOORS_LOG/order\"\n"
            "exit \"${DOORS_CARGO_RC:-0}\"\n",
        )
        _fake(
            bindir / "core",
            "#!/bin/sh\nprintf '%s\\n' \"core $*\" >> \"$DOORS_LOG/order\"\n"
            "exit \"${DOORS_CORE_RC:-0}\"\n",
        )
        _fake(
            bindir / "ship",
            "#!/bin/sh\nprintf '%s\\n' \"ship $*\" >> \"$DOORS_LOG/order\"\nexit 0\n",
        )
        _fake(
            bindir / "reload",
            "#!/bin/sh\nprintf '%s\\n' \"reload $*\" >> \"$DOORS_LOG/order\"\nexit 0\n",
        )
        _fake(debug / "nsh", "#!/bin/sh\nexit 0\n")
        _fake(
            debug / "nsh-test",
            "#!/bin/sh\nprintf '%s\\n' \"nsh-test NSH_BIN=${NSH_BIN-}\" >> \"$DOORS_LOG/order\"\n"
            "exit \"${DOORS_NSH_TEST_RC:-0}\"\n",
        )
        env = os.environ.copy()
        env["PATH"] = str(bindir) + ":/usr/bin:/bin"
        env["DOORS_LOG"] = str(log)
        env["DOORS_CARGO_RC"] = cargo_rc
        env.pop("NO_COLOR", None)
        if no_color is not None:
            env["NO_COLOR"] = no_color
        argv = [str(dest)]
        if tty:
            code, out = _pty(argv, env, root)
        else:
            proc = subprocess.run(argv, cwd=root, env=env, capture_output=True)
            code, out = proc.returncode, proc.stdout
        order_path = log / "order"
        text = order_path.read_text() if order_path.is_file() else ""
        return code, out, {"text": text, "root": root}


def _pty(argv, env, cwd):
    pid, fd = pty.fork()
    if pid == 0:
        os.chdir(cwd)
        os.execvpe(argv[0], argv, env)
        os._exit(127)
    chunks = []
    while True:
        try:
            chunk = os.read(fd, 4096)
        except OSError:
            break
        if not chunk:
            break
        chunks.append(chunk)
    _pid, status = os.waitpid(pid, 0)
    os.close(fd)
    return os.waitstatus_to_exitcode(status), b"".join(chunks)


if __name__ == "__main__":
    unittest.main()
