from pathlib import Path
import signal
import subprocess
import sys
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
from check_native import classification, execution, native_cases, run_executables


class NativeContract(unittest.TestCase):
    def test_supplemental_views_do_not_weaken_native_requirements(self):
        manifest = {"mode": "--emit-lowir", "additional_modes": ["--emit-layouts"]}
        layout = {"mode": "--emit-layouts", "exit_status": 0}
        accepted = {"exit_status": 0, "rustc": {"kind": "program", "status": 0}}
        with self.assertRaises(ValueError):
            native_cases(manifest, [layout, accepted])
        with self.assertRaises(ValueError):
            native_cases(manifest, [layout])
        accepted["native"] = {"outcome": "pass"}
        self.assertEqual(native_cases(manifest, [layout, accepted]), [accepted])

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

    def test_runtime_differences_require_declared_outcomes_and_reasons(self):
        case = {"exit_status": 0, "rustc": {"kind": "program", "status": 0},
                "native": {"outcome": "pass", "rustc_outcome": "panic"}}
        for reason in (None, "", "   ", True):
            case["native"]["reason"] = reason
            with self.assertRaises(ValueError):
                classification(case)
        case["native"]["reason"] = "Reference requires two guard retries; pinned rustc performs one"
        classification(case)
        for outcome in ("failure", 1, "signal", ""):
            case["native"]["rustc_outcome"] = outcome
            with self.assertRaises(ValueError):
                classification(case)
            with self.assertRaises(ValueError):
                execution(subprocess.CompletedProcess([], 134, b"", b""), outcome, "cppgm")

    def test_only_explicitly_qualified_panic_terminations_pass(self):
        execution(subprocess.CompletedProcess([], 134, b"", b""), "panic", "cppgm")
        execution(subprocess.CompletedProcess([], -signal.SIGABRT, b"", b"panicked at x"), "panic", "rustc")
        for compiler, status, stderr in [("cppgm", -signal.SIGABRT, b""), ("cppgm", 86, b""),
                                          ("rustc", -signal.SIGABRT, b""), ("rustc", -signal.SIGSEGV, b"panicked at x")]:
            with self.assertRaises(ValueError):
                execution(subprocess.CompletedProcess([], status, b"", stderr), "panic", compiler)

    def test_codegen_controls_are_qualified_but_never_executed(self):
        case = {"exit_status": 0, "rustc": {"kind": "program", "status": 0},
                "native": {"kind": "codegen", "reason": "main intentionally diverges"}}
        classification(case)
        def forbidden(*args):
            self.fail("codegen-only binaries must not execute")
        run_executables(case["native"], Path("rustc"), Path("cppgm"), forbidden)
        for extra in ({"outcome": "pass"}, {"rustc_outcome": "panic"}, {"reason": " "},
                      {"kind": "skip"}):
            invalid = dict(case, native=case["native"] | extra)
            with self.assertRaises(ValueError):
                classification(invalid)
        with self.assertRaises(ValueError):
            classification(dict(case, exit_status=1))
