#!/usr/bin/env python3
"""A finite contract closes an assignment at its released units and moves the rest forward.

An assignment declares one in assignments.json:

    "contract": {"kind": "finite", "carried_by": "pa45",
                 "deferred_units": [...], "deferred_requirements": [...]}

It is complete when at least one of its units is released, its tests/manifest.json is
exactly the union of its released unit manifests, and its deferred lists name every
unreleased unit it owns or its requirement rows name, and every requirement row that
still has unreleased work. The carrier repeats those lists under "carries", cannot
itself defer, depends on every deferred unit, and is complete only when everything it
carries is released. A requirement can move; it cannot disappear.

    python3 scripts/finite_contracts.py owed PA      print the lists the plan requires
    python3 scripts/finite_contracts.py compose PA   write PA/tests/manifest.json from its released units

Composition copies declared cases. It runs no compiler and writes no reference: the
references and their receipts stay those of the released units.
"""
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
FIELDS = {"kind", "carried_by", "deferred_units", "deferred_requirements"}
PROFILE = ("schema", "language", "edition", "target")


def declared(stage):
    """The stage's finite contract, or None for an assignment that owes its whole handout."""
    contract = stage.get("contract")
    if contract is None:
        return None
    if (not isinstance(contract, dict) or set(contract) != FIELDS or contract["kind"] != "finite"
            or not isinstance(contract["carried_by"], str)
            or any(not isinstance(contract[key], list) or any(not isinstance(item, str) for item in contract[key])
                   or len(set(contract[key])) != len(contract[key])
                   for key in ("deferred_units", "deferred_requirements"))):
        raise ValueError(f"malformed assignment contract: {stage['path']}")
    return contract


def lookup(root, path):
    """The finite contract of the assignment at `path`, read from assignments.json."""
    listing = root / "assignments.json"
    if not listing.is_file():
        return None
    for stage in json.loads(listing.read_text())["stages"]:
        if stage["path"] == path:
            return declared(stage)
    return None


def released(units, stage):
    """The released units an assignment owns, in plan order."""
    return [unit for unit in units if unit["owner"] == stage["path"] and unit["publication"] == "released"]


def owed(stage, units, rows):
    """What a closure of this assignment must defer: (unreleased units, requirement rows with unreleased work).

    The units are those the assignment owns and those its requirement rows name, exactly
    the set a whole-handout completion requires released.
    """
    state = {unit["id"]: unit["publication"] == "released" for unit in units}
    own = [row for row in rows if row["pa"] == stage["number"]]
    named = [unit["id"] for unit in units if unit["owner"] == stage["path"]]
    named += [name for row in own for name in row["units"]]
    return (list(dict.fromkeys(name for name in named if not state[name])),
            [row["id"] for row in own if not all(state[name] for name in row["units"])])


def prerequisites(units, owner):
    """Every unit the owner's units depend on, transitively."""
    by_id = {unit["id"]: unit for unit in units}
    reached, pending = set(), [unit["id"] for unit in units if unit["owner"] == owner]
    while pending:
        for dependency in by_id[pending.pop()]["depends_on"]:
            if dependency not in reached:
                reached.add(dependency)
                pending.append(dependency)
    return reached


def difference(label, stage, declared_items, required_items):
    dropped = [item for item in required_items if item not in declared_items]
    if dropped:
        raise ValueError(f"finite contract drops {label}: {stage['path']}: {dropped}")
    extra = [item for item in declared_items if item not in required_items]
    if extra:
        raise ValueError(f"finite contract defers {label} it does not owe: {stage['path']}: {extra}")


