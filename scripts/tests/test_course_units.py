"""Component publication shares the course runner without activating its parent PA."""
import contextlib
import copy
import io
import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import assignment_metadata
import assignment_plan
import check_layout
import course
from course_suites import run_released, suite_paths
import reference_batches


class CourseUnits(unittest.TestCase):
    def fixture(self, directory):
        root = Path(directory)
        for path in ("docs", "dev", "pa27/tests/places", "pa27/tests/units", "pa27/tests/later"):
            (root / path).mkdir(parents=True)
        for path in ("pa27/README.md", "pa27/Makefile", "pa27/tests/README.md", "docs/proof.md"):
            (root / path).write_text("fixture")
        (root / "dev/compiler.rs").write_text("fn compiler() {}")
        stage = {"path": "pa27", "number": 27, "status": "planned", "mode": "--emit-types"}
        (root / "assignments.json").write_text(json.dumps({"stages": [stage]}))
        (root / "Makefile").write_text("ACTIVE_PAS :=\n")
        units, rows = [], []
        for index, key in enumerate(("unsafe-places", "raw-validity"), 1):
            units.append({"id": key, "owner": "pa27", "implementation": "partial",
                          "publication": "unpublished", "scope": key, "exit": "real evidence",
                          "requirements": [f"27.{index}"], "depends_on": []})
            rows.append({"id": f"27.{index}", "pa": 27, "implementation": "partial",
                         "source": [], "tests": [], "evidence": [], "units": [key]})
        units[0]["manifest"] = "pa27/tests/units/unsafe-places.json"
        (root / "assignment-plan.json").write_text(json.dumps({"schema": 1, "units": units}))
        (root / "docs/assignment-requirements.json").write_text(json.dumps({"rows": rows}))
        cases = [{"path": f"tests/places/{name}.rs", "exit_status": 0,
                  "work_units": ["unsafe-places"], "features": ["pa27.unsafe"]} for name in ("x", "y")]
        for case in cases:
            (root / "pa27" / case["path"]).write_text("fn main() {}")
        (root / "pa27/tests/later/unreleased.rs").write_text("retained later requirement")
        manifest = {"mode": "--emit-types", "discovery": ["tests/places"], "cases": cases}
        self.write_manifest(root, manifest)
        return root, manifest

    def write_manifest(self, root, manifest, unit="unsafe-places"):
        (root / f"pa27/tests/units/{unit}.json").write_text(json.dumps(manifest))

    def generate(self, root, paths=None, command="print('qualified')", unit="unsafe-places"):
        with contextlib.redirect_stdout(io.StringIO()):
            return course.run_stage(root, "pa27", [sys.executable, "-c", command], unit=unit,
                                    bless=True, jobs=2, case_paths=paths)

    def verify(self, root, unit="unsafe-places"):
        manifest, cases = course.load_cases(root, "pa27", unit=unit)
        check_layout.check_suite(root, "pa27", manifest, cases, unit=unit)

    def snapshot(self, root):
        return {str(p.relative_to(root)): p.read_bytes() for p in (root / "pa27").rglob("*")
                if p.is_file() and (".ref" in p.name or "reference" in p.name or "reference-batches" in p.parts)}

    def test_unit_runs_its_claimed_sources_while_parent_and_later_work_stay_planned(self):
        with tempfile.TemporaryDirectory() as name:
            root, manifest = self.fixture(name)
            with self.assertRaisesRegex(ValueError, "planned assignment"):
                course.load_cases(root, "pa27")
            self.assertEqual(course.load_cases(root, "pa27", unit="unsafe-places")[0], manifest)
            self.assertEqual(assignment_metadata.collect(root)[1], [])
            self.assertEqual(assignment_metadata.collect_units(root), [])
            self.assertEqual(len(assignment_metadata.collect_units(root, unit="unsafe-places")), 1)
            self.assertTrue(self.generate(root))
            self.verify(root)
            with contextlib.redirect_stdout(io.StringIO()):
                check_layout.check(root, unit="unsafe-places")
            self.assertFalse((root / "pa27/reference.json").exists())
            self.assertEqual((root / "Makefile").read_text(), "ACTIVE_PAS :=\n")
            self.assertEqual(json.loads((root / "assignments.json").read_text())["stages"][0]["status"], "planned")
            runs = list((root / "obj/tests/pa27/units/unsafe-places").glob("*/results.json"))
            self.assertEqual(len(runs), 1)
            self.assertEqual(json.loads(runs[0].read_text())["unit"], "unsafe-places")

    def test_wrong_unit_owner_manifest_or_symlink_fails_before_execution(self):
        for mutation in ("unknown", "owner", "manifest", "undeclared", "symlink", "escape"):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as name:
                root, _ = self.fixture(name)
                plan = json.loads((root / "assignment-plan.json").read_text())
                selected = "unsafe-places"
                if mutation == "unknown":
                    selected = "unknown"
                elif mutation == "owner":
                    plan["units"][0]["owner"] = "pa28"
                elif mutation == "manifest":
                    plan["units"][0]["manifest"] = "docs/proof.md"
                elif mutation == "undeclared":
                    del plan["units"][0]["manifest"]
                elif mutation == "escape":
                    selected = "../unsafe-places"
                else:
                    target = root / "pa27/tests/units/unsafe-places.json"
                    target.rename(root / "saved.json")
                    target.symlink_to(root / "saved.json")
                (root / "assignment-plan.json").write_text(json.dumps(plan))
                with mock.patch("course.execute") as execute, self.assertRaises(ValueError):
                    self.generate(root, unit=selected)
                execute.assert_not_called()
                self.assertEqual(self.snapshot(root), {})

    def test_claimed_discovery_is_complete_nonoverlapping_and_contained(self):
        for roots in ([], ["tests/places", "tests/places/x.rs"], ["tests/places/x.rs"] ,
                      ["tests/places", "tests/places"], ["tests/../places"], ["../outside"],
                      ["tests/missing"], ["tests/units"], "tests/places"):
            with self.subTest(roots=roots), tempfile.TemporaryDirectory() as name:
                root, manifest = self.fixture(name)
                manifest["discovery"] = roots
                self.write_manifest(root, manifest)
                with self.assertRaises(ValueError):
                    course.load_cases(root, "pa27", unit="unsafe-places")
        with tempfile.TemporaryDirectory() as name:
            root, manifest = self.fixture(name)
            (root / "pa27/tests/places/unregistered.rs").write_text("missing input")
            with self.assertRaisesRegex(ValueError, "undiscovered"):
                course.load_cases(root, "pa27", unit="unsafe-places")
            (root / "pa27/tests/places/unregistered.rs").unlink()
            (root / "outside.rs").write_text("outside")
            (root / "pa27/tests/places/link.rs").symlink_to(root / "outside.rs")
            with self.assertRaisesRegex(ValueError, "invalid unit discovery"):
                course.load_cases(root, "pa27", unit="unsafe-places")

    def test_primary_publishers_and_primary_as_auxiliary_cannot_overlap(self):
        for auxiliary in (False, True):
            with self.subTest(auxiliary=auxiliary), tempfile.TemporaryDirectory() as name:
                root, manifest = self.fixture(name)
                plan = json.loads((root / "assignment-plan.json").read_text())
                plan["units"][1]["manifest"] = "pa27/tests/units/raw-validity.json"
                (root / "assignment-plan.json").write_text(json.dumps(plan))
                other = copy.deepcopy(manifest)
                if auxiliary:
                    other["cases"] = [{"path": "tests/later/unreleased.rs", "exit_status": 0,
                                       "auxiliary": ["tests/places/x.rs"]}]
                self.write_manifest(root, other, "raw-validity")
                before = self.snapshot(root)
                with mock.patch("course.execute") as execute, self.assertRaisesRegex(ValueError, "publisher|auxiliary"):
                    self.generate(root)
                execute.assert_not_called()
                self.assertEqual(self.snapshot(root), before)

    def test_active_parent_contract_and_output_ownership_survive_unit_generation(self):
        with tempfile.TemporaryDirectory() as name:
            root, manifest = self.fixture(name)
            (root / "pa27/tests/manifest.json").write_text(json.dumps(manifest))
            stages = json.loads((root / "assignments.json").read_text())
            stages["stages"][0]["status"] = "implemented-draft"
            (root / "assignments.json").write_text(json.dumps(stages))
            other = copy.deepcopy(manifest)
            other["mode"] = "--emit-lowir"
            for case in other["cases"]:
                case.update(rustc={"kind": "program", "status": 0}, native={"outcome": "pass"})
            self.write_manifest(root, other)
            with mock.patch("course.execute") as execute, self.assertRaisesRegex(ValueError, "contract disagrees"):
                self.generate(root)
            execute.assert_not_called()
            self.write_manifest(root, manifest)
            before = self.snapshot(root)
            with self.assertRaisesRegex(ValueError, "active parent"):
                self.generate(root)
            self.assertEqual(self.snapshot(root), before)
            # Existing parent bytes may be adopted without changing their authority.
            for case in manifest["cases"]:
                fixture = root / "pa27" / case["path"]
                for suffix, value in zip(course.SIDECARS, (b"qualified\n", b"", b"0\n")):
                    Path(str(fixture) + suffix).write_bytes(value)
            (root / "pa27/reference.json").write_text("parent receipt stays exact")
            self.generate(root)
            saved = self.snapshot(root)
            with self.assertRaisesRegex(ValueError, "active parent"):
                self.generate(root, command="print('changed')")
            self.assertEqual(self.snapshot(root), saved)
            self.assertEqual((root / "pa27/reference.json").read_text(), "parent receipt stays exact")

    def test_oracle_symlinks_cannot_overwrite_other_units_or_external_files(self):
        for suffix in course.SIDECARS:
            with self.subTest(suffix=suffix), tempfile.TemporaryDirectory() as name:
                root, _ = self.fixture(name)
                protected = root / "protected.ref"
                protected.write_bytes(b"keep exact")
                Path(str(root / "pa27/tests/places/y.rs") + suffix).symlink_to(protected)
                with mock.patch("course.execute") as execute, self.assertRaisesRegex(ValueError, "symlink"):
                    self.generate(root)
                execute.assert_not_called()
                self.assertEqual(protected.read_bytes(), b"keep exact")
                self.assertFalse((root / "pa27/tests/places/x.rs.ref").exists())
                self.assertFalse(suite_paths(root, "pa27", "unsafe-places").reference.exists())

    def test_unit_feature_ownership_and_explicit_dependency_control_admission(self):
        for changes in ({"work_units": []}, {"work_units": ["raw-validity"]},
                        {"features": ["pa42.async"]}, {"work_units": ["unsafe-places", "unknown"]}):
            with self.subTest(changes=changes), tempfile.TemporaryDirectory() as name:
                root, manifest = self.fixture(name)
                manifest["cases"][0].update(changes)
                self.write_manifest(root, manifest)
                with self.assertRaises(ValueError):
                    course.load_cases(root, "pa27", unit="unsafe-places")
        with tempfile.TemporaryDirectory() as name:
            root, manifest = self.fixture(name)
            plan = json.loads((root / "assignment-plan.json").read_text())
            plan["units"].insert(0, {"id": "source-bindings", "owner": "pa31", "scope": "declarations",
                "exit": "checked declarations", "requirements": [], "depends_on": [],
                "implementation": "partial", "publication": "unpublished"})
            plan["units"][1]["depends_on"] = ["source-bindings"]
            (root / "assignment-plan.json").write_text(json.dumps(plan))
            stages = json.loads((root / "assignments.json").read_text())
            stages["stages"].append({"path": "pa31", "number": 31, "status": "planned"})
            (root / "assignments.json").write_text(json.dumps(stages))
            manifest["cases"][0]["features"] = ["pa31.source"]
            self.write_manifest(root, manifest)
            course.load_cases(root, "pa27", unit="unsafe-places")

    def test_scoped_generation_uses_only_own_receipt_and_preserves_other_sidecars(self):
        with tempfile.TemporaryDirectory() as name:
            root, manifest = self.fixture(name)
            legacy = root / "pa27/reference.json"
            legacy.write_text('{"stage": "pa27", "fixtures": {}}')
            before = self.snapshot(root)
            with self.assertRaisesRegex(ValueError, "full reference receipt"):
                self.generate(root, ["tests/places/x.rs"])
            self.assertEqual(self.snapshot(root), before)
            self.assertTrue(self.generate(root))
            base = suite_paths(root, "pa27", "unsafe-places").reference.read_bytes()
            y = (root / "pa27/tests/places/y.rs.ref").read_bytes()
            self.assertTrue(self.generate(root, ["tests/places/x.rs"], "print('changed')"))
            self.verify(root)
            self.assertEqual(suite_paths(root, "pa27", "unsafe-places").reference.read_bytes(), base)
            self.assertEqual((root / "pa27/tests/places/y.rs.ref").read_bytes(), y)
            self.assertEqual(legacy.read_bytes(), before["pa27/reference.json"])
            for selected in ([], ["tests/later/unreleased.rs"], ["tests/places/x.rs"] * 2):
                saved = self.snapshot(root)
                with self.assertRaises(ValueError):
                    self.generate(root, selected)
                self.assertEqual(self.snapshot(root), saved)

    def test_identity_contract_input_and_all_output_channels_are_bound(self):
        for partial in (False, True):
            for mutation in ("unit", "stage", "manifest_path", "manifest", "contract", "input",
                             "output", "missing-output", "exit-status"):
                with self.subTest(partial=partial, mutation=mutation), tempfile.TemporaryDirectory() as name:
                    root, manifest = self.fixture(name)
                    self.generate(root)
                    paths = suite_paths(root, "pa27", "unsafe-places")
                    if partial:
                        self.generate(root, ["tests/places/x.rs"], "print('partial')")
                    receipt_path = next(paths.batches.glob("*.json")) if partial else paths.reference
                    receipt = json.loads(receipt_path.read_text())
                    if mutation in ("unit", "stage", "manifest_path"):
                        receipt[mutation] = "wrong"
                        receipt_path.write_text(json.dumps(receipt))
                    elif mutation == "manifest":
                        paths.manifest.write_bytes(paths.manifest.read_bytes() + b" ")
                    elif mutation == "contract":
                        manifest["cases"][0]["args"] = ["--cfg", "changed"]
                        self.write_manifest(root, manifest)
                    elif mutation == "input":
                        (root / "pa27/tests/places/x.rs").write_text("changed input")
                    elif mutation == "output":
                        (root / "pa27/tests/places/x.rs.ref").write_text("changed output")
                    elif mutation == "exit-status":
                        (root / "pa27/tests/places/x.rs.ref.exit_status").write_text("1\n")
                    else:
                        values = receipt["cases"]["tests/places/x.rs"]["outputs"] if partial else receipt["outputs"]["tests/places/x.rs"]
                        del values[".ref.stderr"]
                        receipt_path.write_text(json.dumps(receipt))
                    with self.assertRaises((ValueError, FileNotFoundError)):
                        self.verify(root)

    def test_failed_outcomes_cannot_publish_and_full_generation_supersedes_scoped_batches(self):
        with tempfile.TemporaryDirectory() as name:
            root, manifest = self.fixture(name)
            self.generate(root)
            before = self.snapshot(root)
            self.assertFalse(self.generate(root, ["tests/places/x.rs"], "raise SystemExit(1)"))
            self.assertEqual(self.snapshot(root), before)
            self.generate(root, ["tests/places/x.rs"], "print('partial')")
            batch = next(suite_paths(root, "pa27", "unsafe-places").batches.glob("*.json"))
            retained = batch.read_bytes()
            self.generate(root, command="print('full')")
            self.verify(root)
            self.assertEqual(batch.read_bytes(), retained)

    def test_course_cli_and_independent_runners_share_the_selected_unit_identity(self):
        import check_native
        import check_syntax
        with tempfile.TemporaryDirectory() as name:
            root, manifest = self.fixture(name)
            app = root / "candidate"
            app.write_text(f"#!{sys.executable}\nprint('qualified')\n")
            app.chmod(0o755)
            argv = ["course.py", "pa27", "--unit", "unsafe-places", "--bless", "--app", str(app)]
            with mock.patch("course.ROOT", root), mock.patch.object(sys, "argv", argv), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(course.main(), 0)
            manifest["mode"] = "--emit-lowir"
            for case in manifest["cases"]:
                case.update(exit_status=1, rustc={"kind": "program", "status": 1})
            self.write_manifest(root, manifest)
            (root / "backend.lock.json").write_text('{"pin": "test only"}')
            rejected = (None, (b"", b"ERROR: fixture rejection\n", b"1\n"))
            result = SimpleNamespace(returncode=1, stdout=b"", stderr=b"fixture rejection")
            with mock.patch("check_native.ROOT", root), mock.patch("check_syntax.ROOT", root), \
                    mock.patch("check_native.load_tools", return_value={}), \
                    mock.patch("subprocess.check_output", return_value="rustc 1.98.0 pinned"), \
                    mock.patch("subprocess.run", return_value=result), \
                    mock.patch("check_native.run_case", return_value=rejected), \
                    mock.patch("check_syntax.run_case", return_value=rejected), \
                    contextlib.redirect_stdout(io.StringIO()):
                check_native.check(app, 1, "pa27", unit="unsafe-places")
                check_syntax.check("pa27", app, 1, unit="unsafe-places")
            receipt = json.loads((root / "obj/native/pa27/units/unsafe-places/evidence.json").read_text())
            self.assertEqual(receipt["unit"], "unsafe-places")
            self.assertEqual(receipt["manifest_path"], "pa27/tests/units/unsafe-places.json")
            self.assertTrue((root / "obj/rustc-syntax/pa27/units/unsafe-places/tests/places/x/rustc.stderr").is_file())
            self.assertFalse((root / "obj/native/pa27/tests").exists())

    def test_unknown_unit_stops_independent_runners_before_external_commands(self):
        import check_native
        import check_syntax
        with tempfile.TemporaryDirectory() as name:
            root, _ = self.fixture(name)
            with mock.patch("check_native.ROOT", root), mock.patch("check_syntax.ROOT", root), \
                    mock.patch("subprocess.check_output") as version:
                with self.assertRaises(ValueError):
                    check_native.check(root / "missing", 1, "pa27", unit="missing")
                with self.assertRaises(ValueError):
                    check_syntax.check("pa27", root / "missing", 1, unit="missing")
                version.assert_not_called()

    def test_releasing_component_checks_provenance_and_never_closes_residual_parent_work(self):
        with tempfile.TemporaryDirectory() as name:
            root, _ = self.fixture(name)
            plan = json.loads((root / "assignment-plan.json").read_text())
            plan["units"][0].update(implementation="implemented", implementation_evidence=["docs/proof.md"],
                                     publication="released", release_evidence="docs/proof.md")
            (root / "assignment-plan.json").write_text(json.dumps(plan))
            requirements = json.loads((root / "docs/assignment-requirements.json").read_text())
            requirements["rows"][0]["implementation"] = "implemented_component"
            (root / "docs/assignment-requirements.json").write_text(json.dumps(requirements))
            self.assertEqual(len(assignment_metadata.collect_units(root)), 1)
            with self.assertRaises(FileNotFoundError), contextlib.redirect_stdout(io.StringIO()):
                check_layout.check(root)
            self.generate(root)
            with contextlib.redirect_stdout(io.StringIO()):
                check_layout.check(root)
            (root / "pa27/tests/places/x.rs.ref").unlink()
            with self.assertRaises(FileNotFoundError), contextlib.redirect_stdout(io.StringIO()):
                check_layout.check(root)
            stages = json.loads((root / "assignments.json").read_text())
            stages["stages"][0]["status"] = "implemented"
            (root / "assignments.json").write_text(json.dumps(stages))
            with self.assertRaisesRegex(ValueError, "whole PA completion has unpublished work"):
                assignment_plan.validate(root)

    def test_release_scheduler_skips_unpublished_work_and_propagates_real_failures(self):
        with tempfile.TemporaryDirectory() as name:
            root, _ = self.fixture(name)
            called = []
            stream = io.StringIO()
            with contextlib.redirect_stdout(stream):
                run_released(root, lambda stage, unit: called.append((stage, unit)))
            self.assertEqual(called, [])
            self.assertIn("skipped", stream.getvalue())
            self.generate(root)
            plan = json.loads((root / "assignment-plan.json").read_text())
            plan["units"][0].update(implementation="implemented", implementation_evidence=["docs/proof.md"],
                                     publication="released", release_evidence="docs/proof.md")
            (root / "assignment-plan.json").write_text(json.dumps(plan))
            rows = json.loads((root / "docs/assignment-requirements.json").read_text())
            rows["rows"][0]["implementation"] = "implemented_component"
            (root / "docs/assignment-requirements.json").write_text(json.dumps(rows))
            run_released(root, lambda stage, unit: called.append((stage, unit)))
            self.assertEqual(called, [("pa27", "unsafe-places")])
            with self.assertRaisesRegex(ValueError, "released component failed"):
                run_released(root, lambda stage, unit: False)
            app = root / "candidate"
            app.write_text(f"#!{sys.executable}\nprint('qualified')\n")
            app.chmod(0o755)
            argv = ["course.py", "--released-units", "--app", str(app)]
            with mock.patch("course.ROOT", root), mock.patch.object(sys, "argv", argv), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(course.main(), 0)
            for extra in (["pa27"], ["--unit", "unsafe-places"], ["--bless"], ["--case", "tests/places/x.rs"]):
                with mock.patch.object(sys, "argv", argv + extra), contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
                    course.main()


if __name__ == "__main__":
    unittest.main()
