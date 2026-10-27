#!/usr/bin/env python3
"""Independent source-to-LowIR execution against pinned rustc and explicit outcomes."""
import argparse
import os
import re
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import shutil
import signal
import subprocess
import sys

from backend import digest, emit_object, load_tools, verify_object
from course import ROOT, case_environment, library_arguments, load_cases, run_case
from course_suites import run_released, suite_paths
from check_syntax import rustc_arguments


REVIEWED_WARNING_CODES = (
    "unused_must_use", "unused_attributes", "duplicate_macro_attributes",
    "malformed_diagnostic_attributes", "malformed_diagnostic_format_literals",
    "misplaced_diagnostic_attributes",
    "unsafe_op_in_unsafe_fn", "static_mut_refs",
)


def classification(case):
    metadata = case["rustc"]
    library = metadata["kind"] == "library"
    if metadata["kind"] not in ("program", "library") or metadata["status"] not in (0, 1):
        raise ValueError("native fixtures must declare complete program or library rustc outcomes")
    if "warnings" in case:
        warnings = case["warnings"]
        if (case["exit_status"] != 0 or not isinstance(warnings, list) or not warnings
                or any(code not in REVIEWED_WARNING_CODES for code in warnings)
                or len(set(warnings)) != len(warnings)):
            raise ValueError("successful warnings require explicit unique reviewed manifest codes")
    library_arguments(case, "--emit-lowir")
    if metadata["status"] != case["exit_status"] and not metadata.get("reason"):
        raise ValueError("acceptance differences require a phase-boundary reason")
    if case["exit_status"] == 0:
        native = case.get("native", {})
        if metadata["status"] != 0:
            raise ValueError("accepted native fixtures require successful rustc code generation")
        kind = native.get("kind", "execute")
        if library != (kind == "object"):
            raise ValueError("library native checks require object emission; programs require executables")
        if kind in ("codegen", "object"):
            if (not isinstance(native.get("reason"), str) or not native["reason"].strip()
                    or "outcome" in native or "rustc_outcome" in native):
                raise ValueError("compile-only fixtures require a reason and cannot declare runtime outcomes")
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
    if metadata.get("kind", "execute") in ("codegen", "object"):
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


def accepted_frontend(actual, expected_stderr=b"", declared_codes=()):
    """Accept only the exact independently reviewed successful-warning reference."""
    stdout, stderr, status = actual
    if status != b"0\n" or not stdout:
        raise ValueError("successful LowIR emission requires exit 0 and nonempty stdout")
    if stderr != expected_stderr:
        raise ValueError("successful frontend stderr differs from its declared reference")
    if not expected_stderr:
        if declared_codes:
            raise ValueError("declared successful warnings have an empty diagnostic reference")
        return
    try:
        lines = expected_stderr.decode("utf-8").splitlines()
    except UnicodeError as error:
        raise ValueError("successful frontend warnings must be UTF-8 diagnostics") from error
    header = re.compile(r"warning\[(" + "|".join(REVIEWED_WARNING_CODES) + r")\]: file [0-9]+: [0-9]+: .+")
    if not lines or header.fullmatch(lines[0]) is None:
        raise ValueError("successful frontend stderr requires a declared warning code")
    actual_codes = {match[1] for line in lines if (match := header.fullmatch(line))}
    if actual_codes != set(declared_codes):
        raise ValueError("successful warning codes differ from the explicit manifest declaration")
    for line in lines:
        if line.startswith(("ERROR:", "error:", "error[")):
            raise ValueError("an error diagnostic cannot masquerade as a successful warning")
        if line.startswith("warning[") and header.fullmatch(line) is None:
            raise ValueError("successful frontend warning code was not reviewed")


