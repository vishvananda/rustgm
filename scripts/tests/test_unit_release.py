"""The component release recipe declares only what was observed and never drops a fixture."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
import unit_release
import unit_tables

REPOSITORY = SCRIPTS.parent


def quiet(function, *arguments, **options):
    with contextlib.redirect_stdout(io.StringIO()) as output:
        function(*arguments, **options)
    return output.getvalue()


class Tree:
    """A minimal repository with one implemented, unpublished unit and its draft tables."""

    def __init__(self, directory, *, config=None, rows=None):
        self.root = Path(directory)
        for folder in ("scripts", "docs", "pa27/tests/spec/places", "pa27/tests/spec/other", "pa27/tests/units"):
            (self.root / folder).mkdir(parents=True)
        self.source("tests/spec/places/accept.rs", "fn main() {}\n")
        self.source("tests/spec/places/reject.rs", "fn main() { let x: u8 = (); }\n")
        self.source("tests/spec/places/entry-free.rs", "pub fn f() {}\n")
        self.source("tests/spec/other/skipped.rs", "fn main() {}\n")
        rows = rows if rows is not None else [
            {"path": "tests/spec/places/accept.rs", "rustc_expected": 0, "native": True},
            {"path": "pa27/tests/spec/places/reject.rs", "rustc": 1, "exit_status": 1},
            {"path": "tests/spec/places/entry-free.rs", "accepted": True},
            {"path": "tests/spec/other/skipped.rs", "accepted": True, "maintainer_origin": "dev/tests/skip.rs:9"},
            {"path": "pa31/tests/spec/later.rs", "accepted": True}]
        (self.root / "pa27/tests/places-outcomes.json").write_text(json.dumps({"cases": rows}))
        unit = {"tables": [{"table": "tests/places-outcomes.json", "feature": "pa27.places"}],
                "entry_free": {"crate_name": "fixture", "disambiguator": "pa27"},
                "skip_origins": ["dev/tests/skip.rs"], **(config or {})}
        (self.root / "scripts/unit_tables.json").write_text(json.dumps({"schema": 1, "units": {"places": unit}}))
        plan = {"schema": 1, "units": [{"id": "places", "owner": "pa27", "implementation": "implemented",
                                        "publication": "unpublished", "depends_on": [],
                                        "manifest": "pa27/tests/units/places.json"}]}
        (self.root / "assignment-plan.json").write_text(json.dumps(plan, indent=2) + "\n")

    def source(self, path, text):
        (self.root / "pa27" / path).write_text(text)

    def probe(self, verdicts, messages=None):
        """Record observations as the probe does, without running a compiler."""
        candidates, _ = unit_tables.inventory(self.root, "places")
        unit_release.write(self.root, "places", "candidates.json", candidates)
        records = []
        for case in candidates["cases"]:
            passed = verdicts.get(case["path"], True)
            message = (messages or {}).get(case["path"], "ERROR: file 0: 3: mismatched types")
            record = {"path": case["path"], "verdict": "pass" if passed else "defer", "stage": "native",
                      "message": None if passed else "backend failed (1): unsupported",
                      "frontend_message": message if case["exit_status"] else ""}
            if passed:
                record["case"] = case
            records.append(record)
        unit_release.write(self.root, "places", "probe.json", {"cases": records})
        return candidates


class DraftTables(unittest.TestCase):
    def test_rows_keep_the_independent_outcome_and_name_recorded_candidate_gaps(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            options = {"pa": "pa27", "unit": "places", "table": "t", "default_feature": "pa27.places"}
            row = {"path": "tests/spec/places/accept.rs", "rustc_expected": 0, "exit_status": 1}
            case, candidate, notes = unit_tables.normalize(tree.root, row, **options)
            self.assertEqual((case["exit_status"], case["rustc"]["status"], candidate), (0, 0, 1))
            self.assertEqual(case["native"]["kind"], "codegen")
            self.assertIn("draft candidate status 1 differs from Rust 0 (recorded gap)", notes)
            self.assertEqual(case["source_sha256"], hashlib.sha256(b"fn main() {}\n").hexdigest())
            for row, status in (({"rustc": {"kind": "program", "status": 1, "reason": "draft"}}, 1),
                                ({"rustc_hypothesis": 1}, 1), ({"accepted": False}, 1),
                                ({"rustc_run_status": 0}, 0), ({"expected_compile_status": 1}, 1)):
                case, _, _ = unit_tables.normalize(tree.root, {"path": "tests/spec/places/accept.rs", **row}, **options)
                self.assertEqual(case["exit_status"], status)
                self.assertNotIn("reason", case["rustc"])
                self.assertEqual("native" in case, status == 0)

    def test_native_rows_execute_and_trapping_rows_declare_a_panic(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            options = {"pa": "pa27", "unit": "places", "table": "t", "default_feature": "f", "native_rows": True}
            passing, _, _ = unit_tables.normalize(tree.root, {"path": "tests/spec/places/accept.rs", "status": 0}, **options)
            trapping, _, _ = unit_tables.normalize(
                tree.root, {"path": "tests/spec/places/accept.rs", "status": 0, "traps": True}, **options)
            self.assertEqual(passing["native"], {"kind": "execute", "outcome": "pass"})
            self.assertEqual(trapping["native"], {"kind": "execute", "outcome": "panic"})

    def test_undeclared_missing_and_conflicting_rows_fail(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            options = {"pa": "pa27", "unit": "places", "table": "t", "default_feature": "f"}
            for row in ({"path": "tests/spec/places/accept.rs"}, {"path": "tests/spec/places/absent.rs", "rustc": 0},
                        {"rustc": 0}, {"path": "tests/spec/places/accept.rs", "rustc": 2}):
                with self.subTest(row=row), self.assertRaises(ValueError):
                    unit_tables.normalize(tree.root, row, **options)
            cases, bindings = {}, {}
            accept, _, _ = unit_tables.normalize(tree.root, {"path": "tests/spec/places/accept.rs", "rustc": 0}, **options)
            reject, _, _ = unit_tables.normalize(tree.root, {"path": "tests/spec/places/accept.rs", "rustc": 1}, **options)
            unit_tables.merge(cases, accept, "a", [], bindings)
            with self.assertRaises(ValueError):
                unit_tables.merge(cases, reject, "b", [], bindings)

    def test_inventory_applies_filters_entry_free_identity_and_reports_every_source(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            tree.source("tests/spec/places/unlisted.rs", "fn main() {}\n")
            document, foreign = unit_tables.inventory(tree.root, "places")
            cases = {case["path"]: case for case in document["cases"]}
            self.assertEqual(sorted(cases), ["tests/spec/places/accept.rs", "tests/spec/places/entry-free.rs",
                                             "tests/spec/places/reject.rs"])
            self.assertEqual(foreign, ["pa31/tests/spec/later.rs"])
            self.assertEqual(document["undeclared_sources"], ["tests/spec/places/unlisted.rs"])
            self.assertEqual(cases["tests/spec/places/accept.rs"]["native"], {"kind": "execute", "outcome": "pass"})
            library = cases["tests/spec/places/entry-free.rs"]
            self.assertEqual((library["rustc"]["kind"], library["library"], library["native"]["kind"]),
                             ("library", {"crate_name": "fixture", "disambiguator": "pa27"}, "object"))
            self.assertNotIn("native", cases["tests/spec/places/reject.rs"])

    def test_a_directory_is_claimed_only_when_every_source_in_it_is_published(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            sources = ["tests/spec/places/accept.rs", "tests/spec/places/reject.rs", "tests/spec/places/entry-free.rs"]
            self.assertEqual(unit_tables.discovery_claims(tree.root, "pa27", sources, sources), ["tests/spec/places"])
            self.assertEqual(unit_tables.discovery_claims(tree.root, "pa27", sources[:2], sources), sources[:2])

    def test_committed_configuration_names_declared_units_and_existing_tables(self):
        config = json.loads((REPOSITORY / "scripts/unit_tables.json").read_text())["units"]
        for unit, entry in config.items():
            owner, loaded = unit_tables.load_config(REPOSITORY, unit)
            self.assertEqual(loaded, entry)
            for table in entry["tables"]:
                with self.subTest(unit=unit, table=table["table"]):
                    self.assertTrue(unit_tables.table_rows(REPOSITORY, owner, table))
        with self.assertRaises(ValueError):
            unit_tables.load_config(REPOSITORY, "no-such-unit")


class Declaration(unittest.TestCase):
    def test_only_observed_passes_are_declared_and_every_other_candidate_is_deferred(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            tree.probe({"tests/spec/places/entry-free.rs": False})
            quiet(unit_release.run_declare, tree.root, "places")
            manifest = json.loads((tree.root / "pa27/tests/units/places.json").read_text())
            self.assertEqual([case["path"] for case in manifest["cases"]],
                             ["tests/spec/places/accept.rs", "tests/spec/places/reject.rs"])
            self.assertEqual(manifest["discovery"], ["tests/spec/places/accept.rs", "tests/spec/places/reject.rs"])
            self.assertEqual(unit_release.read(tree.root, "places", "deferred.json"),
                             [{"path": "tests/spec/places/entry-free.rs", "rust": 0, "stage": "native",
                               "message": "backend failed (1): unsupported"}])

    def test_the_declared_case_is_the_probed_one(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            tree.probe({})
            probe = unit_release.read(tree.root, "places", "probe.json")
            probe["cases"][0]["case"]["warnings"] = ["unused_must_use"]
            unit_release.write(tree.root, "places", "probe.json", probe)
            quiet(unit_release.run_declare, tree.root, "places")
            manifest = json.loads((tree.root / "pa27/tests/units/places.json").read_text())
            self.assertEqual(manifest["cases"][0]["warnings"], ["unused_must_use"])
            self.assertEqual(manifest["discovery"], ["tests/spec/places"])

    def test_gap_worded_rejections_need_a_reviewed_deferral(self):
        gap = {"tests/spec/places/reject.rs": "ERROR: file 0: 9: this range form requires actual source declarations"}
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            tree.probe({}, gap)
            with self.assertRaises(ValueError):
                unit_release.run_declare(tree.root, "places")
            self.assertFalse((tree.root / "pa27/tests/units/places.json").exists())
            quiet(unit_release.run_declare, tree.root, "places", ["tests/spec/places/reject.rs=wrong reason"])
            self.assertEqual(unit_release.read(tree.root, "places", "deferred.json"),
                             [{"path": "tests/spec/places/reject.rs", "rust": 1, "stage": "review",
                               "message": "wrong reason"}])
            self.assertIn("1 (1 undecided)", quiet(unit_release.run_review, tree.root, "places"))
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name, config={"review": {"tests/spec/places/reject.rs": "committed decision"}})
            tree.probe({}, gap)
            quiet(unit_release.run_declare, tree.root, "places")
            self.assertEqual(len(json.loads((tree.root / "pa27/tests/units/places.json").read_text())["cases"]), 2)

    def test_a_stale_or_missing_probe_cannot_declare(self):
        with tempfile.TemporaryDirectory() as name:
            tree = Tree(name)
            with self.assertRaises(ValueError):
                unit_release.run_declare(tree.root, "places")
            tree.probe({})
            probe = unit_release.read(tree.root, "places", "probe.json")
            unit_release.write(tree.root, "places", "probe.json", {"cases": probe["cases"][:-1]})
            with self.assertRaises(ValueError):
                unit_release.run_declare(tree.root, "places")
            for exclusion in ("tests/spec/places/reject.rs", "tests/spec/places/reject.rs= "):
                with self.assertRaises(ValueError):
                    unit_release.decisions(tree.root, "places", [exclusion])


class Wiring(unittest.TestCase):
    ORDER = "| 4 | PA27 | Places (`places`) | implemented | unpublished |\n"
    ROADMAP = "| Unit | PA | Cases | Deferred | Release record |\n| --- | --- | --- | --- | --- |\n" \
              "| `earlier` | 26 | 7 | 1 | [record](docs/earlier.md) |\n\ntext\n"
    README = "| Component | Manifest | Cases | Deferred list |\n| --- | --- | --- | --- |\n" \
             "| `earlier` | [earlier.json](tests/units/earlier.json) | 7 | [1 fixture](../docs/earlier.md#deferred-list) |\n"

    def tree(self, directory):
        tree = Tree(directory)
        tree.probe({})
        quiet(unit_release.run_declare, tree.root, "places")
        (tree.root / "docs/assignment-order.md").write_text(self.ORDER)
        (tree.root / "ROADMAP.md").write_text(self.ROADMAP)
        (tree.root / "pa27/README.md").write_text(self.README)
        (tree.root / "docs/places.md").write_text("record")
        return tree

    def test_release_is_wired_once_and_a_later_count_replaces_its_rows(self):
        with tempfile.TemporaryDirectory() as name:
            tree = self.tree(name)
            quiet(unit_release.run_wire, tree.root, "places", "docs/places.md", 2, "1 needs a provider")
            plan = json.loads((tree.root / "assignment-plan.json").read_text())["units"][0]
            self.assertEqual((plan["publication"], plan["release_evidence"]), ("released", "docs/places.md"))
            self.assertIn("| implemented | released |", (tree.root / "docs/assignment-order.md").read_text())
            roadmap = (tree.root / "ROADMAP.md").read_text()
            self.assertIn("| `earlier` | 26 | 7 | 1 | [record](docs/earlier.md) |\n"
                          "| `places` | 27 | 3 | 2 (1 needs a provider) | [record](docs/places.md) |\n", roadmap)
            self.assertIn("| `places` | [places.json](tests/units/places.json) | 3 | "
                          "[2 fixtures](../docs/places.md#deferred-list) |", (tree.root / "pa27/README.md").read_text())
            quiet(unit_release.run_wire, tree.root, "places", "docs/places.md", 1)
            roadmap = (tree.root / "ROADMAP.md").read_text()
            self.assertEqual(roadmap.count("`places`"), 1)
            self.assertIn("| `places` | 27 | 3 | 1 | [record](docs/places.md) |", roadmap)
            self.assertIn("[1 fixture](../docs/places.md#deferred-list)", (tree.root / "pa27/README.md").read_text())

    def test_wiring_needs_a_record_and_cannot_change_a_released_record(self):
        with tempfile.TemporaryDirectory() as name:
            tree = self.tree(name)
            for record in ("docs/absent.md", "pa27/README.md"):
                with self.assertRaises(ValueError):
                    unit_release.run_wire(tree.root, "places", record, 0)
            quiet(unit_release.run_wire, tree.root, "places", "docs/places.md", 0)
            (tree.root / "docs/other.md").write_text("record")
            with self.assertRaises(ValueError):
                unit_release.run_wire(tree.root, "places", "docs/other.md", 0)

    def test_ledger_edits_only_the_named_publication_string(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            (root / "docs").mkdir()
            text = ('{"rows": [\n {"id": "27.1",\n  "publication": "pending",\n  "units": ["places"]},\n'
                    ' {"id": "27.2",\n  "publication": "pending",\n  "units": ["places"]}\n]}\n')
            (root / "docs/assignment-requirements.json").write_text(text)
            unit_release.run_ledger(root, ['27.2=released in "places"'])
            rows = json.loads((root / "docs/assignment-requirements.json").read_text())["rows"]
            self.assertEqual([row["publication"] for row in rows], ["pending", 'released in "places"'])
            for item in ("27.9=x", "27.1"):
                with self.assertRaises(ValueError):
                    unit_release.run_ledger(root, [item])


if __name__ == "__main__":
    unittest.main()
