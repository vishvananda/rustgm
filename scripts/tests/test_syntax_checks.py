import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_syntax import check_classification, expression_source, check_configured_owners


class IndependentSyntaxTests(unittest.TestCase):
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
