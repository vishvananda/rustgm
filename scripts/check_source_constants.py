#!/usr/bin/env python3
"""Check source constant artifacts against separately compiled Rust clients."""
import argparse
import json
from pathlib import Path

from crate_qualification import Check


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "pa29/tests/spec/named-constants"
IDENTITY = "base@source_constants"
COMMON = ["--edition=2024", "--crate-type=lib"]


def provider_source(folder, fixture, contract):
    foundation = (ROOT / contract["foundation"]).read_bytes()
    prefix, marker, _ = foundation.partition(contract["prefix_marker"].encode())
    if not marker:
        raise ValueError("constant provider foundation changed its prefix")
    source = folder / "provider.rs"
    source.write_bytes(prefix + (fixture / contract["provider"]).read_bytes())
    return source


def independent(check, rustc, source, fixture, contract):
    artifact = check.folder / "libbase.rmeta"
    options = [rustc.resolve(strict=True), *COMMON, "--emit=metadata", "--cfg", "rustc_oracle",
               *contract.get("rustc_options", ())]
    environment = {"RUSTC_BOOTSTRAP": "1"}
    check.run("rustc-provider", [*options, "--crate-name", "base", "-o", artifact, source],
              timeout=20, environment=environment)
    for case in contract["cases"]:
        test = fixture / case["source"]
        stem = test.stem
        check.run(f"rustc-{stem}", [*options, "--extern", f"base={artifact}",
                  "-o", check.folder / f"{stem}.rmeta", test], case["status"],
                  [case["rustc_diagnostic"]] if "rustc_diagnostic" in case else (),
                  timeout=20, environment=environment)


def candidate(check, source, fixture, cases):
    artifact = check.folder / "base.rgcrate"
    options = [check.app, *COMMON, "--emit-lowir", "--cap-lints=allow"]
    command = [*options, "--crate-name", "base", "--crate-disambiguator", "source_constants",
               "--source-core", IDENTITY, "--source-profile", "module-body-operations-v14",
               "--metadata-output", artifact, source]
    check.run("rustgm-provider", command, timeout=20)
    if not artifact.is_file() or artifact.stat().st_size == 0:
        raise ValueError("source constant provider emitted no metadata")
    for case in cases:
        test = fixture / case["source"]
        command = [*options, "--crate-name", test.stem.replace("-", "_"),
                   "--crate-disambiguator", "source_constants",
                   "--source-client", IDENTITY, "--extern", f"base={artifact}", test]
        result = check.run(f"rustgm-{test.stem}", command, case["status"],
                           [case["diagnostic"]] if "diagnostic" in case else (), timeout=20)
        for fragment, count in case.get("lowir", {}).items():
            if result.stdout.count(fragment.encode() + b"\n") != count:
                raise ValueError(f"{test.stem}: expected {count} LowIR lines ending in {fragment!r}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", default="specialized-inherent",
                        choices=("specialized-inherent", "generic-inherent"))
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--app", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    fixture = FIXTURES / args.case
    contract = json.loads((fixture / "outcomes.json").read_text())
    check = Check(args.app or args.rustc, args.out)
    source = provider_source(check.folder, fixture, contract)
    independent(check, args.rustc, source, fixture, contract)
    if args.app is not None:
        candidate(check, source, fixture, contract["cases"])
    print(f"Checked {len(contract['cases'])} independent source constant clients")


if __name__ == "__main__":
    main()
