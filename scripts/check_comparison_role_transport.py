#!/usr/bin/env python3
"""Recheck comparison roles and their provided bodies in fresh clients."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_location_role_transport import role_address
from check_source_nominal_transport import FIXTURES, RUSTGM, source_prefix
from crate_qualification import Check


FIXTURE = FIXTURES / "source-comparison-role-transport"
FOUNDATION = "pa31/tests/spec/source-enum-role-transport/provider.rs"
IDENTITY = "base@compare_roles"


def rejected_bindings(check, importer, artifact, client, contract, baseline):
    packet = artifact.read_bytes()
    offset = role_address(packet, 255, "eq") - 4
    forged = check.folder / "foreign-comparison.rgcrate"
    forged.write_bytes(packet[:offset] + (1).to_bytes(4, "little") + packet[offset + 4:])
    check.run("foreign-role", [*importer, "--extern", f"base={forged}", client], 1,
              [contract["foreign_role"]], timeout=30)
    if baseline is not None:
        check.run("older-reader", [baseline.resolve(strict=True), *importer[1:],
                  "--extern", f"base={artifact}", client], 1, [contract["old_reader"]],
                  timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    check = Check(args.app, args.out)
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    source = check.folder / "provider.rs"
    source.write_bytes(b"#![feature(auto_traits)]\n"
                       + source_prefix(FOUNDATION, '#[rustc_diagnostic_item = "Option"]')
                       + (FIXTURE / contract["provider"]).read_bytes())
    client = FIXTURE / contract["client"]
    observer = check.folder / "observer.o"
    check.run("observer", ["cc", "-c", FIXTURE / contract["observer"], "-o", observer], timeout=15)
    expected = contract["stdout"].encode()
    pinned(check, args.rustc, source, client, observer, expected)
    common = [check.app, *RUSTGM, "--crate-type=lib"]
    artifact = check.folder / "base.rgcrate"
    provider_lowir = check.folder / "provider.lowir"
    client_lowir = check.folder / "client.lowir"
    exporter = [*common, "--crate-name", "base", "--crate-disambiguator", "compare_roles",
                "--source-core", IDENTITY]
    check.run("provider", [*exporter, "--source-profile", "module-body-operations-v14",
                           "--metadata-output", artifact, "-o", provider_lowir, source], timeout=30)
    importer = [*common, "--crate-name", "client", "--crate-disambiguator", "compare_roles_client",
                "--source-client", IDENTITY]
    check.run("client", [*importer, "--extern", f"base={artifact}", "-o", client_lowir, client],
              timeout=30)
    native = check.folder / "rustgm-comparison-client"
    backend.build_many([provider_lowir, client_lowir], native, native_objects=[observer])
    if check.run("execute", [native], timeout=15).stdout != expected:
        raise ValueError("a fresh native client observed different comparisons")
    rejected_bindings(check, importer, artifact, client, contract, args.baseline)
    old = check.folder / "older-profile.rgcrate"
    check.run("older-profile", [*exporter, "--source-profile", "module-bodies-v13",
                                "--metadata-output", old, source], 1, timeout=30)
    if old.exists():
        raise ValueError("older source profile published a conditional comparison default")
    print(expected.decode(), end="")


if __name__ == "__main__":
    main()
