"""Fast feedback must fail on expired budgets and unsafe native output scopes."""
from argparse import Namespace
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
