"""Metadata can be checked before references or compiler binaries exist."""
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from assignment_metadata import collect
import check_layout


class AssignmentMetadata(unittest.TestCase):
    def fixture(self, directory):
        root = Path(directory)
        (root / "pa1/tests").mkdir(parents=True)
        stage = {"path": "pa1", "number": 1, "status": "implemented-draft", "mode": "--emit-lowir"}
        case = {"path": "tests/x.rs", "exit_status": 0,
                "rustc": {"kind": "program", "status": 0}, "native": {"outcome": "pass"}}
        manifest = {"mode": "--emit-lowir", "cases": [case]}
        for name in ("README.md", "Makefile", "tests/README.md"):
            (root / "pa1" / name).write_text("fixture")
        (root / "pa1/tests/x.rs").write_text("struct S; fn main(){}")
        self.write(root, stage, manifest)
        return root, stage, manifest, case

    def write(self, root, stage, manifest):
        (root / "assignments.json").write_text(json.dumps({"stages": [stage]}))
        active = "" if stage["status"] == "planned" else "pa1"
        (root / "Makefile").write_text(f"ACTIVE_PAS := {active}\n")
        (root / "pa1/tests/manifest.json").write_text(json.dumps(manifest))

    def test_pending_references_are_allowed_but_full_layout_still_requires_them(self):
        with tempfile.TemporaryDirectory() as directory:
            root, _, _, _ = self.fixture(directory)
            _, loaded = collect(root)
            self.assertEqual(len(loaded[0][2]), 1)
            self.assertFalse((root / "pa1/reference.json").exists())
            with self.assertRaises(FileNotFoundError):
                check_layout.check(root)

    def test_implemented_status_requires_existing_release_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stage, manifest, _ = self.fixture(directory)
            stage["status"] = "implemented"
            self.write(root, stage, manifest)
            with self.assertRaisesRegex(ValueError, "needs release evidence"):
                collect(root)
            stage["release_evidence"] = ["release.md"]
            self.write(root, stage, manifest)
            with self.assertRaisesRegex(ValueError, "missing or unsafe"):
                collect(root)
            (root / "release.md").write_text("complete profile qualification")
            self.assertEqual(len(collect(root)[1]), 1)

    def test_missing_native_outcome_uses_the_actual_native_classifier(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stage, manifest, case = self.fixture(directory)
            del case["native"]
            self.write(root, stage, manifest)
            with self.assertRaisesRegex(ValueError, "every executed native fixture needs a declared outcome"):
                collect(root)

    def test_supplemental_mode_must_be_declared_in_manifest_and_root(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stage, manifest, case = self.fixture(directory)
            case["mode"] = "--emit-patterns"
            self.write(root, stage, manifest)
            with self.assertRaisesRegex(ValueError, "undeclared mode"):
                collect(root)
            manifest["additional_modes"] = ["--emit-patterns"]
            self.write(root, stage, manifest)
            with self.assertRaisesRegex(ValueError, "assignment modes disagree"):
                collect(root)
            stage["additional_modes"] = ["--emit-patterns"]
            self.write(root, stage, manifest)
            collect(root)

    def test_ignored_cfg_field_rejects_and_supported_args_remain_valid(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stage, manifest, case = self.fixture(directory)
            case["cfg"] = ["test"]
            self.write(root, stage, manifest)
            with self.assertRaisesRegex(ValueError, r'per-case cfg.*args: \["--cfg", "test"\]'):
                collect(root)
            del case["cfg"]
            case["args"] = ["--cfg", "test"]
            self.write(root, stage, manifest)
            collect(root)

    def test_layout_vectors_use_the_actual_layout_classifier(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stage, manifest, case = self.fixture(directory)
            stage["mode"] = manifest["mode"] = "--emit-layouts"
            del case["native"]
            self.write(root, stage, manifest)
            with self.assertRaisesRegex(ValueError, "declared vectors"):
                collect(root)
            case["layout_checks"] = [{"name": "S", "rust_type": "S", "size": 0, "align": 1}]
            self.write(root, stage, manifest)
            collect(root)

    def test_future_feature_owner_and_stale_active_stage_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stage, manifest, case = self.fixture(directory)
            case["features"] = ["pa2.future"]
            self.write(root, stage, manifest)
            with self.assertRaisesRegex(ValueError, "future-stage feature"):
                collect(root)
            case["features"] = ["pa1.owned"]
            self.write(root, stage, manifest)
            (root / "Makefile").write_text("ACTIVE_PAS := pa1 pa2\n")
            with self.assertRaisesRegex(ValueError, "root report stages disagree"):
                collect(root)

    def test_explicit_unit_dependencies_allow_source_bindings_but_not_unrelated_features(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stage, manifest, case = self.fixture(directory)
            case.update(work_units=["consumer"], features=["pa31.source-binding"])
            self.write(root, stage, manifest)
            (root / "docs").mkdir()
            folder = root / "pa31/tests"
            folder.mkdir(parents=True)
            for path in ("README.md", "Makefile", "tests/README.md"):
                (root / "pa31" / path).write_text("source declarations")
            stages = [stage, {"path": "pa31", "number": 31, "status": "planned"}]
            (root / "assignments.json").write_text(json.dumps({"stages": stages}))
            common = {"implementation": "partial", "publication": "unpublished",
                      "scope": "explicit source contracts", "exit": "native consumer", "requirements": []}
            units = [{**common, "id": "source", "owner": "pa31", "depends_on": []},
                     {**common, "id": "consumer", "owner": "pa1", "depends_on": ["source"]}]
            (root / "assignment-plan.json").write_text(json.dumps({"schema": 1, "units": units}))
            (root / "docs/assignment-requirements.json").write_text('{"rows": []}')
            self.assertEqual(len(collect(root)[1]), 1)
            case["features"] = ["pa42.async"]
            (root / "pa1/tests/manifest.json").write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, "future-stage feature"):
                collect(root)

    def test_planned_stage_cannot_publish_an_executed_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stage, manifest, _ = self.fixture(directory)
            stage["status"] = "planned"
            self.write(root, stage, manifest)
            with self.assertRaisesRegex(ValueError, "planned stage has an undeclared executed suite"):
                collect(root)
            (root / "pa1/tests/manifest.json").unlink()
            self.assertEqual(collect(root)[1], [])
            stage["status"] = "unknown"
            (root / "assignments.json").write_text(json.dumps({"stages": [stage]}))
            with self.assertRaisesRegex(ValueError, "unknown assignment status"):
                collect(root)


if __name__ == "__main__":
    unittest.main()
