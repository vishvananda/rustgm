#!/usr/bin/env python3
"""Independent whole-program outcomes; raw lexer fragments are not programs."""
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main():
    version = subprocess.check_output(["rustc", "--version"], text=True)
    if not version.startswith("rustc 1.98.0 "):
        raise ValueError(f"expected pinned rustc 1.98.0, got {version.strip()}")
    sources = json.loads((ROOT / "third_party/sources.json").read_text())
    output = ROOT / "obj/rustc-check"
    output.mkdir(parents=True, exist_ok=True)
    count = 0
    for item in sources["files"]:
        if item["oracle"] == "lexical-fragment":
            continue
        path = ROOT / item["destination"]
        result = subprocess.run(["rustc", "--edition=2024", "--crate-name", "upstream_check",
                                 "--emit=metadata", "-o", str(output / "check.rmeta"), str(path)],
                                capture_output=True, timeout=30)
        wanted = 0 if item["oracle"] == "check-pass" else 1
        (output / (path.stem + ".stderr")).write_bytes(result.stderr)
        if result.returncode != wanted:
            raise ValueError(f"rustc outcome disagrees: {path.name}: {result.returncode}, wanted {wanted}")
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
