"""Resolve course-owned suite identities and their explicitly claimed source files."""
from dataclasses import dataclass
import json
from pathlib import Path
import re


def unit_manifest(stage, unit):
    if (not isinstance(stage, str) or not re.fullmatch(r"pa[1-9][0-9]*", stage)
            or not isinstance(unit, str) or not re.fullmatch(r"[a-z]+(?:-[a-z0-9]+)*", unit)):
        raise ValueError("invalid assignment/unit identity")
    return f"{stage}/tests/units/{unit}.json"


@dataclass(frozen=True)
class SuitePaths:
    manifest: Path
    reference: Path
    batches: Path
    evidence: Path
    identity: dict


def suite_paths(root, stage, unit=None):
    folder = root / stage
    if unit is None:
        return SuitePaths(folder / "tests/manifest.json", folder / "reference.json",
                          folder / "reference-batches", Path(stage), {})
    manifest = unit_manifest(stage, unit)
    matches = [entry for entry in json.loads((root / "assignment-plan.json").read_text())["units"]
               if entry["id"] == unit]
    if len(matches) != 1 or matches[0]["owner"] != stage:
        raise ValueError(f"unknown or non-owning assignment unit: {stage}: {unit}")
    if matches[0].get("manifest") != manifest:
        raise ValueError(f"undeclared or noncanonical unit manifest: {stage}: {unit}")
    folder = folder / "units" / unit
    paths = SuitePaths(root / manifest, folder / "reference.json", folder / "reference-batches",
                       Path(stage) / "units" / unit,
                       {"stage": stage, "unit": unit, "manifest_path": manifest})
    for path in (paths.manifest, paths.reference, paths.batches):
        if not path.resolve().is_relative_to((root / stage).resolve()):
            raise ValueError(f"unit suite path escapes assignment: {path}")
        for parent in (path, *path.parents):
            if parent == root:
                break
            if parent.is_symlink():
                raise ValueError(f"unit suite path uses a symlink: {path}")
    return paths


def check_identity(paths, receipt):
    if paths.identity:
        if any(receipt.get(key) != value for key, value in paths.identity.items()):
            raise ValueError("reference receipt has a different assignment/unit/manifest identity")
    elif receipt.get("unit") is not None:
        raise ValueError("unit reference receipt cannot authorize a whole assignment")


def discovery(root, stage, manifest, unit):
    folder = root / stage
    tests = (folder / "tests").resolve()
    if unit is None:
        return {p.relative_to(folder).as_posix() for p in (folder / "tests").rglob("*.rs")}
    from course import check_relative_path
    claims = manifest.get("discovery")
    if not isinstance(claims, list) or not claims:
        raise ValueError("unit discovery must be a nonempty list")
    claimed = []
    found = set()
    for name in claims:
        check_relative_path(name, stage)
        path = folder / name
        resolved = path.resolve()
        if (not resolved.is_relative_to(tests) or not path.exists()
                or any(resolved == previous or resolved.is_relative_to(previous)
                       or previous.is_relative_to(resolved) for previous in claimed)):
            raise ValueError(f"missing, escaping or overlapping unit discovery: {name}")
        claimed.append(resolved)
        files = list(path.rglob("*.rs")) if path.is_dir() else [path]
        if not files:
            raise ValueError(f"empty unit discovery: {name}")
        for source in files:
            if not source.is_file() or source.suffix != ".rs" or not source.resolve().is_relative_to(tests):
                raise ValueError(f"invalid unit discovery source: {source}")
            found.add(source.relative_to(folder).as_posix())
    return found


def check_publishers(root):
    """Sidecars have one component publisher; auxiliary source can be shared."""
    from reference_batches import case_digest
    units = [entry for entry in json.loads((root / "assignment-plan.json").read_text())["units"]
             if "manifest" in entry]
    if not units:
        return
    owners = {unit["owner"] for unit in units}
    parents = {}
    for stage in json.loads((root / "assignments.json").read_text())["stages"]:
        if stage["path"] in owners and stage["status"] != "planned":
            folder = root / stage["path"]
            manifest = json.loads((folder / "tests/manifest.json").read_text())
            parents.update({(folder / case["path"]).resolve(): case_digest(case, manifest["mode"])
                            for case in manifest["cases"]})
    primary, auxiliary = {}, set()
    for unit in units:
        paths = suite_paths(root, unit["owner"], unit["id"])
        manifest = json.loads(paths.manifest.read_text())
        for case in manifest["cases"]:
            from course import check_relative_path
            check_relative_path(case["path"], unit["owner"])
            key = (root / unit["owner"] / case["path"]).resolve()
            if key in primary:
                raise ValueError(f"duplicate unit primary publisher: {key}")
            if key in parents and parents[key] != case_digest(case, manifest["mode"]):
                raise ValueError(f"unit case contract disagrees with active parent: {key}")
            primary[key] = unit["id"]
            auxiliary.update((root / unit["owner"] / name).resolve()
                             for name in case.get("auxiliary", []))
    if set(primary) & auxiliary:
        raise ValueError("unit primary source is also claimed as an auxiliary")


def shared_outputs(root, stage, results):
    """A unit can adopt existing parent outputs, but only the parent changes them."""
    stages = json.loads((root / "assignments.json").read_text())["stages"]
    if not any(entry["path"] == stage and entry["status"] != "planned" for entry in stages):
        return
    folder = root / stage
    manifest = json.loads((folder / "tests/manifest.json").read_text())
    shared = {(folder / case["path"]).resolve() for case in manifest["cases"]}
    from reference_batches import SUFFIXES
    for _, fixture, actual, _ in results:
        if fixture.resolve() in shared and any(
                not Path(str(fixture) + suffix).is_file()
                or Path(str(fixture) + suffix).read_bytes() != value
                for suffix, value in zip(SUFFIXES, actual)):
            raise ValueError(f"shared unit outputs must first be generated by their active parent: {fixture}")


def run_released(root, check):
    """Schedule only declared releases through the existing selected-suite checks."""
    from assignment_metadata import collect_units
    components = collect_units(root)
    if not components:
        print("No released component suites declared; skipped.")
        return
    for component, _, _ in components:
        if check(component["owner"], component["id"]) is False:
            raise ValueError(f"released component failed: {component['id']}")
