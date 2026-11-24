#!/usr/bin/env python3
"""Check declaration export observation against a prebuilt RustGM."""
import argparse
import json
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "pa31/tests/spec/source-export-census"
COMMAND = ["--emit-lowir", "--edition=2024", "--crate-type=lib", "--crate-name", "base",
           "--crate-disambiguator", "census", "--source-core", "base@census",
           "--source-profile", "module-body-operations-v14"]
HEADER = "rustgm-export-census\t1\tdeclaration-admission\tanalysis-only"


def run(app, out):
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    source = FIXTURE / contract["source"]
    out.mkdir(parents=True, exist_ok=False)
    census = out / "census.tsv"
    command = [app, *COMMAND, "--export-census", census, source]
    result = subprocess.run(command, capture_output=True, text=True, timeout=60, check=False)
    (out / "census.stderr").write_text(result.stderr)
    if result.returncode != 1 or result.stdout or not census.is_file():
        raise ValueError(f"census did not exclusively produce an analysis report: {result.stderr}")
    lines = census.read_text().splitlines()
    if lines[0] != HEADER:
        raise ValueError("missing declaration admission census header")
    issues = [line.split("\t") for line in lines[1:-1]]
    if [row[3] for row in issues] != contract["refused"]:
        raise ValueError(f"wrong declaration export failures: {issues}")
    for row in issues:
        _, declaration, _, path, location, diagnostic = row
        name = path.rsplit("::", 1)[-1]
        if not declaration.isdigit() or diagnostic != contract["diagnostic"]:
            raise ValueError(f"incorrect census diagnostic: {row}")
        file, line, _ = location.rsplit(":", 2)
        if Path(file) != source or f"type {name}" not in source.read_text().splitlines()[int(line) - 1]:
            raise ValueError(f"incorrect declaration location: {location}")
    summary = lines[-1].split("\t")
    if len(summary) != 5 or summary[0] != "summary":
        raise ValueError(f"incorrect census summary: {lines[-1]}")
    if summary[1::2] != ["checked", "failed"] or int(summary[4]) != len(issues):
        raise ValueError(f"incorrect census summary: {lines[-1]}")
    rejected = subprocess.run(
        [*command[:-1], "--metadata-output", out / "invalid.rgcrate", source],
        capture_output=True, text=True, timeout=60, check=False,
    )
    if rejected.returncode != 2 or (out / "invalid.rgcrate").exists():
        raise ValueError("export census accepted publication output paths")
    print(f"Export census checked {summary[2]} declarations, preserved {len(issues)} failures and emitted no library")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    run(args.app.resolve(strict=True), args.out)


if __name__ == "__main__":
    main()
