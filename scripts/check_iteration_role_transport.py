#!/usr/bin/env python3
"""Qualify required for-loop roles in a separately built native client."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_location_role_transport import role_address
from check_source_nominal_transport import FIXTURES, RUSTGM, source_prefix
from crate_qualification import Check

FIXTURE = FIXTURES / "source-iteration-transport"
IDENTITY = "base@iteration_roles"
PROFILE = "module-body-operations-v14"


def rejected_providers(check, command, source, contract):
    text = source.read_bytes()
    bound = b"Iterator<Item=Self::Item>"
    if text.count(bound) != 1:
        raise ValueError("iteration provider lost its exact associated result equality")
    wrong = check.folder / "wrong-bound.rs"
    wrong.write_bytes(text.replace(bound, b"Iterator<Item=usize>"))
    output = check.folder / "wrong-bound.rgcrate"
    check.run("wrong-bound", [*command, "--source-profile", PROFILE,
              "--metadata-output", output, wrong], 1, [contract["wrong_bound"]], timeout=30)
    older = check.folder / "old-provider.rs"
    independent_v14 = (b'#[lang="Option"] ', b'#[lang="Some"] ', b'#[lang="None"] ',
                       b'#[rustc_skip_during_method_dispatch(array, boxed_slice)]\n')
    for marker in independent_v14:
        if text.count(marker) != 1:
            raise ValueError("old profile control lost an independent v14 declaration")
        text = text.replace(marker, b"")
    older.write_bytes(text)
    old = check.folder / "old.rgcrate"
    check.run("old-profile", [*command, "--source-profile", "module-bodies-v13",
              "--metadata-output", old, older], 1, [contract["old_profile"]], timeout=30)
    if old.exists() or output.exists():
        raise ValueError("unsupported declarations produced an artifact")


def rejected_consumers(check, importer, client, artifact, baseline, contract):
    packet = bytearray(artifact.read_bytes())
    next_method = role_address(packet, 255, "next")
    into_iter = role_address(packet, 255, "into_iter")
    packet[next_method:next_method + 4], packet[into_iter:into_iter + 4] = (
        packet[into_iter:into_iter + 4], packet[next_method:next_method + 4]
    )
    forged = check.folder / "wrong-owner.rgcrate"
    forged.write_bytes(packet)
    check.run("wrong-owner", [*importer, "--extern", f"base={forged}", client],
              1, [contract["wrong_owner"]], timeout=30)
    if baseline is not None:
        check.run("old-reader", [baseline.resolve(strict=True), *importer[1:],
                  "--extern", f"base={artifact}", client],
                  1, [contract["old_reader"]], timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", required=True, type=Path)
    parser.add_argument("--rustc", required=True, type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    check = Check(args.app, args.out)
    source = check.folder / "provider.rs"
    foundation = source_prefix(contract["foundation"], contract["prefix_marker"])
    source.write_bytes(b"#![feature(const_trait_impl)]\n" + foundation
                       + (FIXTURE / contract["provider"]).read_bytes())
    client = FIXTURE / contract["client"]
    observer = check.folder / "observer.o"
    check.run("observer", ["cc", "-c", FIXTURE / contract["observer"], "-o", observer], timeout=15)
    expected = contract["stdout"].encode()
    pinned(check, args.rustc, source, client, observer, expected)

    common = [check.app, *RUSTGM, "--crate-type=lib"]
    exporter = [*common, "--crate-name", "base", "--crate-disambiguator", "iteration_roles",
                "--source-core", IDENTITY]
    importer = [*common, "--crate-name", "client", "--crate-disambiguator", "iteration_client",
                "--source-client", IDENTITY]
    artifact = check.folder / "base.rgcrate"
    provider_lowir = check.folder / "provider.lowir"
    client_lowir = check.folder / "client.lowir"
    check.run("rustgm-provider", [*exporter, "--source-profile", PROFILE, "--metadata-output",
              artifact, "-o", provider_lowir, source], timeout=30)
    check.run("rustgm-client", [*importer, "--extern", f"base={artifact}", "-o", client_lowir,
              client], timeout=30)
    native = check.folder / "rustgm-iteration-client"
    backend.build_many([provider_lowir, client_lowir], native, native_objects=[observer])
    if check.run("rustgm-observe", [native], timeout=15).stdout != expected:
        raise ValueError("fresh RustGM client changed either branch of the imported iteration")
    rejected_providers(check, exporter, source, contract)
    rejected_consumers(check, importer, client, artifact, args.baseline, contract)
    print(expected.decode(), end="")


if __name__ == "__main__":
    main()
