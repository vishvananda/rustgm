#!/usr/bin/env python3
"""Run fresh native clients of separately published source artifacts."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_source_nominal_transport import FIXTURES, RUSTGM, source_prefix
from crate_qualification import Check


FOUNDATION = "pa31/tests/spec/source-enum-role-transport/provider.rs"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", required=True,
                        choices=("source-reference-cast-transport", "source-marker-trait-transport"))
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    check = Check(args.app, args.out)
    fixture = FIXTURES / args.case
    contract = json.loads((fixture / "outcomes.json").read_text())
    disambiguator = args.case.replace("-", "_")
    identity = f"base@{disambiguator}"
    source = check.folder / "provider.rs"
    features = ", ".join(contract["features"])
    source.write_bytes(f"#![feature({features})]\n".encode()
                       + source_prefix(FOUNDATION, '#[rustc_diagnostic_item = "Option"]')
                       + (fixture / "provider.rs").read_bytes())
    client = fixture / "client.rs"
    observer = check.folder / "observer.o"
    check.run("observer", ["cc", "-c", fixture / "observe.c", "-o", observer], timeout=15)
    expected = contract["stdout"].encode()
    pinned(check, args.rustc, source, client, observer, expected)
    common = [check.app, *RUSTGM, "--crate-type=lib"]
    exporter = [*common, "--crate-name", "base", "--crate-disambiguator", disambiguator,
                "--source-core", identity]
    artifact, provider, consumer = (check.folder / name for name in
                                    ("base.rgcrate", "provider.lowir", "client.lowir"))
    check.run("provider", [*exporter, "--source-profile", "module-body-operations-v14",
                           "--metadata-output", artifact, "-o", provider, source], timeout=30)
    importer = [*common, "--crate-name", "client", "--crate-disambiguator", f"{disambiguator}_client",
                "--source-client", identity]
    check.run("client", [*importer, "--extern", f"base={artifact}", "-o", consumer, client], timeout=30)
    native = check.folder / "observe"
    backend.build_many([provider, consumer], native, native_objects=[observer])
    if check.run("execute", [native], timeout=15).stdout != expected:
        raise ValueError("the fresh native client differed from the declared stdout")
    old = check.folder / "old.rgcrate"
    check.run("older-profile", [*exporter, "--source-profile", "module-bodies-v13",
                                "--metadata-output", old, source], 1, [contract["older_profile"]],
              timeout=30)
    if old.exists():
        raise ValueError("older source profile published the new metadata")
    if args.baseline is not None:
        check.run("older-reader", [args.baseline.resolve(strict=True), *importer[1:],
                  "--extern", f"base={artifact}", client], 1, [contract["older_reader"]], timeout=30)
    print(expected.decode(), end="")


if __name__ == "__main__":
    main()
