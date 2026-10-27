"""A passing component cannot silently discard a prerequisite or residual PA work."""
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from assignment_plan import fixture_owners, validate


class AssignmentPlan(unittest.TestCase):
    def fixture(self, directory):
        root = Path(directory)
        (root / "docs").mkdir()
        stages = [{"path": "pa25", "number": 25, "status": "planned"}]
        unit = {"id": "const-machine", "owner": "pa25", "implementation": "partial",
                "publication": "unpublished", "depends_on": [], "scope": "typed CFG",
                "exit": "execute actual constants", "requirements": ["25.1"]}
        row = {"id": "25.1", "pa": 25, "implementation": "partial", "source": [],
               "tests": [], "evidence": [], "units": ["const-machine"]}
        plan = {"schema": 1, "units": [unit]}
        return root, stages, plan, [row]

    def write(self, root, stages, plan, rows):
        (root / "assignments.json").write_text(json.dumps({"stages": stages}))
        (root / "assignment-plan.json").write_text(json.dumps(plan))
        (root / "docs/assignment-requirements.json").write_text(json.dumps({"rows": rows}))

    def test_number_is_ownership_and_dependency_order_is_explicit(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stages, plan, rows = self.fixture(directory)
            stages.append({"path": "pa31", "number": 31, "status": "planned"})
            source = {**plan["units"][0], "id": "source-bindings", "owner": "pa31",
                      "requirements": []}
            plan["units"].insert(0, source)
            plan["units"][1]["depends_on"] = ["source-bindings"]
            self.write(root, stages, plan, rows)
            self.assertEqual([unit["owner"] for unit in validate(root)[0]], ["pa31", "pa25"])

    def test_cycles_and_missing_dependencies_cannot_be_hidden_by_order(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stages, plan, rows = self.fixture(directory)
            for dependency in ("missing", "const-machine"):
                plan["units"][0]["depends_on"] = [dependency]
                self.write(root, stages, plan, rows)
                with self.assertRaisesRegex(ValueError, "missing, cyclic or later dependency"):
                    validate(root)

    def test_unfinished_requirement_cannot_be_mapped_only_to_complete_component(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stages, plan, rows = self.fixture(directory)
            plan["units"][0]["implementation"] = "implemented"
            plan["units"][0]["implementation_evidence"] = ["docs/proof.md"]
            (root / "docs/proof.md").write_text("finite component qualification")
            self.write(root, stages, plan, rows)
            with self.assertRaisesRegex(ValueError, "lost its remaining work"):
                validate(root)

    def test_unmapped_requirement_and_stale_evidence_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stages, plan, rows = self.fixture(directory)
            plan["units"][0]["requirements"] = []
            self.write(root, stages, plan, rows)
            with self.assertRaisesRegex(ValueError, "unmapped or inconsistent"):
                validate(root)
            plan["units"][0]["requirements"] = ["25.1"]
            rows[0]["evidence"] = ["docs/missing.md"]
            self.write(root, stages, plan, rows)
            with self.assertRaisesRegex(ValueError, "missing or unsafe"):
                validate(root)

    def test_whole_pa_requires_component_publication(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stages, plan, rows = self.fixture(directory)
            rows[0]["implementation"] = "implemented_component"
            plan["units"][0]["implementation"] = "implemented"
            plan["units"][0]["implementation_evidence"] = ["docs/proof.md"]
            (root / "docs/proof.md").write_text("finite component qualification")
            stages[0]["status"] = "implemented"
            self.write(root, stages, plan, rows)
            with self.assertRaisesRegex(ValueError, "whole PA completion has unpublished work"):
                validate(root)
            plan["units"][0].update(publication="released", manifest="docs/cases.json",
                                     release_evidence="docs/release.md")
            (root / "docs/release.md").write_text("qualified source, native and release evidence")
            (root / "docs/cases.json").write_text('{"cases": []}')
            self.write(root, stages, plan, rows)
            with self.assertRaisesRegex(ValueError, "empty manifest"):
                validate(root)
            (root / "docs/cases.json").write_text('{"cases": [{"path": "positive.rs"}]}')
            self.assertEqual(len(validate(root)[0]), 1)

    def test_requirement_owned_elsewhere_still_blocks_whole_pa_completion(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stages, plan, rows = self.fixture(directory)
            stages[0]["status"] = "implemented"
            stages.append({"path": "pa27", "number": 27, "status": "planned"})
            plan["units"][0]["owner"] = "pa27"
            plan["units"].append({**plan["units"][0], "id": "inspection", "owner": "pa25",
                                   "requirements": [], "implementation": "implemented",
                                   "publication": "released", "manifest": "docs/cases.json",
                                   "implementation_evidence": ["docs/proof.md"],
                                   "release_evidence": "docs/proof.md"})
            (root / "docs/proof.md").write_text("complete inspection qualification")
            (root / "docs/cases.json").write_text('{"cases": [{"path": "inspection.rs"}]}')
            self.write(root, stages, plan, rows)
            with self.assertRaisesRegex(ValueError, "whole PA completion has unpublished work"):
                validate(root)

    def test_complete_components_need_existing_implementation_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stages, plan, rows = self.fixture(directory)
            plan["units"][0]["implementation"] = "implemented"
            self.write(root, stages, plan, rows)
            with self.assertRaisesRegex(ValueError, "implemented component needs evidence"):
                validate(root)

    def test_fixture_admits_owned_unit_dependencies_without_future_feature_exemption(self):
        stage = {"path": "pa26", "number": 26}
        units = {"hooks": {"owner": "pa26", "depends_on": ["source"]},
                 "source": {"owner": "pa31", "depends_on": []},
                 "async": {"owner": "pa42", "depends_on": []}}
        case = {"path": "tests/hooks.rs", "work_units": ["hooks"]}
        self.assertEqual(fixture_owners(stage, case, units, []), {"pa26", "pa31"})
        for selection in (["source"], ["async"], ["missing"], [], "hooks", ["hooks", "hooks"]):
            with self.subTest(selection=selection), self.assertRaises(ValueError):
                fixture_owners(stage, {**case, "work_units": selection}, units, [])
        rows = [{"pa": 26, "units": ["source"]}]
        self.assertEqual(fixture_owners(stage, {**case, "work_units": ["source"]}, units, rows),
                         {"pa31"})

    def test_duplicate_units_and_unknown_owners_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            root, stages, plan, rows = self.fixture(directory)
            plan["units"].append(plan["units"][0])
            self.write(root, stages, plan, rows)
            with self.assertRaisesRegex(ValueError, "invalid or duplicate"):
                validate(root)
            plan["units"].pop()
            plan["units"][0]["owner"] = "pa99"
            self.write(root, stages, plan, rows)
            with self.assertRaisesRegex(ValueError, "unknown PA owner"):
                validate(root)


if __name__ == "__main__":
    unittest.main()
