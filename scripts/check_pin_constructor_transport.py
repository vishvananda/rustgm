#!/usr/bin/env python3
"""Run a fresh client through the imported generic const Pin constructor."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_location_role_transport import role_address
from check_source_constants import COMMON, ROOT
from crate_qualification import Check


FIXTURE = ROOT / "pa31/tests/spec/source-pin-constructor-transport"
IDENTITY = "base@pin_constructor"


def source_provider(check, contract):
    original = (ROOT / contract["foundation"]).read_bytes()
    prefix, marker, _ = original.partition(contract["prefix_marker"].encode())
    _, start, tail = original.partition(contract["cleanup_start"].encode())
    cleanup, end, _ = tail.partition(contract["cleanup_end"].encode())
    if not all((marker, start, end)):
        raise ValueError("the provider foundation changed its declared boundaries")
    path = check.folder / "provider.rs"
    path.write_bytes(prefix.replace(b",pin_ergonomics", b"") +
                     (FIXTURE / contract["provider"]).read_bytes() + b"\n" + start + cleanup)
    return path


def opponents(check, exporter, importer, source, artifact, client, contract):
    weaker = check.folder / "no-deref.rs"
    weaker.write_bytes(source.read_bytes().replace(b"impl<Ptr: Deref> Pin<Ptr>",
                                                  b"impl<Ptr> Pin<Ptr>", 1))
    check.run("source-without-deref", [*exporter, "--metadata-output",
              check.folder / "no-deref.rgcrate", weaker], 1, [contract["missing_deref"]], timeout=30)
    packet = artifact.read_bytes()
    coordinate = role_address(packet, 255, "new_unchecked")
    for name, offset, replacement in [("foreign_role", coordinate - 4, b"\x01\0\0\0"),
                                      ("unknown_role", coordinate, b"\xff" * 4)]:
        malformed = check.folder / f"{name}.rgcrate"
        malformed.write_bytes(packet[:offset] + replacement + packet[offset + 4:])
        check.run(name, [*importer, "--extern", f"base={malformed}", client], 1,
                  [contract[name]], timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    check = Check(args.app, args.out)
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    source = source_provider(check, contract)
    client = FIXTURE / contract["client"]
    observer = check.folder / "observer.o"
    check.run("cc-observer", ["cc", "-c", FIXTURE / contract["observer"], "-o", observer], timeout=15)
    expected = contract["stdout"].encode()
    pinned(check, args.rustc, source, client, observer, expected)
    check.run("rustc-wrong-pointer", [args.rustc.resolve(strict=True), *COMMON, "--emit=metadata",
              "--cfg", "wrong_pointer", "--extern", f"base={check.folder / 'libbase.rlib'}",
              "-o", check.folder / "wrong-pointer.rmeta", client], 1,
              [contract["rustc_wrong_pointer"]], timeout=15, environment={"RUSTC_BOOTSTRAP": "1"})
    options = [check.app, *COMMON, "--emit-lowir", "--cap-lints=allow",
               "--crate-disambiguator", "pin_constructor"]
    exporter = [*options, "--crate-name", "base", "--source-core", IDENTITY,
                "--source-profile", "module-body-operations-v14"]
    artifact = check.folder / "base.rgcrate"
    provider, consumer = (check.folder / name for name in ("provider.lowir", "client.lowir"))
    check.run("provider", [*exporter, "--metadata-output", artifact, "-o", provider, source], timeout=30)
    importer = [*options, "--crate-name", "client", "--source-client", IDENTITY]
    check.run("client", [*importer, "--extern", f"base={artifact}", "-o", consumer, client], timeout=30)
    check.run("rustgm-wrong-pointer", [*importer, "--extern", f"base={artifact}",
              "--cfg", "wrong_pointer", client], 1, [contract["rustgm_wrong_pointer"]], timeout=30)
    executable = check.folder / "client"
    backend.build_many([provider, consumer], executable, native_objects=[observer])
    if check.run("rustgm-observe", [executable], timeout=15).stdout != expected:
        raise ValueError("the transported constructor changed the pinned pointer")
    opponents(check, exporter, importer, source, artifact, client, contract)
    print(expected.decode(), end="")


if __name__ == "__main__":
    main()
