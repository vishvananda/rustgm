#!/usr/bin/env python3
"""Check fresh implementations of an imported associated type family."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_source_constants import COMMON, ROOT, provider_source
from crate_qualification import Check


FIXTURE = ROOT / "pa31/tests/spec/source-associated-family-transport"
IDENTITY = "base@associated_family"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    check = Check(args.app, args.out)
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    source = provider_source(check.folder, FIXTURE, contract)
    client = FIXTURE / contract["client"]
    observer = check.folder / "observer.o"
    check.run("cc-observer", ["cc", "-c", FIXTURE / contract["observer"], "-o", observer], timeout=15)
    expected = contract["stdout"].encode()
    pinned(check, args.rustc, source, client, observer, expected)
    check.run("rustc-rejected", [args.rustc.resolve(strict=True), *COMMON, "--emit=metadata",
                                 "--cfg", "missing_family_bound", "--extern",
                                 f"base={check.folder / 'libbase.rlib'}", "-o",
                                 check.folder / "rejected.rmeta", client], 1, [contract["rustc_rejected"]],
              timeout=15, environment={"RUSTC_BOOTSTRAP": "1"})
    common = [check.app, *COMMON, "--emit-lowir", "--cap-lints=allow", "--crate-disambiguator", "associated_family"]
    exporter = [*common, "--crate-name", "base", "--source-core", IDENTITY]
    artifact = check.folder / "base.rgcrate"
    provider_lowir = check.folder / "provider.lowir"
    check.run("provider", [*exporter, "--source-profile", "module-body-operations-v14",
                           "--metadata-output", artifact, "-o", provider_lowir, source], timeout=30)
    importer = [*common, "--crate-name", "client", "--source-client", IDENTITY, "--extern", f"base={artifact}"]
    client_lowir = check.folder / "client.lowir"
    check.run("client", [*importer, "-o", client_lowir, client], timeout=30)
    check.run("rustgm-rejected", [*importer, "--cfg", "missing_family_bound", "-o",
                                  check.folder / "rejected.lowir", client], 1,
              [contract["rustgm_rejected"]], timeout=15)
    check.run("older-profile", [*exporter, "--source-profile", "module-bodies-v13",
                                "--metadata-output", check.folder / "v13.rgcrate", source], 1,
              [contract["older_profile"]], timeout=15)
    native = check.folder / "rustgm-client"
    backend.build_many([provider_lowir, client_lowir], native, native_objects=[observer])
    if check.run("rustgm-observe", [native], timeout=15).stdout != expected:
        raise ValueError("the associated type family returned a different reference")
    print(expected.decode(), end="")


if __name__ == "__main__":
    main()
