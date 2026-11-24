#!/usr/bin/env python3
"""Check exact Rust panic linkage from two private provider declarations."""
import argparse
import json
from pathlib import Path
import struct

import backend
from check_source_nominal_transport import FIXTURES, RUSTGM, provider
from crate_qualification import Check


FIXTURE = FIXTURES / "source-weak-panic-transport"
IDENTITY = "base@weak_panic_control"


def exact_marker(check, importer, artifact, contract, client):
    symbol = contract["symbol"].encode()
    data = artifact.read_bytes()
    marker = struct.pack("<II", 0xFFFFFFFE, len(symbol)) + symbol
    if data.count(marker) != contract["calls"]:
        raise ValueError("the artifact omitted an independently named weak panic declaration")
    for name, mutation in [
        ("wrong_symbol", marker[:-1] + b"X"),
        ("wrong_form", b"\xff" + marker[1:]),
    ]:
        forged = check.folder / f"{name}.rgcrate"
        forged.write_bytes(data.replace(marker, mutation, 1))
        check.run(name, [*importer, "--extern", f"base={forged}", "-o",
                         check.folder / f"{name}.lowir", client], 1, [contract[name]], timeout=15)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    check = Check(args.app, args.out)
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    prefix = FIXTURES / contract["nominal_fixture"]
    source = provider(check, prefix, json.loads((prefix / "outcomes.json").read_text()))
    source.write_bytes(source.read_bytes() + b"\n" + (FIXTURE / contract["provider"]).read_bytes())
    client = FIXTURE / contract["client"]
    metadata = check.folder / "libbase.rmeta"
    oracle = [args.rustc.resolve(strict=True), "--edition=2024", "--emit=metadata",
              "--crate-type=lib", "--cfg", "rustc_oracle"]
    bootstrap = {"RUSTC_BOOTSTRAP": "1"}
    check.run("rustc-provider", [*oracle, "--crate-name", "base", "-o", metadata, source],
              timeout=15, environment=bootstrap)
    check.run("rustc-client", [*oracle, "--crate-name", "client", "--extern", f"base={metadata}",
                              "-o", check.folder / "libclient.rmeta", client],
              timeout=15, environment=bootstrap)
    artifact = check.folder / "base.rgcrate"
    provider_lowir = check.folder / "provider.lowir"
    client_lowir = check.folder / "client.lowir"
    check.run("provider", [check.app, *RUSTGM, "--crate-type=lib", "--crate-name", "base",
                           "--crate-disambiguator", "weak_panic_control", "--source-core", IDENTITY,
                           "--source-profile", "module-body-operations-v14", "--metadata-output",
                           artifact, "-o", provider_lowir, source], timeout=30)
    importer = [check.app, *RUSTGM, "--crate-type=bin", "--crate-name", "client",
                "--crate-disambiguator", "weak_panic_consumer", "--source-client", IDENTITY]
    check.run("client", [*importer, "--extern", f"base={artifact}", "-o", client_lowir, client],
              timeout=30)
    text = client_lowir.read_text()
    symbol = contract["symbol"]
    if text.count(f"function @{symbol}(") != 1 or text.count(f"call void @{symbol}(") != contract["calls"]:
        raise ValueError("both freshly imported calls must reach the one user handler symbol")
    program = check.folder / "client"
    backend.build_many([provider_lowir, client_lowir], program)
    check.run("linked-client", [program], timeout=15)
    exact_marker(check, importer, artifact, contract, client)
    print("Two fresh private panic calls linked to the same user handler.")


if __name__ == "__main__":
    main()
