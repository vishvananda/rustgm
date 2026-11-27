"""The student export selects consecutive assignments directly without renumbering."""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from export_assignments import selection, student_stage


class ExportAssignments(unittest.TestCase):
    def test_self_hosting_and_follow_on_use_unified_consecutive_paths(self):
        stages = [{"number": n, "path": f"pa{n}"} for n in range(1, 46)]
        assignments = {"stages": stages}
        track = selection(assignments, None)
        self.assertEqual(track, stages[:37])
        self.assertEqual(track, selection(assignments, "pa37"))
        self.assertEqual(selection(assignments, "pa38")[-1], stages[37])
        full = [student_stage(stage) for stage in selection(assignments, "pa45")]
        self.assertEqual(full, stages)

    def test_student_stage_drops_maintainer_release_evidence_only(self):
        stage = {
            "number": 30, "path": "pa30",
            "contract": {"kind": "finite", "carried_by": "pa45"},
            "release_evidence": ["docs/pa30-c-abi-release.md"],
        }
        exported = student_stage(stage)
        self.assertNotIn("release_evidence", exported)
        self.assertEqual(exported["contract"]["carried_by"], "pa45")
        self.assertIn("release_evidence", stage)


if __name__ == "__main__":
    unittest.main()
