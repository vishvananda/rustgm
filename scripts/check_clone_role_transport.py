#!/usr/bin/env python3
"""Qualify const Clone and its checked TrivialClone marker in a fresh client."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_location_role_transport import role_address
from check_source_nominal_transport import FIXTURES, RUSTGM
from crate_qualification import Check

FIXTURE = FIXTURES / "source-clone-transport"
IDENTITY = "base@clone_fixture"
PROFILE = "module-body-operations-v14"


def reject_providers(check, exporter, source, contract):
    text = source.read_bytes()
    bound = b"trait TrivialClone: [const] Clone"
    if text.count(bound) != 1:
        raise ValueError("Clone fixture lost its exact conditional supertrait")
    wrong = check.folder / "ordinary-supertrait.rs"
    wrong.write_bytes(text.replace(bound, b"trait TrivialClone: Clone"))
    unrelated = check.folder / "unrelated-specialization.rs"
    unrelated.write_bytes(text + b"\n#[rustc_unsafe_specialization_marker]\npub trait Unrelated {}\n")
    for path, expected in ((wrong, "ordinary_supertrait"), (unrelated, "unrelated_marker")):
        output = path.with_suffix(".rgcrate")
        check.run(path.stem, [*exporter, "--source-profile", PROFILE,
                  "--metadata-output", output, path], 1, [contract[expected]], timeout=30)
        if output.exists():
            raise ValueError("unsupported specialization policy published a source artifact")


def reject_consumers(check, importer, artifact, client, baseline, contract):
    packet = bytearray(artifact.read_bytes())
    offset = role_address(packet, 255, "trivial_clone") - 4
    packet[offset:offset + 4] = (1).to_bytes(4, "little")
    forged = check.folder / "foreign-marker.rgcrate"
    forged.write_bytes(packet)
    check.run("foreign-marker", [*importer, "--extern", f"base={forged}", client],
              1, [contract["foreign_owner"]], timeout=30)
    if baseline is not None:
        check.run("old-reader", [baseline.resolve(strict=True), *importer[1:],
                  "--extern", f"base={artifact}", client], 1, [contract["old_reader"]], timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", required=True, type=Path)
    parser.add_argument("--rustc", required=True, type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    check = Check(args.app, args.out)
    source = FIXTURE / contract["provider"]
    client = FIXTURE / contract["client"]
    observer = check.folder / "observer.o"
    check.run("observer", ["cc", "-c", FIXTURE / contract["observer"], "-o", observer], timeout=15)
    expected = contract["stdout"].encode()
    pinned(check, args.rustc, source, client, observer, expected)

    common = [check.app, *RUSTGM, "--crate-type=lib"]
    exporter = [*common, "--crate-name", "base", "--crate-disambiguator", "clone_fixture",
                "--source-core", IDENTITY]
    importer = [*common, "--crate-name", "client", "--crate-disambiguator", "clone_fixture_client",
                "--source-client", IDENTITY]
    artifact = check.folder / "base.rgcrate"
    provider_lowir = check.folder / "provider.lowir"
    client_lowir = check.folder / "client.lowir"
    check.run("provider", [*exporter, "--source-profile", PROFILE, "--metadata-output",
              artifact, "-o", provider_lowir, source], timeout=30)
    check.run("client", [*importer, "--extern", f"base={artifact}", "-o", client_lowir,
              client], timeout=30)
    native = check.folder / "observe"
    backend.build_many([provider_lowir, client_lowir], native, native_objects=[observer])
    if check.run("execute", [native], timeout=15).stdout != expected:
        raise ValueError("the fresh client changed the imported const Clone contract")
    reject_providers(check, exporter, source, contract)
    reject_consumers(check, importer, artifact, client, args.baseline, contract)
    print(expected.decode(), end="")


if __name__ == "__main__":
    main()
