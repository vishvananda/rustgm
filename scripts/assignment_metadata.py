#!/usr/bin/env python3
"""Validate assignment declarations without requiring generated references."""
import json
import re
import sys

from course import ROOT, load_cases
from check_native import classification
from check_layouts import classification as layout_classification


def collect(root=ROOT):
    stages = json.loads((root / "assignments.json").read_text())["stages"]
    for stage in stages:
        if stage["status"] not in ("planned", "implemented-draft"):
            raise ValueError(f"unknown assignment status: {stage['path']}: {stage['status']}")
    active = [stage["path"] for stage in stages if stage["status"] != "planned"]
    configured = re.search(r"^ACTIVE_PAS\s*:=\s*(.*)$", (root / "Makefile").read_text(), re.M)
    if configured is None or configured[1].split() != active:
        raise ValueError("root report stages disagree with assignments.json")
    loaded = []
    for stage in stages:
        folder = root / stage["path"]
        for name in ("README.md", "Makefile", "tests/README.md"):
            if not (folder / name).is_file():
                raise ValueError(f"missing assignment document: {folder / name}")
        if stage["status"] == "planned":
            if (folder / "tests/manifest.json").exists():
                raise ValueError(f"planned stage has an undeclared executed suite: {folder}")
            continue
        manifest, cases = load_cases(root, stage["path"])
        if (manifest["mode"] != stage["mode"]
                or manifest.get("additional_modes", []) != stage.get("additional_modes", [])):
            raise ValueError(f"assignment modes disagree with manifest: {folder}")
        for case in cases:
            mode = case.get("mode", manifest["mode"])
            try:
                if mode == "--emit-lowir":
                    classification(case)
                elif mode == "--emit-layouts":
                    layout_classification(case)
            except (ValueError, KeyError) as error:
                raise ValueError(f"{stage['path']}/{case['path']}: {error}") from error
            for feature in case.get("features", []):
                owner = re.match(r"pa(\d+)\.", feature)
                if owner and int(owner[1]) > stage["number"]:
                    raise ValueError(f"fixture uses a future-stage feature: {folder / case['path']}: {feature}")
        loaded.append((stage, manifest, cases))
    return stages, loaded


def main():
    try:
        stages, loaded = collect()
        print(f"Assignment metadata: {len(stages)} stages; {sum(len(cases) for _, _, cases in loaded)} active cases; references not required")
        return 0
    except (ValueError, OSError, KeyError) as error:
        print(error, file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
