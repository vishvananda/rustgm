"""The integration check runs every part, reports each on one line and fails if any part failed."""
from argparse import Namespace
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import fastcheck_run
import integration_check


class RecordedGates:
    """Gate rows as the background gates would leave them: one skip, one failure, one pass."""

    def __init__(self, _root, steps, _python, _digests):
        self.errors = [ValueError("format failed with status 1")]
        steps.skip("assignment-metadata", "f" * 64, "obj/qualification/fast/old")
        with steps.lock:
            steps.receipt["steps"] += [{"name": "format", "gate": "format", "status": 1, "seconds": 1.0},
                                       {"name": "architecture", "gate": "architecture", "status": 0,
                                        "seconds": 2.0}]

    def join(self, *, report=True):
        assert not report


OUTPUTS = {
    "lib": (0, "test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured\n"),
    "suite_frontend": (101, "failures:\n\n---- lexer::bad stdout ----\npanicked\n\nfailures:\n"
                            "    lexer::bad\n    lexer::worse\n\n"
                            "test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured\n"),
    "suite_traits": (0, "running 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured\n"),
    "pa1": (0, "pa1/a.rs: PASS (1/1)\npa1: 1/1\n"),
    "pa2": (1, "pa2/b.rs: FAIL (0/1)\nstdout differs\npa2/c.rs: PASS (1/1)\npa2: 1/2\n"),
}


class IntegrationCheck(unittest.TestCase):
    def run_check(self, root, *, build_fails=False):
        (root / "dev").mkdir()
        (root / "dev/Cargo.toml").write_text('[package]\nname="test"\nversion="1.0.0"\n')
        (root / "obj/audit-venv/bin").mkdir(parents=True)
        (root / "obj/audit-venv/bin/python").touch()
        binaries = {name: root / name for name in ("lib", "suite_frontend", "suite_traits")}
        commands = []

        def run(command, *, stdout, **_):
            commands.append(command)
            if command[:2] == ["cargo", "build"]:
                (root / "obj/rust/dev-fast").mkdir(parents=True)
                (root / "obj/rust/dev-fast/rustgm").touch()
                return mock.Mock(returncode=0)
            if command[:2] == ["cargo", "check"]:
                # The CLI's strict check, beside its build.
                return mock.Mock(returncode=0)
            name = Path(command[0]).name if command[0] != sys.executable else command[2]
            if name == "pa3":
                raise subprocess.TimeoutExpired(command, 1)
            status, text = OUTPUTS[name]
            stdout.write(text)
            return mock.Mock(returncode=status)

        def build(*_):
            if build_fails:
                raise ValueError("clippy-private failed with status 101")
            return binaries

        args = Namespace(stages=["pa1", "pa2", "pa3"], seconds=600, compile_seconds=600,
                         evidence_root=root / "obj/qualification/integration/run")
        with mock.patch.object(integration_check, "discover",
                               return_value={"lexer": "suite_frontend", "traits": "suite_traits"}), \
                mock.patch.object(integration_check, "fingerprint", return_value={}), \
                mock.patch.object(integration_check, "Beside", RecordedGates), \
                mock.patch.object(integration_check, "build", side_effect=build), \
                mock.patch.object(integration_check, "check_space"), \
                mock.patch.object(fastcheck_run, "controller", return_value=None), \
                mock.patch.object(fastcheck_run.shutil, "which", return_value="clippy-driver"), \
                mock.patch.object(fastcheck_run.subprocess, "run", side_effect=run), \
                mock.patch("builtins.print") as printed:
            status = integration_check.check(root, args)
        receipt = json.loads((args.evidence_root / "receipt.json").read_text())
        lines = [call.args[0] for call in printed.call_args_list if call.args]
        return status, receipt, lines, commands

    def test_every_part_runs_after_failures_and_each_gets_one_line(self):
        with tempfile.TemporaryDirectory() as directory:
            status, receipt, lines, commands = self.run_check(Path(directory))
        self.assertEqual(status, 1)
        self.assertEqual(receipt["state"], "failed")
        rows = {row["part"]: row for row in receipt["parts"]}
        self.assertEqual({name: row["passed"] for name, row in rows.items()}, {
            "build": True, "gate assignment-metadata": True, "gate format": False, "gate architecture": True,
            "lib": True, "suite_frontend": False, "suite_traits": False, "pa1": True, "pa2": False, "pa3": False})
        self.assertIn("skipped: fingerprint ffffffffffffffff", rows["gate assignment-metadata"]["detail"])
        self.assertEqual(rows["suite_frontend"]["failures"], ["lexer::bad", "lexer::worse"])
        self.assertEqual(rows["suite_frontend"]["detail"], "2 of 5 failed; status 101")
        # An empty test binary is a failure, not empty-suite success.
        self.assertIn("ran no tests", rows["suite_traits"]["detail"])
        self.assertEqual(rows["pa2"]["failures"], ["pa2/b.rs"])
        self.assertEqual(rows["pa3"]["detail"], "status 124; no case summary")
        # The course runs against the CLI just built, at the granted width.
        course = [command for command in commands if command[0] == sys.executable]
        self.assertTrue(all(command[3:5] == ["--app", str(Path(directory) / "obj/rust/dev-fast/rustgm")]
                            for command in course))
        self.assertTrue(all(command[5] == "--jobs" for command in course))
        summary = "\n".join(lines).split("\n")
        self.assertEqual(summary[-2], "Integration: 5 of 10 parts passed; "
                                      "failed: gate format, suite_frontend, suite_traits, pa2, pa3")
        self.assertIn("  FAIL pa2: 1/2 cases; status 1 (pa2/b.rs)", summary)

    def test_memory_marks_charge_each_sample_to_the_steps_running_then(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            stat = folder / "memory.stat"
            stat.write_text("anon 1073741824\nfile 5\n")
            steps = mock.Mock(receipt={}, running={"clippy-private", "format"})
            steps.lock = threading.RLock()
            marks = integration_check.MemoryMarks(steps, folder, interval=0.01)
            time.sleep(0.1)
            with steps.lock:
                steps.running = {"suite_patterns"}
                stat.write_text("anon 3221225472\n")
            time.sleep(0.1)
            highest = marks.close()
        self.assertEqual(highest, ("suite_patterns", 3.0))
        self.assertEqual(steps.receipt["memory_anon_peak_gib"],
                         {"suite_patterns": 3.0, "clippy-private": 1.0, "format": 1.0})
        # Without a readable cgroup nothing is sampled and nothing is claimed.
        quiet = mock.Mock(receipt={}, running=set())
        quiet.lock = threading.RLock()
        self.assertIsNone(integration_check.MemoryMarks(quiet, None).close())
        self.assertEqual(quiet.receipt["memory_anon_peak_gib"], {})

    def test_a_failed_build_reports_every_test_part_as_not_run(self):
        with tempfile.TemporaryDirectory() as directory:
            status, receipt, _, commands = self.run_check(Path(directory), build_fails=True)
        self.assertEqual(status, 1)
        rows = {row["part"]: row for row in receipt["parts"]}
        self.assertEqual(rows["build"]["detail"], "clippy-private failed with status 101")
        for name in ("lib", "suite_frontend", "suite_traits", "pa1", "pa2", "pa3"):
            self.assertEqual((rows[name]["passed"], rows[name]["detail"]), (False, "not run: the build failed"))
        self.assertEqual(commands, [])


if __name__ == "__main__":
    unittest.main()
