"""A finite contract closes an assignment only at released work, and moves the rest to a carrier."""
import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import assignment_metadata
import assignment_plan
import check_layout
import course
from course_suites import run_released, suite_paths
import finite_contracts


class Tree:
    """PA25 closed at one released unit; PA31 planned with nothing released; PA44 carrying the rest."""

    def __init__(self, directory):
        self.root = Path(directory)
        for folder in ("docs", "dev", "pa25/tests/machine", "pa25/tests/units", "pa25/tests/later",
                       "pa31/tests", "pa44/tests"):
            (self.root / folder).mkdir(parents=True)
        for stage in ("pa25", "pa31", "pa44"):
            for name in ("README.md", "Makefile", "tests/README.md"):
                (self.root / stage / name).write_text("fixture")
        for name in ("docs/proof.md", "docs/release.md"):
            (self.root / name).write_text("qualified evidence")
        (self.root / "dev/compiler.rs").write_text("fn compiler() {}")
        (self.root / "Makefile").write_text("ACTIVE_PAS := pa25\n")
        self.stages = [
            {"path": "pa25", "number": 25, "status": "implemented", "mode": "--emit-types",
             "release_evidence": ["docs/release.md"],
             "contract": {"kind": "finite", "carried_by": "pa44",
                          "deferred_units": ["const-memory", "source-runtime"],
                          "deferred_requirements": ["25.2", "25.3"]}},
            {"path": "pa31", "number": 31, "status": "planned", "mode": "--emit-types"},
            {"path": "pa44", "number": 44, "status": "planned", "mode": "--emit-types",
             "carries": {"pa25": {"units": ["const-memory", "source-runtime"], "requirements": ["25.2", "25.3"]}}}]
        common = {"scope": "finite profile", "exit": "real evidence", "depends_on": [], "requirements": []}
        partial = {**common, "implementation": "partial", "publication": "unpublished"}
        self.units = [
            {**common, "id": "const-machine", "owner": "pa25", "implementation": "implemented",
             "publication": "released", "requirements": ["25.1", "25.2"],
             "implementation_evidence": ["docs/proof.md"], "release_evidence": "docs/release.md",
             "manifest": "pa25/tests/units/const-machine.json"},
            {**partial, "id": "const-memory", "owner": "pa25", "requirements": ["25.2"]},
            {**partial, "id": "source-runtime", "owner": "pa31", "requirements": ["25.3"]},
            {**partial, "id": "frontend-acceptance", "owner": "pa44", "implementation": "missing",
             "depends_on": ["const-machine", "const-memory", "source-runtime"]}]
        row = {"pa": 25, "source": [], "tests": [], "evidence": []}
        self.rows = [
            {**row, "id": "25.1", "implementation": "implemented_component", "units": ["const-machine"]},
            {**row, "id": "25.2", "implementation": "partial", "units": ["const-machine", "const-memory"]},
            {**row, "id": "25.3", "implementation": "partial", "units": ["source-runtime"]}]
        cases = [{"path": f"tests/machine/{name}.rs", "exit_status": 0, "work_units": ["const-machine"],
                  "features": ["pa25.machine"]} for name in ("x", "y")]
        for case in cases:
            (self.root / "pa25" / case["path"]).write_text("fn main() {}")
        (self.root / "pa25/tests/later/unreleased.rs").write_text("deferred with const-memory")
        self.unit_manifest = {"schema": 1, "mode": "--emit-types", "discovery": ["tests/machine"], "cases": cases}
        self.write()
        (self.root / "pa25/tests/manifest.json").write_text(json.dumps(self.compose()))

    def write(self):
        (self.root / "assignments.json").write_text(json.dumps({"stages": self.stages}))
        (self.root / "assignment-plan.json").write_text(json.dumps({"schema": 1, "units": self.units}))
        (self.root / "docs/assignment-requirements.json").write_text(json.dumps({"rows": self.rows}))
        (self.root / "pa25/tests/units/const-machine.json").write_text(json.dumps(self.unit_manifest))

    def compose(self):
        return finite_contracts.compose(self.root, self.stages[0], self.units)

    def generate(self, command="print('qualified')", **options):
        with contextlib.redirect_stdout(io.StringIO()):
            return course.run_stage(self.root, "pa25", [sys.executable, "-c", command], jobs=2, **options)


