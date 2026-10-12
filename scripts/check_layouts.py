#!/usr/bin/env python3
"""Check declared layout vectors against the text view and guaranteed rustc layouts."""
import argparse
import ast
import json
from pathlib import Path
import re
import subprocess
import sys

from backend import digest
from course import ROOT, case_environment, load_cases, run_case
from check_syntax import check_classification, rustc_arguments


def classification(case):
    check_classification(case)
    if "native" in case:
        raise ValueError("layout inspection is not native execution evidence")
    if case["rustc"]["kind"] != "program":
        raise ValueError("layout checks require complete crate sources")
    checks = case.get("layout_checks", [])
    if case["exit_status"]:
        if checks:
            raise ValueError("rejected layout sources cannot declare layout values")
        return
    if case["rustc"]["status"] or not checks:
        raise ValueError("accepted layout sources need rustc acceptance and declared vectors")
    names = set()
    for check in checks:
        if check["name"] in names:
            raise ValueError("duplicate inspected definition name")
        names.add(check["name"])
        if check["size"] < 0 or check["align"] <= 0 or check["align"] & (check["align"] - 1):
            raise ValueError("invalid declared layout size/alignment")
        if check.get("rustc_layout", True) and check.get("rustc_offsets", True):
            if len(check.get("offsets", [])) != len(check.get("field_names", [])):
                raise ValueError("independent field offsets require source field names")
        elif not check.get("reason"):
            raise ValueError("a course-specific layout needs an explicit reason")


def view_values(output, checks):
    definitions = {}
    layouts = {}
    offsets = {}
    for line in output.decode().splitlines():
        if match := re.fullmatch(r'def \d+ type=(\d+) name=("(?:[^"\\]|\\.)*")(?: source=\d+:\d+)?', line):
            name = ast.literal_eval("b" + match[2]).decode("utf-8")
            definitions.setdefault(name, set()).add(int(match[1]))
        elif match := re.fullmatch(r"layout (\d+) size=(\d+) align=(\d+) inhabited=(?:true|false) references=(?:true|false) .+", line):
            identity = int(match[1])
            if identity in layouts:
                raise ValueError("duplicate layout record")
            layouts[identity] = (int(match[2]), int(match[3]))
        elif match := re.fullmatch(r"field (\d+):(\d+):(\d+) type=\d+ offset=(\d+)", line):
            key = tuple(map(int, match.group(1, 2, 3)))
            if key in offsets:
                raise ValueError("duplicate field offset")
            offsets[key] = int(match[4])
    for check in checks:
        candidates = definitions.get(check["name"], set())
        if len(candidates) != 1:
            raise ValueError(f"missing or ambiguous inspected type {check['name']}")
        identity = next(iter(candidates))
        if layouts.get(identity) != (check["size"], check["align"]):
            raise ValueError(f"layout differs for {check['name']}: {layouts.get(identity)}")
        if "offsets" in check:
            actual = {field: offset for (ty, variant, field), offset in offsets.items()
                      if ty == identity and variant == 0}
            if actual != dict(enumerate(check["offsets"])):
                raise ValueError(f"field offsets differ for {check['name']}: {actual}")


def probe(checks):
    statements = []
    expected = []
    for check in checks:
        if not check.get("rustc_layout", True):
            continue
        ty = check["rust_type"]
        statements.append(f'println!("{{}} {{}}",std::mem::size_of::<{ty}>(),std::mem::align_of::<{ty}>());')
        expected.append(f"{check['size']} {check['align']}\n")
        fields = check.get("field_names", []) if check.get("rustc_offsets", True) else []
        for name, offset in zip(fields, check.get("offsets", [])):
            statements.append(f'println!("{{}}",std::mem::offset_of!({ty},{name}));')
            expected.append(f"{offset}\n")
    return "\nfn main(){\n" + "\n".join(statements) + "\n}\n", "".join(expected).encode()


