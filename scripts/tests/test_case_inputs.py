"""A prelude or host provider is a declared, hashed input of its case."""
import contextlib
import io
import json
from pathlib import Path
import re
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
from case_inputs import source_bytes, support_files
import check_layout
from check_native import classification
from course import load_cases, run_case, run_stage

REPOSITORY = SCRIPTS.parent
ECHO = [sys.executable, "-c", "import sys; sys.stdout.buffer.write(sys.stdin.buffer.read())"]


class DeclaredInputs(unittest.TestCase):
    def stage(self, directory, case):
        root = Path(directory)
        tests = root / "pa1/tests"
        (tests / "prelude").mkdir(parents=True)
        (root / "dev").mkdir()
        (tests / "body.rs").write_text("fn main(){check(true);}\n")
        (tests / "prelude/check.rs").write_text("fn check(ok:bool){}\n")
        (tests / "provider.c").write_text("int imported(void) { return 3; }\n")
        self.write(root, case)
        return root

    def write(self, root, *cases):
        manifest = {"mode": "--emit-lowir", "cases": list(cases)}
        (root / "pa1/tests/manifest.json").write_text(json.dumps(manifest))
        return manifest

    def case(self, **changes):
        return {"path": "tests/body.rs", "exit_status": 0, "prelude": "tests/prelude/check.rs",
                "rustc": {"kind": "program", "status": 0},
                "native": {"kind": "execute", "outcome": "pass", "providers": ["tests/provider.c"]}} | changes

    def test_the_compiler_reads_the_prelude_then_the_fixture_as_one_unit(self):
        with tempfile.TemporaryDirectory() as name:
            case = self.case()
            root = self.stage(name, case)
            _, cases = load_cases(root, "pa1")
            self.assertEqual(support_files(cases[0]), ["tests/prelude/check.rs", "tests/provider.c"])
            joined = b"fn check(ok:bool){}\nfn main(){check(true);}\n"
            self.assertEqual(source_bytes(root, "pa1", case), joined)
            _, actual = run_case(root, "pa1", "--emit-lowir", case, ECHO, 5)
            self.assertEqual(actual, (joined, b"", b"0\n"))
            plain = {key: value for key, value in case.items() if key != "prelude"}
            self.assertEqual(source_bytes(root, "pa1", plain), b"fn main(){check(true);}\n")

    def test_malformed_preludes_and_providers_reject_before_any_run(self):
        with tempfile.TemporaryDirectory() as name:
            root = self.stage(name, self.case())
            (root / "pa1/outside.rs").write_text("fn check(ok:bool){}\n")
            native = self.case()["native"]
            for changes in ({"input": "file"}, {"prelude": "tests/absent.rs"}, {"prelude": "tests/body.rs"},
                            {"prelude": "tests/provider.c"}, {"prelude": "outside.rs"}, {"prelude": 7},
                            {"prelude": "tests/../tests/prelude/check.rs"},
                            {"rustc": {"kind": "expression", "status": 0}},
                            {"native": native | {"providers": []}},
                            {"native": native | {"providers": "tests/provider.c"}},
                            {"native": native | {"providers": ["tests/provider.c", "tests/provider.c"]}},
                            {"native": native | {"providers": ["tests/prelude/check.rs"]}},
                            {"native": native | {"providers": ["tests/absent.c"]}}):
                self.write(root, self.case(**changes))
                with self.subTest(changes=changes), self.assertRaises(ValueError):
                    load_cases(root, "pa1")

    def test_an_undeclared_prelude_source_is_an_orphan_fixture(self):
        with tempfile.TemporaryDirectory() as name:
            case = self.case()
            del case["prelude"]
            root = self.stage(name, case)
            with self.assertRaises(ValueError):
                load_cases(root, "pa1")

    def test_host_providers_link_programs_only(self):
        program = self.case()
        classification(program)
        classification(program | {"native": {"kind": "codegen", "reason": "compile only",
                                             "providers": ["tests/provider.c"]}})
        library = program | {"rustc": {"kind": "library", "status": 0},
                             "library": {"crate_name": "unit", "disambiguator": "pa1"},
                             "native": {"kind": "object", "reason": "entry-free",
                                        "providers": ["tests/provider.c"]}}
        for case in (library, program | {"native": program["native"] | {"providers": []}}):
            with self.subTest(case=case), self.assertRaises(ValueError):
                classification(case)

    def test_reference_receipts_bind_every_declared_input(self):
        with tempfile.TemporaryDirectory() as name:
            case = self.case()
            root = self.stage(name, case)
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertTrue(run_stage(root, "pa1", ECHO, bless=True, jobs=1))
            receipt = json.loads((root / "pa1/reference.json").read_text())
            self.assertEqual(sorted(receipt["auxiliary"]), ["tests/prelude/check.rs", "tests/provider.c"])
            manifest, cases = load_cases(root, "pa1")
            check_layout.check_suite(root, "pa1", manifest, cases)
            for changed in ("tests/prelude/check.rs", "tests/provider.c"):
                path = root / "pa1" / changed
                original = path.read_bytes()
                path.write_bytes(original + b"\n")
                with self.subTest(changed=changed), self.assertRaises(ValueError):
                    check_layout.check_suite(root, "pa1", manifest, cases)
                path.write_bytes(original)
            check_layout.check_suite(root, "pa1", manifest, cases)

    def test_committed_check_preludes_are_the_maintainer_harness_prelude(self):
        harness = (REPOSITORY / "dev/tests/harness/native_references.rs").read_text()
        literal = re.search(r'^pub const CHECK: &str = "(.*)";$', harness, re.M)
        self.assertIsNotNone(literal)
        expected = literal[1].encode().decode("unicode_escape").encode()
        for prelude in sorted(REPOSITORY.glob("pa*/tests/prelude/check.rs")):
            with self.subTest(prelude=prelude):
                self.assertEqual(prelude.read_bytes(), expected)


if __name__ == "__main__":
    unittest.main()
