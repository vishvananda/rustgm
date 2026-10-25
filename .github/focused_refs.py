"""Read-only text qualification; new references require a separate reviewed step."""
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

SOURCE = "6824f6cf5a9a457093242cd41fc2e37c3ec88888"
EXPECTED = "321dec585789ce1b64c905ab233343c19ab52805f02d7779dfa9dad78a6721aa"
SELECTED = {'pa5': {'tests/spec/items/default-function.rs': 0, 'tests/spec/items/default-implementation.rs': 0, 'tests/spec/items/default-associated-constant.rs': 0}, 'pa19': {'tests/spec/default-methods/trait-blanket.rs': 0, 'tests/spec/default-methods/inherent.rs': 0, 'tests/spec/default-methods/unused-body-type.rs': 1}}
OUT = ROOT / "obj/parallel/all-attribute-selected-output-v1"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def worker(cli=None):
    started = time.monotonic()
    deadline = started + 185
    OUT.mkdir(parents=True, exist_ok=False)
    cli = Path(cli) if cli is not None else ROOT / "obj/rust/dev-fast/rustgm"
    assert sha(cli) == EXPECTED, "compiled CLI differs from qualified bytes"
    before = fingerprint(ROOT)
    stages = {f"pa{number}": course.load_cases(ROOT, f"pa{number}")
              for number in range(1, 25)}
    inputs = {}
    for stage, (_, cases) in stages.items():
        for case in cases:
            for path in [case["path"], *case.get("auxiliary", [])]:
                inputs[f"{stage}/{path}"] = sha(ROOT / stage / path)
    record = {
        "state": "running", "source_commit": SOURCE,
        "ci_commit": os.environ.get("GITHUB_SHA"),
        "candidate_sha256": EXPECTED, "sources": before,
        "compiler_source_sha256": {path: digest for path, digest in before.items()
                                   if path.startswith("dev/") and path.endswith(".rs")},
        "selected": SELECTED, "reference_writes": False,
        "input_sha256": inputs,
        "old_passes": 0, "new_review_cases": 0,
    }
    try:
        for number in range(1, 25):
            assert time.monotonic() < deadline, "text qualification deadline"
            stage = f"pa{number}"
            manifest, cases = stages[stage]
            with (OUT / f"{stage}.log").open("w") as stream, contextlib.redirect_stdout(stream):
                passed = course.run_stage(ROOT, stage, [str(cli)], jobs=2, timeout=10,
                                          evidence_root=OUT / "text")
            folders = sorted((OUT / "text" / stage).iterdir())
            assert len(folders) == 1
            results = json.loads((folders[0] / "results.json").read_text())
            expected = SELECTED.get(stage, {})
            assert passed == (not expected)
            assert len(results["cases"]) == len(cases)
            for case, result in zip(cases, results["cases"]):
                assert case["path"] == result["path"]
                if case["path"] not in expected:
                    assert result["passed"], (stage, result)
                    record["old_passes"] += 1
                    continue
                assert case["exit_status"] == expected[case["path"]]
                assert len(result["errors"]) == 3
                assert all(error.startswith("missing oracle:") for error in result["errors"])
                actual = folders[0] / case["path"]
                assert Path(str(actual) + ".exit_status").read_text() == f"{expected[case['path']]}\n"
                destination = OUT / "review" / stage / case["path"]
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / stage / case["path"], destination)
                for suffix in (".stdout", ".stderr", ".exit_status"):
                    shutil.copyfile(Path(str(actual) + suffix), Path(str(destination) + suffix))
                record["new_review_cases"] += 1
        assert record["old_passes"] == 8493
        assert record["new_review_cases"] == 6
        assert fingerprint(ROOT) == before
        assert all(sha(ROOT / path) == digest for path, digest in inputs.items())
        assert sha(cli) == EXPECTED
        record.update(state="passed; six new text oracles await review",
                      sources_unchanged=True)
    except Exception as error:
        record.update(state="failed", error=str(error))
        raise
    finally:
        record["seconds"] = time.monotonic() - started
        (OUT / "binding.json").write_text(json.dumps(record, indent=2) + "\n")
        print(json.dumps({key: value for key, value in record.items()
                          if key not in ("sources", "compiler_source_sha256", "input_sha256")}, indent=2))


if __name__ == "__main__":
    if "--worker" in sys.argv:
        worker()
    else:
        raise SystemExit(subprocess.run([
            "timeout", "--signal=TERM", "--kill-after=3s", "190s", sys.executable,
            "scripts/storage.py", "run", "--no-prune", "--", sys.executable,
            __file__, "--worker"], check=False).returncode)