def check(stage, app):
    version = subprocess.check_output(["rustc", "--version"], text=True).strip()
    if not version.startswith("rustc 1.98.0 "):
        raise ValueError("layout checks require pinned rustc 1.98.0")
    manifest, cases = load_cases(ROOT, stage)
    cases = [c for c in cases if c.get("mode", manifest["mode"]) == "--emit-layouts"]
    if not cases:
        raise ValueError("assignment has no declared layout cases")
    directory = ROOT / "obj/layouts" / stage
    directory.mkdir(parents=True, exist_ok=True)
    candidate = digest(app)
    records = []
    for ordinal, case in enumerate(cases):
        classification(case)
        folder = directory / str(ordinal)
        folder.mkdir(exist_ok=True)
        source = ROOT / stage / case["path"]
        _, output = run_case(ROOT, stage, manifest["mode"], case, [str(app)], 30)
        for suffix, data in zip(("stdout", "stderr", "status"), output):
            (folder / ("frontend." + suffix)).write_bytes(data)
        if output[2] != f"{case['exit_status']}\n".encode():
            raise ValueError(f"frontend outcome differs: {case['path']}: {output[1]!r}")
        if case["exit_status"] == 0 and (not output[0] or output[1]):
            raise ValueError("layout success needs nonempty output and empty diagnostics")
        if case["exit_status"] and (output[0] or not output[1]):
            raise ValueError("layout rejection needs empty output and a diagnostic")
        # Compile the original fixture at its original path, so declared module
        # resolution and cfg arguments have the same source base as the frontend.
        command = ["rustc", "--edition=2024", "--target", "x86_64-unknown-linux-gnu",
                   *rustc_arguments(case, "layout_check"), str(source),
                   "--crate-type=lib", "--emit=metadata", "-o", str(folder / "output.rmeta")]
        rust = subprocess.run(command, capture_output=True, timeout=30, env=case_environment(case))
        (folder / "rustc.stderr").write_bytes(rust.stderr)
        if rust.returncode != case["rustc"]["status"]:
            raise ValueError(f"rustc outcome differs: {case['path']}: {rust.stderr!r}")
        if not case["exit_status"]:
            checks = case["layout_checks"]
            view_values(output[0], checks)
            oracle = Path(str(source) + ".ref")
            if oracle.exists():
                view_values(oracle.read_bytes(), checks)
            program, expected = probe(checks)
            # Required value probes are self-contained declaration fixtures.
            # Metadata-only package checks can still declare a rejected outcome.
            if case.get("auxiliary"):
                raise ValueError("layout value probes require self-contained source fixtures")
            copied = folder / "probe.rs"
            copied.write_bytes(source.read_bytes() + program.encode())
            binary = folder / "probe"
            compile_probe = ["rustc", "--edition=2024", "--target", "x86_64-unknown-linux-gnu",
                             *rustc_arguments(case, "layout_probe"), str(copied), "-o", str(binary)]
            built = subprocess.run(compile_probe, capture_output=True, timeout=30, env=case_environment(case))
            (folder / "probe.stderr").write_bytes(built.stderr)
            if built.returncode:
                raise ValueError(f"independent layout probe failed: {built.stderr!r}")
            executed = subprocess.run([str(binary)], capture_output=True, timeout=10, env=case_environment(case))
            (folder / "probe.stdout").write_bytes(executed.stdout)
            if executed.returncode or executed.stderr or executed.stdout != expected:
                raise ValueError(f"independent layout values differ: {case['path']}")
        records.append({"path": case["path"], "source_sha256": digest(source), "passed": True})
    if digest(app) != candidate:
        raise ValueError("layout candidate changed during checks")
    (directory / "evidence.json").write_text(json.dumps({"candidate_sha256": candidate,
        "manifest_sha256": digest(ROOT / stage / "tests/manifest.json"), "rustc": version, "cases": records}, indent=2) + "\n")
    print(f"PASS: {stage}: {len(records)} independent layout cases")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", nargs="?", default="pa12")
    parser.add_argument("--app", type=Path, default=ROOT / "dev/rustgm")
    args = parser.parse_args()
    try:
        check(args.stage, args.app.resolve())
        return 0
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        print(f"layout check failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
