#!/usr/bin/env python3
"""Recheck the actual Option, Result, and Ordering roles in a fresh crate."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_location_role_transport import role_address
from check_source_nominal_transport import FIXTURES, RUSTGM
from crate_qualification import Check


FIXTURE = FIXTURES / "source-enum-role-transport"
IDENTITY = "base@enum_roles"


def invalid_roles(check, importer, artifact, client, contract, baseline):
    packet = artifact.read_bytes()
    some, none, option = [role_address(packet, 255, item) for item in ("Some", "None", "Option")]
    result = role_address(packet, 254, "Result")
    for name, patches in (("swapped_option", ((some, none), (none, some))),
                          ("redirected_result", ((result, option),))):
        changed = bytearray(packet)
        for target, origin in patches:
            changed[target:target + 4] = packet[origin:origin + 4]
        forged = check.folder / f"{name}.rgcrate"
        forged.write_bytes(changed)
        check.run(name, [*importer, "--extern", f"base={forged}", client], 1,
                  [contract[name]], timeout=30)
    if baseline is not None:
        check.run("older-reader", [baseline.resolve(strict=True), *importer[1:],
                  "--extern", f"base={artifact}", client], 1,
                  [contract["old_reader"]], timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    check = Check(args.app, args.out)
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    source, client = (FIXTURE / contract[name] for name in ("provider", "client"))
    observer = check.folder / "observer.o"
    check.run("observer", ["cc", "-c", FIXTURE / contract["observer"], "-o", observer], timeout=15)
    expected = contract["stdout"].encode()
    pinned(check, args.rustc, source, client, observer, expected)
    common = [check.app, *RUSTGM, "--crate-type=lib"]
    artifact = check.folder / "base.rgcrate"
    provider_lowir = check.folder / "provider.lowir"
    client_lowir = check.folder / "client.lowir"
    check.run("provider", [*common, "--crate-name", "base", "--crate-disambiguator", "enum_roles",
                           "--source-core", IDENTITY, "--source-profile", "module-body-operations-v14",
                           "--metadata-output", artifact, "-o", provider_lowir, source], timeout=30)
    importer = [*common, "--crate-name", "client", "--crate-disambiguator", "enum_roles_client",
                "--source-client", IDENTITY]
    check.run("client", [*importer, "--extern", f"base={artifact}", "-o", client_lowir, client],
              timeout=30)
    native = check.folder / "rustgm-enum-client"
    backend.build_many([provider_lowir, client_lowir], native, native_objects=[observer])
    if check.run("execute", [native], timeout=15).stdout != expected:
        raise ValueError("a fresh native client observed different enum results")
    invalid_roles(check, importer, artifact, client, contract, args.baseline)
    print(expected.decode(), end="")


if __name__ == "__main__":
    main()
