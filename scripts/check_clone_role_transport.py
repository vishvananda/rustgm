#!/usr/bin/env python3
"""Qualify imported Clone declarations and the provided clone_from body."""
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


def default_provider(check, source, contract):
    prefix, marker, _ = source.read_bytes().partition(contract["end"].encode())
    required = contract["required_method"].encode()
    if not marker or prefix.count(required) != 1:
        raise ValueError("Clone fixture lost the exact default insertion point")
    provider = check.folder / "provider.rs"
    provider.write_bytes(prefix.replace(required, required + (FIXTURE / contract["method"]).read_bytes()))
    return provider


def reject_coupled_borrows(check, exporter, source, contract):
    text = source.read_bytes()
    signature = contract["signature"].encode()
    if text.count(signature) != 1:
        raise ValueError("Clone fixture lost the exact clone_from signature")
    source = check.folder / "coupled-borrows.rs"
    source.write_bytes(text.replace(signature, contract["coupled_borrows"].encode()))
    output = source.with_suffix(".rgcrate")
    check.run("coupled-borrows", [*exporter, "--source-profile", PROFILE,
              "--metadata-output", output, source], 1, [contract["coupled_diagnostic"]], timeout=30)
    if output.exists():
        raise ValueError("altered clone_from borrow binders published a source artifact")


def reject_ordinary_const_impl(check, importer, artifact, client, rustc, contract):
    text = client.read_bytes()
    marker = contract["ordinary_impl"].encode()
    if text.count(marker) != 1:
        raise ValueError("Clone const client lost its exact const implementation")
    source = check.folder / "ordinary-impl.rs"
    source.write_bytes(text.replace(marker, marker.removeprefix(b"const ")))
    check.run("rustc-ordinary-impl", [rustc.resolve(strict=True), "--edition=2024", "--crate-type=lib",
              "--emit=metadata", "--extern", f"base={check.folder / 'libbase.rlib'}",
              "-o", check.folder / "ordinary-impl.rmeta", source], 1, timeout=30,
              environment={"RUSTC_BOOTSTRAP": "1"})
    check.run("rustgm-ordinary-impl", [*importer, "--extern", f"base={artifact}",
              "-o", check.folder / "ordinary-impl.lowir", source], 1, timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", required=True, type=Path)
    parser.add_argument("--rustc", required=True, type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--case", choices=("roles", "default", "constant"), default="roles")
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    check = Check(args.app, args.out)
    source = FIXTURE / contract["provider"]
    case = contract.get(args.case, contract)
    if args.case != "roles":
        source = default_provider(check, source, contract["default"])
    client = FIXTURE / case["client"]
    observer = check.folder / "observer.o"
    observe = FIXTURE / case.get("observer", contract["observer"])
    check.run("observer", ["cc", "-c", observe, "-o", observer], timeout=15)
    expected = case.get("stdout", contract["stdout"]).encode()
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
    if args.case == "constant":
        reject_ordinary_const_impl(check, importer, artifact, client, args.rustc, case)
    elif args.case == "default":
        reject_coupled_borrows(check, exporter, source, case)
    else:
        reject_providers(check, exporter, source, contract)
        reject_consumers(check, importer, artifact, client, args.baseline, contract)
    print(expected.decode(), end="")


if __name__ == "__main__":
    main()
