"""Verify consumer suite loading, guest graph assembly, and execution oracle checks."""
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import grade_consumers


class GradeConsumers(unittest.TestCase):
    def test_committed_consumer_suites_validate_hashes_and_inventory(self):
        expected_counts = {"pa32": 18, "pa33": 12, "pa35": 4, "pa36": 14}
        for stage, expected in expected_counts.items():
            with self.subTest(stage=stage):
                suite, outcomes, _ = grade_consumers.load_suite(grade_consumers.ROOT, stage)
                self.assertEqual(suite["stage"], stage)
                self.assertEqual(len(outcomes["cases"]), expected)

    def test_whole_graph_guests_include_expected_crates_by_strategy(self):
        lib = Path("/sysroot/library")
        vendor = Path("/vendor")
        _, alloc_guests, _, _ = grade_consumers.whole_graph_guests("whole-graph-alloc", lib, vendor)
        self.assertEqual(alloc_guests, ["--source-guest", f"alloc@2024={lib}/alloc/src/lib.rs"])

        cfgs, std_guests, _, env = grade_consumers.whole_graph_guests("whole-graph-std", lib, vendor)
        self.assertIn("backtrace_in_libstd", cfgs)
        self.assertEqual(env["STD_ENV_ARCH"], "x86_64")
        self.assertTrue(any(arg.startswith("std@2024=") for arg in std_guests))

        _, uni_guests, uni_cfgs, _ = grade_consumers.whole_graph_guests("whole-graph-unicode", lib, vendor)
        self.assertTrue(any(arg.startswith("unicode_normalization@2018=") for arg in uni_guests))
        self.assertIn('tinyvec=feature="alloc"', uni_cfgs)

    def test_verify_execution_checks_stdout_status_and_stderr(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "expected.out").write_text("ok\n")
            script = root / "prog.sh"
            script.write_text("#!/bin/sh\nprintf 'ok\\n'\n")
            script.chmod(0o755)
            case = {
                "name": "demo", "stdout": "expected.out", "status": 0,
                "args": [], "stdin": "", "env": {}, "stderr": "",
            }
            runs = grade_consumers.verify_execution(script, case, {"folder": root, "work": root}, repeats=2)
            self.assertEqual(len(runs), 2)

            (root / "expected.out").write_text("different\n")
            with self.assertRaisesRegex(ValueError, "stdout differs"):
                grade_consumers.verify_execution(script, case, {"folder": root, "work": root}, repeats=1)

            (root / "expected.out").write_text("ok\n")
            case["status"] = 1
            with self.assertRaisesRegex(ValueError, "exit 0, expected 1"):
                grade_consumers.verify_execution(script, case, {"folder": root, "work": root}, repeats=1)


if __name__ == "__main__":
    unittest.main()
