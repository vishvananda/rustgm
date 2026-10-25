"""Observe the original pinned source using the qualified defaultness compiler."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path.cwd()
EXPECTED = "321dec585789ce1b64c905ab233343c19ab52805f02d7779dfa9dad78a6721aa"

if "--worker" not in sys.argv:
    raise SystemExit(subprocess.run([
        "timeout", "--signal=TERM", "--kill-after=3s", "190s", sys.executable,
        "scripts/storage.py", "run", "--no-prune", "--", sys.executable,
        __file__, "--worker"], check=False).returncode)

sys.path.insert(0, str(ROOT / "scripts"))
from fastcheck_run import fingerprint

out = ROOT / "obj/parallel/all-attribute-selected-output-v1"
out.mkdir(parents=True, exist_ok=False)
freeze = json.loads((ROOT / ".github/observer_freeze.json").read_text())
assert fingerprint(ROOT) == freeze["source_fingerprints"]
app = ROOT / "obj/rust/dev-fast/rustgm"
assert hashlib.sha256(app.read_bytes()).hexdigest() == EXPECTED
record = {
    "source_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
    "product_source": "16c0476bd9c535ac9e2e312cc3671a77765115aa",
    "candidate_sha256": EXPECTED, "profile": "dev-fast",
    "qualified_changed_two_run": 40236064195,
    "qualified_retained_four_run": 40235524814,
    "text_reference_publication_run": 40238779679,
    "compiler_source_sha256": {
        path: digest for path, digest in freeze["source_fingerprints"].items()
        if path.startswith("dev/") and path.endswith(".rs")},
}
(out / "binding.json").write_text(json.dumps(record, indent=2) + "\n")
subprocess.run([
    sys.executable, "scripts/observe_original_core.py", "--worker",
    "--expected-cli-sha256", EXPECTED, "--cli", str(app),
    "--producer-binding", str(out / "binding.json"),
    "--out", str(out / "whole-core-defaultness-v1")], check=True)
assert fingerprint(ROOT) == freeze["source_fingerprints"]
