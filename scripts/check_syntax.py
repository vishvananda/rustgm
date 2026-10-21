#!/usr/bin/env python3
"""Independent stable-rustc checks for explicitly classified syntax fixtures."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

from course import ROOT, load_cases, run_case, case_environment


def check_classification(case):
    metadata = case["rustc"]
    rustc_emission(metadata)
    if metadata["kind"] == "fragment":
        return
    if metadata.get("status", 0) != case["exit_status"] and not metadata.get("reason"):
        raise ValueError(f"independent outcome mismatch requires an explicit reason: {case['path']}")


def rustc_emission(metadata):
    emit = metadata.get("emit", "metadata")
    if emit not in ("metadata", "obj"):
        raise ValueError("independent emission must be metadata or obj")
    if emit == "obj" and metadata["kind"] not in ("program", "library"):
        raise ValueError("object generation requires a complete program or library")
    return "--emit=" + emit


def expression_source(data, metadata):
    return (metadata.get("prefix", "fn main() { let _ = ").encode() + data
            + metadata.get("suffix", "; }\n").encode())


def rustc_arguments(case, default_name):
    """Preserve cfg arguments and the CLI's last explicit crate-name choice."""
    arguments = case.get("args", [])
    name = case.get("library", {}).get("crate_name", default_name)
    extra = []
    for index in range(0, len(arguments), 2):
        if arguments[index] == "--crate-name":
            name = arguments[index + 1]
        else:
            extra.extend(arguments[index:index + 2])
    return ["--crate-name", name, *extra]


def check_configured_owners(metadata, output):
    if "configured" in metadata:
        actual = [value == b"true" for value in re.findall(rb"^configured \d+ (true|false)$", output, re.M)]
        if actual != metadata["configured"]:
            raise ValueError(f"configured owner facts disagree: {actual}, wanted {metadata['configured']}")


def check(stage, app, jobs=None):
    jobs = int(os.environ.get("RUSTGM_TEST_JOBS", "4")) if jobs is None else jobs
    if not 1 <= jobs <= 8:
        raise ValueError("independent check jobs must be between 1 and 8")
    version = subprocess.check_output(["rustc", "--version"], text=True)
    if not version.startswith("rustc 1.98.0 "):
        raise ValueError(f"expected rustc 1.98.0, got {version.strip()}")
    manifest, cases = load_cases(ROOT, stage)
    directory = ROOT / "obj/rustc-syntax" / stage
    directory.mkdir(parents=True, exist_ok=True)

    def worker(case):
        check_classification(case)
        metadata = case["rustc"]
        kind = metadata["kind"]
        if kind == "fragment":
            return False
        if kind not in ("program", "library", "predicate", "expression"):
            raise ValueError(f"unknown independent check kind: {kind}")
        fixture = ROOT / stage / case["path"]
        data = fixture.read_bytes()
        case_directory = directory / case["path"].removesuffix(".rs")
        if case_directory.exists():
            shutil.rmtree(case_directory)
        case_directory.mkdir(parents=True)
        source = case_directory / fixture.name
        suffix = "" if kind == "predicate" else ".o" if metadata.get("emit") == "obj" else ".rmeta"
        binary = case_directory / ("output" + suffix)
        for auxiliary in case.get("auxiliary", []):
            original = ROOT / stage / auxiliary
            destination = case_directory / original.relative_to(fixture.parent)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(original, destination)
        if kind == "predicate":
            source.write_text('fn main() { println!("{}", cfg!(' + metadata["predicate"] + ')); }\n')
        elif kind == "expression":
            source.write_bytes(expression_source(data, metadata))
        else:
            source.write_bytes(data)
        command = ["rustc", "--edition=2024", "--target", "x86_64-unknown-linux-gnu",
                   "-C", "panic=abort", "-C", "debug-assertions=off",
                   *rustc_arguments(case, "syntax_check"), str(source), "-o", str(binary)]
        if not metadata.get("default_lints", False):
            command += ["--cap-lints", "allow"]
        if kind != "predicate":
            mode = case.get("mode", manifest["mode"])
            command += ["--crate-type=bin" if mode == "--emit-lowir" and kind != "library" else "--crate-type=lib",
                        rustc_emission(metadata)]
        result = subprocess.run(command, capture_output=True, timeout=30, env=case_environment(case))
        (case_directory / "rustc.stderr").write_bytes(result.stderr)
        wanted = 0 if kind == "predicate" else metadata["status"]
        if result.returncode != wanted:
            raise ValueError(f"rustc disagreement: {fixture}: {result.returncode}, wanted {wanted}\n{result.stderr.decode(errors='replace')}")
        _, actual = run_case(ROOT, stage, manifest["mode"], case, [str(app)], 10)
        if actual[2] != f"{case['exit_status']}\n".encode():
            raise ValueError(f"candidate outcome disagreement: {fixture}: {actual[1]!r}")
        check_configured_owners(metadata, actual[0])
        if kind == "predicate":
            runtime = subprocess.run([str(binary)], capture_output=True, timeout=10)
            expected = b"true\n" if metadata["value"] else b"false\n"
            if runtime.returncode or runtime.stderr or runtime.stdout != expected:
                raise ValueError(f"independent predicate expectation is wrong: {fixture}: {runtime.stdout!r}")
            facts = re.findall(rb"^attribute .*? cfg=(true|false) ", actual[0], re.M)
            if facts != [expected.strip()]:
                raise ValueError(f"cfg evaluation disagrees with rustc: {fixture}: {facts!r}")
        elif "enabled" in metadata:
            enabled = not re.search(rb"^attribute .*? cfg=false ", actual[0], re.M)
            if enabled != metadata["enabled"]:
                raise ValueError(f"configuration fact disagrees with declared ownership case: {fixture}")
        return True

    with ThreadPoolExecutor(max_workers=jobs) as pool:
        count = sum(pool.map(worker, cases))
    if count == 0:
        raise ValueError("empty independent syntax check suite")
    print(f"PASS: {stage}: {count} independent rustc checks; {len(cases) - count} declared stream fragments")
    differences = sum(c.get("rustc", {}).get("status", 0) != c["exit_status"]
                      for c in cases if c["rustc"]["kind"] != "fragment")
    if differences:
        print(f"  {differences} documented acceptance differences, not acceptance equivalence")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage")
    parser.add_argument("--app", type=Path, default=ROOT / "dev/rustgm")
    parser.add_argument("--jobs", type=int, choices=range(1, 9))
    args = parser.parse_args()
    try:
        check(args.stage, args.app.resolve(), args.jobs)
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
