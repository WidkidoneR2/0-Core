"""INT-278: one test per fpatch defect. Run: python3 -m unittest -q test_fpatch

Each test works in its own temp directory. Nothing outside it is touched.
"""

import contextlib
import io
import os
import pathlib
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
os.environ["FPATCH_COLOR"] = "0"
os.environ.pop("FPATCH_DEBUG", None)
import fpatch  # noqa: E402

SAB = "CORRUPTED\n"


class Base(unittest.TestCase):
    def setUp(self):
        self.d = tempfile.TemporaryDirectory()
        self.dir = self.d.name

    def tearDown(self):
        self.d.cleanup()

    def mk(self, name, text):
        p = os.path.join(self.dir, name)
        with open(p, "w", newline="") as f:
            f.write(text)
        return p

    def rd(self, p):
        with open(p, newline="") as f:
            return f.read()

    def call(self, fn, *a, **k):
        """Run fn; return (exit code or None, stdout, stderr)."""
        out, err = io.StringIO(), io.StringIO()
        code = None
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            try:
                fn(*a, **k)
            except SystemExit as e:
                code = e.code
        return code, out.getvalue(), err.getvalue()

    def sabotage(self):
        """Corrupt whatever lands on disk, by either write route."""
        real_replace, real_wt = os.replace, pathlib.Path.write_text

        def bad_replace(src, dst):
            real_replace(src, dst)
            with open(dst, "w") as f:
                f.write(SAB)

        def bad_wt(self_, data, *a, **k):
            return real_wt(self_, SAB)

        return mock.patch.multiple(os, replace=bad_replace), mock.patch.object(pathlib.Path, "write_text", bad_wt)


class Defect1Dry(Base):
    def test_dry_writes_nothing(self):
        p = self.mk("a.txt", "alpha\nbeta\n")
        code, out, _ = self.call(fpatch.patch, p, "beta", "gamma", dry=True)
        self.assertIsNone(code)
        self.assertEqual(self.rd(p), "alpha\nbeta\n")
        self.assertIn("DRY RUN -- nothing written. Seal:", out)


class Defect2AllOrNothing(Base):
    def test_third_edit_refused_writes_nothing(self):
        a = self.mk("a.txt", "one\n")
        b = self.mk("b.txt", "two\n")
        plan = fpatch.Plan().patch(a, "one", "ONE").patch(b, "two", "TWO").patch(b, "missing", "x")
        code, _, err = self.call(plan.show)
        self.assertEqual(code, 1)
        self.assertIn("edit 3 of 3 refused", err)
        self.assertEqual((self.rd(a), self.rd(b)), ("one\n", "two\n"))

    def test_apply_needs_the_seal(self):
        a = self.mk("a.txt", "one\n")
        plan = fpatch.Plan().patch(a, "one", "ONE")
        code, _, _ = self.call(plan.apply, "0000000000000000")
        self.assertEqual(code, 1)
        self.assertEqual(self.rd(a), "one\n")
        seal = plan._build()[4]
        code, out, _ = self.call(plan.apply, seal)
        self.assertIsNone(code)
        self.assertEqual(self.rd(a), "ONE\n")
        self.assertIn("verified on disk", out)

    def test_seal_moves_when_the_file_moves(self):
        a = self.mk("a.txt", "one\nx\n")
        plan = fpatch.Plan().patch(a, "one", "ONE")
        seal = plan._build()[4]
        self.mk("a.txt", "one\ny\n")
        code, _, _ = self.call(plan.apply, seal)
        self.assertEqual(code, 1)
        self.assertEqual(self.rd(a), "one\ny\n")

    def test_seal_ignores_colour(self):
        a = self.mk("a.txt", "one\n")
        plan = fpatch.Plan().patch(a, "one", "ONE")
        s0 = plan._build()[4]
        with mock.patch.dict(os.environ, {"FPATCH_COLOR": "1"}):
            s1 = plan._build()[4]
        self.assertEqual(s0, s1)

    def test_two_edits_one_file_chain(self):
        a = self.mk("a.txt", "one two\n")
        plan = fpatch.Plan().patch(a, "one", "ONE").patch(a, "ONE two", "ONE TWO")
        self.call(plan.apply, plan._build()[4])
        self.assertEqual(self.rd(a), "ONE TWO\n")