def check(stages, units, rows):
    """Reject an empty, incomplete or uncarried closure, and a carrier completed before what it carries."""
    by_path = {stage["path"]: stage for stage in stages}
    state = {unit["id"]: unit["publication"] == "released" for unit in units}
    row_units = {row["id"]: row["units"] for row in rows}
    for stage in stages:
        contract = declared(stage)
        carries = stage.get("carries", {})
        if not isinstance(carries, dict):
            raise ValueError(f"malformed carried work: {stage['path']}")
        unreleased, open_rows = owed(stage, units, rows)
        if contract is None:
            pending = list(unreleased)
            for source, entry in carries.items():
                origin = by_path.get(source)
                if origin is None or declared(origin) is None or declared(origin)["carried_by"] != stage["path"]:
                    raise ValueError(f"carrier holds work no finite contract deferred to it: "
                                     f"{stage['path']}: {source}")
                if not isinstance(entry, dict) or set(entry) != {"units", "requirements"}:
                    raise ValueError(f"malformed carried work: {stage['path']}: {source}")
                pending += [name for name in entry["units"] if not state.get(name, False)]
                pending += [name for key in entry["requirements"] for name in row_units.get(key, [])
                            if not state[name]]
            if stage["status"] == "implemented" and pending:
                raise ValueError(f"whole PA completion has unpublished work: {stage['path']}")
            continue
        if carries:
            raise ValueError(f"a carrier cannot defer its own work: {stage['path']}")
        if stage["status"] != "implemented":
            raise ValueError(f"finite contract on an assignment that is not closed: {stage['path']}")
        if not released(units, stage):
            raise ValueError(f"finite contract has no released unit: {stage['path']}")
        difference("unreleased units", stage, contract["deferred_units"], unreleased)
        difference("requirement rows", stage, contract["deferred_requirements"], open_rows)
        carrier = by_path.get(contract["carried_by"])
        if carrier is None or carrier is stage or declared(carrier) is not None:
            raise ValueError(f"finite contract needs a carrier that owes its whole handout: {stage['path']}")
        entry = carrier.get("carries", {}).get(stage["path"], {})
        lost = ([name for name in unreleased if name not in entry.get("units", [])]
                + [key for key in open_rows if key not in entry.get("requirements", [])])
        if lost:
            raise ValueError(f"deferred work is not carried: {stage['path']} -> {carrier['path']}: {lost}")
        stale = ([name for name in entry.get("units", []) if name not in unreleased]
                 + [key for key in entry.get("requirements", []) if key not in open_rows])
        if stale:
            raise ValueError(f"carrier holds work the finite contract does not defer: "
                             f"{stage['path']} -> {carrier['path']}: {stale}")
        unreachable = [name for name in unreleased if name not in prerequisites(units, carrier["path"])]
        if unreachable:
            raise ValueError(f"carrier does not depend on deferred units: {carrier['path']}: {unreachable}")


def compose(root, stage, units):
    """The whole-assignment manifest of a finite contract: its released unit manifests, in plan order."""
    members = released(units, stage)
    if not members:
        raise ValueError(f"finite contract has no released unit: {stage['path']}")
    profile, cases, claims, modes = None, [], [], []
    for unit in members:
        manifest = json.loads((root / unit["manifest"]).read_text())
        if profile is None:
            profile = {key: manifest[key] for key in PROFILE if key in manifest}
        elif profile != {key: manifest[key] for key in PROFILE if key in manifest}:
            raise ValueError(f"released units disagree on the course profile: {stage['path']}")
        for declared_case in manifest["cases"]:
            # A case keeps its effective mode; only the manifest default it is written against changes.
            mode = declared_case.get("mode", manifest["mode"])
            case = {key: value for key, value in declared_case.items() if key != "mode"}
            if mode != stage["mode"]:
                case["mode"] = mode
                if mode not in modes:
                    modes.append(mode)
            cases.append(case)
        claims += [claim for claim in manifest["discovery"] if claim not in claims]
    schema = {"schema": profile.pop("schema")} if "schema" in profile else {}
    return {**schema, "mode": stage["mode"], **({"additional_modes": modes} if modes else {}), **profile,
            "contract": "finite", "units": [unit["id"] for unit in members], "discovery": claims, "cases": cases}


def check_manifest(root, stage, manifest, units):
    """A finite contract publishes nothing of its own: the union of its released units, and their records."""
    if manifest != compose(root, stage, units):
        raise ValueError(f"whole-assignment manifest is not the union of its released unit manifests: "
                         f"{stage['path']}; run scripts/finite_contracts.py compose {stage['path']}")
    if (root / stage["path"] / "reference.json").exists():
        raise ValueError(f"finite contract cannot have its own reference receipt: {stage['path']}")
    missing = [unit["release_evidence"] for unit in released(units, stage)
               if unit["release_evidence"] not in stage.get("release_evidence", [])]
    if missing:
        raise ValueError(f"finite contract release evidence omits a released unit's record: "
                         f"{stage['path']}: {missing}")


def main(argv=None, root=ROOT):
    import argparse
    from assignment_plan import validate
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=("owed", "compose"))
    parser.add_argument("stage")
    args = parser.parse_args(argv)
    try:
        stages = json.loads((root / "assignments.json").read_text())["stages"]
        stage = next((stage for stage in stages if stage["path"] == args.stage), None)
        if stage is None:
            raise ValueError(f"unknown assignment: {args.stage}")
        plan = json.loads((root / "assignment-plan.json").read_text())["units"]
        rows = json.loads((root / "docs/assignment-requirements.json").read_text())["rows"]
        if args.command == "owed":
            unreleased, open_rows = owed(stage, plan, rows)
            print(json.dumps({"released_units": [unit["id"] for unit in released(plan, stage)],
                              "deferred_units": unreleased, "deferred_requirements": open_rows}, indent=1))
            return 0
        if declared(stage) is None:
            raise ValueError(f"{args.stage} declares no finite contract in assignments.json")
        validate(root, stages)
        path = root / args.stage / "tests/manifest.json"
        document = compose(root, stage, plan)
        path.write_text(json.dumps(document, indent=1, ensure_ascii=False) + "\n")
        print(f"{path.relative_to(root)}: {len(document['cases'])} cases from {', '.join(document['units'])}")
        return 0
    except (ValueError, OSError, KeyError) as error:
        print(f"finite contract failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
