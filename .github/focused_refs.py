"""CI-only genuine trait-coordinate test correction with authenticated eight-pass retention."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import time
from types import SimpleNamespace
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
from fastcheck_run import Steps, fingerprint
from maintainer_batch import artifacts, inventory
from storage import check_space, exclusive, minimum_reserve
CANDIDATE = "22e00b6e734d10fd5135eba7216578787af13282"
MANIFEST_SHA = "b33bdd94cceb1bd2531fb251139561d8af67961adcf2f3404126015153da8dde"
OUTPUT = ROOT / "obj/parallel/all-attribute-selected-output-v1"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def worker(deadline, folder):
    with exclusive(ROOT):
        check_space(ROOT, minimum_reserve())
        steps = Steps(ROOT, SimpleNamespace(deadline=deadline, evidence_root=folder, prune_under=[]))
        try:
            retained_root = ROOT / ".github/lib026-versioned-proof"
            if sha(retained_root / "manifest.json") != MANIFEST_SHA:
                raise ValueError("retained proof manifest changed")
            manifest = json.loads((retained_root / "manifest.json").read_bytes())
            for path, expected in manifest["files"].items():
                if sha(retained_root / path) != expected:
                    raise ValueError("retained raw proof changed: " + path)
            proof = json.loads((retained_root / manifest["receipt"]).read_bytes())
            if proof["state"] != "failed" or [(s["name"], s["status"]) for s in proof["steps"]] != [(name, 0) for name in ["assignment-metadata", "format", "architecture", "clippy-private", "build", "lib-inventory", "lib-ignored"]] + [("lib", 101)]:
                raise ValueError("original static/eight-pass failure state differs")
            original_output = (retained_root / manifest["stdout"]).read_text()
            if any(f"test {name} ... ok" not in original_output for name in manifest["retained_exact8"]):
                raise ValueError("original exact-eight passes differ")
            sources = fingerprint(ROOT)
            source_sha = hashlib.sha256(json.dumps(sources, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
            if len(sources) != 2590 or source_sha != manifest["current_sources_sha256"]:
                raise ValueError("current full source fingerprint changed")
            changes = {p for p in sources if sources[p] != proof["sources"].get(p)}
            if set(sources) != set(proof["sources"]) or changes != {"dev/tests/invariants/transported_annotations.rs"}:
                raise ValueError("test correction changed production or unrelated sources")
            fixtures = manifest["fixtures"]
            if any(sha(ROOT / p) != h for p, h in fixtures.items()):
                raise ValueError("provider fixture changed")
            groups = {"lib": [manifest["selector"]]}
            target = OUTPUT / "cli/candidate/rustgm"
            target.parent.mkdir(parents=True, exist_ok=True)
            (target.parent / ".keep-artifacts").touch()
            shutil.copy2(ROOT / "obj/rust/dev-fast/rustgm", target)
            target.chmod(0o755)
            binding = {"source_commit": CANDIDATE, "compiler_unchanged_from": manifest["prior_candidate"],
                "ci_commit": os.environ.get("GITHUB_SHA"), "candidate_sha256": sha(target), "sources": sources,
                "compiler_source_sha256": {p: h for p, h in sources.items() if p.endswith(".rs") and (p.startswith("dev/src/") or p.startswith("dev/bin/"))},
                "fixtures": fixtures, "profile": "dev-fast", "qualification": {"state": "pending"}, "build": "mandatory unchanged workflow CLI step; no helper CLI rebuild"}
            (OUTPUT / "binding.json").write_text(json.dumps(binding, indent=2) + "\n")
            steps.receipt.update(sources=sources, fixtures=fixtures, groups=groups, cli=binding,
                retained_functional={"run": manifest["prior_run"], "candidate": manifest["prior_candidate"],
                    "files": manifest["files"], "receipt_sha256": manifest["receipt_sha256"],
                    "exact8": manifest["retained_exact8"], "original_ci_source_guard": manifest["original_ci_source_guard"]},
                retained_metadata=proof["steps"][0])
            steps.environment["RUST_TEST_NOCAPTURE"] = "1"
            steps.save()
            steps.run("format", ["cargo", "fmt", "--manifest-path", "dev/Cargo.toml", "--all", "--", "--check"])
            steps.run("architecture", [ROOT / "obj/audit-venv/bin/python", "scripts/rust_audit.py", "--root", ".", "--cache", "obj/fast-audit.json"])
            steps.run("clippy-private", ["cargo", "clippy", "--manifest-path", "dev/Cargo.toml", "--locked", "--profile", "dev-fast", "--lib", "--", "--test", "-D", "warnings"], overrides={"CARGO_TARGET_DIR": str(ROOT / "obj/rust/private-lint")})
            built = steps.run("build", ["cargo", "test", "--manifest-path", "dev/Cargo.toml", "--locked", "--profile", "dev-fast", "--lib", "--no-run", "--message-format=json"])
            binary = artifacts(built, groups)["lib"]
            listed = steps.run("lib-inventory", [binary, "--list"], cwd=ROOT / "dev")
            ignored = steps.run("lib-ignored", [binary, "--ignored", "--list"], cwd=ROOT / "dev")
            ignored_rows = set(ignored.splitlines())
            selected = inventory("\n".join(row for row in listed.splitlines() if row not in ignored_rows), "lib", groups["lib"])
            if selected != groups["lib"]:
                raise ValueError("nonignored inventory differs from exact new selector")
            steps.receipt.update(selected={"lib": selected}, binaries={"lib": {"path": str(binary), "sha256": sha(binary)}})
            steps.save()
            output = steps.run("lib", [binary, "--exact", *selected], cwd=ROOT / "dev")
            counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output)
            if len(counts) != 1 or tuple(map(int, counts[0])) != (1, 0, 0):
                raise ValueError("executed count differs from exact new selector")
            if fingerprint(ROOT) != sources or any(sha(ROOT / p) != h for p, h in fixtures.items()):
                raise ValueError("source or provider fixture changed during qualification")
            steps.receipt["state"] = "passed"
            binding["qualification"] = {"state": "passed", "fresh_selector": groups["lib"], "retained_exact8": manifest["retained_exact8"], "receipt": str(folder.relative_to(ROOT))}
            (OUTPUT / "binding.json").write_text(json.dumps(binding, indent=2) + "\n")
            print("Passed exact new v1 domain control with fresh fmt/audit/privateClippy; original exact8 retained.", flush=True)
            return 0
        except BaseException as error:
            steps.receipt.update(state="failed", error=str(error))
            raise
        finally:
            steps.receipt["sources_after"] = fingerprint(ROOT)
            steps.save()
            OUTPUT.mkdir(parents=True, exist_ok=True)
            (OUTPUT / "result.json").write_text(json.dumps({"candidate": CANDIDATE, "state": steps.receipt["state"],
                "receipt": str(folder.relative_to(ROOT)), "receipt_sha256": sha(folder / "receipt.json"), "retained_run": 40215420059}, indent=2) + "\n")
            print(f"Continuation evidence: {folder}", flush=True)


def main():
    if len(sys.argv) == 4 and sys.argv[1] == "--worker":
        return worker(float(sys.argv[2]), Path(sys.argv[3]))
    started = time.monotonic()
    folder = ROOT / f"obj/qualification/fast/{time.time_ns()}"
    command = [sys.executable, "scripts/resources.py", "run", "--wait-seconds", "0", "--", sys.executable,
        ".github/focused_refs.py", "--worker", str(started + 197), str(folder)]
    process = subprocess.Popen(command, cwd=ROOT, start_new_session=True)
    status = 1
    try:
        status = process.wait(timeout=max(0.01, started + 198 - time.monotonic()))
        return status
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=max(0.01, started + 199 - time.monotonic()))
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=max(0.01, started + 200 - time.monotonic()))
        status = 124
        return status
    finally:
        if status:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        folder.parent.mkdir(parents=True, exist_ok=True)
        folder.with_suffix(".launcher.json").write_text(json.dumps({"state": "passed" if status == 0 else "failed", "status": status,
            "seconds": time.monotonic() - started, "absolute_budget_seconds": 200, "command": command, "retained_run": 40215420059}, indent=2) + "\n")
        print(f"Focused v1 total: {time.monotonic() - started:.2f}s", flush=True)


if __name__ == "__main__":
    sys.exit(main())
