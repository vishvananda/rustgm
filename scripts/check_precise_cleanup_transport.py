#!/usr/bin/env python3
"""Check an imported precise-drop definition with a feature-independent reader."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_source_nominal_transport import RUSTGM
from crate_qualification import Check

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "pa31/tests/spec/source-precise-cleanup-transport"
IDENTITY = "base@precise_cleanup"
PROFILE = "module-body-operations-v14"


def candidate(check, sources, observer, contract, baseline):
    source, conservative, client = sources
    targets = [check.app, *RUSTGM, "--crate-type=lib", "--crate-disambiguator", "precise_cleanup"]
    exporter = [*targets, "--crate-name", "base", "--source-core", IDENTITY,
                "--source-profile", PROFILE]
    artifact = check.folder / "base.rgcrate"
    provider = check.folder / "provider.lowir"
    check.run("rustgm-provider", [*exporter, "--metadata-output", artifact,
                                 "-o", provider, source], timeout=30)
    lowered = check.folder / "client.lowir"
    importer = [*targets, "--crate-name", "client", "--source-client", IDENTITY,
                "--extern", f"base={artifact}"]
    check.run("rustgm-client", [*importer, "-o", lowered, client], timeout=30)
    if baseline is not None:
        check.run("older-reader", [baseline.resolve(strict=True), *importer[1:], client],
                  1, (contract["older_reader"],), timeout=30)
    program = check.folder / "rustgm-native"
    backend.build_many([provider, lowered], program, native_objects=[observer])
    observed = check.run("rustgm-observe", [program], timeout=30).stdout
    if observed != contract["stdout"].encode():
        raise ValueError("fresh RustGM reader changed the moved or empty enum value")
    rejected = check.folder / "conservative.rgcrate"
    check.run("rustgm-conservative", [*exporter, "--metadata-output", rejected,
                                     "-o", check.folder / "conservative.lowir", conservative],
              1, (contract["conservative_rustgm"],), timeout=30)
    if rejected.exists():
        raise ValueError("conservative provider published an unchecked destructor")
    print(observed.decode(), end="")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rustc", required=True, type=Path)
    parser.add_argument("--app", type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    check = Check(args.app or args.rustc, args.out)
    source = FIXTURE / contract["provider"]
    client = FIXTURE / contract["client"]
    observer = check.folder / "observer.o"
    check.run("cc-observer", ["cc", "-c", FIXTURE / contract["observer"], "-o", observer], timeout=30)
    conservative = check.folder / "conservative.rs"
    marker = b", const_precise_live_drops"
    original = source.read_bytes()
    if original.count(marker) != 1:
        raise ValueError("expected one provider precise-live-drop request")
    conservative.write_bytes(original.replace(marker, b""))
    pinned(check, args.rustc, source, client, observer, contract["stdout"].encode())
    check.run("rustc-conservative", [args.rustc.resolve(strict=True), "--edition=2024",
                                    "--crate-type=rlib", "--crate-name", "base", "--emit=metadata",
                                    "-o", check.folder / "conservative.rmeta", conservative],
              1, (contract["conservative_rustc"],), timeout=30, environment={"RUSTC_BOOTSTRAP": "1"})
    if args.app is not None:
        candidate(check, (source, conservative, client), observer, contract, args.baseline)


if __name__ == "__main__":
    main()
