#!/usr/bin/env python3
"""An assignment is active only with a nonempty, fully discovered oracle suite."""
import hashlib
from pathlib import Path
import sys
import argparse
from course import ROOT, SIDECARS
from assignment_metadata import collect, collect_units
import reference_batches


def check(root=ROOT, *, unit=None):
    stages, loaded = collect(root)
    components = collect_units(root, unit=unit)
    for stage, manifest, cases in loaded:
        check_suite(root, stage["path"], manifest, cases)
    for component, manifest, cases in components:
        check_suite(root, component["owner"], manifest, cases, unit=component["id"])
    print(f"Assignment layout: {len(stages)} stages; no orphan fixtures in active suites")
    if components:
        print(f"Component layout: {len(components)} units with complete oracle provenance")


def check_suite(root, stage, manifest, cases, *, unit=None):
    folder = root / stage
    recorded, recorded_auxiliary = reference_batches.validate(root, stage, manifest, cases, unit=unit)
    expected_auxiliary = {p for case in cases for p in case.get("auxiliary", [])}
    if set(recorded_auxiliary) != expected_auxiliary:
        raise ValueError(f"auxiliary receipt disagrees with manifest: {folder}")
    for path in expected_auxiliary:
        if recorded_auxiliary[path] != hashlib.sha256((folder / path).read_bytes()).hexdigest():
            raise ValueError(f"auxiliary changed since reference generation: {folder / path}")
    for case in cases:
        fixture = folder / case["path"]
        if recorded.get(case["path"]) != hashlib.sha256(fixture.read_bytes()).hexdigest():
            raise ValueError(f"fixture changed since reference generation: {fixture}")
        for suffix in SIDECARS:
            if not Path(str(fixture) + suffix).is_file():
                raise ValueError(f"missing oracle: {fixture}{suffix}")
        status = Path(str(fixture) + ".ref.exit_status").read_text()
        if status != f"{case['exit_status']}\n":
            raise ValueError(f"oracle disagrees with declared outcome: {fixture}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--unit", help="also check one declared unpublished component")
    args = parser.parse_args()
    try:
        check(unit=args.unit)
    except (ValueError, OSError, KeyError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
