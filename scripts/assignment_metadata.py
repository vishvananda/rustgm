#!/usr/bin/env python3
"""Validate assignment declarations without requiring generated references."""
import json
import re
import sys

from course import ROOT, load_cases
from check_native import classification
from check_layouts import classification as layout_classification
from assignment_plan import checked_path, fixture_owners, validate as validate_plan
from course_suites import check_publishers
import finite_contracts


def validate_cases(stage, manifest, cases, units, rows, foundational):
    for case in cases:
        owners = fixture_owners(stage, case, units, rows)
        if owners is not None:
            owners |= foundational
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
            future = owner and (f"pa{owner[1]}" not in owners if owners is not None
                                else int(owner[1]) > stage["number"])
            if future:
                raise ValueError(f"fixture uses a future-stage feature: {stage['path']}/{case['path']}: {feature}")


def validate_unit_cases(root, owner, selected, manifest, cases):
    stages = json.loads((root / "assignments.json").read_text())["stages"]
    ordered, rows = validate_plan(root, stages)
    stage = next(stage for stage in stages if stage["path"] == owner)
    for case in cases:
        if selected not in case.get("work_units", []):
            raise ValueError(f"unit fixture must declare its owning work unit: {case['path']}")
    foundational = {stage["path"] for stage in stages
                    if stage["number"] <= 24 and stage["status"] != "planned"}
    validate_cases(stage, manifest, cases, {unit["id"]: unit for unit in ordered}, rows, foundational)


def collect_units(root=ROOT, *, unit=None):
    if not (root / "assignment-plan.json").exists():
        if unit is not None:
            raise ValueError(f"unknown assignment unit: {unit}")
        return []
    units, _ = validate_plan(root)
    check_publishers(root)
    selected = [entry for entry in units if entry["id"] == unit] if unit is not None else [
        entry for entry in units if entry["publication"] == "released" and "manifest" in entry]
    if unit is not None and not selected:
        raise ValueError(f"unknown assignment unit: {unit}")
    return [(entry, *load_cases(root, entry["owner"], unit=entry["id"]))
            for entry in selected if "manifest" in entry]


def validate_workload_stage(root, stage):
    """Validate a released consumers or self-hosting suite manifest."""
    kind = stage.get("suite_kind", "text")
    suite_path = root / stage["path"] / "tests/suite.json"
    if not suite_path.is_file():
        raise ValueError(f"workload assignment lacks tests/suite.json: {stage['path']}")
    suite = json.loads(suite_path.read_text())
    if suite.get("schema") != 1 or suite.get("kind") != kind or suite.get("stage") != stage["path"]:
        raise ValueError(f"workload suite manifest disagrees with assignments.json: {stage['path']}")
    if kind == "consumers":
        import grade_consumers
        _, outcomes, folder = grade_consumers.load_suite(root, stage["path"])
        if not outcomes.get("cases"):
            raise ValueError(f"consumer suite has no cases: {stage['path']}")
        if lint_rel := suite.get("lint_manifest"):
            lint_path = checked_path(root, f"{stage['path']}/{lint_rel}")
            lint_plan = json.loads(lint_path.read_text())
            if not lint_plan.get("cases"):
                raise ValueError(f"empty lint manifest: {lint_path}")
            for case in lint_plan["cases"]:
                if not (lint_path.parent / case["path"]).is_file():
                    raise ValueError(f"missing lint fixture: {lint_path.parent / case['path']}")
        return suite, outcomes["cases"]
    if kind == "self-hosting":
        import build_self_hosting
        config_path = checked_path(root, f"{stage['path']}/{suite['configuration']}")
        build_self_hosting.source_arguments(json.loads(config_path.read_text()))
        if not suite.get("strict_zero_bringup") or not suite.get("compare_outputs") or not suite.get("negative_controls"):
            raise ValueError(f"self-hosting suite manifest is incomplete: {stage['path']}")
        return suite, []
    raise ValueError(f"unknown suite_kind: {stage['path']}: {kind}")


def collect(root=ROOT):
    stages = json.loads((root / "assignments.json").read_text())["stages"]
    for stage in stages:
        if stage["status"] not in ("planned", "implemented-draft", "implemented"):
            raise ValueError(f"unknown assignment status: {stage['path']}: {stage['status']}")
        if stage.get("suite_kind", "text") not in ("text", "consumers", "self-hosting"):
            raise ValueError(f"unknown suite_kind: {stage['path']}: {stage.get('suite_kind')}")
        if stage["status"] == "implemented":
            if not stage.get("release_evidence"):
                raise ValueError(f"implemented assignment needs release evidence: {stage['path']}")
            for path in stage["release_evidence"]:
                checked_path(root, path)
    units, rows = {}, []
    if (root / "assignment-plan.json").exists():
        ordered, rows = validate_plan(root, stages)
        units = {unit["id"]: unit for unit in ordered}
    foundational = {stage["path"] for stage in stages
                    if stage["number"] <= 24 and stage["status"] != "planned"}
    active = [stage["path"] for stage in stages
              if stage["status"] != "planned" and stage.get("suite_kind", "text") == "text"]
    makefile = (root / "Makefile").read_text()
    configured = re.search(r"^ACTIVE_PAS\s*:=\s*(.*)$", makefile, re.M)
    if configured is None or configured[1].split() != active:
        raise ValueError("root report stages disagree with assignments.json")
    workloads = [stage["path"] for stage in stages
                 if stage["status"] != "planned" and stage.get("suite_kind", "text") != "text"]
    if workloads:
        configured_w = re.search(r"^WORKLOAD_PAS\s*:=\s*(.*)$", makefile, re.M)
        if configured_w is None or configured_w[1].split() != workloads:
            raise ValueError("root workload stages disagree with assignments.json")
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
        if stage.get("suite_kind", "text") != "text":
            validate_workload_stage(root, stage)
            continue
        manifest, cases = load_cases(root, stage["path"])
        if (manifest["mode"] != stage["mode"]
                or manifest.get("additional_modes", []) != stage.get("additional_modes", [])):
            raise ValueError(f"assignment modes disagree with manifest: {folder}")
        if finite_contracts.declared(stage) is not None:
            finite_contracts.check_manifest(root, stage, manifest, list(units.values()))
        validate_cases(stage, manifest, cases, units, rows, foundational)
        loaded.append((stage, manifest, cases))
    collect_units(root)
    return stages, loaded


def main():
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--unit", help="also validate one declared unpublished component")
    args = parser.parse_args()
    try:
        stages, loaded = collect()
        components = collect_units(unit=args.unit)
        print(f"Assignment metadata: {len(stages)} stages; {sum(len(cases) for _, _, cases in loaded)} active cases; references not required")
        if components:
            print(f"Component metadata: {len(components)} units; {sum(len(cases) for _, _, cases in components)} cases")
        return 0
    except (ValueError, OSError, KeyError) as error:
        print(error, file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
