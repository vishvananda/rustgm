#!/usr/bin/env python3
"""An assignment is active only with a nonempty, fully discovered oracle suite."""
import json
import hashlib
from pathlib import Path
import re
import sys
from course import ROOT, SIDECARS, load_cases
from check_native import classification


def check(root=ROOT):
    stages = json.loads((root / "assignments.json").read_text())["stages"]
    active = [s["path"] for s in stages if s["status"] != "planned"]
    configured = re.search(r"^ACTIVE_PAS\s*:=\s*(.*)$", (root / "Makefile").read_text(), re.M)
    if configured is None or configured[1].split() != active:
        raise ValueError("root report stages disagree with assignments.json")
    for stage in stages:
        folder = root / stage["path"]
        for name in ("README.md", "Makefile", "tests/README.md"):
            if not (folder / name).is_file():
                raise ValueError(f"missing assignment document: {folder / name}")
        if stage["status"] != "planned":
            _, cases = load_cases(root, stage["path"])
            if stage["mode"] == "--emit-lowir":
                for case in cases:
                    classification(case)
            receipt = json.loads((folder / "reference.json").read_text())
            if receipt.get("manifest_sha256") != hashlib.sha256((folder / "tests/manifest.json").read_bytes()).hexdigest():
                raise ValueError(f"manifest changed since reference generation: {folder}")
            recorded = receipt["fixtures"]
            expected_auxiliary = {p for case in cases for p in case.get("auxiliary", [])}
            if set(receipt.get("auxiliary", {})) != expected_auxiliary:
                raise ValueError(f"auxiliary receipt disagrees with manifest: {folder}")
            for path in expected_auxiliary:
                if receipt["auxiliary"][path] != hashlib.sha256((folder / path).read_bytes()).hexdigest():
                    raise ValueError(f"auxiliary changed since reference generation: {folder / path}")
            for case in cases:
                fixture = folder / case["path"]
                if recorded.get(case["path"]) != hashlib.sha256(fixture.read_bytes()).hexdigest():
                    raise ValueError(f"fixture changed since reference generation: {fixture}")
                for feature in case.get("features", []):
                    owner = re.match(r"pa(\d+)\.", feature)
                    if owner and int(owner[1]) > stage["number"]:
                        raise ValueError(f"fixture uses a future-stage feature: {fixture}: {feature}")
                for suffix in SIDECARS:
                    if not Path(str(fixture) + suffix).is_file():
                        raise ValueError(f"missing oracle: {fixture}{suffix}")
                status = Path(str(fixture) + ".ref.exit_status").read_text()
                if status != f"{case['exit_status']}\n":
                    raise ValueError(f"oracle disagrees with declared outcome: {fixture}")
        elif (folder / "tests/manifest.json").exists():
            raise ValueError(f"planned stage has an undeclared executed suite: {folder}")
    print(f"Assignment layout: {len(stages)} stages; no orphan fixtures in active suites")


if __name__ == "__main__":
    try:
        check()
    except (ValueError, OSError, KeyError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
