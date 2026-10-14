import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_syntax import check_classification, expression_source, check_configured_owners, rustc_arguments, rustc_emission


class IndependentSyntaxTests(unittest.TestCase):
    def test_independent_compilers_keep_cfg_and_one_effective_crate_name(self):
        case = {"args": ["--crate-name", "first", "--cfg", 'feature="two words"',
                         "--crate-name", "last", "--cfg", "selected"]}
        self.assertEqual(rustc_arguments(case, "default"),
                         ["--crate-name", "last", "--cfg", 'feature="two words"', "--cfg", "selected"])
        self.assertEqual(rustc_arguments({}, "default"), ["--crate-name", "default"])

    def test_configured_owners_are_explicit_and_ordered(self):
        output = b"configured 8 false\nconfigured 10 true\nactive [9,10]\n"
        check_configured_owners({"configured": [False, True]}, output)
        with self.assertRaises(ValueError):
            check_configured_owners({"configured": [False]}, output)
        with self.assertRaises(ValueError):
            check_configured_owners({"configured": [True, False]}, output)

    def test_acceptance_differences_cannot_be_silent(self):
        case = {"path": "x.rs", "exit_status": 0, "rustc": {"kind": "expression", "status": 1}}
        with self.assertRaises(ValueError):
            check_classification(case)
        case["rustc"]["reason"] = "types are checked later"
        check_classification(case)

    def test_expression_wrapper_preserves_bytes_and_explicit_context(self):
        data = b'"\xc3\xa9"\r\n'
        self.assertEqual(expression_source(data, {}), b'fn main() { let _ = ' + data + b'; }\n')
        self.assertEqual(expression_source(data, {"prefix": "prefix\n", "suffix": "\nsuffix"}),
                         b"prefix\n" + data + b"\nsuffix")

    def test_monomorphization_requires_object_emission(self):
        self.assertEqual(rustc_emission({"kind": "program"}), "--emit=metadata")
        self.assertEqual(rustc_emission({"kind": "program", "emit": "obj"}), "--emit=obj")
        for metadata in ({"kind": "expression", "emit": "obj"},
                         {"kind": "program", "emit": "invalid"}):
            with self.assertRaises(ValueError):
                rustc_emission(metadata)
