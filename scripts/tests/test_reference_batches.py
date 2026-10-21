import contextlib
import hashlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import course
import reference_batches


class ScopedReferenceTests(unittest.TestCase):
    def fixture(self, directory):
        root = Path(directory)
        tests = root / "pa1/tests"
        tests.mkdir(parents=True)
        (root / "dev").mkdir()
        (root / "dev/compiler.rs").write_text("fn original() {}")
        cases = [{"path": "tests/x.rs", "exit_status": 0},
                 {"path": "tests/y.rs", "exit_status": 0}]
        for case in cases:
            (root / "pa1" / case["path"]).write_text(case["path"])
        manifest = {"mode": "--emit-tokens", "cases": cases}
        (tests / "manifest.json").write_text(json.dumps(manifest))
        self.run_batch(root, "print('original')")
        return root, tests, manifest

    def run_batch(self, root, command, paths=None, timeout=2):
        with contextlib.redirect_stdout(io.StringIO()):
            return course.run_stage(root, "pa1", [sys.executable, "-c", command], bless=True,
                                    case_paths=paths, jobs=2, timeout=timeout)

    def snapshot(self, root):
        folder = root / "pa1"
        return {path.relative_to(folder).as_posix(): path.read_bytes()
                for path in folder.rglob("*") if path.is_file() and
                (".ref" in path.name or "reference-batches" in path.parts or path.name == "reference.json")}

    def test_selector_rejects_duplicates_unknown_empty_and_noncanonical_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root, _, _ = self.fixture(directory)
            original = self.snapshot(root)
            for paths in [[], [""], ["tests/x.rs", "tests/x.rs"], ["tests/no.rs"],
                          ["tests/../tests/x.rs"], ["/tests/x.rs"], ["tests//x.rs"], [1]]:
                with self.subTest(paths=paths), self.assertRaises(ValueError):
                    self.run_batch(root, "print('wrong')", paths)
                self.assertEqual(self.snapshot(root), original)

    def test_selection_is_in_manifest_order_and_validates_unselected_cases(self):
        with tempfile.TemporaryDirectory() as directory:
            root, tests, manifest = self.fixture(directory)
            selected = reference_batches.select(manifest["cases"], ["tests/y.rs", "tests/x.rs"], "pa1")
            self.assertEqual([case["path"] for case in selected], ["tests/x.rs", "tests/y.rs"])
            manifest["cases"][1]["exit_status"] = 86
            (tests / "manifest.json").write_text(json.dumps(manifest))
            with mock.patch("course.execute") as execute, self.assertRaises(ValueError):
                self.run_batch(root, "print('wrong')", ["tests/x.rs"])
            execute.assert_not_called()

    def test_partial_generation_preserves_base_and_unselected_sidecars(self):
        with tempfile.TemporaryDirectory() as directory:
            root, tests, manifest = self.fixture(directory)
            original = self.snapshot(root)
            self.assertTrue(self.run_batch(root, "print('selected')", ["tests/x.rs"]))
            self.assertEqual((tests / "x.rs.ref").read_bytes(), b"selected\n")
            for path, data in original.items():
                if not path.startswith("tests/x.rs.ref"):
                    self.assertEqual((root / "pa1" / path).read_bytes(), data, path)
            batches = list((root / "pa1/reference-batches").glob("*.json"))
            self.assertEqual(len(batches), 1)
            batch = json.loads(batches[0].read_text())
            self.assertEqual(batch["selected_paths"], ["tests/x.rs"])
            self.assertEqual(set(batch["cases"]), {"tests/x.rs"})
            self.assertEqual(set(batch["compiler_source_sha256"]), {"dev/compiler.rs"})
            self.assertEqual(batch["cases"]["tests/x.rs"]["outputs"][".ref"],
                             hashlib.sha256(b"selected\n").hexdigest())
            reference_batches.validate(root, "pa1", manifest, manifest["cases"])

    def test_new_case_overlays_coverage_without_rewriting_full_provenance(self):
        with tempfile.TemporaryDirectory() as directory:
            root, tests, manifest = self.fixture(directory)
            base = (root / "pa1/reference.json").read_bytes()
            (tests / "new.rs").write_text("new input")
            manifest["cases"].append({"path": "tests/new.rs", "exit_status": 0})
            (tests / "manifest.json").write_text(json.dumps(manifest))
            self.assertTrue(self.run_batch(root, "print('new output')", ["tests/new.rs"]))
            self.assertEqual((root / "pa1/reference.json").read_bytes(), base)
            inputs, auxiliary = reference_batches.validate(root, "pa1", manifest, manifest["cases"])
            self.assertEqual(set(inputs), {case["path"] for case in manifest["cases"]})
            self.assertEqual(auxiliary, {})
            for mutation in ("input", "output", "contract", "manifest"):
                with self.subTest(mutation=mutation):
                    target = tests / ("new.rs" if mutation == "input" else "new.rs.ref" if mutation == "output" else "manifest.json")
                    previous = target.read_bytes()
                    if mutation in ("input", "output", "manifest"):
                        target.write_bytes(previous + b" ")
                    else:
                        manifest["cases"][-1]["args"] = ["--crate-name", "changed"]
                    try:
                        with self.assertRaises(ValueError):
                            reference_batches.validate(root, "pa1", manifest, manifest["cases"])
                    finally:
                        target.write_bytes(previous)
                        manifest["cases"][-1].pop("args", None)

    def test_failed_outcomes_and_infrastructure_never_publish_a_partial_batch(self):
        commands = ["raise SystemExit(1)", "raise SystemExit(86)", "raise SystemExit(2)",
                    "import os,signal;os.kill(os.getpid(),signal.SIGKILL)",
                    "import time;time.sleep(2)"]
        for command in commands:
            with self.subTest(command=command), tempfile.TemporaryDirectory() as directory:
                root, _, _ = self.fixture(directory)
                before = self.snapshot(root)
                self.assertFalse(self.run_batch(root, command, ["tests/x.rs", "tests/y.rs"], timeout=.15))
                self.assertEqual(self.snapshot(root), before)
                self.assertFalse((root / "pa1/reference-batches").exists())

    def test_successful_and_failed_selection_is_atomic_across_cases(self):
        with tempfile.TemporaryDirectory() as directory:
            root, _, _ = self.fixture(directory)
            before = self.snapshot(root)
            command = "import sys;data=sys.stdin.read();print('new');raise SystemExit(int('y.rs' in data))"
            self.assertFalse(self.run_batch(root, command, ["tests/x.rs", "tests/y.rs"]))
            self.assertEqual(self.snapshot(root), before)

    def test_scoped_generation_requires_valid_full_reference_before_writes(self):
        with tempfile.TemporaryDirectory() as directory:
            root, tests, _ = self.fixture(directory)
            (root / "pa1/reference.json").write_text("{}")
            before = (tests / "x.rs.ref").read_bytes()
            with self.assertRaises(ValueError):
                self.run_batch(root, "print('wrong')", ["tests/x.rs"])
            self.assertEqual((tests / "x.rs.ref").read_bytes(), before)

    def test_newer_batch_replaces_only_its_selected_provenance(self):
        with tempfile.TemporaryDirectory() as directory:
            root, _, manifest = self.fixture(directory)
            self.assertTrue(self.run_batch(root, "print('first')", ["tests/x.rs"]))
            self.assertTrue(self.run_batch(root, "print('second')", ["tests/x.rs"]))
            self.assertEqual(len(list((root / "pa1/reference-batches").glob("*.json"))), 2)
            reference_batches.validate(root, "pa1", manifest, manifest["cases"])

    def test_full_generation_supersedes_historical_scoped_receipts(self):
        with tempfile.TemporaryDirectory() as directory:
            root, tests, manifest = self.fixture(directory)
            self.assertTrue(self.run_batch(root, "print('partial')", ["tests/x.rs"]))
            batch = next((root / "pa1/reference-batches").glob("*.json"))
            saved = batch.read_bytes()
            self.assertTrue(self.run_batch(root, "print('new full')"))
            self.assertEqual(batch.read_bytes(), saved)
            self.assertEqual((tests / "x.rs.ref").read_bytes(), b"new full\n")
            self.assertEqual((tests / "y.rs.ref").read_bytes(), b"new full\n")
            receipt = json.loads((root / "pa1/reference.json").read_text())
            self.assertEqual(receipt["supersedes_reference_batches"][batch.name], hashlib.sha256(saved).hexdigest())
            reference_batches.validate(root, "pa1", manifest, manifest["cases"])

    def test_cli_repeated_case_option_and_layout_overlay(self):
        import check_layout
        with tempfile.TemporaryDirectory() as directory:
            root, tests, manifest = self.fixture(directory)
            app = root / "candidate"
            app.write_text(f"#!{sys.executable}\nprint('selected')\n")
            app.chmod(0o755)
            manifest["cases"].append({"path": "tests/new.rs", "exit_status": 0})
            (tests / "new.rs").write_text("new input")
            (tests / "manifest.json").write_text(json.dumps(manifest))
            args = ["course.py", "pa1", "--app", str(app), "--bless", "--case", "tests/new.rs", "--case", "tests/x.rs"]
            with mock.patch("course.ROOT", root), mock.patch.object(sys, "argv", args), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(course.main(), 0)
            self.assertEqual((tests / "y.rs.ref").read_bytes(), b"original\n")
            (root / "Makefile").write_text("ACTIVE_PAS := pa1\n")
            (root / "assignments.json").write_text(json.dumps({"stages": [{"path": "pa1", "number": 1, "status": "implemented-draft", "mode": "--emit-tokens"}]}))
            for name in ("README.md", "Makefile", "tests/README.md"):
                (root / "pa1" / name).write_text("fixture")
            with contextlib.redirect_stdout(io.StringIO()):
                check_layout.check(root)

    def test_partial_batch_cannot_attest_unselected_contract_changes(self):
        for mutation in ("args", "exit_status", "mode", "default_mode"):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                root, tests, manifest = self.fixture(directory)
                (tests / "new.rs").write_text("new")
                manifest["cases"].append({"path": "tests/new.rs", "exit_status": 0})
                if mutation == "args":
                    manifest["cases"][0]["args"] = ["--crate-name", "changed"]
                elif mutation == "exit_status":
                    manifest["cases"][0]["exit_status"] = 1
                elif mutation == "mode":
                    manifest["additional_modes"] = ["--emit-types"]
                    manifest["cases"][0]["mode"] = "--emit-types"
                else:
                    manifest["mode"] = "--emit-types"
                (tests / "manifest.json").write_text(json.dumps(manifest))
                before = self.snapshot(root)
                with self.assertRaisesRegex(ValueError, "case contract changed"):
                    self.run_batch(root, "print('new')", ["tests/new.rs"])
                self.assertEqual(self.snapshot(root), before)

    def test_legacy_partial_generation_requires_an_unchanged_full_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            root, tests, manifest = self.fixture(directory)
            base = root / "pa1/reference.json"
            receipt = json.loads(base.read_text())
            del receipt["case_contracts"]
            base.write_text(json.dumps(receipt))
            self.assertTrue(self.run_batch(root, "print('legacy selected')", ["tests/x.rs"]))
            (tests / "new.rs").write_text("new")
            manifest["cases"].append({"path": "tests/new.rs", "exit_status": 0})
            (tests / "manifest.json").write_text(json.dumps(manifest))
            before = self.snapshot(root)
            with self.assertRaisesRegex(ValueError, "legacy reference receipt requires full refresh"):
                self.run_batch(root, "print('new')", ["tests/new.rs"])
            self.assertEqual(self.snapshot(root), before)

    def test_unexplained_or_changed_superseded_batch_is_rejected(self):
        for mutation in ("unexplained", "tamper", "missing"):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                root, _, manifest = self.fixture(directory)
                self.assertTrue(self.run_batch(root, "print('selected')", ["tests/x.rs"]))
                batch = next((root / "pa1/reference-batches").glob("*.json"))
                if mutation == "unexplained":
                    receipt = json.loads(batch.read_text())
                    receipt["base_reference_sha256"] = "0" * 64
                    batch.write_text(json.dumps(receipt))
                else:
                    self.assertTrue(self.run_batch(root, "print('full refresh')"))
                    if mutation == "tamper":
                        batch.write_bytes(batch.read_bytes() + b" ")
                    else:
                        batch.unlink()
                with self.assertRaises(ValueError):
                    reference_batches.validate(root, "pa1", manifest, manifest["cases"])


if __name__ == "__main__":
    unittest.main()
