import copy
import unittest
from check_layouts import classification, probe, view_values


class LayoutContract(unittest.TestCase):
    def test_declared_vectors_check_candidate_offsets_and_unicode_names(self):
        check = {"name": "Σ", "rust_type": "Σ", "size": 16, "align": 8,
                 "offsets": [0, 8], "field_names": ["a", "b"]}
        output = (b'def 17 type=22 name="\\xce\\xa3" source=0:3\n'
                  b'layout 22 size=16 align=8 inhabited=true references=false aggregate tag=-\n'
                  b'field 22:0:0 type=1 offset=0\nfield 22:0:1 type=2 offset=8\n')
        view_values(output, [check])
        for changed in (output.replace(b"align=8", b"align=4"),
                        output.replace(b"offset=8", b"offset=1"),
                        output + b"field 22:0:2 type=1 offset=9\n",
                        output + b'def 18 type=23 name="\\xce\\xa3"\n'):
            with self.assertRaises(ValueError):
                view_values(changed, [check])
        code, expected = probe([check])
        self.assertIn("offset_of!(Σ,b)", code)
        self.assertEqual(expected, b"16 8\n0\n8\n")

    def test_layout_exceptions_and_outcomes_are_explicit(self):
        case = {"path": "tests/layout.rs", "exit_status": 0, "rustc": {"kind": "program", "status": 0},
                "layout_checks": [{"name": "T", "rust_type": "T", "size": 16, "align": 8,
                                   "rustc_layout": False}]}
        with self.assertRaises(ValueError):
            classification(case)
        case["layout_checks"][0]["reason"] = "course source-order Rust layout"
        classification(case)
        for changes in ({"exit_status": 1}, {"layout_checks": []},
                        {"rustc": {"kind": "program", "status": 1, "reason": "not executable"}}):
            invalid = copy.deepcopy(case)
            invalid.update(changes)
            with self.assertRaises(ValueError):
                classification(invalid)
