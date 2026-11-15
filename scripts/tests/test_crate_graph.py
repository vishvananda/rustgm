"""A crate-graph case builds its declared providers, then its consumer against their artifacts."""
import json
from pathlib import Path
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
from case_inputs import support_files
from check_native import execution
import crate_graph
from course import load_cases, run_case
import unit_graphs

# A stand-in compiler: publishes the requested metadata artifact and echoes its arguments.
COMPILER = '''import json, sys
arguments = sys.argv[1:]
if "--fail" in open(arguments[-1]).read():
    sys.stderr.write("ERROR: rejected provider\\n"); sys.exit(1)
if "--metadata-output" in arguments:
    open(arguments[arguments.index("--metadata-output") + 1], "w").write("artifact")
print(json.dumps(arguments))
'''
FIRST = {"name": "first", "path": "tests/first.rs", "crate_name": "shared", "disambiguator": "first", "externs": {}}
SECOND = {"name": "second", "path": "tests/second.rs", "crate_name": "shared", "disambiguator": "second",
          "externs": {"one": "first"}}


class CrateGraphs(unittest.TestCase):
    def stage(self, directory, case):
        root = Path(directory)
        tests = root / "pa1/tests"
        tests.mkdir(parents=True)
        (root / "dev").mkdir()
        for name in ("first", "second", "client"):
            (tests / f"{name}.rs").write_text(f"// {name}\n")
        (root / "compiler.py").write_text(COMPILER)
        self.write(root, case)
        return root, [sys.executable, str(root / "compiler.py")]

    def write(self, root, case):
        (root / "pa1/tests/manifest.json").write_text(json.dumps({"mode": "--emit-lowir", "cases": [case]}))

    def case(self, **changes):
        return {"path": "tests/client.rs", "exit_status": 0, "input": "file", "crates": [FIRST, SECOND],
                "externs": {"renamed": "second"}} | changes

    def test_providers_build_in_order_and_the_consumer_binds_only_its_direct_aliases(self):
        with tempfile.TemporaryDirectory() as name:
            case = self.case()
            root, app = self.stage(name, case)
            _, cases = load_cases(root, "pa1")
            self.assertEqual(support_files(cases[0]), ["tests/first.rs", "tests/second.rs"])
            graph = crate_graph.candidate_graph(root, "pa1", case, app, root / "out", 5)
            first, second = (json.loads(unit.read_text()) for unit in graph.units)
            identity = ["--crate-type=lib", "--crate-name", "shared", "--crate-disambiguator"]
            self.assertEqual(first, ["--emit-lowir", "--edition=2024", *identity, "first",
                                     "--metadata-output", str(root / "out/first.rgm"), "pa1/tests/first.rs"])
            self.assertEqual(second, ["--emit-lowir", "--edition=2024", *identity, "second",
                                      "--extern", f"one={root / 'out/first.rgm'}",
                                      "--metadata-output", str(root / "out/second.rgm"), "pa1/tests/second.rs"])
            self.assertEqual(graph.arguments, ["--extern", f"renamed={root / 'out/second.rgm'}",
                                               "--dependency-metadata", str(root / "out/first.rgm")])
            _, actual = run_case(root, "pa1", "--emit-lowir", case, app, 5, graph=graph)
            self.assertEqual(json.loads(actual[0]), ["--emit-lowir", "--edition=2024", *graph.arguments,
                                                     "pa1/tests/client.rs"])
            _, standalone = run_case(root, "pa1", "--emit-lowir", case, app, 5)
            arguments = json.loads(standalone[0])
            self.assertEqual([arguments[2], arguments[4], arguments[-1]],
                             ["--extern", "--dependency-metadata", "pa1/tests/client.rs"])
            self.assertFalse(Path(arguments[5]).exists())

    def test_a_provider_that_does_not_build_is_a_harness_error_not_an_outcome(self):
        with tempfile.TemporaryDirectory() as name:
            case = self.case()
            root, app = self.stage(name, case)
            (root / "pa1/tests/first.rs").write_text("// --fail\n")
            with self.assertRaises(ValueError):
                run_case(root, "pa1", "--emit-lowir", case, app, 5)

    def test_malformed_graphs_reject_before_any_run(self):
        with tempfile.TemporaryDirectory() as name:
            root, _ = self.stage(name, self.case())
            for changes in ({"crates": []}, {"crates": [SECOND, FIRST]}, {"crates": [FIRST, FIRST]},
                            {"crates": [FIRST | {"name": "First"}]}, {"crates": [FIRST | {"path": "tests/absent.rs"}]},
                            {"crates": [FIRST | {"crate_name": "_"}]}, {"crates": [FIRST | {"args": []}]},
                            {"crates": [{key: value for key, value in FIRST.items() if key != "externs"}]},
                            {"externs": {"renamed": "absent"}}, {"externs": {"not an alias": "second"}},
                            {"externs": {"one": "first"}}, {"externs": None}, {"mode": "--emit-types"}):
                self.write(root, self.case(**changes))
                with self.subTest(changes=changes), self.assertRaises(ValueError):
                    load_cases(root, "pa1")
            self.write(root, {"path": "tests/client.rs", "exit_status": 0, "externs": {"one": "first"}})
            with self.assertRaises(ValueError):
                load_cases(root, "pa1")

    def test_a_host_linked_candidate_aborts_through_the_host_library(self):
        aborted = type("Result", (), {"stdout": b"", "stderr": b"", "returncode": -6})()
        exited = type("Result", (), {"stdout": b"", "stderr": b"", "returncode": 134})()
        execution(aborted, "panic", "cppgm-linked")
        execution(exited, "panic", "cppgm")
        for result, compiler in ((exited, "cppgm-linked"), (aborted, "cppgm")):
            with self.subTest(compiler=compiler), self.assertRaises(ValueError):
                execution(result, "panic", compiler)


