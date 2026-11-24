#!/usr/bin/env python3
"""Check declared bootstrap lint outcomes against pinned rustc and a prebuilt RustGM."""
import argparse
import json
from pathlib import Path
import re

from crate_qualification import Check


def diagnostics(stderr):
    return [row["code"]["code"] for line in stderr.splitlines()
            if (row := json.loads(line)).get("code")]


def check_case(check, manifest, case, index, *, source_core, edition, bootstrap, oracle_only):
    name = f"{manifest.parent.name}-{index}"
    source = manifest.parent / case["path"]
    if not source.resolve().is_relative_to(manifest.parent.resolve()) or not source.is_file():
        raise ValueError(f"{name}: invalid fixture path {source}")
    cfg = ["--cfg", case["cfg"]] if "cfg" in case else []
    status = case["status"]
    lint = case["lint"]
    if (status, lint is not None) not in ((0, False), (1, True)):
        raise ValueError(f"{name}: declare a diagnostic for each rejection")

    rustc = ["rustc", "--crate-name", "lint_case", "--crate-type=lib", f"--edition={edition}",
             "--emit=metadata", "--error-format=json", "-Cpanic=abort",
             "-o", check.folder / f"{name}.rmeta", *cfg, source]
    reference = check.run(f"{name}-rustc", rustc, status, timeout=15,
                          environment={"RUSTC_BOOTSTRAP": "1"} if bootstrap else None)
    if diagnostics(reference.stderr) != ([lint] if lint else []):
        raise ValueError(f"{name}: independent lint verdict differs\n"
                         + reference.stderr.decode(errors="replace"))
    if oracle_only:
        return

    args = [check.app, "--emit-types", f"--edition={edition}", *cfg]
    if source_core:
        args += ["--source-core", "lint_case@lintcase",
                 "--crate-name", "lint_case", "--crate-disambiguator", "lintcase"]
    candidate = check.run(f"{name}-rustgm", [*args, source], status, timeout=15,
                          codes=(case["diagnostic"],) if lint else ())
    if re.search(rb"^warning\[", candidate.stderr, re.M):
        raise ValueError(f"{name}: unexpected warning\n"
                         + candidate.stderr.decode(errors="replace"))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    compiler = parser.add_mutually_exclusive_group(required=True)
    compiler.add_argument("--app", type=Path)
    compiler.add_argument("--oracle-only", action="store_true")
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("manifests", nargs="+", type=Path)
    args = parser.parse_args()
    check = Check(args.app or Path("rustc"), args.out)
    count = 0
    failures = []
    for manifest in args.manifests:
        plan = json.loads(manifest.read_text())
        if not plan["cases"]:
            raise ValueError(f"{manifest}: empty lint suite")
        source_core = plan.get("source_core", False)
        for index, case in enumerate(plan["cases"]):
            try:
                check_case(check, manifest, case, index, source_core=source_core,
                           edition=plan.get("edition", "2024"),
                           bootstrap=source_core or plan.get("bootstrap", False),
                           oracle_only=args.oracle_only)
            except ValueError as error:
                failures.append(str(error))
            count += 1
    if failures:
        parser.exit(1, "\n".join(failures) + "\n")
    print(f"Verified {count} declared bootstrap lint cases")


if __name__ == "__main__":
    main()
