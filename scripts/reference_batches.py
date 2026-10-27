"""Scoped oracle provenance overlays an immutable full-stage reference receipt."""
import hashlib
import json
from pathlib import Path
import time
from course_suites import check_identity, suite_paths

SUFFIXES = (".ref", ".ref.stderr", ".ref.exit_status")


def digest(data):
    return hashlib.sha256(data).hexdigest()


def case_digest(case, mode):
    effective = dict(case, mode=case.get("mode", mode))
    return digest(json.dumps(effective, sort_keys=True, separators=(",", ":")).encode())


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


def prepare(root, stage, manifest, all_cases, source_hashes, results, *, unit=None):
    folder = root / stage
    paths = suite_paths(root, stage, unit)
    base = paths.reference
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
            "case_sha256": case_digest(case, manifest["mode"]),
            "input_sha256": digest(fixture.read_bytes()),
            "auxiliary": {path: digest((folder / path).read_bytes())
                          for path in case.get("auxiliary", [])},
            "outputs": {suffix: digest(value) for suffix, value in zip(SUFFIXES, actual)},
        }
    validate(root, stage, manifest, all_cases, pending=selected, unit=unit)
    batch = {"schema": 1, "kind": "partial-reference", "stage": stage,
             **paths.identity,
             "mode": manifest["mode"], "base_reference_sha256": digest(base.read_bytes()),
             "manifest_sha256": digest(paths.manifest.read_bytes()),
             "compiler_source_sha256": source_hashes,
             "selected_paths": list(selected), "cases": selected}
    data = (json.dumps(batch, indent=2) + "\n").encode()
    path = paths.batches / f"{time.time_ns()}-{digest(data)[:12]}.json"
    return path, data


def _batches(suite, folder, stage, manifest, receipt, base_bytes):
    paths = {path.name: path for path in suite.batches.glob("*.json")}
    superseded = receipt.get("supersedes_reference_batches", {})
    if not isinstance(superseded, dict):
        raise ValueError(f"invalid superseded reference receipts: {folder}")
    for name, expected in superseded.items():
        if name not in paths or digest(paths[name].read_bytes()) != expected:
            raise ValueError(f"superseded reference receipt missing or changed: {name}")
    active = []
    for name, path in sorted(paths.items()):
        if name in superseded:
            continue
        batch = json.loads(path.read_text())
        check_identity(suite, batch)
        if batch.get("base_reference_sha256") != digest(base_bytes):
            raise ValueError(f"unexplained reference receipt base: {path}")
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
        active.append(batch)
    return active


def validate(root, stage, manifest, cases, *, pending=None, unit=None):
    folder = root / stage
    paths = suite_paths(root, stage, unit)
    base_bytes = paths.reference.read_bytes()
    receipt = json.loads(base_bytes)
    check_identity(paths, receipt)
    if unit is not None and (receipt.get("schema") != 1
            or receipt.get("mode") != manifest["mode"]
            or not isinstance(receipt.get("outputs"), dict)
            or not isinstance(receipt.get("compiler_source_sha256"), dict)):
        raise ValueError("invalid full unit reference receipt")
    recorded = dict(receipt["fixtures"])
    auxiliary = dict(receipt.get("auxiliary", {}))
    current_manifest = digest(paths.manifest.read_bytes())
    contracts = receipt.get("case_contracts")
    if contracts is None:
        if receipt.get("manifest_sha256") != current_manifest:
            raise ValueError(f"legacy reference receipt requires full refresh before manifest changes: {folder}")
        contracts = {case["path"]: case_digest(case, manifest["mode"]) for case in cases}
    elif not isinstance(contracts, dict):
        raise ValueError(f"invalid full reference case contracts: {folder}")
    else:
        contracts = dict(contracts)
    latest = {}
    manifest_proved = receipt.get("manifest_sha256") == current_manifest
    for batch in _batches(paths, folder, stage, manifest, receipt, base_bytes):
        latest.update(batch["cases"])
        manifest_proved |= batch.get("manifest_sha256") == current_manifest
    if pending is not None:
        latest.update(pending)
        manifest_proved = True
    if not manifest_proved:
        raise ValueError(f"manifest changed since reference generation: {folder}")
    for path, entry in latest.items():
        contracts[path] = entry.get("case_sha256")
    current_cases = {case["path"]: case for case in cases}
    for path, case in current_cases.items():
        if contracts.get(path) != case_digest(case, manifest["mode"]):
            raise ValueError(f"case contract changed since reference generation: {path}")
        if path in latest:
            entry = latest[path]
            if set(entry.get("outputs", {})) != set(SUFFIXES):
                raise ValueError(f"incomplete scoped output hashes: {path}")
            if set(entry.get("auxiliary", {})) != set(case.get("auxiliary", [])):
                raise ValueError(f"scoped auxiliary receipt disagrees with manifest: {path}")
            if pending is None or path not in pending:
                for suffix, expected in entry["outputs"].items():
                    if digest(Path(str(folder / path) + suffix).read_bytes()) != expected:
                        raise ValueError(f"oracle changed since partial generation: {path}{suffix}")
            recorded[path] = entry["input_sha256"]
            auxiliary.update(entry["auxiliary"])
        if recorded.get(path) != digest((folder / path).read_bytes()):
            raise ValueError(f"fixture changed since reference generation: {path}")
        if unit is not None and path not in latest:
            outputs = receipt["outputs"].get(path, {})
            if set(outputs) != set(SUFFIXES):
                raise ValueError(f"incomplete full unit output hashes: {path}")
            for suffix, expected in outputs.items():
                if digest(Path(str(folder / path) + suffix).read_bytes()) != expected:
                    raise ValueError(f"oracle changed since unit generation: {path}{suffix}")
    return recorded, auxiliary
