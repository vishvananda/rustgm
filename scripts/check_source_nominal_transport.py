#!/usr/bin/env python3
"""Compare fresh nominal clients using separately published RustGM and Rust crates."""
import argparse
import json
from pathlib import Path

from crate_qualification import Check


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "pa31/tests/spec"
IDENTITY = "base@source_nominal_fixture"
RUSTC = ["rustc", "--edition=2024", "--crate-type=lib", "--emit=metadata", "--cfg", "rustc_oracle"]
RUSTGM = ["--emit-lowir", "--edition=2024", "--cap-lints=allow"]


def source_prefix(path, marker):
    prefix, found, _ = (ROOT / path).read_bytes().partition(marker.encode())
    if not found:
        raise ValueError(f"{path}: provider foundation has no declared boundary")
    return prefix


def provider(check, fixture, contract):
    parts = [source_prefix(contract["foundation"], contract["prefix_marker"])]
    parts.extend(source_prefix(prefix["path"], prefix["marker"])
                 for prefix in contract.get("shared_prefixes", ()))
    if features := contract.get("provider_features"):
        parts.insert(0, f"#![feature({', '.join(features)})]\n".encode())
    source = check.folder / "provider.rs"
    source.write_bytes(b"".join(parts) + (fixture / "provider.rs").read_bytes())
    return source


def rustc(check, source, client, cases):
    bootstrap = {"RUSTC_BOOTSTRAP": "1"}
    artifact = check.folder / "libbase.rmeta"
    check.run("rustc-provider", [*RUSTC, "--crate-name", "base", "-o", artifact, source],
              environment=bootstrap, timeout=15)
    for case in cases:
        name = case["cfg"]
        codes = ()
        if case["status"]:
            codes = (case["rustc_message"],) if "rustc_message" in case else (f"error[{case['rustc']}]",)
        check.run(f"rustc-{name}", [*RUSTC, "--crate-name", name, "--extern", f"base={artifact}",
                                 "--cfg", name, "-o", check.folder / f"{name}.rmeta",
                                 client], case["status"], codes,
                  environment=bootstrap, timeout=15)


def candidate(check, source, client, contract):
    artifact = check.folder / "base.rgcrate"
    check.run("rustgm-provider", [check.app, *RUSTGM, "--crate-type=lib", "--crate-name", "base",
              "--crate-disambiguator", "source_nominal_fixture", "--source-core", IDENTITY,
              "--source-profile", "module-body-operations-v14", "--metadata-output", artifact,
              "-o", check.folder / "provider.lowir", source], timeout=30)
    if artifact.stat().st_size == 0:
        raise ValueError("provider emitted no RustGM crate metadata")
    client_kind = contract.get("client_kind", "lib")
    for case in contract["cases"]:
        name = case["cfg"]
        codes = (case["rustgm"],) if case["status"] else ()
        lowir = check.folder / f"{name}.lowir"
        check.run(f"rustgm-{name}", [check.app, *RUSTGM, f"--crate-type={client_kind}", "--crate-name", name,
                  "--crate-disambiguator", "source_nominal_client",
                  "--source-client", IDENTITY, "--extern", f"base={artifact}", "--cfg", name,
                  "-o", lowir, client],
                  case["status"], codes, timeout=15)
        if "lowir" in case:
            text = lowir.read_text()
            for fragment in case["lowir"]:
                if fragment not in text:
                    raise ValueError(f"{name}: missing expected LowIR {fragment!r}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    compiler = parser.add_mutually_exclusive_group(required=True)
    compiler.add_argument("--app", type=Path)
    compiler.add_argument("--oracle-only", action="store_true")
    parser.add_argument(
        "--case",
        choices=("non-exhaustive-transport", "source-nominal-layout",
                 "source-storage-transport", "source-union-transport", "source-wrapper-transport",
                 "source-panic-format-transport", "source-nominal-outlives"),
        default="non-exhaustive-transport",
    )
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    check = Check(args.app or Path("rustc"), args.out)
    fixture = FIXTURES / args.case
    contract = json.loads((fixture / "outcomes.json").read_text())
    source = provider(check, fixture, contract)
    client = fixture / "client.rs"
    rustc(check, source, client, contract["cases"])
    if args.app:
        candidate(check, source, client, contract)
    print(f"Checked {len(contract['cases'])} independent source nominal clients for {args.case}")


if __name__ == "__main__":
    main()
