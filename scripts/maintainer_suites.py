#!/usr/bin/env python3
"""Validate integration-suite registration and run one suite within its binary."""
import argparse
from pathlib import Path
import re
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
MODULE = re.compile(
    r'#\[path = "(?P<path>\.\./\.\./[a-z0-9_/]+\.rs)"\]\s*'
    r'(?P<visibility>pub\(crate\) )?mod (?P<name>[a-z][a-z0-9_]*);')


def discover(root=ROOT):
    """Every original integration root must appear exactly once, unconditionally."""
    manifest = tomllib.loads((root / "dev/Cargo.toml").read_text())
    if manifest["package"].get("autotests") is not False:
        raise ValueError("grouped integration tests require autotests = false")
    suites, paths = {}, set()
    for target in manifest.get("test", []):
        name = target.get("name", "")
        if (set(target) != {"name", "path"}
                or not re.fullmatch(r"suite_[a-z][a-z0-9_]*", name)
                or target["path"] != f"tests/suites/{name[6:]}/mod.rs"):
            raise ValueError(f"invalid unconditional integration target: {target}")
        path = root / "dev" / target["path"]
        if path in paths:
            raise ValueError(f"duplicate integration target: {name}")
        paths.add(path)
        source = path.read_text()
        # Registration files contain only documentation and ordinary path modules.
        source = re.sub(r"(?m)^\s*//[^\n]*", "", source)
        registered = list(MODULE.finditer(source))
        if not registered or MODULE.sub("", source).strip():
            raise ValueError(f"invalid or conditional suite registration: {path}")
        modules = set()
        for module in registered:
            suite = module["name"]
            if suite in modules:
                raise ValueError(f"duplicate group module: {suite}")
            modules.add(suite)
            relative = module["path"]
            if relative != f"../../{suite}.rs":
                helper = path.parent / relative
                if (not re.fullmatch(r"\.\./\.\./(?:helpers|harness|support)/" + suite + r"\.rs", relative)
                        or not helper.is_file() or module["visibility"] is None):
                    raise ValueError(f"invalid shared helper module: {suite}")
                continue
            if module["visibility"] is not None:
                raise ValueError(f"conditional or public suite registration: {suite}")
            if suite in suites:
                raise ValueError(f"duplicate maintainer suite: {suite}")
            suites[suite] = name
    expected_paths = set((root / "dev/tests/suites").rglob("*.rs"))
    if paths != expected_paths:
        raise ValueError("unregistered or missing integration group file")
    expected = {p.stem for p in (root / "dev/tests").glob("*.rs")}
    if not expected or set(suites) != expected:
        raise ValueError(f"unregistered or missing maintainer suites: {sorted(set(suites) ^ expected)}")
    return suites


def command(suites, args):
    if args.suite not in suites:
        raise ValueError(f"unknown maintainer suite: {args.suite}")
    result = ["cargo", "test", "--manifest-path", "dev/Cargo.toml", "--locked"]
    if args.release:
        result.append("--release")
    if args.all_features:
        result.append("--all-features")
    result += ["--test", suites[args.suite], "--"]
    if args.ignored:
        result.append("--ignored")
    return result


def selected_tests(listing, suite, case):
    """Use the actual binary's inventory and match the outer suite exactly."""
    prefix = suite + "::"
    selected = []
    for line in listing.splitlines():
        match = re.fullmatch(r"(\S+): test", line)
        if match and match[1].startswith(prefix) and (case is None or match[1] == prefix + case):
            selected.append(match[1])
    if not selected:
        raise ValueError(f"no runnable tests match {prefix + (case or '')}")
    if len(set(selected)) != len(selected):
        raise ValueError("duplicate test identity in libtest inventory")
    return selected


def run_suite(suites, args, *, root=ROOT, runner=subprocess.run):
    base = command(suites, args)
    listed = runner(base + ["--list"], cwd=root, stdout=subprocess.PIPE, text=True)
    if listed.returncode:
        return listed.returncode
    selected = selected_tests(listed.stdout, args.suite, args.case)
    if args.list:
        print("\n".join(name + ": test" for name in selected))
        print(f"\n{len(selected)} tests selected")
        return 0
    return runner(base + ["--exact", *selected], cwd=root).returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    actions = parser.add_subparsers(dest="action", required=True)
    actions.add_parser("check")
    actions.add_parser("list")
    run = actions.add_parser("run")
    run.add_argument("suite")
    run.add_argument("--case", help="exact test path within the suite, including nested modules")
    for flag in ("release", "all-features", "ignored", "list"):
        run.add_argument("--" + flag, action="store_true")
    args = parser.parse_args()
    try:
        suites = discover()
        if args.action == "run":
            return run_suite(suites, args)
        if args.action == "list":
            for suite, target in sorted(suites.items()):
                print(f"{suite}: {target}")
        else:
            print(f"Maintainer suites: {len(suites)} roots in {len(set(suites.values()))} binaries")
        return 0
    except (OSError, ValueError, KeyError) as error:
        print(error, file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
