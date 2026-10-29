"""Fast feedback must fail on expired budgets and unsafe native output scopes."""
from argparse import Namespace
import json
from pathlib import Path
import sys
import tempfile
import time
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from fastcheck_run import Steps
import fastcheck_run
import fastcheck


class FastcheckBudget(unittest.TestCase):
    def test_metadata_failure_stops_before_format_lint_and_build(self):
        steps = mock.Mock()
        steps.receipt = {}
        steps.run.side_effect = ValueError("invalid assignment metadata")
        args = Namespace(suites=["lexer"], lib_case=[], ignored=False, prune_under=[])
        with mock.patch.object(fastcheck_run, "discover", return_value={"lexer": "suite_frontend"}), \
                mock.patch.object(fastcheck_run, "fingerprint", return_value={}):
            with self.assertRaisesRegex(ValueError, "invalid assignment metadata"):
                fastcheck_run.execute(Path("unused"), args, steps)
        steps.run.assert_called_once_with("assignment-metadata", [sys.executable, "scripts/assignment_metadata.py"])

    def test_subprocess_timeout_retains_failure_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "dev").mkdir()
            (root / "dev/Cargo.toml").write_text('[package]\nname="test"\nversion="1.0.0"\n')
            args = Namespace(evidence_root=root / "obj/check", prune_under=[],
                             deadline=time.monotonic() + 0.2)
            steps = Steps(root, args)
            with self.assertRaises(TimeoutError):
                steps.run("hang", [sys.executable, "-c", "import time; time.sleep(30)"])
            self.assertEqual(steps.receipt["steps"][0]["status"], 124)
            self.assertTrue((root / "obj/check/receipt.json").is_file())

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
                    mock.patch.object(fastcheck.subprocess, "Popen", side_effect=lambda *a, **k: (events.append("admit"), process)[1]):
                self.assertEqual(fastcheck.launch(args, time.monotonic()), 0)
            self.assertEqual(events, ["retire", "admit"])


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

    def test_private_only_links_once_and_mixed_selection_builds_only_remaining_target(self):
        for mixed in (False, True):
            with self.subTest(mixed=mixed), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                record, binary = self.fixture(root)
                audit = root / "obj/audit-venv/bin/python"
                audit.parent.mkdir(parents=True)
                audit.touch()
                commands = []
                steps = mock.Mock()
                steps.receipt = {}

                def run(name, command, **_):
                    commands.append((name, command))
                    if name == "clippy-private":
                        self.assertEqual(command[:2], ["cargo", "test"])
                        self.assertIn("--lib", command)
                        self.assertIn("--no-run", command)
                        self.assertEqual(_["overrides"]["RUSTC_WORKSPACE_WRAPPER"], "test-clippy-driver")
                        self.assertEqual(_["overrides"]["CLIPPY_ARGS"], "-Dwarnings")
                        self.assertNotIn("RUSTFLAGS", _["overrides"])
                        return json.dumps(record)
                    if name == "build":
                        self.assertNotIn("--lib", command)
                        return json.dumps({"reason": "compiler-artifact", "executable": str(binary),
                                           "profile": {"test": True},
                                           "target": {"kind": ["test"], "name": "suite_frontend"}})
                    if name.endswith("-inventory"):
                        return "sample::case: test\n" if name == "lib-inventory" else "lexer::valid: test\n"
                    if name in ("lib", "suite_frontend"):
                        return "test result: ok. 1 passed; 0 failed; 0 ignored;\n"
                    return ""

                steps.run.side_effect = run
                args = Namespace(suites=["lexer::valid"] if mixed else [], lib_case=["sample::case"],
                                 ignored=False, prune_under=[], pa=[])
                with mock.patch.object(fastcheck_run, "discover", return_value={"lexer": "suite_frontend"}), \
                        mock.patch.object(fastcheck_run.shutil, "which", return_value="test-clippy-driver"), \
                        mock.patch.object(fastcheck_run, "fingerprint", return_value={}):
                    fastcheck_run.execute(root, args, steps)
                self.assertEqual(sum(name == "build" for name, _ in commands), int(mixed))
                self.assertEqual(steps.receipt["private_harness_producer"], "clippy-private")
