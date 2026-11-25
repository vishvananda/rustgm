#!/usr/bin/env python3
"""Compare an imported caller-location accessor with separately built pinned rustc."""
import argparse
import json
from pathlib import Path

import backend
from check_location_role_transport import FIXTURE
from check_source_constants import COMMON, provider_source
from crate_qualification import Check

IDENTITY = "base@caller_forwarding"
OPTIONS = [*COMMON, "--emit-lowir", "--cap-lints=allow", "--crate-disambiguator",
           "caller_forwarding"]


def pinned(check, rustc, source, client, observer, expected):
    options = [rustc.resolve(strict=True), "--edition=2024", "-C", "panic=abort",
               "--cfg", "rustc_oracle"]
    environment = {"RUSTC_BOOTSTRAP": "1"}
    library = check.folder / "libbase.rlib"
    check.run("rustc-provider", [*options, "--crate-type=rlib", "--crate-name", "base",
              "-o", library, source], timeout=30, environment=environment)
    consumer = check.folder / "pinned-client.o"
    check.run("rustc-client", [*options, "--crate-type=lib", "--crate-name", "client",
              "--extern", f"base={library}", "--emit=obj", "-o", consumer, client],
              timeout=30, environment=environment)
    program = check.folder / "pinned"
    check.run("rustc-link", ["cc", "-o", program, observer, consumer, library], timeout=30)
    if check.run("rustc-observe", [program], timeout=30).stdout != expected:
        raise ValueError("pinned rustc disagreed with the declared runtime or CTFE coordinates")


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
        raise ValueError("fresh RustGM client changed the runtime or CTFE coordinates")
    opponents(check, source, client, artifact, exporter, importer, contract, baseline)
    print(observed.decode(), end="")


def opponents(check, source, client, artifact, exporter, importer, contract, baseline):
    old = check.folder / "old.rgcrate"
    check.run("old-profile", [*exporter, "--source-profile", "module-bodies-v13",
              "--metadata-output", old, source], 1, [contract["older_profile"]], timeout=30)
    if old.exists():
        raise ValueError("old profile published an unsupported intrinsic")
    if baseline is not None:
        check.run("old-reader", [baseline.resolve(strict=True), *importer[1:],
                  "--extern", f"base={artifact}", client], 1, [contract["older_reader"]],
                  timeout=30)

    text = source.read_bytes()
    marker = b"    #[track_caller]\n"
    if text.count(marker) != 1:
        raise ValueError("expected one tracked forwarding source declaration")
    untracked = check.folder / "untracked-provider.rs"
    untracked.write_bytes(text.replace(marker, b""))
    untracked_artifact = check.folder / "untracked.rgcrate"
    untracked_provider = check.folder / "untracked-provider.lowir"
    check.run("untracked-provider", [*exporter, "--source-profile", "module-body-operations-v14",
              "--metadata-output", untracked_artifact, "-o", untracked_provider, untracked],
              timeout=30)
    untracked_lowered = check.folder / "untracked-client.lowir"
    check.run("untracked-client", [*importer, "--extern", f"base={untracked_artifact}",
              "-o", untracked_lowered, client], timeout=30)
    untracked_program = check.folder / "rustgm-untracked"
    observer = check.folder / "observer.o"
    backend.build_many([untracked_provider, untracked_lowered], untracked_program,
                       native_objects=[observer])
    untracked_observed = check.run("rustgm-untracked-observe", [untracked_program],
                                   timeout=30).stdout
    if untracked_observed != contract["untracked_stdout"].encode():
        raise ValueError("untracked provider differed from expected provider coordinates")

    packet = artifact.read_bytes()
    length = (15).to_bytes(4, "little")
    marker = length + b"caller_location"
    if packet.count(marker) != 2:
        raise ValueError("expected the intrinsic's exact declaration name and path component")
    forged = check.folder / "forged-identity.rgcrate"
    forged.write_bytes(packet.replace(marker, length + b"caller_position"))
    check.run("forged-identity", [*importer, "--extern", f"base={forged}", client],
              1, [contract["forged_identity"]], timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--app", type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    contract = json.loads((FIXTURE / "forwarding.json").read_text())
    check = Check(args.app or args.rustc, args.out)
    source = provider_source(check.folder, FIXTURE, contract)
    source.write_bytes(b"#![feature(intrinsics, pattern_types)]\n" + source.read_bytes())
    client = FIXTURE / contract["client"]
    observer = check.folder / "observer.o"
    check.run("cc-observer", ["cc", "-c", FIXTURE / contract["observer"], "-o", observer], timeout=30)
    pinned(check, args.rustc, source, client, observer, contract["stdout"].encode())
    if args.app is not None:
        candidate(check, source, client, observer, contract, args.baseline)


if __name__ == "__main__":
    main()
