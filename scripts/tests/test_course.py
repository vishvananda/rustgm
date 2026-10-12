import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from course import compare, execute, load_cases, run_stage, run_case


class HarnessTests(unittest.TestCase):
    def test_supplemental_mode_must_be_declared_and_is_passed_exactly(self):
        with tempfile.TemporaryDirectory() as name:
            case = {"path": "tests/x.rs", "exit_status": 0, "mode": "--emit-layouts"}
            root, tests = self.fixture_root(name, [case])
            (tests / "x.rs").write_text("input")
            with self.assertRaises(ValueError):
                load_cases(root, "pa1")
            manifest = {"mode": "--emit-lowir", "additional_modes": ["--emit-layouts"], "cases": [case]}
            (tests / "manifest.json").write_text(json.dumps(manifest))
            load_cases(root, "pa1")
            app = [sys.executable, "-c", "import json,sys; print(json.dumps(sys.argv[1:]))"]
            _, actual = run_case(root, "pa1", "--emit-lowir", case, app, 2)
            self.assertEqual(json.loads(actual[0]), ["--emit-layouts", "--edition=2024"])
            for modes in (["--emit-lowir"], ["-o"], [1], [["--emit-layouts"]], "--emit-layouts"):
                manifest["additional_modes"] = modes
                (tests / "manifest.json").write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):
                    load_cases(root, "pa1")

    def test_cfg_arguments_are_preserved_without_shell_interpretation(self):
        with tempfile.TemporaryDirectory() as name:
            args = ["--cfg", 'feature="a b"', "--cfg", "custom"]
            case = {"path": "tests/x.rs", "exit_status": 0, "args": args}
            root, tests = self.fixture_root(name, [case])
            (tests / "x.rs").write_text("input")
            load_cases(root, "pa1")
            app = [sys.executable, "-c", "import json,sys; print(json.dumps(sys.argv[1:]))"]
            _, actual = run_case(root, "pa1", "--emit-tokens", case, app, 2)
            self.assertEqual(json.loads(actual[0]), ["--emit-tokens", "--edition=2024", *args])
            for invalid in (["--emit-literals"], ["--cfg"], ["-o", "x"], "--cfg x", ["--cfg", 1]):
                case["args"] = invalid
                (tests / "manifest.json").write_text(json.dumps({"mode": "--emit-tokens", "cases": [case]}))
                with self.assertRaises(ValueError):
                    load_cases(root, "pa1")

    def test_environment_is_explicit_and_null_removes_an_inherited_value(self):
        with tempfile.TemporaryDirectory() as name:
            case = {"path": "tests/x.rs", "exit_status": 0,
                    "environment": {"RUSTGM_HARNESS_TEST": "a b\né", "PATH": None}}
            root, tests = self.fixture_root(name, [case])
            (tests / "x.rs").write_text("input")
            load_cases(root, "pa1")
            app = [sys.executable, "-c", "import json,os; print(json.dumps([os.getenv('RUSTGM_HARNESS_TEST'),os.getenv('PATH')]))"]
            _, actual = run_case(root, "pa1", "--emit-tokens", case, app, 2)
            self.assertEqual(json.loads(actual[0]), ["a b\né", None])
            for invalid in ({"": "x"}, {"a=b": "x"}, {"x": 1}, {"x": "a\0b"}, []):
                case["environment"] = invalid
                (tests / "manifest.json").write_text(json.dumps({"mode": "--emit-tokens", "cases": [case]}))
                with self.assertRaises(ValueError):
                    load_cases(root, "pa1")

    def test_module_inputs_are_explicit_discovered_and_hashed(self):
        with tempfile.TemporaryDirectory() as name:
            case = {"path": "tests/package/lib.rs", "exit_status": 0, "input": "file",
                    "auxiliary": ["tests/package/child.rs"]}
            root, tests = self.fixture_root(name, [case])
            (tests / "package").mkdir()
            (tests / "package/lib.rs").write_text("mod child;")
            (tests / "package/child.rs").write_text("pub fn f(){}")
            load_cases(root, "pa1")
            app = [sys.executable, "-c", "import pathlib,sys; p=pathlib.Path(sys.argv[-1]); print(p.name, p.with_name('child.rs').read_text())"]
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertTrue(run_stage(root, "pa1", app, bless=True))
                self.assertTrue(run_stage(root, "pa1", app))
            receipt = json.loads((root / "pa1/reference.json").read_text())
            self.assertEqual(set(receipt["auxiliary"]), {"tests/package/child.rs"})
            for changes in ({"input": "stdin"}, {"input": "unknown"},
                            {"path": str(tests / "package/lib.rs")},
                            {"path": "tests/package/../package/lib.rs"},
                            {"auxiliary": []}, {"auxiliary": ["tests/package/lib.rs"]},
                            {"auxiliary": ["../../outside.rs"]}):
                invalid = {**case, **changes}
                (tests / "manifest.json").write_text(json.dumps({"mode": "--emit-tokens", "cases": [invalid]}))
                with self.assertRaises(ValueError):
                    load_cases(root, "pa1")

    def test_file_input_names_are_stable_when_checkout_is_relocated(self):
        outputs = []
        for _ in range(2):
            with tempfile.TemporaryDirectory() as name:
                case = {"path": "tests/input.rs", "exit_status": 0, "input": "file",
                        "args": ["--crate-name", "chosen"]}
                root, tests = self.fixture_root(name, [case])
                (tests / "input.rs").write_text("input")
                load_cases(root, "pa1")
                app = [sys.executable, "-c", "import pathlib,sys; print(sys.argv[-1]); print(pathlib.Path(sys.argv[-1]).read_text()); print(sys.argv[-3:-1])"]
                _, actual = run_case(root, "pa1", "--emit-expanded-crate", case, app, 2)
                outputs.append(actual)
        self.assertEqual(outputs[0], outputs[1])
        self.assertEqual(outputs[0], (b"pa1/tests/input.rs\ninput\n['--crate-name', 'chosen']\n", b"", b"0\n"))

    def test_signals_stub_and_other_exits_are_failures(self):
        for code in (2, 86, 101, -9):
            command = "import os,signal; os.kill(os.getpid(),signal.SIGKILL)" if code < 0 else f"raise SystemExit({code})"
            with self.subTest(code=code), self.assertRaises(ValueError):
                execute([sys.executable, "-c", command], b"", 2)

    def test_timeout_is_never_rejection(self):
        with self.assertRaises(ValueError):
            execute([sys.executable, "-c", "import time; time.sleep(5)"], b"", .03)

    def test_text_stderr_and_status_are_all_exact(self):
        with tempfile.TemporaryDirectory() as name:
            fixture = Path(name) / "x.rs"
            actual = (b"out\n", b"err\n", b"1\n")
            for suffix, value in zip((".ref", ".ref.stderr", ".ref.exit_status"), actual):
                Path(str(fixture) + suffix).write_bytes(value)
            self.assertEqual(compare(fixture, actual), [])
            for position in range(3):
                changed = list(actual)
                changed[position] += b" "
                self.assertTrue(compare(fixture, changed))

    def fixture_root(self, name, cases):
        root = Path(name)
        tests = root / "pa1/tests"
        tests.mkdir(parents=True)
        (tests / "manifest.json").write_text(json.dumps({"mode": "--emit-tokens", "cases": cases}))
        (root / "dev").mkdir()
        return root, tests

    def test_empty_and_planned_suites_fail(self):
        with tempfile.TemporaryDirectory() as name:
            root, _ = self.fixture_root(name, [])
            with self.assertRaises(ValueError):
                load_cases(root, "pa1")
            with self.assertRaises(ValueError):
                load_cases(root, "pa2")

    def test_orphan_fixture_fails_discovery(self):
        with tempfile.TemporaryDirectory() as name:
            case = {"path": "tests/x.rs", "exit_status": 0}
            root, tests = self.fixture_root(name, [case])
            (tests / "x.rs").write_text("x")
            (tests / "orphan.rs").write_text("y")
            with self.assertRaises(ValueError):
                load_cases(root, "pa1")

    def test_duplicate_cases_fail(self):
        with tempfile.TemporaryDirectory() as name:
            case = {"path": "tests/x.rs", "exit_status": 0}
            root, _ = self.fixture_root(name, [case, case])
            with self.assertRaises(ValueError):
                load_cases(root, "pa1")

    def test_failed_bless_leaves_references_unchanged(self):
        with tempfile.TemporaryDirectory() as name:
            cases = [{"path": "tests/x.rs", "exit_status": 0}, {"path": "tests/y.rs", "exit_status": 1}]
            root, tests = self.fixture_root(name, cases)
            for case in cases:
                p = root / "pa1" / case["path"]
                p.write_text("input")
                Path(str(p) + ".ref").write_bytes(b"original")
            app = [sys.executable, "-c", "print('new'); raise SystemExit(0)"]
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertFalse(run_stage(root, "pa1", app, bless=True))
            self.assertEqual((tests / "x.rs.ref").read_bytes(), b"original")
            self.assertEqual((tests / "y.rs.ref").read_bytes(), b"original")

    def test_missing_oracles_fail(self):
        with tempfile.TemporaryDirectory() as name:
            self.assertEqual(len(compare(Path(name) / "x.rs", (b"", b"", b"0\n"))), 3)

    def test_bless_then_test_and_changed_output_fail(self):
        with tempfile.TemporaryDirectory() as name:
            root, tests = self.fixture_root(name, [{"path": "tests/x.rs", "exit_status": 0}])
            (tests / "x.rs").write_text("input")
            app = [sys.executable, "-c", "print('gold')"]
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertTrue(run_stage(root, "pa1", app, bless=True))
                self.assertTrue(run_stage(root, "pa1", app))
                self.assertFalse(run_stage(root, "pa1", [sys.executable, "-c", "print('changed')"]))


if __name__ == "__main__":
    unittest.main()