def check(app, jobs, stage="pa11", *, unit=None):
    paths = suite_paths(ROOT, stage, unit)
    manifest, cases = load_cases(ROOT, stage, unit=unit)
    cases = native_cases(manifest, cases)
    version = subprocess.check_output(["rustc", "--version"], text=True)
    if not version.startswith("rustc 1.98.0 "):
        raise ValueError(f"expected pinned rustc 1.98.0: {version.strip()}")
    candidate_hash = digest(app)
    required = {"lowir", "lowir2native"} if any(c["rustc"]["kind"] == "program" for c in cases) else set()
    if any(c.get("native", {}).get("kind") == "object" for c in cases):
        required.add("cppgm++")
    tools = load_tools(tuple(sorted(required)))  # Hash each required immutable tool once.
    directory = ROOT / "obj/native" / paths.evidence
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
            if not case["exit_status"]:
                warning_reference = Path(str(fixture) + ".ref.stderr")
                accepted_frontend(actual, warning_reference.read_bytes(), case.get("warnings", []))
                evidence["warning_reference"] = str(warning_reference.relative_to(ROOT))
                evidence["warning_reference_sha256"] = digest(warning_reference)
                evidence["declared_warning_codes"] = case.get("warnings", [])
            if case["exit_status"] and (actual[0] or not actual[1]):
                raise ValueError("rejections require empty stdout and a diagnostic")
            library = case["rustc"]["kind"] == "library"
            rustexe = folder / ("rustc-output.o" if library else "rustc-output")
            command = ["rustc", "--edition=2024", "--target", "x86_64-unknown-linux-gnu",
                       *rustc_arguments(case, "native_check"), "-C", "panic=abort", "-C", "overflow-checks=yes",
                       "-C", "debug-assertions=off", str(source), "-o", str(rustexe)]
            if library:
                command += ["--crate-type=lib", "--emit=obj"]
            rust = run(command, "rustc")
            if rust.returncode != case["rustc"]["status"]:
                raise ValueError(f"rustc outcome differs: {rust.returncode}: {rust.stderr.decode(errors='replace')}")
            if case["exit_status"] == 0:
                text = folder / "output.lowir"
                text.write_bytes(actual[0])
                native = folder / ("native-output.o" if library else "native-output")
                if library:
                    verify_object(rustexe)
                    def object_run(command):
                        result = run(command, "backend-object")
                        if result.returncode:
                            raise ValueError(f"object backend failed ({result.returncode}): {result.stderr.decode(errors='replace')}")
                    emit_object(text, native, tools, run=object_run)
                    evidence["objects"] = {"rustc_sha256": digest(rustexe), "native_sha256": digest(native)}
                else:
                    validated = folder / "validated.lowir"
                    for command, label in [([tools["lowir"], "-o", str(validated), str(text)], "validate"),
                                           ([tools["lowir2native"], "-O0", "-o", str(native), str(validated)], "backend")]:
                        result = run(command, label)
                        if result.returncode:
                            raise ValueError(f"{label} failed ({result.returncode}): {result.stderr.decode(errors='replace')}")
                evidence["native"] = case["native"]
                run_executables(case["native"], rustexe, native, run)
            for executable in (rustexe, folder / "native-output", folder / "native-output.o"):
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
        **paths.identity, "manifest_sha256": digest(paths.manifest),
        "backend": json.loads((ROOT / "backend.lock.json").read_text()), "cases": results}, indent=2) + "\n")
    failures = [item for item in results if not item["passed"]]
    for item in failures:
        print(f"FAIL {item['path']}: {item['error']}")
    if failures:
        raise ValueError(f"{len(failures)}/{len(cases)} independent native checks failed")
    accepted = [c["native"] for c in cases if "native" in c]
    compiled = sum(n.get("kind", "execute") == "codegen" for n in accepted)
    objects = sum(n.get("kind") == "object" for n in accepted)
    print(f"PASS: {stage.upper()}: {len(cases)} independent checks; {len(accepted) - compiled - objects} executable comparisons; {compiled} codegen-only comparisons; {objects} object comparisons")
    differences = sum(c.get("native", {}).get("rustc_outcome", c.get("native", {}).get("outcome"))
                      != c.get("native", {}).get("outcome") for c in cases)
    if differences:
        print(f"  {differences} documented runtime differences, not same-outcome parity")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", nargs="?")
    parser.add_argument("--released-units", action="store_true")
    parser.add_argument("--unit", help="exact declared component owned by this assignment")
    parser.add_argument("--app", type=Path, default=ROOT / "dev/rustgm")
    parser.add_argument("--jobs", type=int, choices=range(1, 9),
                        default=int(os.environ.get("RUSTGM_TEST_JOBS", "4")))
    args = parser.parse_args()
    if args.released_units and (args.stage or args.unit):
        parser.error("released-unit scheduling cannot select a stage or draft unit")
    try:
        if args.released_units:
            run_released(ROOT, lambda stage, unit: check(args.app.resolve(), args.jobs, stage, unit=unit))
        else:
            check(args.app.resolve(), args.jobs, args.stage or "pa11", unit=args.unit)
        return 0
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"native check failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