class GraphTables(unittest.TestCase):
    def test_named_rows_declare_the_providers_their_dependencies_reach(self):
        rows = [{"name": "first", "path": "a.rs", "kind": "rlib", "crate": "shared", "deps": {}, "rustc": 0,
                 "frontend_expected_after_implementation": 0, "native": False},
                {"name": "second-part", "path": "b.rs", "kind": "rlib", "crate": "shared", "deps": {"one": "first"},
                 "rustc": 0, "frontend_expected_after_implementation": 0, "native": False},
                {"name": "client", "path": "c.rs", "kind": "bin", "crate": "client", "deps": {"two": "second-part"},
                 "rustc": 1, "frontend_expected_after_implementation": 1, "native": False},
                {"name": "again", "path": "c.rs", "kind": "bin", "crate": "client", "deps": {}, "rustc": 0,
                 "frontend_expected_after_implementation": 0, "native": True}]
        entry = {"table": "t", "graph": "named", "skip_rows": {"again": "one source has one graph"}}
        cases, skipped = unit_graphs.graph_rows({"cases": rows}, entry)
        self.assertEqual(skipped, {"again": "one source has one graph"})
        self.assertNotIn("crates", cases[0])
        self.assertEqual(cases[1]["library"], {"crate_name": "shared", "disambiguator": "second_part"})
        self.assertEqual(cases[2]["rustc"], {"kind": "program", "status": 1})
        self.assertEqual([provider["name"] for provider in cases[2]["crates"]], ["first", "second-part"])
        self.assertEqual(cases[2]["crates"][1]["externs"], {"one": "first"})
        self.assertEqual(cases[2]["externs"], {"two": "second-part"})
        for changed in ({"skip_rows": {"absent": "typo"}}, {"skip_rows": {}}):
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                unit_graphs.graph_rows({"cases": rows[:3] + [rows[3] | {"name": "client"}]}, entry | changed)
        rejected = [rows[0] | {"rustc": 1}, rows[1]]
        with self.assertRaises(ValueError):
            unit_graphs.graph_rows({"cases": rejected}, {"table": "t", "graph": "named"})

    def test_library_and_crate_tables_share_the_same_row_form(self):
        table = {"crate": "arrays", "cases": [
            {"path": "tests/spec/a/library.rs", "role": "library", "rustc": 0, "native": False,
             "frontend_expected_after_implementation": 0},
            {"path": "tests/spec/a/pass.rs", "role": "client", "rustc": 0, "native": True,
             "frontend_expected_after_implementation": 0}]}
        cases, _ = unit_graphs.graph_rows(table, {"table": "t", "graph": "library", "disambiguator": "v1"})
        self.assertEqual(cases[0]["library"], {"crate_name": "arrays", "disambiguator": "v1"})
        self.assertEqual((cases[1]["externs"], cases[1]["crates"][0]["name"], cases[1]["native"]),
                         ({"arrays": "library"}, "library", True))
        table = {"crates": [
            {"source": "spec/s/first.rs", "kind": "lib", "name": "shared", "disambiguator": "first", "externs": {},
             "rustc": "accept"},
            {"source": "spec/s/client.rs", "kind": "bin", "externs": {"one": "first"}, "rustc": "accept",
             "native_exit": 0}]}
        cases, _ = unit_graphs.graph_rows(table, {"table": "t", "graph": "crates"})
        self.assertEqual((cases[0]["path"], cases[0]["library"]),
                         ("tests/spec/s/first.rs", {"crate_name": "shared", "disambiguator": "first"}))
        self.assertEqual((cases[1]["native"], cases[1]["crates"][0]["path"]), (True, "tests/spec/s/first.rs"))


if __name__ == "__main__":
    unittest.main()
