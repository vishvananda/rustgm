#!/usr/bin/env python3
"""Compare separately built Try consumers with pinned Rust and reject altered roles."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_location_role_transport import role_address
from check_source_nominal_transport import FIXTURES, RUSTGM
from crate_qualification import Check


FIXTURE = FIXTURES / "source-try-protocol-transport"
IDENTITY = "base@try_roles"


def invalid_variants(check, importer, artifact, client, contract, baseline):
    packet = bytearray(artifact.read_bytes())
    continued = role_address(packet, 255, "Continue")
    interrupted = role_address(packet, 255, "Break")
    packet[continued:continued + 4], packet[interrupted:interrupted + 4] = (
        packet[interrupted:interrupted + 4], packet[continued:continued + 4]
    )
    forged = check.folder / "swapped-variants.rgcrate"
    forged.write_bytes(packet)
    check.run("swapped-variants", [*importer, "--extern", f"base={forged}",
              "-o", check.folder / "swapped.lowir", client],
              1, [contract["wire_variant"]], timeout=30)
    if baseline is not None:
        check.run("older-reader", [baseline.resolve(strict=True), *importer[1:],
                  "--extern", f"base={artifact}", "-o", check.folder / "older.lowir", client],
                  1, [contract["old_reader"]], timeout=30)


def qualify(check, contract, source, rustc, baseline):
    client = FIXTURE / contract["client"]
    observer = check.folder / "observer.o"
    check.run("observer", ["cc", "-c", FIXTURE / contract["observer"], "-o", observer], timeout=15)
    expected = contract["stdout"].encode()
    pinned(check, rustc, source, client, observer, expected)
    common = [check.app, *RUSTGM, "--crate-type=lib"]
    artifact = check.folder / "base.rgcrate"
    provider_lowir = check.folder / "provider.lowir"
    client_lowir = check.folder / "client.lowir"
    check.run("provider", [*common, "--crate-name", "base", "--crate-disambiguator", "try_roles",
                           "--source-core", IDENTITY, "--source-profile", "module-body-operations-v14",
                           "--metadata-output", artifact, "-o", provider_lowir, source], timeout=30)
    importer = [*common, "--crate-name", "client", "--crate-disambiguator", "try_roles_client",
                "--source-client", IDENTITY]
    check.run("client", [*importer, "--extern", f"base={artifact}", "-o", client_lowir, client],
              timeout=30)
    native = check.folder / "rustgm-try-client"
    backend.build_many([provider_lowir, client_lowir], native, native_objects=[observer])
    if check.run("execute", [native], timeout=15).stdout != expected:
        raise ValueError("the fresh client's actual success or residual path changed")
    invalid_variants(check, importer, artifact, client, contract, baseline)
    print(expected.decode(), end="")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    for variant in ("provider", "const_provider"):
        check = Check(args.app, args.out / variant)
        qualify(check, contract, FIXTURE / contract[variant], args.rustc, args.baseline)


if __name__ == "__main__":
    main()
