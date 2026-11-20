"""A cached gate is skipped only when every input it reads is unchanged."""
from argparse import Namespace
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import time
import unittest
from unittest import mock

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
import fastcheck_gates
from fastcheck_gates import Digests, Gates, interpreter_paths
import fastcheck_run
from fastcheck_run import Steps, format_gate, traced_gate


def aged(path, seconds=60):
    """Move a file's mtime out of the racy window; its ctime still changes."""
    old = time.time() - seconds
    os.utime(path, (old, old))


class ToyGates(Gates):
    """A traced Python gate over one declared file, for end-to-end checks."""

    def toy(self):
        command = [sys.executable, "scripts/toy_gate.py"]
        inputs = self.digests.files(["declared.txt", "scripts/toy_gate.py"])
        allowed = [self.root / "declared.txt", self.root / "scripts/toy_gate.py", *interpreter_paths()]
        return command, fastcheck_gates.digest_text([command, inputs]), allowed, []


class GateTests(unittest.TestCase):
    def project(self, root):
        (root / "dev/src").mkdir(parents=True)
        (root / "dev/Cargo.toml").write_text('[package]\nname = "toy"\nversion = "1.0.0"\nedition = "2024"\n')
        for name in ("a", "b"):
            (root / f"dev/src/{name}.rs").write_text(f"fn {name}() {{}}\n")
            aged(root / f"dev/src/{name}.rs")
        (root / "scripts").mkdir()
        for name in fastcheck_gates.IMPLEMENTATION:
            shutil.copy(SCRIPTS.parent / name, root / name)

    def steps(self, root, name="run"):
        args = Namespace(evidence_root=root / f"obj/qualification/fast/{name}", prune_under=[],
                         seconds=300, compile_seconds=1800)
        with mock.patch.object(fastcheck_run, "controller", return_value=None):
            return Steps(root, args)

    def test_content_change_is_seen_even_with_the_same_size_and_mtime(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "input.txt"
            path.write_text("first\n")
            aged(path)
            digests = Digests(root)
            first = digests.file(path)
            digests.save()
            stamp = path.stat().st_mtime_ns
            path.write_text("other\n")
            os.utime(path, ns=(stamp, stamp))
            self.assertNotEqual(Digests(root).file(path), first)
            # A file written moments ago is hashed again next time, never trusted.
            fresh = root / "fresh.txt"
            fresh.write_text("new\n")
            digests = Digests(root)
            digests.file(fresh)
            self.assertNotIn("fresh.txt", digests.used)

    def test_format_checks_changed_files_and_falls_back_to_the_complete_check(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.project(root)
            with mock.patch.object(fastcheck_gates, "tool_version", return_value=[0, "rustfmt 1", ""]):
                gates = Gates(root, root / "obj/audit-venv/bin/python")
                command, key = gates.format_key()
                self.assertEqual(gates.format_plan(key), ("full", []))
                gates.passed("format", gates.format_fingerprint(key), "obj/r", key=key, files=gates.sources)
                self.assertEqual(Gates(root, None).format_plan(key), ("skip", []))
                (root / "dev/src/a.rs").write_text("fn a() { }\n")
                (root / "dev/src/c.rs").write_text("fn c() {}\n")
                gates = Gates(root, None)
                self.assertEqual(gates.format_plan(key), ("partial", ["dev/src/a.rs", "dev/src/c.rs"]))
                steps = mock.Mock()
                steps.folder = root / "obj/qualification/fast/partial"
                runs = []

                def run(name, command, **_):
                    runs.append((name, command[:4]))
                    if name == "format":
                        raise ValueError("format failed with status 1")
                    return ""

                steps.run.side_effect = run
                format_gate(root, steps, gates)
                self.assertEqual(runs, [("format", ["rustfmt", "--check", "--edition", "2024"]),
                                        ("format-full", command[:4])])
                # Only the complete check's pass is recorded, under this fingerprint.
                record = gates.record("format")
                self.assertEqual(record["passed"][0]["receipt"], "obj/qualification/fast/partial")
                self.assertEqual(record["files"], gates.sources)
                (root / "dev/src/b.rs").unlink()
                self.assertEqual(Gates(root, None).format_plan(key), ("full", []))
            with mock.patch.object(fastcheck_gates, "tool_version", return_value=[0, "rustfmt 2", ""]):
                self.assertEqual(Gates(root, None).format_plan(Gates(root, None).format_key()[1])[0], "full")

    def test_a_skip_needs_a_receipt_that_still_records_the_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.project(root)
            gates = Gates(root, None)
            gates.passed("architecture", "f" * 64, "obj/qualification/fast/old")
            self.assertIsNone(gates.matched("architecture", "f" * 64))
            folder = root / "obj/qualification/fast/old"
            folder.mkdir(parents=True)
            for status, skipped, expected in ((1, False, None), (0, True, None),
                                              (0, False, "obj/qualification/fast/old")):
                (folder / "receipt.json").write_text(json.dumps({"steps": [{
                    "name": "architecture", "gate": "architecture", "status": status,
                    "fingerprint": "f" * 64, **({"skipped": True} if skipped else {})}]}))
                with self.subTest(status=status, skipped=skipped):
                    self.assertEqual(gates.matched("architecture", "f" * 64), expected)
            self.assertIsNone(gates.matched("architecture", "e" * 64))

    def run_toy(self, root, name, source):
        (root / "scripts/toy_gate.py").write_text(source)
        aged(root / "scripts/toy_gate.py")
        steps = self.steps(root, name)
        with mock.patch("builtins.print"):
            try:
                traced_gate(root, steps, ToyGates(root, None), "toy", "toy")
            except ValueError as error:
                return steps, str(error)
        return steps, None

    def test_traced_gate_is_cached_only_when_it_read_nothing_undeclared(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.project(root)
            (root / "declared.txt").write_text("input\n")
            (root / "outside.txt").write_text("hidden\n")
            for path in (root / "declared.txt", root / "outside.txt"):
                aged(path)
            reads = "import pathlib, sys\nsys.exit(pathlib.Path({!r}).read_text() != 'input\\n')\n"
            first, error = self.run_toy(root, "first", reads.format("declared.txt"))
            self.assertIsNone(error)
            again, error = self.run_toy(root, "again", reads.format("declared.txt"))
            row = again.receipt["steps"][0]
            self.assertTrue(row["skipped"])
            self.assertEqual(row["matched_receipt"], "obj/qualification/fast/first")
            self.assertEqual(row["fingerprint"], first.receipt["steps"][0]["fingerprint"])
            # Reading an undeclared file passes this time but is never reused.
            hidden = "import pathlib\npathlib.Path('outside.txt').read_text()\n"
            for name in ("hidden", "hidden-again"):
                steps, error = self.run_toy(root, name, hidden)
                self.assertIsNone(error)
                row = steps.receipt["steps"][0]
                self.assertNotIn("skipped", row)
                self.assertFalse(row["cached"])
                self.assertTrue(any("outside.txt" in reason for reason in row["uncached_because"]))
            # Listing an undeclared directory is an input too.
            (root / "elsewhere").mkdir()
            steps, error = self.run_toy(root, "listing", "import os\nos.listdir('elsewhere')\n")
            self.assertIsNone(error)
            self.assertTrue(any("listed" in reason and "elsewhere" in reason
                                for reason in steps.receipt["steps"][0]["uncached_because"]))
            # A failing gate is recorded as failed and runs again.
            for name in ("failing", "failing-again"):
                steps, error = self.run_toy(root, name, "raise SystemExit(3)\n")
                self.assertEqual(error, "toy failed with status 3")
                self.assertNotIn("skipped", steps.receipt["steps"][0])


if __name__ == "__main__":
    unittest.main()
