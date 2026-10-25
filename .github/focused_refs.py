"""Hosted source formatting and focused syntax qualification of frozen inputs."""
import concurrent.futures
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
OUT = ROOT / "obj/parallel/all-attribute-selected-output-v1"
if "--worker" not in sys.argv:
    raise SystemExit(subprocess.run([
        "timeout", "--signal=TERM", "--kill-after=3s", "190s", sys.executable,
        "scripts/storage.py", "run", "--no-prune", "--", sys.executable,
        __file__, "--worker"], check=False).returncode)

sys.path.insert(0, str(ROOT / "scripts"))
from fastcheck_run import Steps, fingerprint
from maintainer_batch import artifacts, inventory, selection
from maintainer_suites import discover

started = time.monotonic()
OUT.mkdir(parents=True, exist_ok=False)
plan = json.loads((ROOT / ".github/alias-inputs.json").read_text())
for name, digest in plan["inputs"].items():
    source = ROOT / ".github/alias-overlay" / name
    assert hashlib.sha256(source.read_bytes()).hexdigest() == digest
    shutil.copyfile(source, ROOT / name)

# Formatting precedes the qualifying source snapshot. Preserve both freezes.
result = subprocess.run([
    "cargo", "fmt", "--manifest-path", "dev/Cargo.toml", "--all"],
    capture_output=True, timeout=15, check=False)
(OUT / "source-format.stdout").write_bytes(result.stdout)
(OUT / "source-format.stderr").write_bytes(result.stderr)
assert result.returncode == 0
changed = subprocess.check_output(["git", "diff", "--name-only"], text=True).splitlines()
assert set(changed) <= set(plan["inputs"]), "formatter changed an undeclared source"
for name in plan["inputs"]:
    target = OUT / "formatted" / name
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(ROOT / name, target)
sources = fingerprint(ROOT)
binding = {"state": "running", "ci_commit": os.environ.get("GITHUB_SHA"),
           "input_plan": plan, "sources": sources, "formatted_before_snapshot": True,
           "scope": "syntax foundation only; semantic aliases remain explicitly unsupported"}

args = SimpleNamespace(evidence_root=ROOT / "obj/qualification/fast/alias-grammar-v1",
                       deadline=started + 182, prune_under=[],
                       suites=plan["selected"], lib_case=[], ignored=False, pa=[])
steps = Steps(ROOT, args)
steps.receipt["sources"] = sources
steps.save()
audit_args = SimpleNamespace(evidence_root=args.evidence_root / "audit",
                             deadline=args.deadline, prune_under=[])
audit = Steps(ROOT, audit_args)
audit.receipt["sources"] = sources
audit.save()
common = ["--manifest-path", "dev/Cargo.toml", "--locked", "--profile", "dev-fast"]


def audit_check():
    try:
        audit.run("architecture", [ROOT / "obj/audit-venv/bin/python",
                                  "scripts/rust_audit.py", "--root", ".",
                                  "--cache", "obj/fast-audit.json"])
        audit.receipt["state"] = "passed"
    except Exception as error:
        audit.receipt.update(state="failed", error=str(error))
    finally:
        audit.save()


try:
    failures = []
    groups = selection(discover(ROOT), plan["selected"], [])
    assert len(groups) == 1
    group = next(iter(groups))
    steps.run("assignment-metadata", [sys.executable, "scripts/assignment_metadata.py"])
    steps.run("format", ["cargo", "fmt", "--manifest-path", "dev/Cargo.toml", "--all", "--", "--check"])
    with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
        future = pool.submit(audit_check)
        for name, command, overrides in [
            ("clippy-production", ["cargo", "clippy", *common, "--lib", "--bin", "rustgm", "--", "-D", "warnings"], {}),
            ("clippy-private", ["cargo", "clippy", *common, "--lib", "--", "--test", "-D", "warnings"],
             {"CARGO_TARGET_DIR": str(ROOT / "obj/rust/private-lint")}),
            ("clippy-integration", ["cargo", "clippy", *common, "--test", group, "--", "-D", "warnings"], {}),
        ]:
            try:
                steps.run(name, command, overrides=overrides)
            except Exception as error:
                failures.append(str(error))
        future.result()
    assert audit.receipt["state"] == "passed", "architecture audit failed"
    assert not failures, "; ".join(failures)
    built = steps.run("build", ["cargo", "test", *common, "--test", group,
                                "--no-run", "--message-format=json"])
    binary = artifacts(built, groups)[group]
    listed = steps.run("inventory", [binary, "--list"], cwd=ROOT / "dev")
    ignored = steps.run("ignored", [binary, "--ignored", "--list"], cwd=ROOT / "dev")
    ignored_rows = set(ignored.splitlines())
    listed = "\n".join(row for row in listed.splitlines() if row not in ignored_rows)
    selected = inventory(listed, group, groups[group])
    assert len(selected) == len(plan["selected"])
    steps.receipt.update(groups=groups, selected={group: selected},
                         binaries={group: {"path": str(binary), "sha256": hashlib.sha256(binary.read_bytes()).hexdigest()}})
    steps.save()
    output = steps.run("selected-runtime", [binary, "--exact", *selected], cwd=ROOT / "dev")
    counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output)
    assert len(counts) == 1 and tuple(map(int, counts[0])) == (len(selected), 0, 0)
    assert fingerprint(ROOT) == sources
    steps.receipt.update(state="passed", sources_after=sources)
    binding.update(state="passed", selected=steps.receipt["selected"])
except Exception as error:
    steps.receipt.update(state="failed", error=str(error))
    binding.update(state="failed", error=str(error))
    raise
finally:
    steps.save()
    binding["seconds"] = time.monotonic() - started
    (OUT / "binding.json").write_text(json.dumps(binding, indent=2) + "\n")
    print(json.dumps({key: value for key, value in binding.items()
                      if key not in ("sources", "input_plan")}, indent=2))
