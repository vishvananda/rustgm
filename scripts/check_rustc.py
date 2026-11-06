#!/usr/bin/env python3
"""Independent whole-program outcomes; raw lexer fragments are not programs."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def program_case(root, item):
    """Retained modules use their complete, independently frozen provider crate."""
    oracle = item["oracle"]
    if oracle == "lexical-fragment":
        return None
    if oracle == "source-fragment":
        companion = item.get("oracle_companion")
        if companion is None:
            return None
        relative = Path(companion["path"])
        if relative.is_absolute() or ".." in relative.parts:
            raise ValueError("oracle companion must be a repository fixture")
        path = root / relative
        data = path.read_bytes()
        if hashlib.sha256(data).hexdigest() != companion["sha256"]:
            raise ValueError(f"oracle companion bytes changed: {relative}")
        if (root / item["destination"]).read_bytes() not in data:
            raise ValueError(f"oracle companion does not embed its original source: {relative}")
        wanted = companion["expected_status"]
        if type(wanted) is not int or wanted not in (0, 1):
            raise ValueError("oracle companion must declare success or rejection")
        return path, wanted, ["--crate-type=lib"]
    if oracle not in ("check-pass", "compile-fail"):
        raise ValueError(f"unknown upstream oracle: {oracle}")
    return root / item["destination"], 0 if oracle == "check-pass" else 1, []


def main():
    version = subprocess.check_output(["rustc", "--version"], text=True)
    if not version.startswith("rustc 1.98.0 "):
        raise ValueError(f"expected pinned rustc 1.98.0, got {version.strip()}")
    sources = json.loads((ROOT / "third_party/sources.json").read_text())
    output = ROOT / "obj/rustc-check"
    output.mkdir(parents=True, exist_ok=True)
    count = 0
    for item in sources["files"]:
        case = program_case(ROOT, item)
        if case is None:
            continue
        path, wanted, flags = case
        result = subprocess.run(["rustc", "--edition=2024", "--crate-name", "upstream_check",
                                 "--emit=metadata", *flags, "-o", str(output / "check.rmeta"), str(path)],
                                capture_output=True, timeout=30,
                                env={**os.environ, "RUSTC_BOOTSTRAP": "1"})
        label = "--".join(path.relative_to(ROOT).parts)
        (output / (label + ".stderr")).write_bytes(result.stderr)
        if result.returncode != wanted:
            raise ValueError(f"rustc outcome disagrees: {path.relative_to(ROOT)}: "
                             f"{result.returncode}, wanted {wanted}; see {output / (label + '.stderr')}")
        count += 1
    if not count:
        raise ValueError("no independent whole-program checks")
    print(f"PASS: {count} independent rustc acceptance/rejection checks")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
