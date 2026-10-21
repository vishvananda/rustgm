"""Scoped oracle provenance overlays an immutable full-stage reference receipt."""
import hashlib
import json
from pathlib import Path
import time

SUFFIXES = (".ref", ".ref.stderr", ".ref.exit_status")


def digest(data):
    return hashlib.sha256(data).hexdigest()


def case_digest(case):
    return digest(json.dumps(case, sort_keys=True, separators=(",", ":")).encode())


def select(cases, paths, stage):
    if paths is None:
        return cases
    from course import check_relative_path
    if not paths:
        raise ValueError(f"{stage}: an empty case selection cannot pass")
    for path in paths:
        check_relative_path(path, stage)
    if len(paths) != len(set(paths)):
        raise ValueError(f"{stage}: duplicate case selector")
    known = {case["path"] for case in cases}
    unknown = set(paths) - known
    if unknown:
        raise ValueError(f"{stage}: unknown case selectors: {sorted(unknown)}")
    selected = set(paths)
    return [case for case in cases if case["path"] in selected]


def prepare(root, stage, manifest, source_hashes, results):
    folder = root / stage
    base = folder / "reference.json"
    if not base.is_file():
        raise ValueError(f"{stage}: partial generation requires a full reference receipt")
    # Parse before any sidecar write; malformed base provenance must not authorize a batch.
    receipt = json.loads(base.read_text())
    if receipt.get("schema") != 1 or not isinstance(receipt.get("fixtures"), dict):
        raise ValueError(f"{stage}: malformed full reference receipt")
    selected = {}
    for case, fixture, actual, errors in results:
        if errors or actual is None:
            raise ValueError(f"{stage}: failed cases cannot publish partial references")
        selected[case["path"]] = {
            "case_sha256": case_digest(case),
            "input_sha256": digest(fixture.read_bytes()),
            "auxiliary": {path: digest((folder / path).read_bytes())
                          for path in case.get("auxiliary", [])},
            "outputs": {suffix: digest(value) for suffix, value in zip(SUFFIXES, actual)},
        }
    batch = {"schema": 1, "kind": "partial-reference", "stage": stage,
             "mode": manifest["mode"], "base_reference_sha256": digest(base.read_bytes()),
             "manifest_sha256": digest((folder / "tests/manifest.json").read_bytes()),
             "compiler_source_sha256": source_hashes,
             "selected_paths": list(selected), "cases": selected}
    data = (json.dumps(batch, indent=2) + "\n").encode()
    path = folder / "reference-batches" / f"{time.time_ns()}-{digest(data)[:12]}.json"
    return path, data


def validate(root, stage, manifest, cases):
    folder = root / stage
    base_bytes = (folder / "reference.json").read_bytes()
    receipt = json.loads(base_bytes)
    recorded = dict(receipt["fixtures"])
    auxiliary = dict(receipt.get("auxiliary", {}))
    current_manifest = digest((folder / "tests/manifest.json").read_bytes())
    latest = {}
    manifest_proved = receipt.get("manifest_sha256") == current_manifest
    for path in sorted((folder / "reference-batches").glob("*.json")):
        batch = json.loads(path.read_text())
        if batch.get("base_reference_sha256") != digest(base_bytes):
            continue
        if (batch.get("schema") != 1 or batch.get("kind") != "partial-reference"
                or batch.get("stage") != stage or batch.get("mode") != manifest["mode"]
                or not isinstance(batch.get("compiler_source_sha256"), dict)):
            raise ValueError(f"invalid scoped reference receipt: {path}")
        selected = batch.get("selected_paths")
        entries = batch.get("cases")
        if (not isinstance(selected, list) or not selected or len(selected) != len(set(selected))
                or not isinstance(entries, dict) or set(entries) != set(selected)):
            raise ValueError(f"invalid scoped reference selection: {path}")
        from course import check_relative_path
        for selected_path in selected:
            check_relative_path(selected_path, stage)
        latest.update(entries)
        manifest_proved |= batch.get("manifest_sha256") == current_manifest
    if not manifest_proved:
        raise ValueError(f"manifest changed since reference generation: {folder}")
    current_cases = {case["path"]: case for case in cases}
    for path, entry in latest.items():
        if path not in current_cases:
            continue
        case = current_cases[path]
        if entry.get("case_sha256") != case_digest(case):
            raise ValueError(f"case contract changed since partial generation: {path}")
        if set(entry.get("outputs", {})) != set(SUFFIXES):
            raise ValueError(f"incomplete scoped output hashes: {path}")
        if set(entry.get("auxiliary", {})) != set(case.get("auxiliary", [])):
            raise ValueError(f"scoped auxiliary receipt disagrees with manifest: {path}")
        for suffix, expected in entry["outputs"].items():
            if digest(Path(str(folder / path) + suffix).read_bytes()) != expected:
                raise ValueError(f"oracle changed since partial generation: {path}{suffix}")
        if digest((folder / path).read_bytes()) != entry.get("input_sha256"):
            raise ValueError(f"fixture changed since partial generation: {path}")
        recorded[path] = entry["input_sha256"]
        auxiliary.update(entry["auxiliary"])
    return recorded, auxiliary
