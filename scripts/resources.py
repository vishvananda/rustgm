#!/usr/bin/env python3
"""Configure, inspect and enter RustGM's shared parallel-work resource controller."""
import argparse
import json
from pathlib import Path
import sys

from resource_locks import active_records, lease
from resource_policy import DEFAULTS, atomic_json, common_directory, discover, output_roots, validate
from resource_runner import run

ROOT = Path(__file__).resolve().parents[1]


def register(root, name, outputs):
    common = common_directory(ROOT)
    if common is None or common_directory(root) != common:
        raise ValueError("only worktrees of this repository can share the controller")
    path = common / "rustgm-resources.json"
    policy = json.loads(path.read_text()) if path.exists() else {**DEFAULTS, "worktrees": {}}
    validate(policy)
    directory = common / "rustgm-resources"
    directory.mkdir(exist_ok=True)
    if active_records(directory, 3):
        raise ValueError("stop guarded commands before changing resource registration")
    entry = {"name": name, "outputs": outputs or ["obj"]}
    output_roots(root, entry)
    with lease(directory, root, policy, "exclusive", wait=0):
        if path.exists() and json.loads(path.read_text()) != policy:
            raise ValueError("resource policy changed; retry configuration")
        policy["worktrees"][str(root)] = entry
        validate(policy)
        atomic_json(path, policy)
    print(f"Registered {name}: {root}")


def set_jobs(root, jobs):
    found = discover(root)
    if found is None:
        raise ValueError("shared controller is not configured")
    directory, policy = found
    updated = {**policy, "jobs": jobs}
    validate(updated)
    if active_records(directory, max(policy["jobs"], jobs)):
        raise ValueError("finish guarded commands before changing job slots")
    path = directory.parent / "rustgm-resources.json"
    with lease(directory, root, policy, "exclusive", wait=0):
        if json.loads(path.read_text()) != policy:
            raise ValueError("resource policy changed; retry configuration")
        atomic_json(path, updated)
    print(f"Shared heavy-job slots: {jobs}; other resource limits unchanged")


def set_schema(root, schema):
    """An explicit idle-only barrier keeps schema 1 wrappers out of variable caps."""
    found = discover(root)
    if found is None:
        raise ValueError("shared controller is not configured")
    directory, policy = found
    if schema != 2:
        raise ValueError("only explicit activation of controller schema 2 is supported")
    updated = {**policy, "schema": schema}
    validate(updated)
    if active_records(directory, 3):
        raise ValueError("finish guarded commands before activating controller schema 2")
    path = directory.parent / "rustgm-resources.json"
    # The exclusive gate also excludes old wrappers, which do not know the new
    # admission lock. Queued wrappers recheck their captured policy before launch.
    with lease(directory, root, policy, "exclusive", wait=0) as (record, _, _):
        if json.loads(path.read_text()) != policy:
            raise ValueError("resource policy changed; retry configuration")
        if any(row.get("slot") != record["slot"] for row in active_records(directory, 3)):
            raise ValueError("finish acquiring leases before activating controller schema 2")
        atomic_json(path, updated)
    print("Controller schema 2 activated; all resource limits unchanged")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    registration = sub.add_parser("register")
    registration.add_argument("worktree", type=Path)
    registration.add_argument("--name", required=True)
    registration.add_argument("--output", action="append", default=[])
    sub.add_parser("status")
    configuration = sub.add_parser("set-jobs")
    configuration.add_argument("jobs", type=int, choices=(2, 3))
    protocol = sub.add_parser("set-schema")
    protocol.add_argument("schema", type=int, choices=(2,))
    launch = sub.add_parser("run")
    launch.add_argument("--exclusive", action="store_true")
    launch.add_argument("--wait-seconds", type=float, default=3600)
    launch.add_argument("--max-output-gib", type=float,
                        help="absolute output cap, including existing caches and pins; requires schema 2")
    launch.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    try:
        if args.action == "register":
            register(args.worktree.resolve(), args.name, args.output)
        elif args.action == "set-jobs":
            set_jobs(ROOT, args.jobs)
        elif args.action == "set-schema":
            set_schema(ROOT, args.schema)
        elif args.action == "status":
            found = discover(ROOT)
            if found is None:
                print("Shared controller is not configured")
                return 1
            directory, policy = found
            print(json.dumps({"policy": policy,
                              "active": active_records(directory, policy["jobs"])}, indent=2))
        else:
            command = args.command[1:] if args.command[:1] == ["--"] else args.command
            if not command or not 0 <= args.wait_seconds <= 3600:
                raise ValueError("run requires a command and wait between zero and 3600 seconds")
            return run(ROOT, command, kind="exclusive" if args.exclusive else "ordinary",
                       wait=args.wait_seconds, max_output_gib=args.max_output_gib)
        return 0
    except (OSError, ValueError, KeyError) as error:
        print(f"resources: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
