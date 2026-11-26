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

    def test_source_core_client_and_forge_mutations_work_and_reject_invalid_specs(self):
        with tempfile.TemporaryDirectory() as name:
            provider = FIRST | {"source_core": True, "source_profile": "provided-contracts-v1", "rustc_oracle": True}
            case = {"path": "tests/client.rs", "exit_status": 0, "input": "file",
                    "crates": [provider], "externs": {"base": "first"}, "source_client": "first"}
            root, app = self.stage(name, case)
            (root / "pa1/tests/second.rs").unlink()
            load_cases(root, "pa1")
            graph = crate_graph.candidate_graph(root, "pa1", case, app, root / "out", 5)
            built = json.loads(graph.units[0].read_text())
            self.assertEqual(built[:8], ["--emit-lowir", "--edition=2024", "--source-core",
                                         "shared@first", "--crate-type=lib", "--crate-name",
                                         "shared", "--crate-disambiguator"])
            self.assertIn("--source-profile", built)
            self.assertNotIn("rustc_oracle", built)
            self.assertEqual(graph.arguments[:2], ["--source-client", "shared@first"])

            mismatch = case | {"exit_status": 1, "source_client_mismatch": True}
            self.write(root, mismatch)
            load_cases(root, "pa1")
            mismatch_graph = crate_graph.candidate_graph(root, "pa1", mismatch, app, root / "out-mismatch", 5)
            self.assertEqual(mismatch_graph.arguments[:2], ["--source-client", "shared@first_mismatch"])

            packet = (b"RGMETA\r\n\x00\x00\x00\x00\x0a\x01\x00\x00"
                      + b"\xff" + (4).to_bytes(4, "little") + b"next" + (0).to_bytes(4, "little") + (11).to_bytes(4, "little")
                      + b"\xff" + (9).to_bytes(4, "little") + b"into_iter" + (0).to_bytes(4, "little") + (22).to_bytes(4, "little")
                      + b"\xfe" + (7).to_bytes(4, "little") + b"NonNull" + (0).to_bytes(4, "little") + (33).to_bytes(4, "little"))
            raw = root / "raw.rgm"
            raw.write_bytes(packet)
            corrupt = crate_graph.apply_forge(raw, {"provider": "first", "kind": "corrupt_version"}, root / "c.rgm").read_bytes()
            self.assertEqual(corrupt[13:15], b"\xff\xff")
            swapped = crate_graph.apply_forge(raw, {"provider": "first", "kind": "swap_roles", "tag": 255, "first": "next", "second": "into_iter"}, root / "s.rgm").read_bytes()
            self.assertIn(b"next" + (0).to_bytes(4, "little") + (22).to_bytes(4, "little"), swapped)
            self.assertIn(b"into_iter" + (0).to_bytes(4, "little") + (11).to_bytes(4, "little"), swapped)
            foreign = crate_graph.apply_forge(raw, {"provider": "first", "kind": "foreign_role", "tag": 255, "spelling": "next"}, root / "f.rgm").read_bytes()
            self.assertIn(b"next" + (1).to_bytes(4, "little") + (11).to_bytes(4, "little"), foreign)
            redirect = crate_graph.apply_forge(raw, {"provider": "first", "kind": "redirect_role", "tag": 254, "target": "NonNull", "source_tag": 255, "source": "next"}, root / "r.rgm").read_bytes()
            self.assertIn(b"NonNull" + (0).to_bytes(4, "little") + (11).to_bytes(4, "little"), redirect)

            for bad in (case | {"crates": [FIRST | {"source_profile": "module-bodies-v13"}]},
                        case | {"crates": [FIRST | {"source_core": True, "source_profile": "unknown"}]},
                        case | {"crates": [FIRST | {"rustc_oracle": True}]},
                        case | {"crates": [FIRST | {"source_core": True, "rustc_oracle": "yes"}]},
                        case | {"crates": [FIRST], "source_client": "first"},
                        case | {"source_client_mismatch": True},
                        case | {"forge": {"provider": "first", "kind": "corrupt_version"}},
                        case | {"exit_status": 1, "forge": {"provider": "absent", "kind": "corrupt_version"}},
                        case | {"exit_status": 1, "forge": {"provider": "first", "kind": "swap_roles", "tag": 255, "first": "a", "second": "a"}}):
                self.write(root, bad)
                with self.subTest(bad=bad), self.assertRaises(ValueError):
                    load_cases(root, "pa1")

    def test_provider_editions_features_cfgs_environment_and_auxiliary(self):
        with tempfile.TemporaryDirectory() as name:
            provider = FIRST | {
                "edition": "2018",
                "cap_lints": "allow",
                "features": ["extra"],
                "cfgs": ["ready"],
                "environment": {"OUT_DIR": "out", "LIB_TAG": "v1"},
                "auxiliary": ["tests/out/gen.rs"],
            }
            case = {"path": "tests/client.rs", "exit_status": 0, "input": "file", "edition": "2021",
                    "crates": [provider], "externs": {"renamed": "first"}}
            root, app = self.stage(name, case)
            (root / "pa1/tests/second.rs").unlink()
            (root / "pa1/tests/out").mkdir()
            (root / "pa1/tests/out/gen.rs").write_text("pub const N: u8 = 7;\n")
            _, cases = load_cases(root, "pa1")
            self.assertEqual(support_files(cases[0]), ["tests/first.rs", "tests/out/gen.rs"])
            graph = crate_graph.candidate_graph(root, "pa1", case, app, root / "out", 5)
            built = json.loads(graph.units[0].read_text())
            self.assertEqual(built[:7], ["--emit-lowir", "--edition=2018", "--cap-lints=allow",
                                         "--cfg", 'feature="extra"', "--cfg", "ready"])
            _, actual = run_case(root, "pa1", "--emit-lowir", case, app, 5, graph=graph)
            self.assertEqual(json.loads(actual[0])[:2], ["--emit-lowir", "--edition=2021"])
            for bad in (provider | {"edition": "2020"},
                        provider | {"cap_lints": "warn"},
                        provider | {"features": []},
                        provider | {"features": ["dup", "dup"]},
                        provider | {"cfgs": [""]},
                        provider | {"environment": {"BAD=KEY": "1"}},
                        provider | {"auxiliary": ["tests/first.rs"]},
                        provider | {"auxiliary": ["tests/client.rs"]}):
                self.write(root, case | {"crates": [bad]})
                with self.subTest(bad=bad), self.assertRaises(ValueError):
                    load_cases(root, "pa1")

    def test_host_driver_executes_library_cases_and_rejects_invalid_combinations(self):
        from check_native import classification
        with tempfile.TemporaryDirectory() as name:
            case = {"path": "tests/client.rs", "exit_status": 0, "input": "file",
                    "rustc": {"kind": "library", "status": 0, "emit": "obj"},
                    "library": {"crate_name": "client", "disambiguator": "drv"},
                    "native": {"kind": "execute", "outcome": "pass", "driver": "tests/driver.c"}}
            root, _ = self.stage(name, case)
            (root / "pa1/tests/first.rs").unlink()
            (root / "pa1/tests/second.rs").unlink()
            (root / "pa1/tests/driver.c").write_text("int main(void){return 0;}\n")
            _, cases = load_cases(root, "pa1")
            self.assertEqual(support_files(cases[0]), ["tests/driver.c"])
            classification(cases[0])
            for bad in (case | {"rustc": {"kind": "program", "status": 0}, "library": None},
                        case | {"native": {"kind": "object", "reason": "x", "driver": "tests/driver.c"}},
                        case | {"native": {"kind": "execute", "outcome": "pass", "driver": "tests/driver.c", "providers": ["tests/driver.c"]}}):
                with self.subTest(bad=bad), self.assertRaises(ValueError):
                    classification(bad)

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