class Closure(unittest.TestCase):
    def rejects(self, message, change):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            change(tree)
            tree.write()
            with self.assertRaisesRegex(ValueError, message):
                assignment_plan.validate(tree.root)

    def test_released_work_closes_the_assignment_and_the_rest_is_carried(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            self.assertEqual(len(assignment_plan.validate(tree.root)[0]), 4)
            self.assertEqual(finite_contracts.owed(tree.stages[0], tree.units, tree.rows),
                             (["const-memory", "source-runtime"], ["25.2", "25.3"]))

    def test_an_assignment_with_no_released_unit_cannot_close(self):
        def unpublish(tree):
            tree.units[0].update(publication="unpublished")
            del tree.units[0]["release_evidence"]
        self.rejects("finite contract has no released unit: pa25", unpublish)

        def close_unreleased(tree):
            tree.stages[1].update(status="implemented", release_evidence=["docs/release.md"], contract={
                "kind": "finite", "carried_by": "pa44", "deferred_units": ["source-runtime"],
                "deferred_requirements": []})
            tree.stages[2]["carries"]["pa31"] = {"units": ["source-runtime"], "requirements": []}
        self.rejects("finite contract has no released unit: pa31", close_unreleased)

    def test_a_closure_cannot_drop_an_unreleased_unit_or_a_requirement_row(self):
        def drop(key, value):
            return lambda tree: tree.stages[0]["contract"][key].remove(value)
        self.rejects(r"drops unreleased units: pa25: \['const-memory'\]", drop("deferred_units", "const-memory"))
        # A unit another assignment owns is still owed when this assignment's rows name it.
        self.rejects(r"drops unreleased units: pa25: \['source-runtime'\]", drop("deferred_units", "source-runtime"))
        self.rejects(r"drops requirement rows: pa25: \['25.3'\]", drop("deferred_requirements", "25.3"))
        # A row shared with a released unit keeps its unreleased share.
        self.rejects(r"drops requirement rows: pa25: \['25.2'\]", drop("deferred_requirements", "25.2"))

    def test_a_closure_defers_only_what_it_owes(self):
        def add(key, value):
            return lambda tree: tree.stages[0]["contract"][key].append(value)
        self.rejects(r"defers unreleased units it does not owe: pa25: \['const-machine'\]",
                     add("deferred_units", "const-machine"))
        self.rejects(r"defers requirement rows it does not owe: pa25: \['25.1'\]",
                     add("deferred_requirements", "25.1"))
        self.rejects("malformed assignment contract: pa25", add("deferred_units", "const-memory"))
        self.rejects("malformed assignment contract: pa25",
                     lambda tree: tree.stages[0]["contract"].update(kind="partial"))
        self.rejects("malformed assignment contract: pa25",
                     lambda tree: tree.stages[0]["contract"].pop("deferred_requirements"))

    def test_every_deferred_unit_and_row_is_carried(self):
        def uncarry(key, value):
            return lambda tree: tree.stages[2]["carries"]["pa25"][key].remove(value)
        self.rejects(r"deferred work is not carried: pa25 -> pa44: \['const-memory'\]",
                     uncarry("units", "const-memory"))
        self.rejects(r"deferred work is not carried: pa25 -> pa44: \['25.3'\]", uncarry("requirements", "25.3"))
        self.rejects(r"deferred work is not carried: pa25 -> pa44: \['const-memory', 'source-runtime', '25.2', '25.3'\]",
                     lambda tree: tree.stages[2].pop("carries"))
        self.rejects(r"carrier holds work the finite contract does not defer: pa25 -> pa44: \['25.1'\]",
                     lambda tree: tree.stages[2]["carries"]["pa25"]["requirements"].append("25.1"))
        self.rejects(r"carrier does not depend on deferred units: pa44: \['source-runtime'\]",
                     lambda tree: tree.units[3]["depends_on"].remove("source-runtime"))

    def test_the_carrier_owes_its_whole_handout_and_only_deferred_work(self):
        self.rejects("needs a carrier that owes its whole handout: pa25",
                     lambda tree: tree.stages[0]["contract"].update(carried_by="pa25"))
        self.rejects("needs a carrier that owes its whole handout: pa25",
                     lambda tree: tree.stages[0]["contract"].update(carried_by="pa99"))

        def defer_again(tree):
            tree.stages[2].update(status="implemented", release_evidence=["docs/release.md"], contract={
                "kind": "finite", "carried_by": "pa31", "deferred_units": ["frontend-acceptance"],
                "deferred_requirements": []})
        self.rejects("needs a carrier that owes its whole handout: pa25", defer_again)

        def carrier_first(tree):
            defer_again(tree)
            tree.stages.reverse()
        self.rejects("a carrier cannot defer its own work: pa44", carrier_first)
        self.rejects("carrier holds work no finite contract deferred to it: pa44: pa31",
                     lambda tree: tree.stages[2]["carries"].update(pa31={"units": [], "requirements": []}))
        self.rejects("carrier holds work no finite contract deferred to it: pa31: pa25",
                     lambda tree: tree.stages[1].update(carries=tree.stages[2]["carries"]))

    def test_the_carrier_is_complete_only_when_everything_it_carries_is_released(self):
        def complete_carrier(tree):
            tree.stages[2].update(status="implemented", release_evidence=["docs/release.md"])
            tree.units[3].update(implementation="implemented", implementation_evidence=["docs/proof.md"])
        self.rejects("whole PA completion has unpublished work: pa44", complete_carrier)

    def test_only_a_closed_assignment_declares_a_finite_contract(self):
        self.rejects("finite contract on an assignment that is not closed: pa25",
                     lambda tree: tree.stages[0].update(status="planned"))
        # Without the declaration the whole handout is owed, exactly as before.
        self.rejects("whole PA completion has unpublished work: pa25", lambda tree: tree.stages[0].pop("contract"))


class Manifest(unittest.TestCase):
    def test_the_union_is_active_and_unclaimed_fixtures_stay_out(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            manifest, cases = course.load_cases(tree.root, "pa25")
            self.assertEqual((manifest["contract"], manifest["units"]), ("finite", ["const-machine"]))
            self.assertEqual([case["path"] for case in cases], ["tests/machine/x.rs", "tests/machine/y.rs"])
            stages, loaded = assignment_metadata.collect(tree.root)
            self.assertEqual([(stage["path"], len(found)) for stage, _, found in loaded], [("pa25", 2)])
            # The planned assignments still fail explicitly.
            for planned in ("pa31", "pa44"):
                with self.assertRaisesRegex(ValueError, "planned assignment"):
                    course.load_cases(tree.root, planned)

    def test_the_manifest_is_exactly_the_union_of_the_released_units(self):
        def without_case(manifest):
            manifest["cases"].pop()
            manifest["discovery"] = ["tests/machine/x.rs"]

        def with_deferred_case(manifest):
            manifest["cases"].append({"path": "tests/later/unreleased.rs", "exit_status": 0,
                                      "work_units": ["const-machine"], "features": ["pa25.machine"]})
            manifest["discovery"].append("tests/later")

        def with_other_contract(manifest):
            manifest["cases"][0]["args"] = ["--cfg", "changed"]

        for change in (without_case, with_deferred_case, with_other_contract,
                       lambda manifest: manifest.pop("units")):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as name:
                tree = Tree(name)
                manifest = tree.compose()
                change(manifest)
                (tree.root / "pa25/tests/manifest.json").write_text(json.dumps(manifest))
                with self.assertRaisesRegex(ValueError, "not the union of its released unit manifests: pa25"):
                    assignment_metadata.collect(tree.root)

    def release_second_unit(self, tree, **profile):
        """Release const-memory with one LowIR rejection; requirement 25.2 is then fully released."""
        tree.units[1].update(implementation="implemented", implementation_evidence=["docs/proof.md"],
                             publication="released", release_evidence="docs/proof.md",
                             manifest="pa25/tests/units/const-memory.json", depends_on=["const-machine"])
        tree.stages[0]["contract"].update(deferred_units=["source-runtime"], deferred_requirements=["25.3"])
        tree.stages[2]["carries"]["pa25"] = {"units": ["source-runtime"], "requirements": ["25.3"]}
        tree.rows[1]["implementation"] = "implemented_component"
        later = {"schema": 1, "mode": "--emit-lowir", **profile, "discovery": ["tests/later"], "cases": [
            {"path": "tests/later/unreleased.rs", "exit_status": 1, "work_units": ["const-memory"],
             "features": ["pa25.memory"], "rustc": {"kind": "program", "status": 1}}]}
        (tree.root / "pa25/tests/units/const-memory.json").write_text(json.dumps(later))
        tree.write()

    def test_a_new_release_must_be_composed_before_the_assignment_validates(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            self.release_second_unit(tree)
            with self.assertRaisesRegex(ValueError, "not the union"):
                assignment_metadata.collect(tree.root)
            with self.assertRaisesRegex(ValueError, "omits a released unit's record"):
                finite_contracts.check_manifest(tree.root, tree.stages[0], tree.compose(), tree.units)
            tree.stages[0].update(release_evidence=["docs/release.md", "docs/proof.md"],
                                  additional_modes=["--emit-lowir"])
            tree.write()
            composed = tree.compose()
            (tree.root / "pa25/tests/manifest.json").write_text(json.dumps(composed))
            # The second unit's cases keep their mode under the assignment's default.
            self.assertEqual((composed["mode"], composed["additional_modes"], composed["units"]),
                             ("--emit-types", ["--emit-lowir"], ["const-machine", "const-memory"]))
            self.assertEqual([case.get("mode") for case in composed["cases"]], [None, None, "--emit-lowir"])
            self.assertEqual(composed["discovery"], ["tests/machine", "tests/later"])
            self.assertEqual(len(assignment_metadata.collect(tree.root)[1][0][2]), 3)

    def test_units_that_disagree_on_the_profile_or_have_no_cases_cannot_compose(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            self.release_second_unit(tree, edition="2021")
            with self.assertRaisesRegex(ValueError, "released units disagree on the course profile: pa25"):
                tree.compose()
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            tree.unit_manifest["cases"] = []
            tree.write()
            with self.assertRaisesRegex(ValueError, "released work unit has empty manifest"):
                assignment_metadata.collect(tree.root)

    def test_a_missing_or_empty_suite_never_passes(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            (tree.root / "pa25/tests/manifest.json").unlink()
            with self.assertRaisesRegex(ValueError, "no accepted test contract"):
                assignment_metadata.collect(tree.root)
            empty = dict(tree.compose(), cases=[])
            (tree.root / "pa25/tests/manifest.json").write_text(json.dumps(empty))
            with self.assertRaisesRegex(ValueError, "an empty suite cannot pass"):
                assignment_metadata.collect(tree.root)

    def test_only_a_finite_contract_claims_part_of_its_fixtures(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            manifest = tree.compose()
            del tree.stages[0]["contract"]
            tree.write()
            with self.assertRaisesRegex(ValueError, "only a finite contract claims part"):
                course.load_cases(tree.root, "pa25")
            del manifest["discovery"]
            (tree.root / "pa25/tests/manifest.json").write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, "undiscovered or missing fixtures"):
                course.load_cases(tree.root, "pa25")

    def test_release_evidence_names_every_released_unit_record(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            tree.stages[0]["release_evidence"] = ["docs/proof.md"]
            tree.write()
            with self.assertRaisesRegex(ValueError, "omits a released unit's record"):
                assignment_metadata.collect(tree.root)


class References(unittest.TestCase):
    def test_provenance_is_the_released_units_and_nothing_is_generated_for_the_assignment(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            with self.assertRaises(FileNotFoundError), contextlib.redirect_stdout(io.StringIO()):
                check_layout.check(tree.root)
            self.assertTrue(tree.generate(unit="const-machine", bless=True))
            before = {path: path.read_bytes() for path in sorted((tree.root / "pa25").rglob("*")) if path.is_file()}
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                check_layout.check(tree.root)
                check_layout.check(tree.root, unit="const-machine")
            self.assertIn("1 finite contracts checked through their released units", output.getvalue())
            # The whole assignment runs against the unit's references and writes none.
            self.assertTrue(tree.generate())
            self.assertFalse(tree.generate("print('changed')"))
            self.assertEqual(before, {path: path.read_bytes() for path in sorted((tree.root / "pa25").rglob("*"))
                                      if path.is_file()})
            self.assertFalse((tree.root / "pa25/reference.json").exists())
            (tree.root / "pa25/tests/machine/x.rs.ref").write_text("edited by hand\n")
            with self.assertRaisesRegex(ValueError, "oracle changed since unit generation"), \
                    contextlib.redirect_stdout(io.StringIO()):
                check_layout.check(tree.root)

    def test_whole_assignment_generation_goes_through_the_units(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            self.assertTrue(tree.generate(bless=True))
            receipt = suite_paths(tree.root, "pa25", "const-machine").reference
            self.assertEqual(json.loads(receipt.read_text())["unit"], "const-machine")
            self.assertFalse((tree.root / "pa25/reference.json").exists())
            with contextlib.redirect_stdout(io.StringIO()):
                check_layout.check(tree.root)
            with self.assertRaisesRegex(ValueError, "select the owning unit"):
                tree.generate(bless=True, case_paths=["tests/machine/x.rs"])
            # The unit publishes its own outputs: no parent receipt has to change first.
            self.assertTrue(tree.generate("print('requalified')", unit="const-machine", bless=True))
            self.assertEqual((tree.root / "pa25/tests/machine/x.rs.ref").read_text(), "requalified\n")
            (tree.root / "pa25/reference.json").write_text("{}")
            with self.assertRaisesRegex(ValueError, "cannot have its own reference receipt"):
                assignment_metadata.collect(tree.root)

    def test_released_units_of_a_closed_assignment_are_scheduled_once(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            tree.generate(unit="const-machine", bless=True)
            called, output = [], io.StringIO()
            with contextlib.redirect_stdout(output):
                run_released(tree.root, lambda stage, unit: called.append((stage, unit)))
            self.assertEqual(called, [])
            self.assertIn("1 released component suites run with their closed assignments", output.getvalue())
            # A release in a planned assignment is still scheduled on its own.
            del tree.stages[0]["contract"], tree.stages[2]["carries"]
            tree.stages[0].update(status="planned")
            tree.write()
            (tree.root / "pa25/tests/manifest.json").unlink()
            (tree.root / "Makefile").write_text("ACTIVE_PAS :=\n")
            run_released(tree.root, lambda stage, unit: called.append((stage, unit)))
            self.assertEqual(called, [("pa25", "const-machine")])


if __name__ == "__main__":
    unittest.main()
