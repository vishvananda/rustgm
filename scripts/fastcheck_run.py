"""Focused checks, exact execution receipts and scoped cleanup for fastcheck."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

from maintainer_batch import artifacts, inventory, package_environment, selection
from maintainer_suites import discover
from storage import check_space, cleanup_roots, exclusive, minimum_reserve, prune


def fingerprint(root):
    paths = sorted((root / "dev").rglob("*.rs"))
    paths += [root / path for path in ("dev/Cargo.toml", "dev/Cargo.lock",
              "scripts/fastcheck.py", "scripts/fastcheck_run.py", "scripts/rust_audit.py",
              "scripts/rust-module-roles.json", "scripts/maintainer_batch.py",
              "scripts/maintainer_suites.py", "scripts/storage.py", "scripts/storage_incremental.py", "scripts/resource_runner.py",
              "scripts/resource_policy.py", "scripts/resource_locks.py")]
    paths += [root / path for path in ("assignments.json", "assignment-plan.json",
              "docs/assignment-requirements.json", "Makefile", "scripts/assignment_metadata.py", "scripts/assignment_plan.py",
              "scripts/course.py", "scripts/course_suites.py", "scripts/reference_batches.py",
              "scripts/check_native.py", "scripts/check_layout.py", "scripts/check_layouts.py", "scripts/check_syntax.py")]
    paths += sorted(root.glob("pa*/tests/manifest.json"))
    paths += sorted(root.glob("pa*/tests/units/*.json"))
    return {str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}


class Steps:
    def __init__(self, root, args):
        self.root, self.args = root, args
        self.folder = args.evidence_root
        cleanup_roots(root, [self.folder, *args.prune_under])
        self.folder.mkdir(parents=True, exist_ok=False)
        self.receipt = {"profile": "dev-fast", "state": "running", "steps": []}
        self.environment = {**os.environ, **package_environment(root), "CARGO_INCREMENTAL": "1",
                            "CARGO_TARGET_DIR": str(root / "obj/rust"),
                            "CARGO_BUILD_JOBS": "2", "RUST_TEST_THREADS": "2"}

    def save(self):
        (self.folder / "receipt.json").write_text(json.dumps(self.receipt, indent=2) + "\n")

    def run(self, name, command, *, cwd=None, overrides=None):
        remaining = self.args.deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError("development validation budget exhausted")
        print(f"START {name} ({remaining:.0f}s remaining)", flush=True)
        started = time.monotonic()
        with (self.folder / f"{name}.stdout").open("w") as output, (
                self.folder / f"{name}.stderr").open("w") as errors:
            try:
                result = subprocess.run(list(map(str, command)), cwd=cwd or self.root,
                                        env={**self.environment, **(overrides or {})}, stdout=output, stderr=errors,
                                        timeout=remaining, check=False)
                status = result.returncode
            except subprocess.TimeoutExpired:
                status = 124
        elapsed = time.monotonic() - started
        self.receipt["steps"].append({"name": name, "command": list(map(str, command)),
                                      "seconds": elapsed, "status": status,
                                      "environment": overrides or {}})
        self.save()
        print(f"END {name}: {elapsed:.2f}s, status {status}", flush=True)
        stdout = (self.folder / f"{name}.stdout").read_text()
        if status:
            print((stdout + (self.folder / f"{name}.stderr").read_text())[-10000:], flush=True)
            if status == 124:
                raise TimeoutError(f"{name}: development validation budget exhausted")
            raise ValueError(f"{name} failed with status {status}")
        return stdout


def execute(root, args, steps):
    suites = discover(root)
    names = args.suites or ([] if args.lib_case else ["lexer"])
    groups = selection(suites, names, args.lib_case)
    if args.ignored and not args.prune_under:
        raise ValueError("native fast checks require --prune-under for their disposable output roots")
    steps.receipt["groups"] = groups
    # Fingerprint content rather than just HEAD: dirty worker snapshots are normal.
    steps.receipt["sources"] = fingerprint(root)
    steps.save()
    steps.run("assignment-metadata", [sys.executable, "scripts/assignment_metadata.py"])
    steps.run("format", ["cargo", "fmt", "--manifest-path", "dev/Cargo.toml", "--all", "--", "--check"])
    python = root / "obj/audit-venv/bin/python"
    if not python.is_file():
        raise ValueError("run make setup-audit once before fastcheck")
    steps.run("architecture", [python, "scripts/rust_audit.py", "--root", ".",
                                "--cache", "obj/fast-audit.json"])
    targets = []
    for group in groups:
        targets += ["--lib"] if group == "lib" else ["--test", group]
    common = ["--manifest-path", "dev/Cargo.toml", "--locked", "--profile", "dev-fast"]
    if set(groups) != {"lib"} or args.pa:
        steps.run("clippy", ["cargo", "clippy", *common, *targets, "--bin", "rustgm", "--", "-D", "warnings"])
    if "lib" in groups:
        # --lib alone does not lint cfg(test). Ask rustc for the library harness
        # without Cargo's --tests, which selects every integration target too.
        steps.run("clippy-private", ["cargo", "clippy", *common, "--lib", "--", "--test", "-D", "warnings"],
                  overrides={"CARGO_TARGET_DIR": str(root / "obj/rust/private-lint")})
    built = steps.run("build", ["cargo", "test", *common, *targets, "--no-run", "--message-format=json"])
    binaries = artifacts(built, groups)
    selected = {}
    flags = ["--ignored"] if args.ignored else []
    for group, binary in binaries.items():
        listed = steps.run(group + "-inventory", [binary, *flags, "--list"], cwd=root / "dev")
        if not args.ignored:
            ignored = steps.run(group + "-ignored", [binary, "--ignored", "--list"], cwd=root / "dev")
            ignored_rows = set(ignored.splitlines())
            listed = "\n".join(row for row in listed.splitlines() if row not in ignored_rows)
        selected[group] = inventory(listed, group, groups[group])
    steps.receipt["selected"] = selected
    steps.receipt["binaries"] = {group: {"path": str(binary),
                                         "sha256": hashlib.sha256(binary.read_bytes()).hexdigest()}
                                  for group, binary in binaries.items()}
    steps.save()
    for group, binary in binaries.items():
        started = time.time()
        output = steps.run(group, [binary, *flags, "--exact", *selected[group]], cwd=root / "dev")
        counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output)
        if len(counts) != 1 or tuple(map(int, counts[0])) != (len(selected[group]), 0, 0):
            raise ValueError(f"{group}: execution count differs from exact selection")
        # Never traverse all historical obj output after each focused test.
        if args.prune_under:
            steps.receipt.setdefault("cleanup", []).append({"group": group, "removed": prune(
                root, apply=True, since=started, paths=args.prune_under)})
            steps.save()
    if args.pa:
        steps.run("build-cli", ["cargo", "build", *common, "--bin", "rustgm"])
        for stage in dict.fromkeys(args.pa):
            steps.run(stage, [sys.executable, "scripts/course.py", stage,
                              "--app", root / "obj/rust/dev-fast/rustgm"])
    after = fingerprint(root)
    if after != steps.receipt["sources"]:
        raise ValueError("source changed during validation; evidence does not qualify the current source")
    print(f"Passed {sum(map(len, selected.values()))} exact tests across {len(groups)} target(s).", flush=True)


def check(root, args):
    with exclusive(root):
        check_space(root, minimum_reserve())
        steps = Steps(root, args)
        try:
            execute(root, args, steps)
            steps.receipt["state"] = "passed"
            return 0
        except Exception as error:
            steps.receipt.update(state="failed", error=str(error))
            raise
        finally:
            steps.save()
            print(f"Fastcheck evidence: {steps.folder}", flush=True)
