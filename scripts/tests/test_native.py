import hashlib
from pathlib import Path
import signal
import struct
import subprocess
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
from check_native import (accepted_frontend, candidate_identity, classification, defined_code_or_data,
                          empty_objects, execution, launcher_target, native_cases, native_selection,
                          run_executables)


def relocatable(symbols):
    """Build a minimal ELF64 x86-64 relocatable object whose one symbol table holds these symbols."""
    strings = b"\0" + b"".join(name.encode() + b"\0" for name, _, _ in symbols)
    table = bytearray(24)
    position = 1
    for name, info, section in symbols:
        table += struct.pack("<IBBHQQ", position, info, 0, section, 0, 0)
        position += len(name) + 1
    strings_at = 64
    table_at = strings_at + len(strings)
    headers_at = table_at + len(table)
    header = bytearray(64)
    header[:7] = b"\x7fELF\x02\x01\x01"
    header[16:20] = b"\x01\x00\x3e\x00"
    struct.pack_into("<Q", header, 0x28, headers_at)
    struct.pack_into("<HH", header, 0x3A, 64, 3)
    sections = bytes(64)
    sections += struct.pack("<IIQQQQIIQQ", 0, 2, 0, 0, table_at, len(table), 2, 1, 8, 24)
    sections += struct.pack("<IIQQQQIIQQ", 0, 3, 0, 0, strings_at, len(strings), 0, 0, 1, 0)
    return bytes(header) + strings + bytes(table) + sections


class NativeContract(unittest.TestCase):
    def test_selected_native_receipt_retains_scope_order_and_eligible_count(self):
        manifest = {"mode": "--emit-lowir"}
        cases = [{"path": f"tests/{name}.rs", "exit_status": 0,
                  "rustc": {"kind": "program", "status": 0}, "native": {"outcome": "pass"}}
                 for name in ("a", "b", "c")]
        chosen, receipt = native_selection(manifest, cases, ["tests/c.rs", "tests/a.rs"], "pa24")
        self.assertEqual(chosen, [cases[0], cases[2]])
        self.assertEqual(receipt, {"scope": "selected", "eligible_cases": 3,
                                  "selected_paths": ["tests/a.rs", "tests/c.rs"]})
        all_cases, complete = native_selection(manifest, cases, None, "pa24")
        self.assertEqual(all_cases, cases)
        self.assertEqual(complete["scope"], "complete")

    def test_native_subset_rejects_empty_unknown_duplicate_and_non_lowir_paths(self):
        manifest = {"mode": "--emit-lowir", "additional_modes": ["--emit-layouts"]}
        case = {"path": "tests/a.rs", "exit_status": 0, "rustc": {"kind": "program", "status": 0},
                "native": {"outcome": "pass"}}
        layout = {"path": "tests/layout.rs", "mode": "--emit-layouts", "exit_status": 0}
        for paths in ([], ["tests/missing.rs"], ["tests/a.rs", "tests/a.rs"], ["tests/layout.rs"], ["../a.rs"]):
            with self.subTest(paths=paths), self.assertRaises(ValueError):
                native_selection(manifest, [case, layout], paths, "pa24")

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

    def test_only_library_objects_declare_an_empty_lowir_unit(self):
        case = {"exit_status": 0, "rustc": {"kind": "library", "status": 0},
                "library": {"crate_name": "values", "disambiguator": "test"},
                "native": {"kind": "object", "lowir": "empty", "reason": "Private items generate no code"}}
        classification(case)
        for value in ("", "none", "Empty", True, None):
            with self.subTest(value=value), self.assertRaises(ValueError):
                classification(case | {"native": case["native"] | {"lowir": value}})
        program = {"exit_status": 0, "rustc": {"kind": "program", "status": 0}}
        for native in ({"outcome": "pass", "lowir": "empty"},
                       {"kind": "codegen", "reason": "main diverges", "lowir": "empty"}):
            with self.subTest(native=native), self.assertRaises(ValueError):
                classification(program | {"native": native})

    def test_declared_empty_units_require_objects_without_generated_code_or_data(self):
        file, section, imported = ("unit.rs", 0x04, 0xFFF1), ("", 0x03, 1), ("callee", 0x12, 0)
        with tempfile.TemporaryDirectory() as folder:
            empty = Path(folder) / "empty.o"
            empty.write_bytes(relocatable([file, section, imported]))
            self.assertEqual(defined_code_or_data(empty), [])
            empty_objects((("rustc", empty), ("cppgm", empty)))
            for symbol in (("f", 0x12, 1), ("local", 0x02, 1), ("value", 0x01, 2), ("tls", 0x16, 3),
                           ("common", 0x15, 0xFFF2)):
                with self.subTest(symbol=symbol):
                    generated = Path(folder) / "generated.o"
                    generated.write_bytes(relocatable([file, symbol]))
                    self.assertEqual(defined_code_or_data(generated), [symbol[0]])
                    with self.assertRaises(ValueError):
                        empty_objects((("rustc", empty), ("cppgm", generated)))
            truncated = Path(folder) / "truncated.o"
            truncated.write_bytes(relocatable([file])[:80])
            with self.assertRaises(ValueError):
                defined_code_or_data(truncated)

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

    def test_empty_lowir_is_accepted_exactly_when_declared(self):
        accepted_frontend((b"", b"", b"0\n"), empty=True)
        accepted_frontend((b"", self.warning, b"0\n"), self.warning, ("unused_must_use",), empty=True)
        for actual, empty in (((b"", b"", b"0\n"), False), ((b"program\n", b"", b"0\n"), True),
                              ((b"", b"", b"1\n"), True), ((b"", self.warning, b"0\n"), True)):
            with self.subTest(actual=actual, empty=empty), self.assertRaises(ValueError):
                accepted_frontend(actual, empty=empty)

    def test_exact_declared_warning_is_accepted(self):
        accepted_frontend((b"program\n", self.warning, b"0\n"), self.warning, ("unused_must_use",))

    def test_reviewed_diagnostics_require_explicit_exact_code_and_bytes(self):
        for code in ("malformed_diagnostic_attributes", "malformed_diagnostic_format_literals",
                     "misplaced_diagnostic_attributes", "unsafe_op_in_unsafe_fn", "static_mut_refs",
                     "unreachable_patterns"):
            warning = f"warning[{code}]: file 0: 37: declared diagnostic\n".encode()
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


