"""Grouping must neither omit suites nor weaken focused test selection."""
import argparse
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import Mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from maintainer_suites import command, discover, run_suite, selected_tests


class MaintainerSuitesTests(unittest.TestCase):
    def fixture(self, directory):
        root = Path(directory)
        groups = root / "dev/tests/suites"
        (groups / "types").mkdir(parents=True)
        (root / "dev/Cargo.toml").write_text(
            '[package]\nautotests = false\n[[test]]\n'
            'name = "suite_types"\npath = "tests/suites/types/mod.rs"\n')
        (root / "dev/tests/example.rs").write_text("#[test] fn example() {}\n")
        (groups / "types/mod.rs").write_text('#[path = "../../example.rs"]\nmod example;\n')
        return root

    def test_complete_unconditional_inventory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = self.fixture(directory)
            self.assertEqual(discover(root), {"example": "suite_types"})

    def test_new_or_missing_suite_fails(self):
        for name in ("new.rs", "example.rs"):
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = self.fixture(directory)
                path = root / "dev/tests" / name
                if path.exists():
                    path.unlink()
                else:
                    path.write_text("#[test] fn added() {}\n")
                with self.assertRaisesRegex(ValueError, "unregistered or missing"):
                    discover(root)

    def test_duplicate_and_conditional_registration_fail(self):
        for prefix in ('#[cfg(feature = "hidden")]\n', '#[path = "../../example.rs"]\nmod example;\n'):
            with self.subTest(prefix=prefix), tempfile.TemporaryDirectory() as directory:
                root = self.fixture(directory)
                group = root / "dev/tests/suites/types/mod.rs"
                group.write_text(prefix + group.read_text())
                with self.assertRaises(ValueError):
                    discover(root)

    def test_unregistered_group_and_feature_gated_target_fail(self):
        for change in ("orphan", "required-feature"):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as directory:
                root = self.fixture(directory)
                if change == "orphan":
                    (root / "dev/tests/suites/orphan.rs").write_text("// orphan\n")
                else:
                    manifest = root / "dev/Cargo.toml"
                    manifest.write_text(manifest.read_text() + 'required-features = ["hidden"]\n')
                with self.assertRaises(ValueError):
                    discover(root)

    def test_focused_suite_and_exact_case_keep_filter_boundaries(self):
        args = argparse.Namespace(suite="example", case=None, release=True,
                                  all_features=True, ignored=True, list=False)
        result = command({"example": "suite_types"}, args)
        self.assertEqual(result[-4:], ["--test", "suite_types", "--", "--ignored"])
        args.case = "nested::value"
        self.assertEqual(command({"example": "suite_types"}, args), result)
        with self.assertRaisesRegex(ValueError, "unknown"):
            command({}, args)

    def test_exact_inventory_excludes_suffixes_and_nested_lookalikes(self):
        listing = ("loans::one: test\nloans::nested::two: test\n"
                   "template_loans::one: test\nother::loans::one: test\n"
                   "4 tests, 0 benchmarks\n")
        self.assertEqual(selected_tests(listing, "loans", None),
                         ["loans::one", "loans::nested::two"])
        self.assertEqual(selected_tests(listing, "loans", "nested::two"),
                         ["loans::nested::two"])
        with self.assertRaisesRegex(ValueError, "no runnable tests"):
            selected_tests(listing, "loans", "missing")
        with self.assertRaisesRegex(ValueError, "duplicate"):
            selected_tests("loans::one: test\nloans::one: test\n", "loans", None)

    def test_failed_or_empty_inventory_never_runs_tests(self):
        args = argparse.Namespace(suite="example", case="missing", release=True,
                                  all_features=False, ignored=False, list=False)
        runner = Mock(return_value=SimpleNamespace(returncode=9, stdout=""))
        self.assertEqual(run_suite({"example": "suite_types"}, args, runner=runner), 9)
        self.assertEqual(runner.call_count, 1)
        runner = Mock(return_value=SimpleNamespace(returncode=0, stdout="0 tests, 0 benchmarks\n"))
        with self.assertRaisesRegex(ValueError, "no runnable tests"):
            run_suite({"example": "suite_types"}, args, runner=runner)
        self.assertEqual(runner.call_count, 1)

    def test_execution_uses_every_selected_exact_name_and_preserves_failure(self):
        args = argparse.Namespace(suite="example", case=None, release=True,
                                  all_features=True, ignored=True, list=False)
        runner = Mock(side_effect=[
            SimpleNamespace(returncode=0, stdout="example::one: test\nexample::two: test\n"),
            SimpleNamespace(returncode=7),
        ])
        self.assertEqual(run_suite({"example": "suite_types"}, args, runner=runner), 7)
        self.assertEqual(runner.call_args_list[0].args[0][-2:], ["--ignored", "--list"])
        self.assertEqual(runner.call_args_list[1].args[0][-4:],
                         ["--ignored", "--exact", "example::one", "example::two"])
