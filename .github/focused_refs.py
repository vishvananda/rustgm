"""One explicit reviewed six-case generation, using the official scoped writer."""
import contextlib
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path.cwd()
sys.path.insert(0, str(ROOT / "scripts"))
import course
from fastcheck_run import fingerprint

OUT = ROOT / "obj/parallel/all-attribute-selected-output-v1"
SUFFIXES = (".stdout", ".stderr", ".exit_status")


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def worker():
    started = time.monotonic()
    OUT.mkdir(parents=True, exist_ok=False)
    decision_path = ROOT / ".github/review-decision.json"
    decision = json.loads(decision_path.read_text())
    before = fingerprint(ROOT)
    assert before == decision["source_fingerprints"]
    cli = ROOT / "obj/rust/dev-fast/rustgm"
    assert sha(cli) == decision["candidate_sha256"]
    assert decision["state"] == "integrator reviewed and approved six exact scoped outputs"
    assert len(decision["outputs"]) == 6
    assert all(sha(ROOT / name) == digest for name, digest in decision["inputs"].items())
    originals = {str(path.relative_to(ROOT)): sha(path)
                 for stage in decision["selected"]
                 for path in (ROOT / stage).rglob("*")
                 if path.is_file() and (path.name.endswith(course.SIDECARS)
                                       or path.name == "reference.json"
                                       or "reference-batches" in path.parts)}
    record = {"state": "running", "source_commit": decision["source_commit"],
              "ci_commit": os.environ.get("GITHUB_SHA"), "sources": before,
              "candidate_sha256": sha(cli), "review_decision_sha256": sha(decision_path),
              "selected": decision["selected"], "generated": {},
              "retained_old_text_cases": decision["old_cases_verified"]}
    try:
        for stage, paths in decision["selected"].items():
            assert time.monotonic() - started < 175
            assert sha(ROOT / stage / "reference.json") == decision["base_references"][stage]
            batches = set((ROOT / stage / "reference-batches").glob("*.json"))
            with (OUT / f"{stage}-generation.log").open("w") as stream, contextlib.redirect_stdout(stream):
                assert course.run_stage(ROOT, stage, [str(cli)], bless=True, jobs=2,
                                        timeout=10, evidence_root=OUT / "generated",
                                        case_paths=list(paths))
            for name in paths:
                for sidecar, raw in zip(course.SIDECARS, SUFFIXES):
                    source = Path(str(ROOT / stage / name) + sidecar)
                    assert sha(source) == decision["outputs"][f"{stage}/{name}"][raw]
                    relative = str(source.relative_to(ROOT))
                    target = OUT / "publication" / relative
                    target.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(source, target)
                    record["generated"][relative] = sha(source)
            new_batches = set((ROOT / stage / "reference-batches").glob("*.json")) - batches
            assert len(new_batches) == 1
            for source in new_batches:
                batch = json.loads(source.read_text())
                assert set(batch["selected_paths"]) == set(paths)
                relative = str(source.relative_to(ROOT))
                target = OUT / "publication" / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source, target)
                record["generated"][relative] = sha(source)
        assert len(record["generated"]) == 20
        assert all(sha(ROOT / path) == digest for path, digest in originals.items())
        assert fingerprint(ROOT) == before
        assert sha(cli) == decision["candidate_sha256"]
        record.update(state="passed", reviewed_cases_generated=6,
                      existing_references_unchanged=len(originals), sources_unchanged=True)
    except Exception as error:
        record.update(state="failed", error=str(error))
        raise
    finally:
        record["seconds"] = time.monotonic() - started
        (OUT / "binding.json").write_text(json.dumps(record, indent=2) + "\n")
        shutil.copyfile(decision_path, OUT / "review-decision.json")
        print(json.dumps({key: value for key, value in record.items() if key != "sources"}, indent=2))


if __name__ == "__main__":
    if "--worker" in sys.argv:
        worker()
    else:
        raise SystemExit(subprocess.run([
            "timeout", "--signal=TERM", "--kill-after=3s", "190s", sys.executable,
            "scripts/storage.py", "run", "--no-prune", "--", sys.executable,
            __file__, "--worker"], check=False).returncode)
