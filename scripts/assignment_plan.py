#!/usr/bin/env python3
"""Check dependency order and retained obligations independently of PA numbering."""
import json
import re
import sys
from pathlib import Path
from course_suites import unit_manifest

ROOT = Path(__file__).resolve().parents[1]
STATES = {"implemented", "partial", "missing"}


def validate(root=ROOT, stages=None):
    stages = stages or json.loads((root / "assignments.json").read_text())["stages"]
    stages_by_path = {stage["path"]: stage for stage in stages}
    plan = json.loads((root / "assignment-plan.json").read_text())
    requirements = json.loads((root / "docs/assignment-requirements.json").read_text())["rows"]
    if plan.get("schema") != 1 or not plan.get("units"):
        raise ValueError("assignment plan needs schema 1 and nonempty units")
    rows = {}
    for row in requirements:
        key = row["id"]
        if key in rows or not key.startswith(f"{row['pa']}."):
            raise ValueError(f"duplicate or mismatched requirement: {key}")
        if row["implementation"] not in {"implemented_component", "partial", "missing"}:
            raise ValueError(f"unknown requirement state: {key}")
        for kind in ("source", "tests", "evidence"):
            for path in row[kind]:
                checked_path(root, path)
        rows[key] = row
    units = {}
    mapped = {key: [] for key in rows}
    for unit in plan["units"]:
        key = unit["id"]
        if not re.fullmatch(r"[a-z]+(?:-[a-z0-9]+)*", key) or key in units:
            raise ValueError(f"invalid or duplicate work unit: {key}")
        if unit["owner"] not in stages_by_path:
            raise ValueError(f"unknown PA owner: {key}: {unit['owner']}")
        if unit["implementation"] not in STATES:
            raise ValueError(f"unknown implementation state: {key}")
        if unit["implementation"] == "implemented":
            if not unit.get("implementation_evidence"):
                raise ValueError(f"implemented component needs evidence: {key}")
            for path in unit["implementation_evidence"]:
                checked_path(root, path)
        if not unit["scope"].strip() or not unit["exit"].strip():
            raise ValueError(f"work unit needs scope and concrete exit: {key}")
        for dependency in unique(unit["depends_on"], key):
            if dependency not in units:
                raise ValueError(f"missing, cyclic or later dependency: {key}: {dependency}")
        if unit["publication"] not in {"unpublished", "released"}:
            raise ValueError(f"unknown publication state: {key}")
        if "manifest" in unit:
            if unit["manifest"] != unit_manifest(unit["owner"], key):
                raise ValueError(f"noncanonical unit manifest: {key}")
            checked_path(root, unit["manifest"])
        if unit["publication"] == "released":
            if unit["implementation"] != "implemented":
                raise ValueError(f"released work unit is incomplete: {key}")
            checked_path(root, unit["release_evidence"])
            manifest = json.loads(checked_path(root, unit["manifest"]).read_text())
            if not manifest.get("cases"):
                raise ValueError(f"released work unit has empty manifest: {key}")
            if any(units[dependency]["publication"] != "released" for dependency in unit["depends_on"]):
                raise ValueError(f"released work unit has an unreleased prerequisite: {key}")
        for requirement in unique(unit["requirements"], key):
            if requirement not in rows:
                raise ValueError(f"unknown requirement: {key}: {requirement}")
            mapped[requirement].append(key)
        units[key] = unit
    for key, row in rows.items():
        if not mapped[key] or set(mapped[key]) != set(row["units"]):
            raise ValueError(f"unmapped or inconsistent retained requirement: {key}")
        if row["implementation"] != "implemented_component" and all(
                units[name]["implementation"] == "implemented" for name in mapped[key]):
            raise ValueError(f"unfinished requirement lost its remaining work: {key}")
    for stage in stages:
        owned = [unit for unit in units.values() if unit["owner"] == stage["path"]]
        if int(stage["number"]) >= 25 and not owned:
            raise ValueError(f"planned assignment has no work unit: {stage['path']}")
        required = {unit["id"] for unit in owned}
        for row in rows.values():
            if row["pa"] == stage["number"]:
                required.update(row["units"])
        if stage["status"] == "implemented" and any(
                units[name]["publication"] != "released" for name in required):
            raise ValueError(f"whole PA completion has unpublished work: {stage['path']}")
    return list(units.values()), requirements


def unique(values, owner):
    if len(values) != len(set(values)):
        raise ValueError(f"duplicate dependency or requirement: {owner}")
    return values


def fixture_owners(stage, case, units, rows):
    """An explicit owned unit admits its prerequisites, never arbitrary later PAs."""
    selected = case.get("work_units")
    if selected is None:
        return None
    if (not isinstance(selected, list) or not selected
            or any(not isinstance(value, str) for value in selected)):
        raise ValueError("fixture work_units must be a nonempty list of unit IDs")
    unique(selected, case["path"])
    needed = set()
    for key in selected:
        unit = units.get(key)
        if unit is None or (unit["owner"] != stage["path"] and not any(
                row["pa"] == stage["number"] and key in row["units"] for row in rows)):
            raise ValueError(f"fixture selects a non-owning work unit: {stage['path']}: {key}")
        needed.add(key)
    pending = list(needed)
    while pending:
        for dependency in units[pending.pop()]["depends_on"]:
            if dependency not in needed:
                needed.add(dependency)
                pending.append(dependency)
    return {units[key]["owner"] for key in needed}


def checked_path(root, value):
    path = Path(value)
    if path.is_absolute() or ".." in path.parts or not (root / path).exists():
        raise ValueError(f"missing or unsafe assignment evidence path: {value}")
    return root / path


def main():
    try:
        units, rows = validate()
        complete = sum(unit["implementation"] == "implemented" for unit in units)
        print(f"Assignment plan: {len(units)} ordered units; {len(rows)} retained requirement families; "
              f"{complete} finite components implemented; "
              f"{sum(unit['publication'] == 'released' for unit in units)} component releases")
        for index, unit in enumerate(units, 1):
            print(f"{index:2}. {unit['owner']} {unit['id']}: "
                  f"{unit['implementation']}, {unit['publication']}")
        return 0
    except (ValueError, OSError, KeyError, TypeError) as error:
        print(error, file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
