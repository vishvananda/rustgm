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


def blanket_inputs(check, contract, provider, baseline_client, const):
    provider += (FIXTURE / contract["provider"]).read_bytes()
    baseline_client = baseline_client.read_bytes()
    begin, end = (marker.encode() for marker in contract["client_without_local_impl"])
    head, opening, rest = baseline_client.partition(begin)
    _, closing, tail = rest.partition(end)
    if not opening or not closing:
        raise ValueError("iteration fixture lost the exact local implementation boundary")
    client = head + closing + tail
    if const:
        marker = contract["const"]["implementation"].encode()
        if client.count(marker) != 1:
            raise ValueError("iteration fixture lost its actual Iterator implementation")
        client = (b"#![feature(const_trait_impl)]\n" + client.replace(marker, b"const " + marker)
                  + (FIXTURE / contract["const"]["client_suffix"]).read_bytes())
    client_file = check.folder / "client.rs"
    client_file.write_bytes(client)
    return b"#![feature(const_destruct)]\n" + provider, client_file


def reject_ordinary_premise(check, exporter, source, contract):
    text = source.read_bytes()
    marker = contract["ordinary_bound"].encode()
    if text.count(marker) != 1:
        raise ValueError("iteration fixture lost its const blanket bound")
    wrong = check.folder / "ordinary-premise.rs"
    wrong.write_bytes(text.replace(marker, marker.replace(b"[const] ", b"")))
    output = check.folder / "ordinary-premise.rgcrate"
    check.run("ordinary-premise", [*exporter, "--source-profile", PROFILE,
              "--metadata-output", output, wrong], 1, [contract["ordinary_bound_diagnostic"]], timeout=30)
    if output.exists():
        raise ValueError("an unsupported ordinary premise published the const blanket")


def reject_ordinary_const_client(check, importer, artifact, client, rustc, contract):
    text = client.read_bytes()
    marker = b"const " + contract["implementation"].encode()
    if text.count(marker) != 1:
        raise ValueError("const iteration fixture lost its exact implementation")
    wrong = check.folder / "ordinary-iterator.rs"
    wrong.write_bytes(text.replace(marker, marker.removeprefix(b"const ")))
    check.run("rustc-ordinary-iterator", [rustc.resolve(strict=True), "--edition=2024",
              "--crate-type=lib", "--emit=metadata", "--extern", f"base={check.folder / 'libbase.rlib'}",
              "-o", check.folder / "ordinary-iterator.rmeta", wrong], 1, timeout=30,
              environment={"RUSTC_BOOTSTRAP": "1"})
    check.run("rustgm-ordinary-iterator", [*importer, "--extern", f"base={artifact}",
              "-o", check.folder / "ordinary-iterator.lowir", wrong], 1, timeout=30)


def size_hint_inputs(contract, blanket, provider):
    anchor = contract["provider_anchor"].encode()
    if provider.count(anchor) != 1:
        raise ValueError("iteration fixture lost its exact next declaration anchor")
    member = (FIXTURE / contract["provider_member"]).read_bytes()
    drop = source_prefix(FIXTURE / blanket["provider"], "const impl<I:")
    return b"#![feature(const_destruct)]\n" + provider.replace(anchor, member + anchor) + drop


def reject_size_hint_rows(check, exporter, importer, source, artifact, client, contract):
    text = source.read_bytes()
    receiver = b"fn size_hint(&self)"
    if text.count(receiver) != 1:
        raise ValueError("iteration fixture lost the exact size_hint receiver")
    wrong = check.folder / "mutating-size-hint.rs"
    wrong.write_bytes(text.replace(receiver, b"fn size_hint(&mut self)"))
    rejected = check.folder / "mutating-size-hint.rgcrate"
    check.run("wrong-size-hint-receiver", [*exporter, "--source-profile", PROFILE,
              "--metadata-output", rejected, wrong], 1, [contract["wrong_receiver"]], timeout=30)
    if rejected.exists():
        raise ValueError("an altered size_hint declaration produced an artifact")

    # The actual body has one aggregate node with a range operand [0, 2).
    range_start = bytes((3, 1, 9)) + (0).to_bytes(4, "little")
    tuple_node = range_start + (2).to_bytes(4, "little")
    packet = artifact.read_bytes()
    if packet.count(tuple_node) != 1:
        raise ValueError("size_hint original lost its one two-operand aggregate row")
    forged = check.folder / "short-tuple.rgcrate"
    forged.write_bytes(packet.replace(tuple_node, range_start + (1).to_bytes(4, "little")))
    check.run("wrong-size-hint-arity", [*importer, "--extern", f"base={forged}", client],
              1, [contract["wrong_arity"]], timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", required=True, type=Path)
    parser.add_argument("--rustc", required=True, type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--case", choices=("roles", "blanket", "blanket-const", "size-hint"),
                        default="roles")
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    check = Check(args.app, args.out)
    source = check.folder / "provider.rs"
    foundation = source_prefix(contract["foundation"], contract["prefix_marker"])
    provider = (b"#![feature(const_trait_impl)]\n" + foundation
                + (FIXTURE / contract["provider"]).read_bytes())
    client = FIXTURE / contract["client"]
    constant = args.case == "blanket-const"
    observed = contract
    if args.case == "size-hint":
        observed = contract["size_hint"]
        provider = size_hint_inputs(observed, contract["blanket"], provider)
        client = FIXTURE / observed["client"]
    elif args.case != "roles":
        provider, client = blanket_inputs(check, contract["blanket"], provider, client, constant)
        if constant:
            observed = contract["blanket"]["const"]
    source.write_bytes(provider)
    observer = check.folder / "observer.o"
    check.run("observer", ["cc", "-c", FIXTURE / observed["observer"], "-o", observer], timeout=15)
    expected = observed["stdout"].encode()
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
    if constant or args.case == "size-hint":
        reject_ordinary_const_client(check, importer, artifact, client, args.rustc, observed)
    if args.case == "size-hint":
        reject_size_hint_rows(check, exporter, importer, source, artifact, client, observed)
        if args.baseline is not None:
            check.run("old-size-hint-reader", [args.baseline.resolve(strict=True), *importer[1:],
                      "--extern", f"base={artifact}", client], 1, [observed["old_reader"]],
                      timeout=30)
    elif args.case == "blanket":
        reject_ordinary_premise(check, exporter, source, contract["blanket"])
    elif args.case == "roles":
        rejected_providers(check, exporter, source, contract)
        rejected_consumers(check, importer, client, artifact, args.baseline, contract)
    print(expected.decode(), end="")


if __name__ == "__main__":
    main()
