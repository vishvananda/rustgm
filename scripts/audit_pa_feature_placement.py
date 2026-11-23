#!/usr/bin/env python3
"""Audit assignment fixtures against their inspection mode and feature owners."""

import argparse
from collections import Counter, defaultdict
from dataclasses import asdict, dataclass
from functools import cache
from hashlib import sha256
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys

from assignment_plan import fixture_owners, validate as load_plan
from case_inputs import support_files
from course import ROOT, SIDECARS, check_relative_path
from course_suites import check_publishers
from rust_audit import PARSER


@dataclass(frozen=True)
class Rule:
    feature: str
    owner: int
    modes: frozenset[str]


LOWIR = frozenset({"--emit-lowir"})
TYPED = LOWIR | {"--emit-types", "--emit-body-ir", "--emit-layouts", "--emit-patterns",
                 "--emit-ownership", "--emit-regions", "--emit-loans", "--emit-semantics",
                 "--emit-trait-semantics", "--emit-generic-semantics", "--emit-constants"}
ASYNC = Rule("async lowering", 42, LOWIR)
SOURCE_RULES = {
    "macro_definition": Rule("macro matching and expansion", 7,
                             TYPED | {"--emit-expanded", "--emit-expanded-crate"}),
    "closure_expression": Rule("closure lowering", 24, LOWIR),
    "union_item": Rule("union semantics", 27, TYPED),
    "try_expression": Rule("Try protocol", 26, LOWIR),
    "variadic_parameter": Rule("C variadics", 30, TYPED),
    "async_block": ASYNC,
    "await_expression": ASYNC,
}
ORACLE_RULES = (
    (re.compile(rb"^    (?:%\w+ = )?(atomic_(?:load|store|exchange|add_fetch|compare_exchange|fence))\b"),
     Rule("atomic operations", 28, LOWIR)),
    (re.compile(rb"^    (?:%\w+ = )?(va_(?:start|arg|copy|end))\b"),
     Rule("C variadics", 30, LOWIR)),
    (re.compile(rb"^(assembly) @asm\d+ intel\b"), Rule("inline assembly", 41, LOWIR)),
)
FEATURE_OWNER = re.compile(r"pa([1-9][0-9]*)\..+")
PA_PATH = re.compile(r"pa[1-9][0-9]*")
SOURCE_MODES = frozenset().union(*(rule.modes for rule in SOURCE_RULES.values()))


@dataclass(frozen=True, order=True)
class Evidence:
    owner: int
    feature: str
    kind: str
    path: str
    line: int | None = None


@dataclass(frozen=True)
class Case:
    manifest: str
    mode: str
    entry: dict


def fixture_inventory(root):
    """Use the worktree's Git inventory so ignored run output never enters the scan."""
    result = subprocess.run(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard",
         "--", ":(glob)pa*/tests/**"],
        cwd=root, check=True, stdout=subprocess.PIPE,
    )
    paths = {PurePosixPath(path) for path in result.stdout.decode().split("\0") if path}
    return {path for path in paths if PA_PATH.fullmatch(path.parts[0]) and (root / path).is_file()}


def load_cases(root, stages, units, pas):
    manifests = [(stage["path"], f"{stage['path']}/tests/manifest.json") for stage in stages
                 if stage["path"] in pas and stage["status"] != "planned"]
    manifests += [(unit["owner"], unit["manifest"]) for unit in units
                  if unit["owner"] in pas and "manifest" in unit]
    cases = {}
    for pa, manifest in manifests:
        data = json.loads((root / manifest).read_bytes())
        seen = set()
        for entry in data["cases"]:
            primary = input_path(pa, entry["path"])
            if primary in seen:
                raise ValueError(f"{manifest}: duplicate primary: {primary}")
            seen.add(primary)
            cases[primary] = Case(manifest, entry.get("mode", data["mode"]), entry)
    return cases


def input_path(pa, name):
    check_relative_path(name, pa)
    path = PurePosixPath(pa) / name
    if not path.is_relative_to(PurePosixPath(pa) / "tests"):
        raise ValueError(f"{pa}: fixture input outside tests: {name}")
    return path


def read_input(root, path):
    file = root / path
    tests = root / path.parts[0] / "tests"
    if not file.resolve().is_relative_to(tests.resolve()):
        raise ValueError(f"fixture input escapes its assignment: {path}")
    return file.read_bytes()


