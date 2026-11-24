#!/usr/bin/env python3
"""Check associated type families across independently compiled crates."""
import argparse
import json
from pathlib import Path
from struct import pack, unpack_from

import backend
from check_caller_location_forwarding import pinned
from check_source_constants import COMMON, ROOT, provider_source
from crate_qualification import Check


FIXTURES = ROOT / "pa31/tests/spec"
CASES = (
    "source-associated-family-transport",
    "source-impl-associated-family-transport",
    "source-impl-associated-where-transport",
)
IDENTITY = "base@associated_family"


def reject_empty_family_range(check, importer, artifact, client, diagnostic):
    packet = artifact.read_bytes()
    upper = len(packet) // 8
    marked = []
    for offset in range(len(packet) - 15):
        owner, value, start, end = unpack_from("<IIII", packet, offset)
        if 0 < owner < upper and (1 << 31) <= value < (1 << 31) + upper and 0 < start < end < upper:
            marked.append((offset, start, end))
    if len(marked) != 1 or marked[0][2] != marked[0][1] + 1:
        raise ValueError("expected one associated definition with one owned input")
    offset, start, _ = marked[0]
    altered = check.folder / "empty-range.rgcrate"
    altered.write_bytes(packet[:offset + 12] + pack("<I", start) + packet[offset + 16:])
    check.run("empty-range", [*importer, "--extern", f"base={altered}", client], 1,
              [diagnostic], timeout=15)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--case", choices=CASES, default=CASES[0])
    args = parser.parse_args()
    check = Check(args.app, args.out)
    fixture = FIXTURES / args.case
    contract = json.loads((fixture / "outcomes.json").read_text())
    source = provider_source(check.folder, fixture, contract)
    client = fixture / contract["client"]
    opposite = contract.get("rejected_cfg", "missing_family_bound")
    observer = check.folder / "observer.o"
    check.run("cc-observer", ["cc", "-c", fixture / contract["observer"], "-o", observer], timeout=15)
    expected = contract["stdout"].encode()
    pinned(check, args.rustc, source, client, observer, expected)
    check.run("rustc-rejected", [args.rustc.resolve(strict=True), *COMMON, "--emit=metadata",
                                 "--cfg", opposite, "--extern",
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
    check.run("rustgm-rejected", [*importer, "--cfg", opposite, "-o",
                                  check.folder / "rejected.lowir", client], 1,
              [contract["rustgm_rejected"]], timeout=15)
    check.run("older-profile", [*exporter, "--source-profile", "module-bodies-v13",
                                "--metadata-output", check.folder / "v13.rgcrate", source], 1,
              [contract["older_profile"]], timeout=15)
    if diagnostic := contract.get("empty_range"):
        reject_empty_family_range(check, importer[:-2], artifact, client, diagnostic)
    native = check.folder / "rustgm-client"
    backend.build_many([provider_lowir, client_lowir], native, native_objects=[observer])
    if check.run("rustgm-observe", [native], timeout=15).stdout != expected:
        raise ValueError("the associated type family returned a different reference")
    print(expected.decode(), end="")


if __name__ == "__main__":
    main()
