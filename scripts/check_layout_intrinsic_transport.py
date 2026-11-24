#!/usr/bin/env python3
"""Check fresh layout intrinsic calls and their trait associated defaults."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_location_role_transport import role_address
from check_source_constants import COMMON, provider_source
from crate_qualification import Check

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "pa31/tests/spec/source-layout-intrinsics"
IDENTITY = "base@layout_intrinsics"
OPTIONS = [*COMMON, "--emit-lowir", "--cap-lints=allow", "--crate-disambiguator",
           "layout_intrinsics"]


def opponents(check, source, client, exporter, importer, artifact, contract, baseline):
    old = check.folder / "old.rgcrate"
    check.run("old-profile", [*exporter, "--source-profile", "module-bodies-v13",
              "--metadata-output", old, source], 1, [contract["old_profile"]], timeout=30)
    if old.exists():
        raise ValueError("old profile published unsupported layout declarations")
    prefix, marker, _ = source.read_bytes().partition(b"pub trait SizedTypeProperties")
    if not marker:
        raise ValueError("layout provider lost its trait-default boundary")
    intrinsic_source = check.folder / "intrinsics-only.rs"
    intrinsic_source.write_bytes(prefix)
    check.run("old-intrinsic-profile", [*exporter, "--source-profile", "module-bodies-v13",
              "--metadata-output", old, intrinsic_source], 1,
              [contract["old_intrinsic_profile"]], timeout=30)
    if old.exists():
        raise ValueError("old profile published an unsupported layout intrinsic")
    if baseline is not None:
        check.run("old-reader", [baseline.resolve(strict=True), *importer[1:],
                  "--extern", f"base={artifact}", client], 1, [contract["old_reader"]],
                  timeout=30)

    packet = artifact.read_bytes()
    role = role_address(packet, 255, "mem_size_const")
    foreign = check.folder / "foreign-role.rgcrate"
    foreign.write_bytes(packet[:role - 4] + (1).to_bytes(4, "little") + packet[role:])
    check.run("foreign-role", [*importer, "--extern", f"base={foreign}", client],
              1, [contract["foreign_role"]], timeout=30)

    length = (7).to_bytes(4, "little")
    marker = length + b"size_of"
    if packet.count(marker) != 2:
        raise ValueError("expected the layout intrinsic's exact declaration name and path")
    wrong = check.folder / "wrong-intrinsic.rgcrate"
    wrong.write_bytes(packet.replace(marker, length + b"payload"))
    check.run("wrong-intrinsic", [*importer, "--extern", f"base={wrong}", client],
              1, [contract["wrong_intrinsic"]], timeout=30)


def candidate(check, source, client, observer, contract, baseline):
    artifact = check.folder / "base.rgcrate"
    provider = check.folder / "provider.lowir"
    exporter = [check.app, *OPTIONS, "--crate-name", "base", "--source-core", IDENTITY]
    check.run("rustgm-provider", [*exporter, "--source-profile", "module-body-operations-v14",
              "--metadata-output", artifact, "-o", provider, source], timeout=30)
    lowered = check.folder / "client.lowir"
    importer = [check.app, *OPTIONS, "--crate-name", "client", "--source-client", IDENTITY]
    check.run("rustgm-client", [*importer, "--extern", f"base={artifact}", "-o", lowered,
              client], timeout=30)
    program = check.folder / "rustgm-client"
    backend.build_many([provider, lowered], program, native_objects=[observer])
    observed = check.run("rustgm-observe", [program], timeout=30).stdout
    if observed != contract["stdout"].encode():
        raise ValueError("fresh RustGM client changed its checked size or alignment")
    opponents(check, source, client, exporter, importer, artifact, contract, baseline)
    print(observed.decode(), end="")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--app", type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    check = Check(args.app or args.rustc, args.out)
    source = provider_source(check.folder, FIXTURE, contract)
    source.write_bytes(b"#![feature(intrinsics)]\n" + source.read_bytes())
    client = FIXTURE / contract["client"]
    observer = check.folder / "observer.o"
    check.run("cc-observer", ["cc", "-c", FIXTURE / contract["observer"], "-o", observer], timeout=30)
    pinned(check, args.rustc, source, client, observer, contract["stdout"].encode())
    if args.app is not None:
        candidate(check, source, client, observer, contract, args.baseline)


if __name__ == "__main__":
    main()