def source_markers(data):
    """Use syntax nodes so comments, literals and unexpanded macro tokens cannot match."""
    found = {}
    pending = [PARSER.parse(data).root_node]
    while pending:
        node = pending.pop()
        rule = SOURCE_RULES.get(node.type)
        if node.type == "function_modifiers" and any(child.type == "async" for child in node.children):
            rule = ASYNC
        if rule is not None and not node.has_error:
            found.setdefault(rule, node.start_point.row + 1)
        if node.type not in {"ERROR", "token_tree", "token_tree_pattern"}:
            pending.extend(reversed(node.named_children))
    return found


def oracle_markers(data, path):
    found = {}
    for line, text in enumerate(data.splitlines(), 1):
        for pattern, rule in ORACLE_RULES:
            if pattern.match(text):
                found.setdefault(rule, Evidence(rule.owner, rule.feature, "oracle", path, line))
    return list(found.values())


def declared_evidence(case, mode_owner, numbers):
    evidence = [Evidence(mode_owner, case.mode, "mode", case.manifest)]
    for feature in case.entry.get("features", []):
        owner = FEATURE_OWNER.fullmatch(feature)
        if owner is None or int(owner[1]) not in numbers:
            raise ValueError(f"{case.manifest}: unknown assignment feature: {feature}")
        evidence.append(Evidence(int(owner[1]), feature, "manifest", case.manifest))
    if case.entry.get("crates"):
        evidence.append(Evidence(29, "separate crate inputs", "manifest", case.manifest))
    if case.entry.get("native", {}).get("providers"):
        evidence.append(Evidence(30, "native C providers", "manifest", case.manifest))
    return evidence


def case_row(path, case, inputs, evidence, admitted, explicit_units):
    current = int(path.parts[0][2:])
    early = [item for item in evidence if item.owner not in admitted]
    observed = [item for item in evidence if item.kind != "source"]
    strongest = max(item.owner for item in observed)
    suggestion = max(item.owner for item in evidence)
    if any(item.kind != "source" for item in early):
        status = "early"
    elif early:
        status = "review"
    elif (not explicit_units and suggestion < current
          and any(item.kind in {"manifest", "oracle"} for item in evidence)):
        status = "move-candidate"
    else:
        status = "ok"
    return {
        "path": path.as_posix(), "manifest": case.manifest, "mode": case.mode,
        "status": status, "inputs": sorted(item.as_posix() for item in inputs),
        "evidence_floor": f"pa{strongest}",
        "suggested_pa": f"pa{suggestion}" if status == "move-candidate" else None,
        "evidence": [asdict(item) for item in sorted(set(evidence))],
        "early_evidence": [asdict(item) for item in sorted(set(early))],
    }


def duplicates(root, cases, inventory):
    """Find byte-equal standalone stdin cases with identical observed contracts."""
    by_input = defaultdict(list)
    for path, case in cases.items():
        if case.entry.get("input", "stdin") != "stdin" or support_files(case.entry):
            continue
        settings = {key: value for key, value in case.entry.items()
                    if key not in {"path", "mode", "input", "features", "work_units",
                                   "source_sha256", "maintainer_origin"}}
        settings = json.dumps(settings, sort_keys=True, separators=(",", ":"))
        source = sha256(read_input(root, path)).digest()
        by_input[case.mode, settings, source].append(path)
    findings = []
    for (mode, _, _), paths in by_input.items():
        if len({path.parts[0] for path in paths}) < 2:
            continue
        by_oracles = defaultdict(list)
        for path in paths:
            oracles = [PurePosixPath(f"{path}{suffix}") for suffix in SIDECARS]
            if all(oracle in inventory for oracle in oracles):
                observed = tuple(sha256(read_input(root, oracle)).digest() for oracle in oracles)
                by_oracles[observed].append(path)
        for matching in by_oracles.values():
            if len({path.parts[0] for path in matching}) > 1:
                findings.append({"mode": mode, "paths": sorted(path.as_posix() for path in matching)})
    return sorted(findings, key=lambda group: group["paths"])


