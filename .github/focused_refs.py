"""CI-only continuation: retained static proof plus the missing exact private run."""
import hashlib
import json
import os
from pathlib import Path
import re
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

PROOF_SHA = "965537ec0b892e3e4758f7d76161e0e680abfcbd0755b35c262d14787d6de410"
SOURCE_SHA = "9eb9e5f8f64ec5a1f4ef262b905dbbe12467089546b1ba4bf9032c8264e77d6d"
CANDIDATE = "b6a8bbc4ffca838a6473ca3593f963505c07fa73"


def worker(deadline, folder):
    with exclusive(ROOT):
        check_space(ROOT, minimum_reserve())
        steps = Steps(ROOT, SimpleNamespace(deadline=deadline, evidence_root=folder, prune_under=[]))
        try:
            proof_bytes = (ROOT / ".github/higher-order-static-receipt.json").read_bytes()
            if hashlib.sha256(proof_bytes).hexdigest() != PROOF_SHA:
                raise ValueError("retained static receipt identity changed")
            proof = json.loads(proof_bytes)
            retained = proof["steps"][:4]
            names = [row["name"] for row in retained]
            if names != ["assignment-metadata", "format", "architecture", "clippy-private"]:
                raise ValueError("retained static steps differ")
            if any(row["status"] != 0 for row in retained):
                raise ValueError("retained static proof did not pass")
            if len(proof["steps"]) != 5 or proof["steps"][4]["name"] != "build" or proof["steps"][4]["status"] != 124:
                raise ValueError("original timeout evidence differs")
            sources = fingerprint(ROOT)
            source_bytes = json.dumps(sources, sort_keys=True, separators=(",", ":")).encode()
            if len(sources) != 2585 or hashlib.sha256(source_bytes).hexdigest() != SOURCE_SHA or sources != proof["sources"]:
                raise ValueError("candidate differs from the authenticated full source fingerprint")
            groups = proof["groups"]
            if set(groups) != {"lib"} or len(groups["lib"]) != 3 or len(set(groups["lib"])) != 3:
                raise ValueError("continuation requires the original three exact selectors")
            steps.receipt.update(
                sources=sources, groups=groups,
                retained_static={"run": 40215762916, "candidate": CANDIDATE,
                                 "receipt_sha256": PROOF_SHA, "steps": retained,
                                 "attribution": "Passed previously; not rerun by this continuation.",
                                 "original_result": "Harness build timeout; no original tests executed."})
            fixture = ROOT / "pa31/tests/spec/source-functions-transport/minimal-provider.rs"
            if hashlib.sha256(fixture.read_bytes()).hexdigest() != "554fbcf9e6b7bd55f2ea8deec5eecab53b2b802806664d0eb0419ecbe47d2297":
                raise ValueError("actual minimal provider fixture changed")
            steps.receipt["fixture"] = {"path": str(fixture.relative_to(ROOT)),
                "sha256": "554fbcf9e6b7bd55f2ea8deec5eecab53b2b802806664d0eb0419ecbe47d2297"}
            steps.environment["RUST_TEST_NOCAPTURE"] = "1"
            steps.save()
            command = ["cargo", "test", "--manifest-path", "dev/Cargo.toml", "--locked",
                       "--profile", "dev-fast", "--lib", "--no-run", "--message-format=json"]
            built = steps.run("build", command)
            binary = artifacts(built, groups)["lib"]
            listed = steps.run("lib-inventory", [binary, "--list"], cwd=ROOT / "dev")
            ignored = steps.run("lib-ignored", [binary, "--ignored", "--list"], cwd=ROOT / "dev")
            ignored_rows = set(ignored.splitlines())
            listed = "\n".join(row for row in listed.splitlines() if row not in ignored_rows)
            selected = inventory(listed, "lib", groups["lib"])
            if selected != groups["lib"]:
                raise ValueError("resolved nonignored inventory differs from the original exact three")
            steps.receipt.update(selected={"lib": selected}, binaries={"lib": {
                "path": str(binary), "sha256": hashlib.sha256(binary.read_bytes()).hexdigest()}})
            steps.save()
            output = steps.run("lib", [binary, "--exact", *selected], cwd=ROOT / "dev")
            counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output)
            if len(counts) != 1 or tuple(map(int, counts[0])) != (3, 0, 0):
                raise ValueError("executed count differs from the original exact three")
            if fingerprint(ROOT) != sources:
                raise ValueError("source changed during continuation")
            steps.receipt["state"] = "passed"
            print("Passed missing harness build and original exact3; static checks retained from40215762916.", flush=True)
            return 0
        except BaseException as error:
            steps.receipt.update(state="failed", error=str(error))
            raise
        finally:
            steps.save()
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
        folder.with_suffix(".launcher.json").write_text(json.dumps({
            "state": "passed" if status == 0 else "failed", "status": status,
            "seconds": time.monotonic() - started, "absolute_budget_seconds": 190,
            "command": command, "retained_run": 40215762916}, indent=2) + "\n")
        print(f"Missing-only total: {time.monotonic() - started:.2f}s", flush=True)


if __name__ == "__main__":
    sys.exit(main())
