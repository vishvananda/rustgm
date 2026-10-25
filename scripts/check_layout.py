#!/usr/bin/env python3
"""An assignment is active only with a nonempty, fully discovered oracle suite."""
import hashlib
from pathlib import Path
import sys
from course import ROOT, SIDECARS
from assignment_metadata import collect
import reference_batches


def check(root=ROOT):
    stages, loaded = collect(root)
    for stage, manifest, cases in loaded:
        folder = root / stage["path"]
        recorded, recorded_auxiliary = reference_batches.validate(root, stage["path"], manifest, cases)
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
    print(f"Assignment layout: {len(stages)} stages; no orphan fixtures in active suites")


if __name__ == "__main__":
    try:
        check()
    except (ValueError, OSError, KeyError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
