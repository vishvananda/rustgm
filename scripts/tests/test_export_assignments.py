"""The student sequence puts reproducible self-hosting before the follow-on work."""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from export_assignments import selection, student_labels, student_path, student_stage


class ExportAssignments(unittest.TestCase):
    def test_self_hosting_and_follow_on_have_consecutive_student_paths(self):
        stages = [{"number": n, "path": f"pa{n}"} for n in range(1, 46)]
        assignments = {"stages": stages}
        track = selection(assignments, None)
        self.assertEqual(track, stages[:36] + [stages[44]])
        self.assertEqual(track, selection(assignments, "pa37"))
        self.assertEqual(selection(assignments, "pa38")[-1], stages[36])
        full = [student_stage(stage) for stage in selection(assignments, "pa45")]
        self.assertEqual(full, stages)

    def test_renumbering_updates_links_and_carried_work(self):
        self.assertEqual(
            student_labels("[PA45](../pa45/README.md) precedes PA37–PA44"),
            "[PA37](../pa37/README.md) precedes PA38–PA45",
        )
        self.assertEqual(student_labels("# RustGM Programming Assignment 45"), "# RustGM Programming Assignment 37")
        self.assertEqual(student_path("pa45/tests/pa45.rs"), "pa37/tests/pa45.rs")
        stage = {
            "number": 30, "path": "pa30",
            "contract": {"kind": "finite", "carried_by": "pa44"},
        }
        self.assertEqual(student_stage(stage)["contract"]["carried_by"], "pa45")
        self.assertEqual(stage["contract"]["carried_by"], "pa44")


if __name__ == "__main__":
    unittest.main()
