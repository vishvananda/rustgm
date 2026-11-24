#!/usr/bin/env python3
"""Compare fresh nominal clients using separately published RustGM and Rust crates."""
import argparse
import json
from pathlib import Path

from crate_qualification import Check


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "pa31/tests/spec/non-exhaustive-transport"
IDENTITY = "base@non_exhaustive_fixture"
RUSTC = ["rustc", "--edition=2024", "--crate-type=lib", "--emit=metadata", "--cfg", "rustc_oracle"]
RUSTGM = ["--emit-lowir", "--edition=2024", "--crate-type=lib", "--cap-lints=allow"]


def provider(check, contract):
    prefix, marker, _ = (ROOT / contract["foundation"]).read_bytes().partition(
        contract["prefix_marker"].encode())
    if not marker:
        raise ValueError("provider foundation has no declared boundary")
    source = check.folder / "provider.rs"
    source.write_bytes(prefix + (FIXTURES / "provider.rs").read_bytes())
    return source


def rustc(check, source, cases):
    bootstrap = {"RUSTC_BOOTSTRAP": "1"}
    artifact = check.folder / "libbase.rmeta"
    check.run("rustc-provider", [*RUSTC, "--crate-name", "base", "-o", artifact, source],
              environment=bootstrap, timeout=15)
    for case in cases:
        name = case["cfg"]
        codes = (f"error[{case['rustc']}]",) if case["status"] else ()
        check.run(f"rustc-{name}", [*RUSTC, "--crate-name", name, "--extern", f"base={artifact}",
                                 "--cfg", name, "-o", check.folder / f"{name}.rmeta",
                                 FIXTURES / "client.rs"], case["status"], codes,
                  environment=bootstrap, timeout=15)


def candidate(check, source, cases):
    artifact = check.folder / "base.rgcrate"
    check.run("rustgm-provider", [check.app, *RUSTGM, "--crate-name", "base",
              "--crate-disambiguator", "non_exhaustive_fixture", "--source-core", IDENTITY,
              "--source-profile", "module-body-operations-v14", "--metadata-output", artifact,
              "-o", check.folder / "provider.lowir", source], timeout=30)
    if artifact.stat().st_size == 0:
        raise ValueError("provider emitted no RustGM crate metadata")
    for case in cases:
        name = case["cfg"]
        codes = (case["rustgm"],) if case["status"] else ()
        check.run(f"rustgm-{name}", [check.app, *RUSTGM, "--crate-name", name,
                  "--crate-disambiguator", "non_exhaustive_client",
                  "--source-client", IDENTITY, "--extern", f"base={artifact}", "--cfg", name,
                  "-o", check.folder / f"{name}.lowir", FIXTURES / "client.rs"],
                  case["status"], codes, timeout=15)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    compiler = parser.add_mutually_exclusive_group(required=True)
    compiler.add_argument("--app", type=Path)
    compiler.add_argument("--oracle-only", action="store_true")
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    check = Check(args.app or Path("rustc"), args.out)
    contract = json.loads((FIXTURES / "outcomes.json").read_text())
    source = provider(check, contract)
    rustc(check, source, contract["cases"])
    if args.app:
        candidate(check, source, contract["cases"])
    print(f"Checked {len(contract['cases'])} independent non-exhaustive nominal clients")


if __name__ == "__main__":
    main()
