"""CI-only 58fa continuation: authenticated static checks plus missing exact four."""
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

CANDIDATE = "58fa8509fceadba9cd7bc40788fed8f9a644f433"
PROOF_SHA = "94cbf15c1dadd98a372c302125093f59307f64fe8708d32a84a8b5859c31ed24"
SOURCE_SHA = "a93a555612983e901ec197d63416bd94d7826312f5656df2f0e9074d8f818f8c"
OUTPUT = ROOT / "obj/parallel/all-attribute-selected-output-v1"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def worker(deadline, folder):
    with exclusive(ROOT):
        check_space(ROOT, minimum_reserve())
        steps = Steps(ROOT, SimpleNamespace(deadline=deadline, evidence_root=folder, prune_under=[]))
        try:
            retained_root = ROOT / ".github/lib026-static-proof"
            manifest = json.loads((retained_root / "manifest.json").read_bytes())
            for path, expected in manifest["files"].items():
                if sha(retained_root / path) != expected:
                    raise ValueError("retained static output changed: " + path)
            proof_path = retained_root / "1792946066731745070/receipt.json"
            if sha(proof_path) != PROOF_SHA:
                raise ValueError("retained static receipt identity changed")
            proof = json.loads(proof_path.read_bytes())
            retained = proof["steps"][:4]
            if [row["name"] for row in retained] != ["assignment-metadata", "format", "architecture", "clippy-private"]:
                raise ValueError("retained static steps differ")
            if any(row["status"] != 0 for row in retained):
                raise ValueError("retained static proof did not pass")
            if len(proof["steps"]) != 5 or proof["steps"][4]["name"] != "build" or proof["steps"][4]["status"] != 124:
                raise ValueError("original timeout evidence differs")
            sources = fingerprint(ROOT)
            source_bytes = json.dumps(sources, sort_keys=True, separators=(",", ":")).encode()
            if len(sources) != 2585 or hashlib.sha256(source_bytes).hexdigest() != SOURCE_SHA or sources != proof["sources"]:
                raise ValueError("candidate differs from full authenticated source fingerprint")
            fixtures = manifest["fixtures"]
            if any(sha(ROOT / path) != expected for path, expected in fixtures.items()):
                raise ValueError("candidate fixture differs from frozen 58fa")
            groups = proof["groups"]
            if set(groups) != {"lib"} or len(groups["lib"]) != 4 or len(set(groups["lib"])) != 4:
                raise ValueError("continuation requires original four exact selectors")
            target = OUTPUT / "cli/candidate/rustgm"
            target.parent.mkdir(parents=True, exist_ok=True)
            (target.parent / ".keep-artifacts").touch()
            shutil.copy2(ROOT / "obj/rust/dev-fast/rustgm", target)
            target.chmod(0o755)
            if target.read_bytes()[:4] != b"\x7fELF":
                raise ValueError("mandatory prebuilt CLI is not ELF")
            compiler = {p: value for p, value in sources.items() if p.endswith(".rs") and (p.startswith("dev/src/") or p.startswith("dev/bin/"))}
            binding = {"candidate_commit": CANDIDATE, "ci_commit": os.environ.get("GITHUB_SHA"),
                       "candidate_sha256": sha(target), "mode": oct(target.stat().st_mode & 0o777),
                       "compiler_source_sha256": compiler, "sources": sources, "fixtures": fixtures,
                       "build_command": "mandatory unchanged workflow dev-fast CLI step; no helper CLI rebuild",
                       "retained_prior_cli": manifest["prior_cli"]}
            (OUTPUT / "cli/binding.json").write_text(json.dumps(binding, indent=2) + "\n")
            steps.receipt.update(sources=sources, fixtures=fixtures, groups=groups, cli=binding,
                retained_static={"run": 40212246155, "candidate": CANDIDATE, "receipt_sha256": PROOF_SHA,
                                 "outputs": manifest["files"], "artifacts": manifest["artifacts"], "steps": retained,
                                 "attribution": "Passed previously; not rerun by this continuation.",
                                 "original_result": "Harness build timeout; no original tests executed."})
            steps.environment["RUST_TEST_NOCAPTURE"] = "1"
            steps.save()
            built = steps.run("build", ["cargo", "test", "--manifest-path", "dev/Cargo.toml", "--locked",
                "--profile", "dev-fast", "--lib", "--no-run", "--message-format=json"])
            binary = artifacts(built, groups)["lib"]
            listed = steps.run("lib-inventory", [binary, "--list"], cwd=ROOT / "dev")
            ignored = steps.run("lib-ignored", [binary, "--ignored", "--list"], cwd=ROOT / "dev")
            ignored_rows = set(ignored.splitlines())
            selected = inventory("\n".join(row for row in listed.splitlines() if row not in ignored_rows), "lib", groups["lib"])
            if selected != groups["lib"]:
                raise ValueError("nonignored inventory differs from original exact four")
            steps.receipt.update(selected={"lib": selected}, binaries={"lib": {"path": str(binary), "sha256": sha(binary)}})
            steps.save()
            output = steps.run("lib", [binary, "--exact", *selected], cwd=ROOT / "dev")
            counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output)
            if len(counts) != 1 or tuple(map(int, counts[0])) != (4, 0, 0):
                raise ValueError("executed count differs from original exact four")
            if fingerprint(ROOT) != sources or any(sha(ROOT / p) != h for p, h in fixtures.items()):
                raise ValueError("source or fixtures changed during continuation")
            steps.receipt["state"] = "passed"
            print("Passed missing harness build and original exact4; static checks retained from40212246155.", flush=True)
            return 0
        except BaseException as error:
            steps.receipt.update(state="failed", error=str(error))
            raise
        finally:
            steps.receipt["sources_after"] = fingerprint(ROOT)
            steps.save()
            OUTPUT.mkdir(parents=True, exist_ok=True)
            (OUTPUT / "result.json").write_text(json.dumps({"candidate": CANDIDATE, "state": steps.receipt["state"],
                "receipt": str(folder.relative_to(ROOT)), "receipt_sha256": sha(folder / "receipt.json"),
                "retained_run": 40212246155}, indent=2) + "\n")
            print(f"Continuation evidence: {folder}", flush=True)


def main():
    if len(sys.argv) == 4 and sys.argv[1] == "--worker":
        return worker(float(sys.argv[2]), Path(sys.argv[3]))
    started = time.monotonic()
    folder = ROOT / f"obj/qualification/fast/{time.time_ns()}"
    command = [sys.executable, "scripts/resources.py", "run", "--wait-seconds", "0", "--",
        sys.executable, ".github/focused_refs.py", "--worker", str(started + 187), str(folder)]
    process = subprocess.Popen(command, cwd=ROOT, start_new_session=True)
    status = 1
    try:
        status = process.wait(timeout=max(0.01, started + 188 - time.monotonic()))
        return status
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=max(0.01, started + 189 - time.monotonic()))
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=max(0.01, started + 190 - time.monotonic()))
        status = 124
        return status
    finally:
        if status:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        folder.parent.mkdir(parents=True, exist_ok=True)
        folder.with_suffix(".launcher.json").write_text(json.dumps({"state": "passed" if status == 0 else "failed",
            "status": status, "seconds": time.monotonic() - started, "absolute_budget_seconds": 190,
            "command": command, "retained_run": 40212246155}, indent=2) + "\n")
        print(f"Missing-only total: {time.monotonic() - started:.2f}s", flush=True)


if __name__ == "__main__":
    sys.exit(main())
