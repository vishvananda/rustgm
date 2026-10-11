#!/usr/bin/env python3
"""Explicit LowIR-to-executable bridge; never a Rust frontend fallback."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def configure(checkout):
    checkout = checkout.resolve()
    commit = subprocess.check_output(["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True).strip()
    tools = {name: str(checkout / "dev" / name) for name in ("lowir", "lowir2native")}
    lock = json.loads((ROOT / "backend.lock.json").read_text())
    if commit != lock["commit"]:
        raise ValueError("checkout differs from the reviewed backend commit")
    for name, path in tools.items():
        if digest(Path(path)) != lock["sha256"][name]:
            raise ValueError(f"{name}: binary differs from reviewed backend; qualify a new lock explicitly")
    (ROOT / "backend.local.json").write_text(json.dumps({"tools": tools}, indent=2) + "\n")
    print("Configured pinned CPPGM backend")


def load_tools():
    local = ROOT / "backend.local.json"
    if not local.is_file():
        raise ValueError("configure backend first: python3 scripts/backend.py --configure CPPGM_CHECKOUT")
    tools = json.loads(local.read_text())["tools"]
    lock = json.loads((ROOT / "backend.lock.json").read_text())
    for name in ("lowir", "lowir2native"):
        if digest(Path(tools[name])) != lock["sha256"][name]:
            raise ValueError(f"{name}: backend bytes changed")
    return tools


def invoke(command):
    result = subprocess.run(command, capture_output=True, timeout=60)
    if result.returncode:
        raise ValueError(f"backend command failed ({result.returncode}): {result.stderr.decode(errors='replace')}")


def build(source, output):
    tools = load_tools()
    output.parent.mkdir(parents=True, exist_ok=True)
    validated = output.with_suffix(".validated.lowir")
    invoke([tools["lowir"], "-o", str(validated), str(source)])
    invoke([tools["lowir2native"], "-O0", "-o", str(output), str(validated)])


def test():
    source = ROOT / "tests/backend/return42.lowir"
    output = ROOT / "obj/backend/return42"
    build(source, output)
    result = subprocess.run([str(output)], capture_output=True, timeout=5)
    if result.returncode != 42 or result.stdout or result.stderr:
        raise ValueError("LowIR smoke program did not return exactly 42 with empty output")
    print("PASS: CPPGM parses, validates and executes LowIR (exit 42)")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--configure", type=Path)
    parser.add_argument("--test", action="store_true")
    parser.add_argument("source", nargs="?", type=Path)
    parser.add_argument("-o", type=Path)
    args = parser.parse_args()
    try:
        if args.configure:
            configure(args.configure)
        elif args.test:
            test()
        elif args.source and args.o:
            build(args.source.resolve(), args.o.resolve())
        else:
            parser.error("use --configure, --test, or SOURCE -o EXECUTABLE")
        return 0
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"backend error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