def audit(root, stages, units, requirements, inventory, pas):
    numbers = {stage["number"] for stage in stages}
    definitions = {stage["path"]: stage for stage in stages}
    foundational = {stage["number"] for stage in stages
                    if stage["number"] <= 24 and stage["status"] != "planned"}
    mode_owners = {}
    for stage in sorted(stages, key=lambda stage: stage["number"]):
        for mode in [stage["mode"], *stage.get("additional_modes", [])]:
            mode_owners.setdefault(mode, stage["number"])
    cases = load_cases(root, stages, units.values(), pas)
    claimed = set()
    results = []

    @cache
    def source(path):
        return source_markers(read_input(root, path))

    for path, case in sorted(cases.items()):
        pa = path.parts[0]
        stage = definitions[pa]
        inputs = {path, *(input_path(pa, name) for name in support_files(case.entry))}
        if missing := inputs - inventory:
            raise ValueError(f"{case.manifest}: missing auditable inputs for {path}: {sorted(missing)}")
        claimed.update(inputs)
        if case.mode not in mode_owners:
            raise ValueError(f"{case.manifest}: unknown inspection mode: {case.mode}")
        owners = fixture_owners(stage, case.entry, units, requirements)
        admitted = (set(range(1, stage["number"] + 1)) if owners is None
                    else {int(owner[2:]) for owner in owners} | foundational)
        evidence = declared_evidence(case, mode_owners[case.mode], numbers)
        if case.entry["exit_status"] == 0:
            if case.mode in SOURCE_MODES:
                for item in sorted(inputs):
                    if item.suffix == ".rs":
                        evidence.extend(Evidence(rule.owner, rule.feature, "source", item.as_posix(), line)
                                        for rule, line in source(item).items() if case.mode in rule.modes)
            oracle = PurePosixPath(f"{path}.ref")
            if case.mode in LOWIR and oracle in inventory:
                evidence.extend(oracle_markers(read_input(root, oracle), oracle.as_posix()))
        results.append(case_row(path, case, inputs, evidence, admitted, owners is not None))

    source_only = sorted(path for path in inventory - claimed
                         if path.parts[0] in pas and path.suffix in {".rs", ".c", ".lowir"})
    draft_review = []
    for path in source_only:
        current = int(path.parts[0][2:])
        if path.suffix == ".rs":
            draft_review.extend(Evidence(rule.owner, rule.feature, "source", path.as_posix(), line)
                                for rule, line in source(path).items() if rule.owner > current)
        if path.suffix == ".lowir":
            uses = oracle_markers(read_input(root, path), path.as_posix())
            draft_review.extend(item for item in uses if item.owner > current)
    return {
        "summary": dict(Counter(row["status"] for row in results)),
        "cases": results,
        "outside_course_manifests": [path.as_posix() for path in source_only],
        "outside_course_manifest_review": [asdict(item) for item in sorted(draft_review)],
        "duplicate_contracts": duplicates(root, cases, inventory),
    }


def print_summary(report):
    counts = report["summary"]
    print(f"Placement audit: {len(report['cases'])} cases, {counts.get('early', 0)} early, "
          f"{counts.get('review', 0)} source findings to review, "
          f"{counts.get('move-candidate', 0)} possible earlier placements")
    print(f"Inputs outside course manifests: {len(report['outside_course_manifests'])}")
    print(f"Feature observations in those inputs to review: {len(report['outside_course_manifest_review'])}")
    print(f"Cross-assignment groups with matching inputs and oracles: {len(report['duplicate_contracts'])}")
    for row in report["cases"]:
        if row["status"] == "early":
            for item in row["early_evidence"]:
                if item["kind"] != "source":
                    where = f":{item['line']}" if item["line"] is not None else ""
                    print(f"{row['path']}: requires pa{item['owner']} "
                          f"({item['feature']}, {item['path']}{where})")
        elif row["status"] == "review":
            first, *remaining = row["early_evidence"]
            more = f" and {len(remaining)} more" if remaining else ""
            print(f"{row['path']}: review {first['feature']} in {first['path']}:{first['line']}{more}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--pa", action="append", help="limit the audit to the selected assignment")
    parser.add_argument("--json-out", type=Path, help="write the full inventory and evidence")
    parser.add_argument("--fail-on-early", action="store_true",
                        help="fail on early placement backed by manifest or committed oracle evidence")
    args = parser.parse_args()
    root = args.root.resolve()
    try:
        stages = json.loads((root / "assignments.json").read_bytes())["stages"]
        available = {stage["path"] for stage in stages}
        pas = set(args.pa) if args.pa else available
        if unknown := pas - available:
            raise ValueError(f"unknown assignments: {', '.join(sorted(unknown))}")
        ordered, requirements = load_plan(root, stages)
        check_publishers(root)
        report = audit(root, stages, {unit["id"]: unit for unit in ordered}, requirements,
                       fixture_inventory(root), pas)
        print_summary(report)
        if args.json_out:
            args.json_out.parent.mkdir(parents=True, exist_ok=True)
            args.json_out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        return int(args.fail_on_early and bool(report["summary"].get("early")))
    except (KeyError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"placement audit: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
