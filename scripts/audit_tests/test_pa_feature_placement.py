"""Placement evidence is scoped to a case's actual mode, inputs and ownership."""

from hashlib import sha256
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from audit_pa_feature_placement import audit, fixture_inventory, source_markers
from course import SIDECARS
from reference_batches import case_digest


class Placement(unittest.TestCase):
    def setUp(self):
        folder = tempfile.TemporaryDirectory()
        self.addCleanup(folder.cleanup)
        self.root = Path(folder.name)
        self.files = set()
        self.stages = []

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        self.files.add(path.relative_to(self.root))

    def suite(self, number, mode, cases):
        pa = f"pa{number}"
        self.stages.append({"path": pa, "number": number, "status": "implemented", "mode": mode})
        entries = [{"path": f"tests/{name}.rs", "exit_status": 0, **extra}
                   for name, extra in cases.items()]
        self.write(f"{pa}/tests/manifest.json", json.dumps({"mode": mode, "cases": entries}))

    def report(self, units=None):
        pas = {stage["path"] for stage in self.stages}
        return audit(self.root, self.stages, units or {}, [], self.files, pas)

    def oracles(self, primary, values):
        for suffix, value in zip(SIDECARS, values, strict=True):
            self.write(f"{primary}{suffix}", value)

    def test_source_markers_require_actual_syntax(self):
        markers = source_markers(b'''
// union Comment { field: u32 }
const TEXT: &str = r#"async fn literal() { make().await }"#;
macro_rules! make { () => { union Template { field: u32 } }; }
fn ordinary() { let f = |x| x; }
async fn later() {}
''')
        self.assertEqual({rule.feature: line for rule, line in markers.items()}, {
            "macro matching and expansion": 4, "closure lowering": 5, "async lowering": 6,
        })

    def test_mode_companion_and_outcome_control_source_findings(self):
        self.suite(5, "--emit-crate-ast", {"syntax": {"features": ["pa5.items"]}})
        self.suite(11, "--emit-lowir", {
            "client": {"input": "file", "auxiliary": ["tests/provider.rs"]},
            "rejected": {"exit_status": 1},
            "atomic": {},
        })
        self.suite(28, "--emit-lowir", {})
        self.write("pa5/tests/syntax.rs", "union U { field: u32 }")
        self.write("pa11/tests/client.rs", "mod provider; fn main() {}")
        self.write("pa11/tests/provider.rs", "union U { field: u32 }")
        self.write("pa11/tests/rejected.rs", "union U { field: u32 }")
        self.write("pa11/tests/atomic.rs", "fn main() {}")
        self.write("pa11/tests/atomic.rs.ref", "    %t0 = atomic_load u32 %p0, 0\n")
        self.write("pa28/tests/spec/pending.rs", "async fn later() {}")
        report = self.report()
        cases = {Path(row["path"]).stem: row for row in report["cases"]}
        self.assertEqual(cases["syntax"]["status"], "ok")
        self.assertEqual(cases["rejected"]["status"], "ok")
        self.assertEqual(cases["client"]["status"], "review")
        self.assertEqual(cases["client"]["early_evidence"][0]["path"], "pa11/tests/provider.rs")
        self.assertEqual(cases["atomic"]["status"], "early")
        self.assertEqual(cases["atomic"]["early_evidence"][0]["line"], 1)
        self.assertEqual(report["outside_course_manifests"], ["pa28/tests/spec/pending.rs"])
        self.assertEqual(report["outside_course_manifest_review"][0]["feature"], "async lowering")

    def test_declared_units_admit_reordered_prerequisites(self):
        self.suite(11, "--emit-lowir", {})
        self.suite(25, "--emit-lowir", {
            "approved": {"features": ["pa30.foreign"], "work_units": ["const-machine"]},
            "early": {"features": ["pa30.foreign"]},
        })
        self.suite(30, "--emit-lowir", {})
        for name in ("approved", "early"):
            self.write(f"pa25/tests/{name}.rs", "fn main() {}")
        units = {
            "const-machine": {"id": "const-machine", "owner": "pa25", "depends_on": ["foreign"]},
            "foreign": {"id": "foreign", "owner": "pa30", "depends_on": []},
        }
        self.assertEqual([row["status"] for row in self.report(units)["cases"]], ["ok", "early"])

    def test_review_decisions_require_current_source_and_contract(self):
        self.suite(11, "--emit-lowir", {"early": {}})
        source = "pa11/tests/early.rs"
        draft = "pa11/tests/spec/draft.rs"
        contract = "pa11/tests/spec/contract.json"
        text = "union U { value: u32 }"
        self.write(source, text)
        self.write(draft, text)
        self.write(contract, "{}\n")
        self.write("docs/assignment-placement-reviews.json", json.dumps({"schema": 1, "reviews": [
            {"case": source, "case_sha256": case_digest({"path": "tests/early.rs", "exit_status": 0},
                                                        "--emit-lowir"),
             "path": source, "feature": "union semantics", "reason": "Structural eligibility only.",
             "input_sha256": sha256(text.encode()).hexdigest()},
            {"contract": contract, "contract_sha256": sha256(b"{}\n").hexdigest(),
             "path": draft, "feature": "union semantics", "reason": "Declaration-only draft.",
             "input_sha256": sha256(text.encode()).hexdigest()},
        ]}))
        reviewed = self.report()
        self.assertEqual(reviewed["cases"][0]["status"], "reviewed")
        self.assertEqual(reviewed["cases"][0]["reviewed_evidence"][0]["reason"],
                         "Structural eligibility only.")
        self.assertEqual(len(reviewed["outside_course_manifest_reviewed"]), 1)
        self.write("pa11/tests/early.rs.ref", "    atomic_store u32 %p0, %p1, 0\n")
        self.assertEqual(self.report()["cases"][0]["status"], "early")
        self.files.remove(Path("pa11/tests/early.rs.ref"))
        self.write(source, text + "\n")
        self.write(contract, '{"changed": true}\n')
        reopened = self.report()
        self.assertEqual(reopened["cases"][0]["status"], "review")
        self.assertEqual(len(reopened["outside_course_manifest_review"]), 1)

    def test_equal_stdin_cases_require_the_same_mode_settings_and_oracles(self):
        self.suite(11, "--emit-lowir", {"first": {"features": ["pa11.lowir"]}})
        self.suite(12, "--emit-lowir", {
            "same": {"features": ["pa11.lowir"]},
            "different-oracle": {},
            "different-args": {"args": ["--cfg", "test"]},
            "file-input": {"input": "file"},
        })
        for name in ("pa11/tests/first", "pa12/tests/same", "pa12/tests/different-oracle",
                     "pa12/tests/different-args", "pa12/tests/file-input"):
            self.write(f"{name}.rs", "fn main() {}")
            stderr = "diagnostic difference" if name.endswith("oracle") else ""
            self.oracles(f"{name}.rs", ("program\n", stderr, "0\n"))
        report = self.report()
        self.assertEqual(report["duplicate_contracts"], [{
            "mode": "--emit-lowir", "paths": ["pa11/tests/first.rs", "pa12/tests/same.rs"],
        }])
        cases = {Path(row["path"]).stem: row for row in report["cases"]}
        self.assertEqual(cases["same"]["suggested_pa"], "pa11")

    def test_git_inventory_includes_new_sources_and_excludes_ignored_scratch(self):
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        self.write(".gitignore", "*.scratch\n")
        self.write("pa1/tests/committed.rs", "")
        subprocess.run(["git", "add", "pa1/tests/committed.rs"], cwd=self.root, check=True)
        self.write("pa1/tests/new.rs", "")
        self.write("pa1/tests/ignored.scratch", "")
        self.write("dev/tests/internal.rs", "")
        self.assertEqual(fixture_inventory(self.root), {
            Path("pa1/tests/committed.rs"), Path("pa1/tests/new.rs"),
        })


if __name__ == "__main__":
    unittest.main()
