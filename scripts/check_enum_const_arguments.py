#!/usr/bin/env python3
"""Check enum const arguments in imported generic function bodies."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_source_constants import COMMON, ROOT, provider_source
from crate_qualification import Check


FIXTURE = ROOT / "pa31/tests/spec/source-enum-const-arguments"
IDENTITY = "base@enum_consts"


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
    source.write_bytes(b"#![feature(min_adt_const_params)]\n" + source.read_bytes()
                       + b"\n" + (FIXTURE / contract["provider"]).read_bytes())
    client = FIXTURE / contract["client"]
    observer = check.folder / "observer.o"
    check.run("cc-observer", ["cc", "-c", FIXTURE / contract["observer"], "-o", observer], timeout=15)
    expected = contract["stdout"].encode()
    pinned(check, args.rustc, source, client, observer, expected)
    common = [check.app, *COMMON, "--emit-lowir", "--cap-lints=allow", "--crate-disambiguator", "enum_consts"]
    exporter = [*common, "--crate-name", "base", "--source-core", IDENTITY]
    artifact = check.folder / "base.rgcrate"
    provider_lowir = check.folder / "provider.lowir"
    check.run("provider", [*exporter, "--source-profile", "module-body-operations-v14",
                           "--metadata-output", artifact, "-o", provider_lowir, source], timeout=30)
    importer = [*common, "--crate-name", "client", "--source-client", IDENTITY, "--extern", f"base={artifact}"]
    client_lowir = check.folder / "client.lowir"
    check.run("client", [*importer, "-o", client_lowir, client], timeout=30)
    if args.baseline is not None:
        check.run("older-reader", [args.baseline.resolve(strict=True), *importer[1:],
                  "-o", check.folder / "old.lowir", client], 1, [contract["old_reader"]], timeout=20)
    native = check.folder / "rustgm-client"
    backend.build_many([provider_lowir, client_lowir], native, native_objects=[observer])
    if check.run("rustgm-observe", [native], timeout=15).stdout != expected:
        raise ValueError("the imported functions changed their closed enum arguments")
    print(expected.decode(), end="")


if __name__ == "__main__":
    main()
