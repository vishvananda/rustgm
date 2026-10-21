import sys
from pathlib import Path
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_syntax import check, check_classification, expression_source, check_configured_owners, rustc_arguments, rustc_emission


class IndependentSyntaxTests(unittest.TestCase):
    def test_library_emission_uses_declared_crate_name(self):
        case = {"library": {"crate_name": "explicit", "disambiguator": "batch"},
                "args": ["--cfg", "selected"]}
        self.assertEqual(rustc_arguments(case, "default"),
                         ["--crate-name", "explicit", "--cfg", "selected"])
        self.assertEqual(rustc_emission({"kind": "library", "emit": "obj"}), "--emit=obj")

    def test_invalid_worker_limits_fail_before_starting_compilers(self):
        with mock.patch("check_syntax.subprocess.check_output") as command:
            for jobs in (0, -1, 9):
                with self.assertRaises(ValueError):
                    check("pa1", Path("unused"), jobs)
            with mock.patch.dict("os.environ", {"RUSTGM_TEST_JOBS": "0"}):
                with self.assertRaises(ValueError):
                    check("pa1", Path("unused"))
            command.assert_not_called()

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
