from pathlib import Path
import signal
import subprocess
import sys
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
from check_native import classification, execution


class NativeContract(unittest.TestCase):
    def test_every_accepted_case_requires_execution_evidence(self):
        case = {"exit_status": 0, "rustc": {"kind": "program", "status": 0}}
        with self.assertRaises(ValueError):
            classification(case)
        case["native"] = {"outcome": "pass"}
        classification(case)
        case["rustc"]["status"] = 1
        case["rustc"]["reason"] = "a reason cannot replace a runnable independent reference"
        with self.assertRaises(ValueError):
            classification(case)

    def test_runtime_failure_never_counts_as_success(self):
        for status in (1, 86, 134, -signal.SIGSEGV, -signal.SIGABRT):
            with self.assertRaises(ValueError):
                execution(subprocess.CompletedProcess([], status, b"", b""), "pass", "cppgm")

    def test_only_explicitly_qualified_panic_terminations_pass(self):
        execution(subprocess.CompletedProcess([], 134, b"", b""), "panic", "cppgm")
        execution(subprocess.CompletedProcess([], -signal.SIGABRT, b"", b"panicked at x"), "panic", "rustc")
        for compiler, status, stderr in [("cppgm", -signal.SIGABRT, b""), ("cppgm", 86, b""),
                                          ("rustc", -signal.SIGABRT, b""), ("rustc", -signal.SIGSEGV, b"panicked at x")]:
            with self.assertRaises(ValueError):
                execution(subprocess.CompletedProcess([], status, b"", stderr), "panic", compiler)