class CandidateIdentity(unittest.TestCase):
    launcher = (SCRIPTS.parent / "dev/rustgm").read_bytes()

    def test_repository_launchers_resolve_to_the_release_compiler(self):
        for name in ("rustgm", "rustgm-ref"):
            with self.subTest(name=name):
                launcher = SCRIPTS.parent / "dev" / name
                self.assertEqual(launcher_target(launcher, launcher.read_text()),
                                 SCRIPTS.parent / "obj/rust/release/rustgm")

    def test_evidence_binds_the_executed_compiler_and_its_launcher(self):
        with tempfile.TemporaryDirectory() as folder:
            launcher = Path(folder) / "dev/rustgm"
            launcher.parent.mkdir()
            launcher.write_bytes(self.launcher)
            binary = Path(folder) / "obj/rust/release/rustgm"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"\x7fELF compiler")
            self.assertEqual(candidate_identity(launcher), {
                "candidate": str(binary), "candidate_sha256": hashlib.sha256(b"\x7fELF compiler").hexdigest(),
                "wrapper": str(launcher), "wrapper_sha256": hashlib.sha256(self.launcher).hexdigest()})
            self.assertEqual(candidate_identity(binary), {
                "candidate": str(binary), "candidate_sha256": hashlib.sha256(b"\x7fELF compiler").hexdigest(),
                "wrapper": None, "wrapper_sha256": None})
            before = candidate_identity(launcher)
            binary.write_bytes(b"\x7fELF rebuilt compiler")
            self.assertNotEqual(candidate_identity(launcher)["candidate_sha256"], before["candidate_sha256"])
            self.assertEqual(candidate_identity(launcher)["wrapper_sha256"], before["wrapper_sha256"])

    def test_unresolvable_launchers_fail_closed(self):
        text = self.launcher.decode()
        variants = {
            "environment target": text.replace('"$root/obj/rust/release/rustgm"', '"$HOME/rustgm"'),
            "parent escape": text.replace("$root/obj/rust", "$root/../rust"),
            "absolute target": text.replace('"$root/obj/rust/release/rustgm"', '"/usr/bin/rustgm"'),
            "extra statement": text.replace("set -eu\n", "set -eu\necho side effect\n"),
            "other root": text.replace('/.." && pwd', '" && pwd'),
            "other interpreter": text.replace("#!/bin/sh", "#!/usr/bin/env python3"),
            "dropped arguments": text.replace(' "$@"', ""),
        }
        with tempfile.TemporaryDirectory() as folder:
            launcher = Path(folder) / "dev/rustgm"
            launcher.parent.mkdir()
            binary = Path(folder) / "obj/rust/release/rustgm"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"\x7fELF compiler")
            for name, variant in variants.items():
                self.assertNotEqual(variant, text, name)
                launcher.write_text(variant)
                with self.subTest(name=name), self.assertRaises(ValueError):
                    candidate_identity(launcher)
            launcher.write_bytes(b"\xff\xfe not a launcher")
            with self.assertRaises(ValueError):
                candidate_identity(launcher)
            launcher.write_bytes(self.launcher)
            binary.write_bytes(self.launcher)
            with self.assertRaises(ValueError):
                candidate_identity(launcher)
            binary.unlink()
            with self.assertRaises(ValueError):
                candidate_identity(launcher)
