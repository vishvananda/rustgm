"""The integration check runs every part, reports each on one line and fails if any part failed."""
from argparse import Namespace
import json
import os
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
    def run_check(self, root, *, build_fails=False, room=None, marks=None):
        (root / "dev").mkdir(parents=True)
        (root / "dev/Cargo.toml").write_text('[package]\nname="test"\nversion="1.0.0"\n')
        (root / "obj/audit-venv/bin").mkdir(parents=True)
        (root / "obj/audit-venv/bin/python").touch()
        binaries = {name: root / name for name in ("lib", "suite_frontend", "suite_traits")}
        commands = []
        self.environments = {}

        def run(command, *, stdout, **_):
            commands.append(command)
            name = " ".join(command[:2]) if command[0] == "cargo" else (
                Path(command[0]).name if command[0] != sys.executable else command[2])
            self.environments[name] = _["env"]
            if command[:2] == ["cargo", "build"]:
                (root / "obj/rust/dev-fast").mkdir(parents=True)
                (root / "obj/rust/dev-fast/rustgm").touch()
                return mock.Mock(returncode=0)
            if command[:2] == ["cargo", "check"]:
                # The CLI's strict check, beside its build.
                return mock.Mock(returncode=0)
            if name == "pa3":
                raise subprocess.TimeoutExpired(command, 1)
            status, text = OUTPUTS[name]
            stdout.write(text)
            return mock.Mock(returncode=status)

        def build(*arguments):
            self.build_flags = arguments[4:]
            if build_fails:
                raise ValueError("clippy-private failed with status 101")
            return binaries

        args = Namespace(stages=["pa1", "pa2", "pa3"], seconds=600, compile_seconds=600,
                         evidence_root=root / "obj/qualification/integration/run", compare=None)
        memory = mock.patch.object(integration_check, "MemoryMarks", marks) if marks else mock.patch.object(
            integration_check, "own_cgroup", return_value=None)
        with mock.patch.object(integration_check, "discover",
                               return_value={"lexer": "suite_frontend", "traits": "suite_traits"}), \
                mock.patch.object(integration_check, "fingerprint", return_value={}), \
                mock.patch.object(integration_check, "Beside", RecordedGates), \
                mock.patch.object(integration_check, "build", side_effect=build), \
                mock.patch.object(integration_check, "check_space"), \
                mock.patch.object(integration_check, "lease_room", return_value=room), memory, \
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
        # The public build and the CLI it runs are optimized; their strict checks are not.
        self.assertEqual(self.build_flags, (integration_check.OPTIMIZED,))
        cargo = [command for command in commands if command[0] == "cargo"]
        self.assertEqual([command[1] for command in cargo if "profile.dev-fast.opt-level=1" in command], ["build"])
        summary = "\n".join(lines).split("\n")
        self.assertEqual(summary[-2], "Integration: 5 of 10 parts passed; "
                                      "failed: gate format, suite_frontend, suite_traits, pa2, pa3")
        self.assertIn("  FAIL pa2: 1/2 cases; status 1 (pa2/b.rs)", summary)

    def test_test_binaries_name_the_pinned_toolchain_and_compiles_do_not(self):
        with tempfile.TemporaryDirectory() as directory, mock.patch.dict(os.environ):
            os.environ.pop("RUSTUP_TOOLCHAIN", None)
            root = Path(directory)
            (root / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.98.0"\n')
            self.run_check(root)
        self.assertEqual({name: environment.get("RUSTUP_TOOLCHAIN") for name, environment in self.environments.items()},
                         {"cargo build": None, "cargo check": None, "lib": "1.98.0", "suite_frontend": "1.98.0",
                          "suite_traits": "1.98.0", "pa1": None, "pa2": None, "pa3": None})

    def test_failing_names_are_one_flat_list_and_compare_reports_both_directions(self):
        expected = ["gate format", "pa2::b.rs", "pa3", "suite_frontend::lexer::bad", "suite_frontend::lexer::worse",
                    "suite_traits"]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _, receipt, _, _ = self.run_check(root / "plain")
            self.assertEqual(receipt["failing"], expected)
            folder = root / "plain/obj/qualification/integration/run"
            earlier = root / "earlier.json"
            earlier.write_text(json.dumps({"parts": [], "failing": [*expected[:3], *expected[4:], "lib::old::case"]}))
            with mock.patch("builtins.print") as printed:
                self.assertEqual(integration_check.compared(folder, earlier), 1)
            lines = [call.args[0] for call in printed.call_args_list]
            self.assertEqual(lines[1:], ["  NEW suite_frontend::lexer::bad", "  FIXED lib::old::case"])
            self.assertEqual(json.loads((folder / "receipt.json").read_text())["compare"],
                             {"receipt": str(earlier), "new": ["suite_frontend::lexer::bad"],
                              "fixed": ["lib::old::case"]})
            # Known failures alone pass the comparison, though the run itself failed; an
            # older receipt without the flat list is read from its parts.
            (root / "same").mkdir()
            (root / "same/receipt.json").write_text(json.dumps({"parts": receipt["parts"]}))
            with mock.patch("builtins.print"):
                self.assertEqual(integration_check.compared(folder, root / "same"), 0)
            self.assertEqual(json.loads((folder / "receipt.json").read_text())["state"], "failed")
            for broken in ({"failing": []}, [1]):
                earlier.write_text(json.dumps(broken))
                with self.subTest(broken=broken), self.assertRaisesRegex(ValueError, "not an integration receipt"):
                    integration_check.compared(folder, earlier)

    def test_the_launcher_compares_after_the_run_and_returns_the_comparisons_status(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            folder = root / "run"
            folder.mkdir()
            (folder / "receipt.json").write_text(json.dumps({"parts": [], "state": "failed",
                                                             "failing": ["suite_traits::known"]}))
            earlier = root / "earlier.json"
            earlier.write_text(json.dumps({"parts": [], "failing": ["suite_traits::known"]}))
            argv = ["integration_check.py", "pa1", "--compare", str(earlier)]
            with mock.patch.object(sys, "argv", argv), mock.patch("builtins.print"), \
                    mock.patch.object(integration_check, "resources", return_value=None), \
                    mock.patch.object(integration_check, "launch", return_value=(1, folder)) as launched:
                self.assertEqual(integration_check.main(), 0)
            launched.assert_called_once()
            # An unreadable comparison receipt fails before queueing.
            argv[-1] = str(root / "missing.json")
            with mock.patch.object(sys, "argv", argv), mock.patch("builtins.print"), \
                    mock.patch.object(integration_check, "launch") as launched:
                self.assertEqual(integration_check.main(), 1)
            launched.assert_not_called()

    def test_too_little_room_under_the_output_cap_stops_before_compiling(self):
        with tempfile.TemporaryDirectory() as directory:
            gib = integration_check.GIB
            status, receipt, lines, commands = self.run_check(Path(directory), room=(36 * gib, 35 * gib))
        self.assertEqual(status, 1)
        self.assertEqual([row["part"] for row in receipt["parts"]], ["integration"])
        detail = receipt["parts"][0]["detail"]
        self.assertIn("only 1.0 GiB of the 36 GiB output cap remain", detail)
        self.assertIn("nothing was compiled", detail)
        self.assertIn("make tidy", detail)
        self.assertEqual(commands, [])
        self.assertNotIn("steps", [row.get("name") for row in receipt["steps"]])

    def test_a_step_above_ten_gib_is_warned_about(self):
        class Marks:
            def __init__(self, steps, _folder):
                self.steps = steps

            def close(self):
                self.steps.receipt["memory_anon_peak_gib"] = {"clippy-private": 10.4, "lib": 0.6}
                return "clippy-private", 10.4

        with tempfile.TemporaryDirectory() as directory:
            _, receipt, lines, _ = self.run_check(Path(directory), marks=Marks)
        self.assertEqual(receipt["memory_warnings"], {"clippy-private": 10.4})
        self.assertIn("WARNING: clippy-private peaked at 10.40 GiB of anonymous memory, above 10 GiB of the "
                      "job's memory cap", lines)

    def test_expected_growth_counts_each_crates_newest_session_or_a_cold_cache(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            gib = integration_check.GIB
            crate = root / "obj/rust/dev-fast/incremental/rustgm-abc"
            for name, size in (("s-old-1", 3000), ("s-new-2", 8192)):
                (crate / name).mkdir(parents=True)
                (crate / name / "query-cache.bin").write_bytes(b"x" * size)
                os.utime(crate / name, ns=(1, 1) if name == "s-old-1" else (2, 2))
            # A hardlinked work product is shared with the carried session, so it is not growth.
            os.link(crate / "s-new-2/query-cache.bin", crate / "s-new-2/shared.o")
            (crate / "s-new-2/own.o").write_bytes(b"y" * 4096)
            growth = integration_check.expected_growth(root)
        cold = int(4.5 * gib) + int(1.2 * gib) + 2 * gib
        self.assertEqual(growth, cold + 4096)

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