class Defect3BetweenGuards(Base):
    def test_between_verifies_on_disk(self):
        p = self.mk("a.txt", "a\nSTART\nold\nEND\nz\n")
        s1, s2 = self.sabotage()
        with s1, s2:
            code, out, _ = self.call(fpatch.patch_between, p, "START", "END", ["START", "new"])
        self.assertEqual(code, 2)
        self.assertNotIn("OK ", out)

    def test_between_noop_refused(self):
        p = self.mk("a.txt", "START\nold\nEND\n")
        code, _, _ = self.call(fpatch.patch_between, p, "START", "END", ["START", "old"])
        self.assertEqual(code, 1)

    def test_between_non_ascii_marker_refused(self):
        p = self.mk("a.txt", "## A \u2014 b\nold\nEND\n")
        code, _, _ = self.call(fpatch.patch_between, p, "\u2014", "END", ["x"])
        self.assertEqual(code, 1)
        self.assertEqual(self.rd(p), "## A \u2014 b\nold\nEND\n")


class Defect4ExactReadBack(Base):
    def test_deletion_is_verified(self):
        p = self.mk("a.txt", "keep\ndrop\n")
        s1, s2 = self.sabotage()
        with s1, s2:
            code, out, _ = self.call(fpatch.patch, p, "drop\n", "")
        self.assertEqual(code, 2)
        self.assertNotIn("verified on disk", out)


class Defect5Atomic(Base):
    def test_failed_write_leaves_original_whole(self):
        orig = "line\n" * 50
        p = self.mk("a.txt", orig)
        real_wt = pathlib.Path.write_text

        def half_then_die(self_, data, *a, **k):
            real_wt(self_, data[: len(data) // 2])
            raise OSError("disk went away")

        def die(src, dst):
            raise OSError("disk went away")

        with mock.patch.object(pathlib.Path, "write_text", half_then_die), mock.patch.object(os, "replace", die):
            code, _, _ = self.call(fpatch.patch, p, "line\nline\n", "LINE\nline\n", count=25)
        self.assertEqual(code, 2)
        self.assertEqual(self.rd(p), orig)
        self.assertEqual(sorted(os.listdir(self.dir)), ["a.txt"])

    def test_mode_bits_kept(self):
        p = self.mk("run.sh", "echo a\n")
        os.chmod(p, 0o755)
        self.call(fpatch.patch, p, "echo a", "echo b")
        self.assertEqual(os.stat(p).st_mode & 0o777, 0o755)

    def test_symlink_is_followed_not_replaced(self):
        real = self.mk("real.txt", "a\n")
        link = os.path.join(self.dir, "link.txt")
        os.symlink("real.txt", link)
        self.call(fpatch.patch, link, "a", "b")
        self.assertTrue(os.path.islink(link))
        self.assertEqual(self.rd(real), "b\n")


class Defect6EverySite(Base):
    def test_count_two_shows_both_sites(self):
        p = self.mk("a.txt", "foo\n" + "x\n" * 8 + "foo\n")
        code, out, _ = self.call(fpatch.patch, p, "foo", "bar", count=2)
        self.assertIsNone(code)
        self.assertIn(">> 10: foo", out)


class Defect7ToEndOfFile(Base):
    def test_between_to_eof(self):
        p = self.mk("a.md", "---\nid: 1\n---\n\n## Vision\nold\nmore\n")
        code, _, _ = self.call(fpatch.patch_between, p, "## Vision", None, ["## Vision", "new"])
        self.assertIsNone(code)
        self.assertEqual(self.rd(p), "---\nid: 1\n---\n\n## Vision\nnew\n")


class OldFormStillWorks(Base):
    def test_patch_as_documented(self):
        p = self.mk("a.rs", "fn a() {}\n")
        code, out, _ = self.call(fpatch.patch, p, "fn a() {}", "fn b() {}")
        self.assertIsNone(code)
        self.assertEqual(self.rd(p), "fn b() {}\n")
        self.assertIn("1 replaced, verified on disk", out)

    def test_refusal_still_writes_nothing(self):
        p = self.mk("a.rs", "fn a() {}\n")
        code, _, err = self.call(fpatch.patch, p, "nope", "x")
        self.assertEqual(code, 1)
        self.assertIn("No changes written to", err)
        self.assertEqual(self.rd(p), "fn a() {}\n")


if __name__ == "__main__":
    unittest.main()
