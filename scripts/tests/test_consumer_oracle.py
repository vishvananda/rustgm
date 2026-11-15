"""Milestone consumer declarations stay consistent with their programs and recorded output."""
import hashlib
import importlib.util
from pathlib import Path
import signal
import sys
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


symbols = load("rust_v0_symbols")
with mock.patch.dict(sys.modules, {"rust_v0_symbols": symbols}):
    demand = load("demand_list")
with mock.patch.dict(sys.modules, {"demand_list": demand}):
    oracle = load("consumer_oracle")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class Declarations(unittest.TestCase):
    def test_every_program_is_declared_and_matches_its_recorded_hashes(self):
        for rung in sorted(oracle.RUNGS):
            _, outcomes, assignment = oracle.load(rung)
            self.assertEqual(outcomes["stage"], "planned")
            self.assertTrue(outcomes["expectation_frozen_before_candidate"])
            self.assertGreaterEqual(len(outcomes["cases"]), 4)
            for case in outcomes["cases"]:
                with self.subTest(rung=rung, case=case["name"]):
                    self.assertEqual(digest(assignment / case["path"]), case["source_sha256"])
                    self.assertEqual(digest(assignment / case["stdout"]), case["stdout_sha256"])
                    self.assertTrue(case["status"] == "abort" or 0 <= case["status"] <= 255)
                    self.assertTrue(isinstance(case["stderr"], str) or case["stderr"]["contains"])
                    self.assertTrue(case["features"])

    def test_each_rung_has_a_negative_control_except_the_library_consumers(self):
        for rung in ("m1", "m2", "m3"):
            cases = oracle.load(rung)[1]["cases"]
            self.assertTrue(any(case["status"] == "abort" for case in cases), rung)

    def test_staged_programs_do_not_activate_a_planned_assignment(self):
        for assignment, _ in oracle.RUNGS.values():
            self.assertFalse((ROOT / assignment / "tests" / "manifest.json").exists(), assignment)


class Runs(unittest.TestCase):
    CASE = {"name": "case", "status": 0, "stderr": ""}

    def test_status_standard_error_and_leftover_files_are_enforced(self):
        self.assertIsNone(oracle.mismatch(self.CASE, 0, b"", []))
        self.assertIn("status 1", oracle.mismatch(self.CASE, 1, b"", []))
        self.assertIn("standard error", oracle.mismatch(self.CASE, 0, b"noise", []))
        self.assertIn("left files", oracle.mismatch(self.CASE, 0, b"", ["scratch.txt"]))

    def test_abort_means_sigabrt_and_contains_accepts_surrounding_text(self):
        abort = {"name": "case", "status": "abort", "stderr": {"contains": ["panicked at", "boom"]}}
        text = b"thread 'main' (1) panicked at a.rs:1:1:\nboom\n"
        self.assertIsNone(oracle.mismatch(abort, -signal.SIGABRT, text, []))
        self.assertIn("status 134", oracle.mismatch(abort, 134, text, []))
        self.assertIn("lacks", oracle.mismatch(abort, -signal.SIGABRT, b"panicked at a.rs", []))


if __name__ == "__main__":
    unittest.main()
