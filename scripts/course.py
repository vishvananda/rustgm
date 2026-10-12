#!/usr/bin/env python3
"""Run explicit fixture manifests against committed text oracles."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import difflib
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
SIDECARS = (".ref", ".ref.stderr", ".ref.exit_status")


def check_relative_path(value, stage):
    if (not isinstance(value, str) or not value or Path(value).is_absolute()
            or ".." in Path(value).parts or Path(value).as_posix() != value):
        raise ValueError(f"{stage}: fixture identities must be normalized relative paths")


def load_cases(root, stage):
    assignment = root / stage
    manifest_path = assignment / "tests/manifest.json"
    if not manifest_path.is_file():
        raise ValueError(f"{stage}: planned assignment; no accepted test contract yet")
    manifest = json.loads(manifest_path.read_text())
    cases = manifest["cases"]
    if not cases:
        raise ValueError(f"{stage}: an empty suite cannot pass")
    paths = [case["path"] for case in cases]
    for path in paths:
        check_relative_path(path, stage)
    if len(paths) != len(set(paths)):
        raise ValueError(f"{stage}: duplicate fixture identity")
    auxiliary = set()
    for case in cases:
        path = (assignment / case["path"]).resolve()
        if not path.is_relative_to((assignment / "tests").resolve()) or not path.is_file():
            raise ValueError(f"{stage}: invalid fixture path {path}")
        if case["exit_status"] not in (0, 1):
            raise ValueError(f"{stage}: expected outcome must be success or diagnosed rejection")
        if case.get("input", "stdin") not in ("stdin", "file"):
            raise ValueError(f"{stage}: input must be stdin or file")
        files = case.get("auxiliary", [])
        if not isinstance(files, list) or any(not isinstance(p, str) for p in files):
            raise ValueError(f"{stage}: auxiliary must be a list of paths")
        if len(set(files)) != len(files) or set(files) & set(paths):
            raise ValueError(f"{stage}: duplicate or primary auxiliary identity")
        if files and case.get("input") != "file":
            raise ValueError(f"{stage}: auxiliary files require file input")
        for file in files:
            check_relative_path(file, stage)
            child = (assignment / file).resolve()
            if (not child.is_relative_to(path.parent) or not child.is_file()
                    or not child.is_relative_to((assignment / "tests").resolve())):
                raise ValueError(f"{stage}: invalid auxiliary path {file}")
            auxiliary.add(file)
        args = case.get("args", [])
        if (not isinstance(args, list) or any(not isinstance(arg, str) for arg in args)
                or len(args) % 2 or any(args[i] != "--cfg" for i in range(0, len(args), 2))):
            raise ValueError(f"{stage}: case args must be --cfg/value pairs")
    discovered = {p.relative_to(assignment).as_posix() for p in (assignment / "tests").rglob("*.rs")}
    declared_rs = {p for p in set(paths) | auxiliary if p.endswith(".rs")}
    if discovered != declared_rs:
        raise ValueError(f"{stage}: undiscovered or missing fixtures: {sorted(discovered ^ declared_rs)}")
    return manifest, cases


def execute(command, data, timeout):
    try:
        result = subprocess.run(command, input=data, capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired as error:
        raise ValueError("compiler timed out") from error
    except OSError as error:
        raise ValueError(f"cannot execute compiler: {error}") from error
    if result.returncode not in (0, 1):
        raise ValueError(f"compiler infrastructure failure: exit {result.returncode}")
    return result.stdout, result.stderr, f"{result.returncode}\n".encode()


def run_case(root, stage, mode, case, app, timeout):
    fixture = root / stage / case["path"]
    command = [*app, mode, "--edition=2024", *case.get("args", [])]
    data = fixture.read_bytes()
    if case.get("input") == "file":
        command.append(str(fixture.resolve()))
        data = b""
    actual = execute(command, data, timeout)
    return fixture, actual


def compare(fixture, actual):
    errors = []
    for suffix, value in zip(SIDECARS, actual):
        path = Path(str(fixture) + suffix)
        if not path.is_file():
            errors.append(f"missing oracle: {path.name}")
        elif path.read_bytes() != value:
            diff = difflib.unified_diff(path.read_text(errors="replace").splitlines(),
                                       value.decode(errors="replace").splitlines(),
                                       fromfile=path.name, tofile="actual", lineterm="")
            errors.append("\n".join(list(diff)[:40]))
    return errors


def run_stage(root, stage, app, *, bless=False, jobs=4, timeout=10):
    manifest, cases = load_cases(root, stage)
    results = []

    def worker(case):
        try:
            fixture, actual = run_case(root, stage, manifest["mode"], case, app, timeout)
            errors = [] if bless else compare(fixture, actual)
            if actual[2] != f"{case['exit_status']}\n".encode():
                errors.append(f"outcome differs from independently declared exit {case['exit_status']}")
            return case, fixture, actual, errors
        except ValueError as error:
            return case, None, None, [str(error)]

    with ThreadPoolExecutor(max_workers=jobs) as pool:
        results = list(pool.map(worker, cases))
    artifacts = root / "obj/tests" / stage / str(time.time_ns())
    artifacts.mkdir(parents=True)
    for case, _, actual, _ in results:
        if actual is not None:
            destination = artifacts / case["path"]
            destination.parent.mkdir(parents=True, exist_ok=True)
            for suffix, value in zip((".stdout", ".stderr", ".exit_status"), actual):
                Path(str(destination) + suffix).write_bytes(value)
    (artifacts / "results.json").write_text(json.dumps({"stage": stage, "command": [*app, manifest["mode"], "--edition=2024"],
        "bless": bless, "cases": [{"path": case["path"], "args": case.get("args", []), "passed": not errors, "errors": errors}
                                   for case, _, _, errors in results]}, indent=2) + "\n")
    # Check every independent expected outcome before changing any reference.
    if bless and not any(errors for _, _, _, errors in results):
        for _, fixture, actual, _ in results:
            for suffix, value in zip(SIDECARS, actual):
                Path(str(fixture) + suffix).write_bytes(value)
        source_hashes = {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
                         for p in sorted((root / "dev").rglob("*.rs"))}
        receipt = {"schema": 1, "mode": manifest["mode"], "compiler_source_sha256": source_hashes,
                   "manifest_sha256": hashlib.sha256((root / stage / "tests/manifest.json").read_bytes()).hexdigest(),
                   "fixtures": {case["path"]: hashlib.sha256(fixture.read_bytes()).hexdigest()
                                for case, fixture, _, _ in results}}
        auxiliary = sorted({p for case in cases for p in case.get("auxiliary", [])})
        if auxiliary:
            receipt["auxiliary"] = {p: hashlib.sha256((root / stage / p).read_bytes()).hexdigest()
                                    for p in auxiliary}
        (root / stage / "reference.json").write_text(json.dumps(receipt, indent=2) + "\n")
    passed = 0
    for case, _, _, errors in results:
        if not errors:
            passed += 1
        print(f"{stage}/{case['path']}: {'FAIL (0/1)' if errors else 'PASS (1/1)'}")
        for error in errors:
            print(error)
    print(f"{stage}: {passed}/{len(cases)}")
    print(f"Evidence: {artifacts.relative_to(root)}")
    return passed == len(cases)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage")
    parser.add_argument("--app", default=str(ROOT / "dev/rustgm"))
    parser.add_argument("--bless", action="store_true")
    parser.add_argument("--jobs", type=int, default=4)
    parser.add_argument("--timeout", type=float, default=10)
    args = parser.parse_args()
    if args.jobs < 1 or args.timeout <= 0:
        parser.error("jobs and timeout must be positive")
    try:
        return 0 if run_stage(ROOT, args.stage, [str(Path(args.app).resolve())],
                             bless=args.bless, jobs=args.jobs, timeout=args.timeout) else 1
    except (ValueError, OSError, KeyError) as error:
        print(f"harness error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
