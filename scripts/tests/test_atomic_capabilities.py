"""The declared backend matrix cannot turn invalid-order controls into executions."""
import copy
import importlib.util
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


with mock.patch.dict(sys.modules, {"backend": load("backend")}):
    atomic = load("atomic_capabilities")


class AtomicCapabilities(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        shutil.copytree(ROOT / "pa28/tests/backend", self.root / "pa28/tests/backend")
        shutil.copyfile(ROOT / "backend.lock.json", self.root / "backend.lock.json")
        self.contract = atomic.load_contract(self.root)

    def save(self, contract):
        (self.root / atomic.MANIFEST).write_text(json.dumps(contract))

    def test_complete_set_and_independent_outcomes(self):
        cases = self.contract["cases"]
        self.assertEqual(len(cases), 27)
        self.assertEqual(sum(case["native_valid"] for case in cases), 19)
        self.assertEqual(sum(case.get("rustc_expected") == 1 for case in cases), 5)
        self.assertEqual(sum(case["validator_expected"] == 1 for case in cases), 3)
        self.assertEqual(len(self.contract["additional_independent"]), 1)

    def test_hash_changes_and_noncanonical_paths_reject(self):
        path = self.root / self.contract["cases"][0]["lowir"]
        path.write_bytes(path.read_bytes() + b"\n")
        with self.assertRaisesRegex(ValueError, "fixture bytes changed"):
            atomic.load_contract(self.root)
        for path in ("../outside.lowir", "/outside.lowir", "pa28/tests/backend/../x.lowir"):
            with self.assertRaises(ValueError):
                atomic.checked_input(self.root, {"path": path}, "path", "hash")

    def test_duplicate_names_and_wrong_backend_pin_reject(self):
        changed = copy.deepcopy(self.contract)
        changed["cases"].append(changed["cases"][0])
        self.save(changed)
        with self.assertRaisesRegex(ValueError, "duplicate case"):
            atomic.load_contract(self.root)
        changed = copy.deepcopy(self.contract)
        changed["backend_lock"]["commit"] = "different"
        self.save(changed)
        with self.assertRaisesRegex(ValueError, "differs from pinned backend"):
            atomic.load_contract(self.root)

    def test_invalid_cases_cannot_gain_native_or_rust_acceptance(self):
        original = next(case for case in self.contract["cases"]
                        if case["category"] == "Rust-invalid-order")
        for field, value in (("native_expected", 42), ("native_valid", True),
                             ("rustc_expected", 0), ("rustc_diagnostic", "unrelated")):
            changed = copy.deepcopy(self.contract)
            case = next(case for case in changed["cases"] if case["name"] == original["name"])
            case[field] = value
            self.save(changed)
            with self.assertRaises(ValueError):
                atomic.load_contract(self.root)

    def test_additional_and_abi_inputs_are_also_bound(self):
        for path in (self.contract["additional_independent"][0]["rust"],
                     self.contract["abi"]["source"]):
            original = (self.root / path).read_bytes()
            (self.root / path).write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError, "fixture bytes changed"):
                atomic.load_contract(self.root)
            (self.root / path).write_bytes(original)

    def test_validator_only_cases_never_reach_native_builder(self):
        contract = copy.deepcopy(self.contract)
        contract["cases"] = [case for case in contract["cases"] if not case["native_valid"]]
        with mock.patch.object(atomic, "invoke") as invoke:
            atomic.backend_cases(contract, self.root, self.root, {"lowir": "validator"}, [])
        self.assertEqual(invoke.call_count, 8)
        self.assertTrue(all(call.args[0][0] == "validator" for call in invoke.call_args_list))
        self.assertEqual((self.root / "executables.tsv").read_text(), "\n")

    def test_bad_last_input_prevents_any_tool_invocation(self):
        (self.root / self.contract["cases"][-1]["lowir"]).write_bytes(b"changed")
        with mock.patch.object(atomic, "ROOT", self.root), mock.patch.object(
                sys, "argv", ["atomic_capabilities.py", "backend", str(self.root / "obj")]), \
                mock.patch.object(atomic.backend, "load_tools") as tools:
            with self.assertRaisesRegex(ValueError, "fixture bytes changed"):
                atomic.main()
            tools.assert_not_called()

    def test_rust_rejections_compile_only_and_do_not_invoke_frontend(self):
        contract = copy.deepcopy(self.contract)
        contract["cases"] = [case for case in contract["cases"]
                             if case["category"] == "Rust-invalid-order"]
        contract["additional_independent"] = []
        cfg = "\n".join(f'target_has_atomic="{x}"' for x in (8, 16, 32, 64, "ptr"))
        version = f"commit-hash: {contract['rustc_commit']}\n"
        with mock.patch.object(atomic.subprocess, "check_output", side_effect=[version, cfg]), \
                mock.patch.object(atomic, "invoke") as invoke:
            atomic.rust_cases(contract, self.root, self.root, [])
        calls = invoke.call_args_list
        self.assertEqual(len(calls), 7)  # Five rejected objects plus the ABI compile/run.
        for call in calls[:5]:
            self.assertEqual(call.args[0][0], "rustc")
            self.assertIn("--emit=obj", call.args[0])
            self.assertEqual(call.args[4], 1)
            self.assertEqual(call.kwargs["diagnostic"], "invalid_atomic_ordering")


if __name__ == "__main__":
    unittest.main()
