#!/usr/bin/env python3
"""Independent stable-rustc checks for explicitly classified syntax fixtures."""
import argparse
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import re
import subprocess
import sys

from course import ROOT, execute, load_cases


def check(stage, app):
    version = subprocess.check_output(["rustc", "--version"], text=True)
    if not version.startswith("rustc 1.98.0 "):
        raise ValueError(f"expected rustc 1.98.0, got {version.strip()}")
    _, cases = load_cases(ROOT, stage)
    directory = ROOT / "obj/rustc-syntax" / stage
    directory.mkdir(parents=True, exist_ok=True)

    def worker(case):
        metadata = case["rustc"]
        kind = metadata["kind"]
        if kind == "fragment":
            return False
        if kind not in ("program", "predicate"):
            raise ValueError(f"unknown independent check kind: {kind}")
        fixture = ROOT / stage / case["path"]
        data = fixture.read_bytes()
        name = fixture.parent.name + "-" + fixture.stem
        source = directory / (name + ".rs")
        binary = directory / name
        if kind == "predicate":
            source.write_text('fn main() { println!("{}", cfg!(' + metadata["predicate"] + ')); }\n')
        else:
            source.write_bytes(data)
        command = ["rustc", "--edition=2024", "--target", "x86_64-unknown-linux-gnu",
                   "-C", "panic=abort", "-C", "debug-assertions=off", "--cap-lints", "allow",
                   "--crate-name", "syntax_check", *case.get("args", []), str(source), "-o", str(binary)]
        if kind != "predicate":
            command += ["--crate-type=lib", "--emit=metadata"]
        result = subprocess.run(command, capture_output=True, timeout=30)
        (directory / (name + ".stderr")).write_bytes(result.stderr)
        wanted = 0 if kind == "predicate" else metadata["status"]
        if result.returncode != wanted:
            raise ValueError(f"rustc disagreement: {fixture}: {result.returncode}, wanted {wanted}\n{result.stderr.decode(errors='replace')}")
        actual = execute([str(app), "--emit-token-trees", "--edition=2024", *case.get("args", [])], data, 10)
        if actual[2] != f"{case['exit_status']}\n".encode():
            raise ValueError(f"candidate outcome disagreement: {fixture}: {actual[1]!r}")
        if kind == "predicate":
            runtime = subprocess.run([str(binary)], capture_output=True, timeout=10)
            expected = b"true\n" if metadata["value"] else b"false\n"
            if runtime.returncode or runtime.stderr or runtime.stdout != expected:
                raise ValueError(f"independent predicate expectation is wrong: {fixture}: {runtime.stdout!r}")
            facts = re.findall(rb"^attribute .*? cfg=(true|false) ", actual[0], re.M)
            if facts != [expected.strip()]:
                raise ValueError(f"cfg evaluation disagrees with rustc: {fixture}: {facts!r}")
        elif "enabled" in metadata:
            enabled = not re.search(rb"^attribute .*? cfg=false ", actual[0], re.M)
            if enabled != metadata["enabled"]:
                raise ValueError(f"configuration fact disagrees with declared ownership case: {fixture}")
        return True

    with ThreadPoolExecutor(max_workers=4) as pool:
        count = sum(pool.map(worker, cases))
    if count == 0:
        raise ValueError("empty independent syntax check suite")
    print(f"PASS: {stage}: {count} independent rustc checks; {len(cases) - count} declared stream fragments")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage")
    parser.add_argument("--app", type=Path, default=ROOT / "dev/rustgm")
    args = parser.parse_args()
    try:
        check(args.stage, args.app.resolve())
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
