"""Bounded combined qualification; one Rust command at a time, concurrent audit."""
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
from types import SimpleNamespace

ROOT = Path.cwd()
sys.path.insert(0, str(ROOT / "scripts"))
from fastcheck_run import Steps, fingerprint
from maintainer_batch import artifacts, inventory


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def worker(started):
    freeze = json.loads((ROOT / ".github/combined_freeze.json").read_text())
    before = fingerprint(ROOT)
    assert before == freeze["sources"]
    prior_path = ROOT / ".github/combined_prior_receipt.json"
    assert sha(prior_path) == freeze["retained_metadata_sha256"]
    prior = json.loads(prior_path.read_text())
    assert prior["state"] == "failed"
    assert next(step for step in prior["steps"] if step["name"] == "metadata")["status"] == 0
    changed = {path for path in set(before) | set(prior["sources"])
               if before.get(path) != prior["sources"].get(path)}
    assert changed == {"dev/src/sema/crates/typed_headers/assumed_originals.rs"}
    fixtures = freeze["fixtures"]
    assert all(sha(ROOT / path) == digest for path, digest in fixtures.items())
    out = ROOT / "obj/qualification/fast" / f"{time.time_ns()}-combined"
    deadline = started + 185
    args = SimpleNamespace(evidence_root=out, deadline=deadline, prune_under=[])
    steps = Steps(ROOT, args)
    selected = freeze["selected"]
    cli = ROOT / "obj/rust/dev-fast/rustgm"
    cli_sha = sha(cli)
    steps.receipt.update(sources=before, source_commit=freeze["source_commit"],
                         groups={"lib": selected}, compiler_sha256=cli_sha,
                         retained_component_receipts=freeze["component_receipts"],
                         retained_metadata={"run": 40228080113, "receipt_sha256": sha(prior_path)})
    steps.save()

    def independent(name, commands):
        branch = Steps(ROOT, SimpleNamespace(evidence_root=out / name,
                                            deadline=deadline, prune_under=[]))
        branch.receipt.update(sources=before)
        try:
            for step, command, overrides in commands:
                branch.run(step, command, overrides=overrides)
            branch.receipt["state"] = "passed"
        except BaseException as error:
            branch.receipt.update(state="failed", error=str(error))
            raise
        finally:
            branch.save()

    try:
        steps.run("format", ["cargo", "fmt", "--manifest-path", "dev/Cargo.toml", "--all", "--", "--check"])
        common = ["--manifest-path", "dev/Cargo.toml", "--locked", "--profile", "dev-fast"]
        audit = [("architecture", [ROOT / "obj/audit-venv/bin/python", "scripts/rust_audit.py",
                                    "--root", ".", "--cache", "obj/fast-audit.json"], {})]
        lint = [
            ("clippy-production", ["cargo", "clippy", *common, "--lib", "--bin", "rustgm", "--", "-D", "warnings"], {}),
            ("clippy-private", ["cargo", "clippy", *common, "--lib", "--", "--test", "-D", "warnings"],
             {"CARGO_TARGET_DIR": str(ROOT / "obj/rust/private-lint")}),
        ]
        failures = []
        parallel_started = time.monotonic()
        with ThreadPoolExecutor(max_workers=2) as executor:
            futures = [executor.submit(independent, "audit", audit),
                       executor.submit(independent, "lint", lint)]
            for future in futures:
                try:
                    future.result()
                except BaseException as error:
                    failures.append(str(error))
        steps.receipt.update(parallel_static_seconds=time.monotonic() - parallel_started,
                             independent_static_receipts={name: sha(out / name / "receipt.json")
                                                          for name in ("audit", "lint")})
        steps.save()
        if failures:
            raise ValueError("; ".join(failures))
        built = steps.run("build", ["cargo", "test", *common, "--lib", "--no-run", "--message-format=json"])
        binary = artifacts(built, {"lib": selected})["lib"]
        listed = steps.run("lib-inventory", [binary, "--list"], cwd=ROOT / "dev")
        ignored = set(steps.run("lib-ignored", [binary, "--ignored", "--list"], cwd=ROOT / "dev").splitlines())
        actual = inventory("\n".join(row for row in listed.splitlines() if row not in ignored), "lib", selected)
        assert len(actual) == len(set(actual)) == 11
        steps.receipt.update(selected={"lib": actual}, binaries={"lib": {"sha256": sha(binary)}})
        steps.save()
        output = steps.run("lib", [binary, "--exact", *actual], cwd=ROOT / "dev",
                           overrides={"RUST_TEST_NOCAPTURE": "1"})
        assert re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output) == [("11", "0", "0")]
        assert fingerprint(ROOT) == before and sha(cli) == cli_sha
        assert all(sha(ROOT / path) == digest for path, digest in fixtures.items())
        steps.receipt.update(state="passed", seconds=time.monotonic() - started)
    except BaseException as error:
        steps.receipt.update(state="failed", error=str(error), seconds=time.monotonic() - started)
        raise
    finally:
        steps.save()
        destination = ROOT / "obj/parallel/all-attribute-selected-output-v1"
        destination.mkdir(parents=True, exist_ok=True)
        candidate = destination / "candidate/rustgm"
        candidate.parent.mkdir(exist_ok=True)
        shutil.copy2(cli, candidate)
        (candidate.parent / ".keep-artifacts").touch()
        binding = {"state": steps.receipt["state"], "candidate_sha256": cli_sha,
                   "source_commit": freeze["source_commit"], "sources": before,
                   "compiler_source_sha256": {path: digest for path, digest in before.items()
                                              if path.startswith("dev/") and path.endswith(".rs")},
                   "receipt": str(out.relative_to(ROOT)), "receipt_sha256": sha(out / "receipt.json"),
                   "ci_commit": os.environ.get("GITHUB_SHA"), "profile": "dev-fast"}
        (destination / "binding.json").write_text(json.dumps(binding, indent=2) + "\n")
        print(json.dumps({key: value for key, value in steps.receipt.items() if key != "sources"}, indent=2))


if __name__ == "__main__":
    if "--worker" in sys.argv:
        worker(float(sys.argv[-1]))
    else:
        start = time.monotonic()
        raise SystemExit(subprocess.run([
            "timeout", "--signal=TERM", "--kill-after=3s", "190s", sys.executable,
            "scripts/storage.py", "run", "--no-prune", "--", sys.executable,
            __file__, "--worker", str(start)], check=False).returncode)
