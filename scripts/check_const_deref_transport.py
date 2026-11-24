#!/usr/bin/env python3
"""Check the separately imported const reference implementations used by Pin."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_location_role_transport import role_address
from check_source_constants import COMMON, ROOT
from crate_qualification import Check

FIXTURE = ROOT / "pa31/tests/spec/source-const-deref-transport"
IDENTITY = "base@const_deref"
OPTIONS = [*COMMON, "--emit-lowir", "--cap-lints=allow", "--crate-disambiguator", "const_deref"]


def provider(check, contract):
    foundation = (ROOT / contract["foundation"]).read_bytes()
    prefix, marker, _ = foundation.partition(contract["foundation_end"].encode())
    if not marker or b",pin_ergonomics" not in prefix:
        raise ValueError("const dereference foundation changed its declared boundary")
    source = check.folder / "provider.rs"
    source.write_bytes(prefix.replace(b",pin_ergonomics", b",const_convert") +
                       (FIXTURE / contract["provider"]).read_bytes() + b"\n" +
                       (ROOT / contract["pin"]).read_bytes())
    return source


def opponents(check, contract, paths, commands, rustc, baseline):
    source, client, artifact = paths
    exporter, importer = commands
    check.run("rustc-ordinary-impl", [rustc, *COMMON, "--emit=metadata", "--cfg", "ordinary_deref",
              "--extern", f"base={check.folder / 'libbase.rlib'}", "-o",
              check.folder / "ordinary.rmeta", client], 1, [contract["rustc_ordinary_impl"]],
              timeout=30, environment={"RUSTC_BOOTSTRAP": "1"})
    check.run("rustgm-ordinary-impl", [*importer, "--extern", f"base={artifact}",
              "--cfg", "ordinary_deref", client], 1, [contract["rustgm_ordinary_impl"]], timeout=30)
    old = check.folder / "old.rgcrate"
    check.run("old-profile", [*exporter, "--source-profile", "module-bodies-v13",
              "--metadata-output", old, source], 1, [contract["old_profile"]], timeout=30)
    if old.exists():
        raise ValueError("older profile published the unsupported const dereference roles")
    if baseline is not None:
        check.run("old-reader", [baseline.resolve(strict=True), *importer[1:], "--extern",
                  f"base={artifact}", client], 1, [contract["old_reader"]], timeout=30)
    packet = artifact.read_bytes()
    mutable = role_address(packet, 255, "deref_mut")
    shared = role_address(packet, 255, "deref")
    forged = check.folder / "wrong-role.rgcrate"
    forged.write_bytes(packet[:mutable] + packet[shared:shared + 4] + packet[mutable + 4:])
    check.run("wrong-role", [*importer, "--extern", f"base={forged}", client],
              1, [contract["wrong_role"]], timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    check = Check(args.app, args.out)
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    source, client = provider(check, contract), FIXTURE / contract["client"]
    observer = check.folder / "observer.o"
    check.run("cc-observer", ["cc", "-c", ROOT / contract["observer"], "-o", observer], timeout=30)
    expected = contract["stdout"].encode()
    rustc = args.rustc.resolve(strict=True)
    pinned(check, rustc, source, client, observer, expected)
    exporter = [check.app, *OPTIONS, "--crate-name", "base", "--source-core", IDENTITY]
    artifact = check.folder / "base.rgcrate"
    exported, consumer = (check.folder / path for path in ("provider.lowir", "client.lowir"))
    check.run("rustgm-provider", [*exporter, "--source-profile", "module-body-operations-v14",
              "--metadata-output", artifact, "-o", exported, source], timeout=30)
    importer = [check.app, *OPTIONS, "--crate-name", "client", "--source-client", IDENTITY]
    check.run("rustgm-client", [*importer, "--extern", f"base={artifact}", "-o", consumer, client],
              timeout=30)
    runner = check.folder / "observe"
    backend.build_many([exported, consumer], runner, native_objects=[observer])
    if check.run("rustgm-observe", [runner], timeout=30).stdout != expected:
        raise ValueError("transport changed const dereference or native pinned mutation")
    opponents(check, contract, (source, client, artifact), (exporter, importer), rustc, args.baseline)
    print(expected.decode(), end="")


if __name__ == "__main__":
    main()
