"""Fast feedback must fail on expired budgets and unsafe native output scopes."""
from argparse import Namespace
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from fastcheck_run import Steps
import fastcheck_run
import fastcheck


class QuietGates:
    """Stands in for the background gates while a test drives the compile and tests."""
    failure = None

    def __init__(self, *_):
        self.joined = []

    def join(self, *, report=True):
        self.joined.append(report)
        if report and self.failure is not None:
            raise self.failure


class FailedGate(QuietGates):
    failure = ValueError("assignment-metadata failed with status 1")


class FastcheckBudget(unittest.TestCase):
    def test_failed_gate_lets_the_compile_finish_and_stops_every_test(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "obj/audit-venv/bin").mkdir(parents=True)
            (root / "obj/audit-venv/bin/python").touch()
            binary = root / "suite_frontend"
            binary.touch()
            steps = mock.Mock()
            steps.receipt, steps.environment = {}, {"CARGO_BUILD_JOBS": "8"}
            steps.run.return_value = json.dumps({"reason": "compiler-artifact", "executable": str(binary),
                                                 "profile": {"test": True},
                                                 "target": {"kind": ["test"], "name": "suite_frontend"}})
            args = Namespace(suites=["lexer"], lib_case=[], ignored=False, prune_under=[], pa=[])
            with mock.patch.object(fastcheck_run, "discover", return_value={"lexer": "suite_frontend"}), \
                    mock.patch.object(fastcheck_run, "fingerprint", return_value={}), \
                    mock.patch.object(fastcheck_run.shutil, "which", return_value="clippy-driver"), \
                    mock.patch.object(fastcheck_run, "Beside", FailedGate):
                with self.assertRaisesRegex(ValueError, "assignment-metadata failed"):
                    fastcheck_run.execute(root, args, steps)
            # The compile ran beside the gates and kept its cache; no test step started.
            self.assertEqual(sorted(call.args[0] for call in steps.run.call_args_list),
                             ["build-public", "clippy-public"])

    def test_subprocess_timeout_retains_failure_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "dev").mkdir()
            (root / "dev/Cargo.toml").write_text('[package]\nname="test"\nversion="1.0.0"\n')
            args = Namespace(evidence_root=root / "obj/check", prune_under=[],
                             seconds=0.2, compile_seconds=60)
            steps = Steps(root, args)
            with self.assertRaisesRegex(TimeoutError, "hang: development feedback budget exhausted"):
                steps.run("hang", [sys.executable, "-c", "import time; time.sleep(30)"])
            self.assertEqual(steps.receipt["steps"][0]["status"], 124)
            self.assertTrue((root / "obj/check/receipt.json").is_file())

    def test_compiles_run_together_are_charged_their_common_time_once(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "dev").mkdir()
            (root / "dev/Cargo.toml").write_text('[package]\nname="test"\nversion="1.0.0"\n')
            args = Namespace(evidence_root=root / "obj/check", prune_under=[], seconds=300, compile_seconds=60)
            sleep = [sys.executable, "-c", "import time; time.sleep(0.4)"]
            with mock.patch("builtins.print"):
                steps = Steps(root, args)
                started = time.monotonic()
                fastcheck_run.together(steps, [("one", sleep, {}), ("two", sleep, {})])
                wall = time.monotonic() - started
                used = steps.receipt["budget"]["compile_used"]
                self.assertGreaterEqual(used, 0.4)
                self.assertLessEqual(used, wall)
                self.assertTrue(all(row["together"] for row in steps.receipt["steps"]))
                # A failure is reported only after the other compile has finished.
                failing = [sys.executable, "-c", "raise SystemExit(3)"]
                with self.assertRaisesRegex(ValueError, "fails failed with status 3"):
                    fastcheck_run.together(steps, [("fails", failing, {}), ("slow", sleep, {})])
            self.assertEqual([(row["name"], row["status"]) for row in steps.receipt["steps"][2:]],
                             [("fails", 3), ("slow", 0)])

    def scripted(self, root, script, **limits):
        """Run named steps of scripted durations against a virtual clock."""
        now = [1000.0]
        allowed = []

        def run(command, *, timeout, **_):
            allowed.append(round(timeout))
            seconds = float(command[0])
            now[0] += min(seconds, timeout)
            if seconds > timeout:
                raise subprocess.TimeoutExpired(command, timeout)
            return mock.Mock(returncode=0)

        (root / "dev").mkdir()
        (root / "dev/Cargo.toml").write_text('[package]\nname="test"\nversion="1.0.0"\n')
        args = Namespace(evidence_root=root / "obj/check", prune_under=[],
                         **{"seconds": 300, "compile_seconds": 1800, **limits})
        failure = None
        with mock.patch.object(fastcheck_run.time, "monotonic", lambda: now[0]), \
                mock.patch.object(fastcheck_run.subprocess, "run", side_effect=run), \
                mock.patch.object(fastcheck_run, "controller", return_value=None), \
                mock.patch("builtins.print"):
            steps = Steps(root, args)
            try:
                for name, seconds, compiles in script:
                    steps.run(name, [seconds], compiles=compiles)
            except TimeoutError as error:
                failure = str(error)
            steps.save()
        return allowed, failure, json.loads((root / "obj/check/receipt.json").read_text())

    def test_compiles_finish_outside_the_feedback_budget(self):
        with tempfile.TemporaryDirectory() as directory:
            allowed, failure, receipt = self.scripted(Path(directory), [
                ("format", 40, False), ("build", 900, True), ("suite", 200, False),
                ("build-cli", 100, True), ("pa1", 70, False)])
        # A fifteen-minute compile neither fails nor shortens the gates and tests.
        self.assertEqual(allowed, [300, 1800, 260, 900, 60])
        self.assertEqual(failure, "pa1: development feedback budget exhausted")
        self.assertEqual([(row["charged"], row["status"]) for row in receipt["steps"]],
                         [("feedback", 0), ("compile", 0), ("feedback", 0), ("compile", 0), ("feedback", 124)])
        self.assertEqual(receipt["budget"], {"feedback_seconds": 300, "compile_ceiling_seconds": 1800,
                                             "feedback_used": 300.0, "compile_used": 1000.0})

    def test_exhausted_feedback_budget_starts_no_further_gate_or_test(self):
        with tempfile.TemporaryDirectory() as directory:
            allowed, failure, _ = self.scripted(Path(directory), [
                ("format", 300, False), ("build", 5, True), ("suite", 1, False)])
        self.assertEqual(allowed, [300, 1800])
        self.assertEqual(failure, "suite: development feedback budget exhausted")

    def test_compile_ceiling_only_stops_a_hung_compiler(self):
        with tempfile.TemporaryDirectory() as directory:
            allowed, failure, receipt = self.scripted(Path(directory), [
                ("clippy", 1000, True), ("build", 5000, True)])
        self.assertEqual(allowed, [1800, 800])
        self.assertEqual(failure, "build: compile ceiling exhausted; the compiler looks hung")
        self.assertEqual(receipt["steps"][-1]["status"], 124)

    def test_build_jobs_and_test_threads_use_the_granted_cpus(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "dev").mkdir()
            (root / "dev/Cargo.toml").write_text('[package]\nname="test"\nversion="1.0.0"\n')
            cases = [((root, {"cpu_percent": 800}), 88, "8"), ((root, {"cpu_percent": 400}), 88, "4"),
                     (None, 4, "4"), (None, 88, "8")]
            for index, (found, cpus, expected) in enumerate(cases):
                args = Namespace(evidence_root=root / f"obj/check-{index}", prune_under=[],
                                 seconds=300, compile_seconds=1800)
                with self.subTest(found=found, cpus=cpus), \
                        mock.patch.object(fastcheck_run, "controller", return_value=found), \
                        mock.patch("resource_policy.os.cpu_count", return_value=cpus), \
                        mock.patch.dict("os.environ", {"RUSTGM_RESOURCE_TOKEN": "nonce", "CARGO_BUILD_JOBS": "2"}):
                    steps = Steps(root, args)
                    self.assertEqual((steps.environment["CARGO_BUILD_JOBS"], steps.environment["RUST_TEST_THREADS"]),
                                     (expected, expected))
                    self.assertEqual(steps.receipt["width"], int(expected))
                    self.assertEqual(steps.receipt["resource_receipt"], "obj/parallel/resources/nonce.json")

    def test_output_scope_must_stay_inside_obj(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            args = Namespace(evidence_root=root / "obj/check", prune_under=["dev"])
            with self.assertRaises(ValueError):
                Steps(root, args)

    def test_cache_retirement_precedes_resource_admission(self):
        with tempfile.TemporaryDirectory() as directory:
            events = []
            root = Path(directory)
            args = fastcheck.parser().parse_args(["lexer"])
            process = mock.Mock()
            process.wait.return_value = 0
            with mock.patch.object(fastcheck, "ROOT", root), \
                    mock.patch.object(fastcheck, "retire", side_effect=lambda _: events.append("retire")), \
                    mock.patch.object(fastcheck, "discover", return_value=None), \
                    mock.patch.object(fastcheck.subprocess, "Popen", side_effect=lambda *a, **k: (events.append(a[0]), process)[1]):
                self.assertEqual(fastcheck.launch(args, time.monotonic())[0], 0)
            self.assertEqual(events[0], "retire")
            # Admission queues by default instead of refusing a busy controller.
            self.assertEqual(events[1][2:5], ["run", "--wait-seconds", "3600"])
            self.assertNotIn("--deadline", events[1])
            receipt = json.loads(next((root / "obj/qualification/fast").glob("*.launcher.json")).read_text())
            self.assertEqual((receipt["state"], receipt["queued_seconds"]), ("passed", None))


class FastcheckAdmission(unittest.TestCase):
    def watch(self, admitted_at, finished_at, **limits):
        """Advance a virtual clock 100 seconds per poll of the launched command."""
        with tempfile.TemporaryDirectory() as directory:
            marker = Path(directory) / "run.admitted"
            now = [0.0]

            def wait(timeout):
                now[0] += 100
                if now[0] == admitted_at:
                    marker.touch()
                if now[0] == finished_at:
                    return 0
                raise subprocess.TimeoutExpired("fastcheck", timeout)

            process = mock.Mock()
            process.wait.side_effect = wait
            args = Namespace(**{"seconds": 300, "compile_seconds": 1800, "wait_seconds": 3600, **limits})
            with mock.patch.object(fastcheck, "terminate") as stop, mock.patch("builtins.print"):
                result = fastcheck.watch(process, marker, args, 0.0, clock=lambda: now[0])
            return result, stop.call_count, now[0]

    def test_queue_is_not_charged_and_a_long_compile_is_not_stopped(self):
        # Fifteen queued minutes, then a worker that compiles for half an hour, is a pass.
        self.assertEqual(self.watch(900, 2900), ((0, 900.0), 0, 2900.0))

    def test_only_a_worker_beyond_budget_and_ceiling_is_stopped(self):
        self.assertEqual(self.watch(900, None), ((124, 900.0), 1, 3200.0))
        self.assertEqual(self.watch(900, None, seconds=100, compile_seconds=60), ((124, 900.0), 1, 1200.0))

    def test_admission_wait_is_bounded(self):
        self.assertEqual(self.watch(None, None, wait_seconds=600), ((124, None), 1, 900.0))

    def test_launch_validates_the_admission_wait(self):
        for wait in ("-1", "3601"):
            args = fastcheck.parser().parse_args(["lexer", "--wait-seconds", wait])
            with self.subTest(wait=wait), self.assertRaisesRegex(ValueError, "admission wait"):
                fastcheck.launch(args, time.monotonic())
        for ceiling in ("59", "3601"):
            args = fastcheck.parser().parse_args(["lexer", "--compile-seconds", ceiling])
            with self.subTest(ceiling=ceiling), self.assertRaisesRegex(ValueError, "compile ceiling"):
                fastcheck.launch(args, time.monotonic())

    def test_worker_marks_admission_and_keeps_its_own_clocks(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory) / "run"
            seen = {}

            def check(_root, args):
                seen.update(limits=(args.seconds, args.compile_seconds),
                            marked=fastcheck.admission_marker(folder).is_file())
                return 0

            argv = ["fastcheck.py", "lexer", "--seconds", "100", "--worker", "--evidence-root", str(folder)]
            with mock.patch.object(sys, "argv", argv), mock.patch.object(fastcheck_run, "check", side_effect=check):
                self.assertEqual(fastcheck.main(), 0)
            self.assertEqual(seen, {"marked": True, "limits": (100.0, 1800)})


class PrivateHarnessSession(unittest.TestCase):
    """After a broad change the harness is compiled without its carried session."""

    def tree(self, root, count):
        (root / "dev/src").mkdir(parents=True)
        (root / "dev/tests/helpers").mkdir(parents=True)
        names = [f"dev/src/m{index}.rs" for index in range(count)] + ["dev/tests/helpers/shared.rs"]
        for name in names:
            (root / name).write_text(f"// {name}\n")
        deps = root / "obj/rust/private-lint/dev-fast/deps"
        deps.mkdir(parents=True)
        binary = deps / "rustgm-abcd"
        inputs = [name.removeprefix("dev/") for name in names[:-1]] + ["src/../tests/helpers/shared.rs"]
        (deps / "rustgm-abcd.d").write_text(f"{binary}: {' '.join(inputs)}\n\n" + "".join(
            f"{name}:\n" for name in inputs) + "\n# env-dep:CARGO_PKG_NAME=rustgm\n")
        session = root / "obj/rust/private-lint/dev-fast/incremental/rustgm-x/s-a-b"
        session.mkdir(parents=True)
        (session / "work-products.bin").write_bytes(b"carried")
        return names, binary, session

    def test_a_one_module_edit_keeps_the_session_and_a_broad_one_drops_it(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            names, binary, session = self.tree(root, 12)
            fastcheck_run.record_harness_inputs(root, binary)
            recorded = json.loads((root / fastcheck_run.HARNESS_INPUTS).read_text())["files"]
            self.assertEqual(sorted(recorded), sorted(names))
            self.assertEqual(fastcheck_run.harness_session(root), {"changed_inputs": 0, "carried": True})
            for name in names[:fastcheck_run.BROAD_CHANGE - 1]:
                (root / name).write_text("// edited\n")
            self.assertEqual(fastcheck_run.harness_session(root),
                             {"changed_inputs": fastcheck_run.BROAD_CHANGE - 1, "carried": True})
            self.assertTrue(session.is_dir())
            (root / names[-1]).write_text("// edited too\n")
            with mock.patch("builtins.print") as printed:
                decision = fastcheck_run.harness_session(root)
            self.assertEqual(decision, {"changed_inputs": fastcheck_run.BROAD_CHANGE, "carried": False,
                                        "dropped_sessions": 1})
            self.assertFalse(session.exists())
            self.assertIn("compiling without the carried incremental session", printed.call_args.args[0])
            # The record described the dropped session; the next success writes a new one.
            self.assertFalse((root / fastcheck_run.HARNESS_INPUTS).exists())

    def test_a_session_without_a_record_of_its_inputs_is_dropped(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _, _, session = self.tree(root, 1)
            with mock.patch("builtins.print"):
                self.assertEqual(fastcheck_run.harness_session(root),
                                 {"changed_inputs": None, "carried": False, "dropped_sessions": 1})
            self.assertFalse(session.exists())
            # With nothing carried there is nothing to drop and nothing to say.
            with mock.patch("builtins.print") as printed:
                self.assertEqual(fastcheck_run.harness_session(root),
                                 {"changed_inputs": None, "carried": True, "dropped_sessions": 0})
            printed.assert_not_called()


class LinkedPrivateHarness(unittest.TestCase):
    def fixture(self, root):
        (root / "dev/src").mkdir(parents=True)
        (root / "dev/Cargo.toml").write_text('[package]\nname="rustgm"\nversion="1.0.0"\n')
        source = root / "dev/src/lib.rs"
        source.write_text("// test input\n")
        folder = root / "obj/rust/private-lint/dev-fast/deps"
        folder.mkdir(parents=True)
        binary = folder / ("rustgm-abcd.exe" if sys.platform == "win32" else "rustgm-abcd")
        binary.write_bytes(b"linked harness")
        binary.chmod(0o755)
        record = {"reason": "compiler-artifact", "manifest_path": str(root / "dev/Cargo.toml"),
                  "target": {"kind": ["lib"], "name": "rustgm", "src_path": str(source)},
                  "profile": {"test": True}, "executable": str(binary), "fresh": False}
        return record, binary

    def test_each_selection_is_linted_and_linked_by_one_compile(self):
        for mixed, text in ((False, False), (True, False), (False, True), (True, True)):
            with self.subTest(mixed=mixed, text=text), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                record, binary = self.fixture(root)
                audit = root / "obj/audit-venv/bin/python"
                audit.parent.mkdir(parents=True)
                audit.touch()
                commands = []
                steps = mock.Mock()
                steps.receipt, steps.environment = {}, {"CARGO_BUILD_JOBS": "8"}
                steps.tests = {"RUSTUP_TOOLCHAIN": "pinned"}

                def run(name, command, **_):
                    commands.append((name, command))
                    if command[:2] != ["cargo", "fmt"] and command[0] == "cargo":
                        # Every Cargo compile is never killed; only checks and the harness lint.
                        self.assertTrue(_["compiles"])
                        self.assertNotIn("RUSTFLAGS", _["overrides"])
                        linted = name.startswith("clippy-")
                        self.assertEqual(_["overrides"].get("RUSTC_WORKSPACE_WRAPPER"),
                                         "test-clippy-driver" if linted else None)
                        self.assertEqual(_["overrides"].get("CLIPPY_ARGS"), "-Dwarnings" if linted else None)
                        self.assertEqual(command[1], {"clippy-private": "test", "build-public": "test",
                                                      "build-cli": "build"}.get(name, "check"))
                        if name != "clippy-private":
                            # The build and its check run together on half the width each.
                            self.assertTrue(_["together"])
                            self.assertEqual(_["overrides"]["CARGO_BUILD_JOBS"], "4")
                    if name in ("clippy-public", "clippy-cli"):
                        self.assertTrue(_["overrides"]["CARGO_TARGET_DIR"].endswith("obj/rust/public-lint"))
                        self.assertEqual(command[-1], "--message-format=json-render-diagnostics")
                        self.assertIn("--bins", command)
                        self.assertNotIn("--no-run", command)
                    if name == "clippy-private":
                        self.assertEqual(command[:2], ["cargo", "test"])
                        self.assertIn("--lib", command)
                        self.assertIn("--no-run", command)
                        self.assertTrue(_["overrides"]["CARGO_TARGET_DIR"].endswith("obj/rust/private-lint"))
                        return json.dumps(record)
                    if name == "build-public":
                        self.assertNotIn("--lib", command)
                        self.assertIn("--no-run", command)
                        self.assertIn("--message-format=json-render-diagnostics", command)
                        self.assertNotIn("CARGO_TARGET_DIR", _["overrides"])
                        return json.dumps({"reason": "compiler-artifact", "executable": str(binary),
                                           "profile": {"test": True},
                                           "target": {"kind": ["test"], "name": "suite_frontend"}})
                    if name == "build-cli":
                        self.assertEqual(command[:2], ["cargo", "build"])
                        self.assertEqual(command[-2:], ["--bin", "rustgm"])
                    if name.endswith("-inventory"):
                        return "sample::case: test\n" if name == "lib-inventory" else "lexer::valid: test\n"
                    if name in ("lib", "suite_frontend"):
                        # Only test executions name the toolchain; the builds keep the file's selection.
                        self.assertEqual(_["overrides"], {"RUSTUP_TOOLCHAIN": "pinned"})
                        return "test result: ok. 1 passed; 0 failed; 0 ignored;\n"
                    return ""

                steps.run.side_effect = run
                args = Namespace(suites=["lexer::valid"] if mixed else [], lib_case=["sample::case"],
                                 ignored=False, prune_under=[], pa=["pa1"] if text else [])
                with mock.patch.object(fastcheck_run, "discover", return_value={"lexer": "suite_frontend"}), \
                        mock.patch.object(fastcheck_run.shutil, "which", return_value="test-clippy-driver"), \
                        mock.patch.object(fastcheck_run, "fingerprint", return_value={}), \
                        mock.patch.object(fastcheck_run, "Beside", QuietGates):
                    fastcheck_run.execute(root, args, steps)
                compiles = [name for name, command in commands if command[0] == "cargo" and name != "format"]
                self.assertEqual(sorted(compiles), sorted(["clippy-private"] + ["build-public", "clippy-public"] * mixed
                                                          + ["build-cli", "clippy-cli"] * text))
                self.assertEqual(steps.receipt["private_harness_producer"], "clippy-private")

    def test_public_selection_builds_once_and_checks_once_beside_it(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _, binary = self.fixture(root)
            audit = root / "obj/audit-venv/bin/python"
            audit.parent.mkdir(parents=True)
            audit.touch()
            commands = []
            steps = mock.Mock()
            steps.receipt, steps.environment = {}, {"CARGO_BUILD_JOBS": "8"}

            def run(name, command, **_):
                if name != "format":
                    commands.append(command[:2])
                if name == "build-public":
                    return json.dumps({"reason": "compiler-artifact", "executable": str(binary),
                                       "profile": {"test": True},
                                       "target": {"kind": ["test"], "name": "suite_frontend"}})
                if name == "suite_frontend-inventory":
                    return "lexer::valid: test\n"
                return "test result: ok. 1 passed; 0 failed; 0 ignored;\n" if name == "suite_frontend" else ""

            steps.run.side_effect = run
            args = Namespace(suites=["lexer"], lib_case=[], ignored=False, prune_under=[], pa=[])
            with mock.patch.object(fastcheck_run, "discover", return_value={"lexer": "suite_frontend"}), \
                    mock.patch.object(fastcheck_run.shutil, "which", return_value="test-clippy-driver"), \
                    mock.patch.object(fastcheck_run, "fingerprint", return_value={}), \
                    mock.patch.object(fastcheck_run, "Beside", QuietGates):
                fastcheck_run.execute(root, args, steps)
            # rustc compiles the edit once and clippy-driver checks it once, at the same time.
            self.assertEqual(sorted(command for command in commands if command[0] == "cargo"),
                             [["cargo", "check"], ["cargo", "test"]])
            steps.charge.assert_called_once()
            with mock.patch.object(fastcheck_run, "discover", return_value={"lexer": "suite_frontend"}), \
                    mock.patch.object(fastcheck_run.shutil, "which", return_value=None), \
                    mock.patch.object(fastcheck_run, "fingerprint", return_value={}), \
                    mock.patch.object(fastcheck_run, "Beside", QuietGates):
                with self.assertRaisesRegex(ValueError, "clippy-driver"):
                    fastcheck_run.execute(root, args, steps)
