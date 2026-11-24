#!/usr/bin/env python3
"""Check omitted const arguments across a source interface."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_source_constants import COMMON, ROOT, provider_source
from crate_qualification import Check


FIXTURE = ROOT / "pa31/tests/spec/source-const-default-transport"
IDENTITY = "base@const_defaults"


def reject_forged_record(check, contract, artifact, importer, client, baseline):
    packet = artifact.read_bytes()
    # Record tag, two fields, then the two u128 field values in logical order.
    marker = b"\x01" + (2).to_bytes(4, "little") + (41).to_bytes(16, "little") + (1).to_bytes(16, "little")
    if packet.count(marker) != 1:
        raise ValueError("const default fixture lost its unique record payload")
    forged = check.folder / "forged-record.rgcrate"
    forged.write_bytes(packet.replace(marker, marker[:-16] + (2).to_bytes(16, "little")))
    options = ["-o", check.folder / "bad.lowir", client]
    check.run("forged-record", [*importer[:-2], "--extern", f"base={forged}", *options],
              1, [contract["forged_record"]], timeout=15)
    if baseline is not None:
        check.run("older-reader", [baseline.resolve(strict=True), *importer[1:], *options],
                  1, [contract["old_reader"]], timeout=15)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    args = parser.parse_args()
    check = Check(args.app, args.out)
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    marker = ROOT / contract["marker_provider"]
    marker_contract = json.loads((marker / "outcomes.json").read_text())
    source = provider_source(check.folder, marker, marker_contract)
    source.write_bytes(b"#![feature(adt_const_params)]\n" + source.read_bytes()
                       + b"\n" + (FIXTURE / contract["provider"]).read_bytes())
    client = FIXTURE / contract["client"]
    observer = check.folder / "observer.o"
    check.run("cc-observer", ["cc", "-c", FIXTURE / contract["observer"], "-o", observer], timeout=15)
    expected = contract["stdout"].encode()
    pinned(check, args.rustc, source, client, observer, expected)
    check.run("rustc-rejected", [args.rustc.resolve(strict=True), *COMMON, "--emit=metadata",
                                 "--cfg", "wrong_record", "--extern",
                                 f"base={check.folder / 'libbase.rlib'}", "-o",
                                 check.folder / "rejected.rmeta", client], 1, [contract["rustc_rejected"]],
              timeout=15, environment={"RUSTC_BOOTSTRAP": "1"})
    common = [check.app, *COMMON, "--emit-lowir", "--cap-lints=allow", "--crate-disambiguator", "const_defaults"]
    exporter = [*common, "--crate-name", "base", "--source-core", IDENTITY]
    artifact = check.folder / "base.rgcrate"
    provider_lowir = check.folder / "provider.lowir"
    check.run("provider", [*exporter, "--source-profile", "module-body-operations-v14",
                           "--metadata-output", artifact, "-o", provider_lowir, source], timeout=30)
    importer = [*common, "--crate-name", "client", "--source-client", IDENTITY, "--extern", f"base={artifact}"]
    client_lowir = check.folder / "client.lowir"
    check.run("client", [*importer, "-o", client_lowir, client], timeout=30)
    check.run("rustgm-rejected", [*importer, "--cfg", "wrong_record", "-o",
                                  check.folder / "rejected.lowir", client], 1,
              [contract["rustgm_rejected"]], timeout=15)
    reject_forged_record(check, contract, artifact, importer, client, args.baseline)
    native = check.folder / "rustgm-client"
    backend.build_many([provider_lowir, client_lowir], native, native_objects=[observer])
    if check.run("rustgm-observe", [native], timeout=15).stdout != expected:
        raise ValueError("fresh const defaults selected unexpected values")
    print(expected.decode(), end="")


if __name__ == "__main__":
    main()
