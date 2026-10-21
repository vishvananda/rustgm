#!/usr/bin/env python3
"""Independent source-to-LowIR execution against pinned rustc and explicit outcomes."""
import argparse
import os
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import shutil
import signal
import subprocess
import sys

from backend import digest, load_tools
from course import ROOT, case_environment, load_cases, run_case
from check_syntax import rustc_arguments


def classification(case):
    metadata = case["rustc"]
    if metadata["kind"] != "program" or metadata["status"] not in (0, 1):
        raise ValueError("native fixtures must declare full-program rustc outcomes")
    if metadata["status"] != case["exit_status"] and not metadata.get("reason"):
        raise ValueError("acceptance differences require a phase-boundary reason")
    if case["exit_status"] == 0:
        native = case.get("native", {})
        if metadata["status"] != 0:
            raise ValueError("accepted native fixtures require successful rustc code generation")
        kind = native.get("kind", "execute")
        if kind == "codegen":
            if (not isinstance(native.get("reason"), str) or not native["reason"].strip()
                    or "outcome" in native or "rustc_outcome" in native):
                raise ValueError("codegen-only fixtures require a reason and cannot declare runtime outcomes")
        elif kind == "execute":
            if native.get("outcome") not in ("pass", "panic"):
                raise ValueError("every executed native fixture needs a declared outcome")
            independent = native.get("rustc_outcome", native["outcome"])
            if independent not in ("pass", "panic"):
                raise ValueError("rustc runtime outcomes must be pass or qualified panic")
            if independent != native["outcome"] and (
                    not isinstance(native.get("reason"), str) or not native["reason"].strip()):
                raise ValueError("runtime differences require an explicit Reference/rustc reason")
        else:
            raise ValueError("unknown native check kind")
    elif "native" in case:
        raise ValueError("rejected sources cannot declare native execution")


def execution(result, outcome, compiler):
    if outcome not in ("pass", "panic"):
        raise ValueError("unknown runtime outcome")
    if result.stdout:
        raise ValueError(f"{compiler}: unexpected runtime stdout")
    if outcome == "pass":
        if result.returncode or result.stderr:
            raise ValueError(f"{compiler}: expected clean exit 0, got {result.returncode}: {result.stderr!r}")
    elif compiler == "cppgm":
        # The pinned course runtime implements abort as exit(134), not SIGABRT.
        if result.returncode != 134 or result.stderr:
            raise ValueError(f"cppgm: expected qualified abort exit 134, got {result.returncode}")
    elif result.returncode != -signal.SIGABRT or b"panicked at" not in result.stderr:
        raise ValueError(f"rustc: expected a runtime panic with abort, got {result.returncode}")


def run_executables(metadata, rustexe, native, run):
    """Compilation-only controls can diverge; never execute either produced binary."""
    if metadata.get("kind", "execute") == "codegen":
        return
    outcome = metadata["outcome"]
    independent = metadata.get("rustc_outcome", outcome)
    execution(run([str(rustexe)], "rustc-run", 10), independent, "rustc")
    execution(run([str(native)], "native-run", 10), outcome, "cppgm")


def native_cases(manifest, cases):
    selected = [case for case in cases if case.get("mode", manifest["mode"]) == "--emit-lowir"]
    if not selected:
        raise ValueError("native assignment has no LowIR cases")
    for case in selected:
        classification(case)
    return selected


