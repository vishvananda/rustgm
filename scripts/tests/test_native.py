from pathlib import Path
import signal
import subprocess
import sys
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
from check_native import accepted_frontend, classification, execution, native_cases, run_executables


class NativeContract(unittest.TestCase):
    def test_libraries_require_objects_and_never_execute(self):
        case = {"exit_status": 0, "rustc": {"kind": "library", "status": 0},
                "library": {"crate_name": "values", "disambiguator": "test"},
                "native": {"kind": "object", "reason": "Public source has no main"}}
        classification(case)
        def forbidden(*args):
            self.fail("relocatable objects cannot execute")
        run_executables(case["native"], Path("rust.o"), Path("cppgm.o"), forbidden)
        for native in ({"outcome": "pass"}, {"kind": "codegen", "reason": "library"},
                       {"kind": "object"}, {"kind": "object", "reason": " "},
                       case["native"] | {"outcome": "pass"},
                       case["native"] | {"rustc_outcome": "pass"}):
            with self.subTest(native=native), self.assertRaises(ValueError):
                classification(case | {"native": native})
        with self.assertRaises(ValueError):
            classification(case | {"rustc": {"kind": "program", "status": 0}})
        with self.assertRaises(ValueError):
            classification(case | {"exit_status": 1})
        rejected = {k: v for k, v in case.items() if k != "native"}
        rejected.update(exit_status=1, rustc={"kind": "library", "status": 1})
        classification(rejected)

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


class SuccessfulFrontendDiagnostics(unittest.TestCase):
    warning = b"warning[unused_must_use]: file 0: 37: unused must_use result: retain result\n"

    def test_empty_diagnostics_preserve_the_existing_success_contract(self):
        accepted_frontend((b"program\n", b"", b"0\n"))
        for actual in ((b"", b"", b"0\n"), (b"program\n", b"", b"1\n")):
            with self.subTest(actual=actual), self.assertRaises(ValueError):
                accepted_frontend(actual)

    def test_exact_declared_warning_is_accepted(self):
        accepted_frontend((b"program\n", self.warning, b"0\n"), self.warning, ("unused_must_use",))

    def test_explicit_diagnostic_attribute_warnings_require_exact_code_and_bytes(self):
        for code in ("malformed_diagnostic_attributes", "malformed_diagnostic_format_literals",
                     "misplaced_diagnostic_attributes"):
            warning = f"warning[{code}]: file 0: 37: declared attribute diagnostic\n".encode()
            accepted_frontend((b"program\n", warning, b"0\n"), warning, (code,))
            with self.assertRaises(ValueError):
                accepted_frontend((b"program\n", warning, b"0\n"), warning)
            with self.assertRaises(ValueError):
                accepted_frontend((b"program\n", warning + b"error: failure\n", b"0\n"),
                                  warning + b"error: failure\n", (code,))

    def test_undeclared_changed_or_additional_warnings_reject(self):
        for expected, actual in ((b"", self.warning), (self.warning, b""),
                                 (self.warning, self.warning + self.warning),
                                 (self.warning, self.warning.replace(b"37", b"38"))):
            with self.subTest(expected=expected, actual=actual), self.assertRaises(ValueError):
                accepted_frontend((b"program\n", actual, b"0\n"), expected, ("unused_must_use",) if expected else ())

    def test_reference_presence_cannot_invent_a_warning_declaration(self):
        with self.assertRaises(ValueError):
            accepted_frontend((b"program\n", self.warning, b"0\n"), self.warning)
        with self.assertRaises(ValueError):
            accepted_frontend((b"program\n", self.warning, b"0\n"), self.warning, ("unused_attributes",))
        with self.assertRaises(ValueError):
            accepted_frontend((b"program\n", b"", b"0\n"), b"", ("unused_must_use",))

    def test_error_status_or_error_text_cannot_masquerade_as_warning(self):
        with self.assertRaises(ValueError):
            accepted_frontend((b"program\n", self.warning, b"1\n"), self.warning)
        for text in (b"ERROR: type failure\n", self.warning + b"error: type failure\n",
                     b"warning[unknown]: file 0: 37: arbitrary stderr\n"):
            with self.subTest(text=text), self.assertRaises(ValueError):
                accepted_frontend((b"program\n", text, b"0\n"), text, ("unused_must_use",))