def check(app, jobs, stage="pa11"):
    version = subprocess.check_output(["rustc", "--version"], text=True)
    if not version.startswith("rustc 1.98.0 "):
        raise ValueError(f"expected pinned rustc 1.98.0: {version.strip()}")
    tools = load_tools()  # Hash the immutable backend tools once for this run.
    candidate_hash = digest(app)
    manifest, cases = load_cases(ROOT, stage)
    cases = native_cases(manifest, cases)
    directory = ROOT / "obj/native" / stage
    directory.mkdir(parents=True, exist_ok=True)

    def worker(case):
        evidence = {"path": case["path"], "commands": [], "passed": False}
        folder = directory / case["path"].removesuffix(".rs")
        try:
            classification(case)
            if folder.exists():
                shutil.rmtree(folder)
            folder.mkdir(parents=True)
            fixture = ROOT / stage / case["path"]
            source = folder / fixture.name
            source.write_bytes(fixture.read_bytes())
            for auxiliary in case.get("auxiliary", []):
                original = ROOT / stage / auxiliary
                destination = folder / original.relative_to(fixture.parent)
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(original, destination)
            env = case_environment(case)

            def run(command, label, timeout=60):
                result = subprocess.run(command, capture_output=True, timeout=timeout, cwd=ROOT, env=env)
                evidence["commands"].append({"argv": command, "status": result.returncode})
                (folder / (label + ".stdout")).write_bytes(result.stdout)
                (folder / (label + ".stderr")).write_bytes(result.stderr)
                return result

            _, actual = run_case(ROOT, stage, manifest["mode"], case, [str(app)], 30)
            for suffix, data in zip(("stdout", "stderr", "status"), actual):
                (folder / ("frontend." + suffix)).write_bytes(data)
            if actual[2] != f"{case['exit_status']}\n".encode():
                raise ValueError(f"frontend outcome differs: {actual[2]!r}, {actual[1]!r}")
            if not case["exit_status"] and (actual[1] or not actual[0]):
                raise ValueError("successful LowIR emission requires nonempty stdout and empty stderr")
            if case["exit_status"] and (actual[0] or not actual[1]):
                raise ValueError("rejections require empty stdout and a diagnostic")
            rustexe = folder / "rustc-output"
            command = ["rustc", "--edition=2024", "--target", "x86_64-unknown-linux-gnu",
                       *rustc_arguments(case, "native_check"), "-C", "panic=abort", "-C", "overflow-checks=yes",
                       "-C", "debug-assertions=off", str(source), "-o", str(rustexe)]
            rust = run(command, "rustc")
            if rust.returncode != case["rustc"]["status"]:
                raise ValueError(f"rustc outcome differs: {rust.returncode}: {rust.stderr.decode(errors='replace')}")
            if case["exit_status"] == 0:
                text = folder / "output.lowir"
                text.write_bytes(actual[0])
                validated = folder / "validated.lowir"
                native = folder / "native-output"
                for command, label in [([tools["lowir"], "-o", str(validated), str(text)], "validate"),
                                       ([tools["lowir2native"], "-O0", "-o", str(native), str(validated)], "backend")]:
                    result = run(command, label)
                    if result.returncode:
                        raise ValueError(f"{label} failed ({result.returncode}): {result.stderr.decode(errors='replace')}")
                evidence["native"] = case["native"]
                run_executables(case["native"], rustexe, native, run)
            for executable in (rustexe, folder / "native-output"):
                executable.unlink(missing_ok=True)
            evidence["passed"] = True
        except (ValueError, OSError, subprocess.SubprocessError) as error:
            evidence["error"] = str(error)
        return evidence

    with ThreadPoolExecutor(max_workers=jobs) as pool:
        results = list(pool.map(worker, cases))
    if digest(app) != candidate_hash:
        raise ValueError("candidate bytes changed during the independent run")
    (directory / "evidence.json").write_text(json.dumps({"rustc": version.strip(), "candidate_sha256": candidate_hash,
        "manifest_sha256": digest(ROOT / stage / "tests/manifest.json"),
        "backend": json.loads((ROOT / "backend.lock.json").read_text()), "cases": results}, indent=2) + "\n")
    failures = [item for item in results if not item["passed"]]
    for item in failures:
        print(f"FAIL {item['path']}: {item['error']}")
    if failures:
        raise ValueError(f"{len(failures)}/{len(cases)} independent native checks failed")
    accepted = [c["native"] for c in cases if "native" in c]
    compiled = sum(n.get("kind", "execute") == "codegen" for n in accepted)
    print(f"PASS: {stage.upper()}: {len(cases)} independent checks; {len(accepted) - compiled} executable comparisons; {compiled} codegen-only comparisons")
    differences = sum(c.get("native", {}).get("rustc_outcome", c.get("native", {}).get("outcome"))
                      != c.get("native", {}).get("outcome") for c in cases)
    if differences:
        print(f"  {differences} documented runtime differences, not same-outcome parity")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", nargs="?", default="pa11")
    parser.add_argument("--app", type=Path, default=ROOT / "dev/rustgm")
    parser.add_argument("--jobs", type=int, choices=range(1, 9),
                        default=int(os.environ.get("RUSTGM_TEST_JOBS", "4")))
    args = parser.parse_args()
    try:
        check(args.app.resolve(), args.jobs, args.stage)
        return 0
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"native check failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
