#!/usr/bin/env python3
"""Check native use and malformed bindings of the imported drop glue role."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_location_role_transport import role_address
from check_source_constants import COMMON, provider_source
from crate_qualification import Check

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "pa31/tests/spec/source-drop-glue-transport"
IDENTITY = "base@drop_glue"
OPTIONS = [*COMMON, "--emit-lowir", "--cap-lints=allow", "--crate-disambiguator", "drop_glue"]


def retargeted_role(check, exporter, source, packet, role):
    text = source.read_bytes()
    attribute = b'#[lang = "drop_glue"]\n'
    clause = b"where\n    T: [const] Destruct,\n"
    control = b"pub const unsafe fn ordinary<T: PointeeSized>(_: &mut T) {}"
    if any(text.count(marker) != 1 for marker in (attribute, clause, control)):
        raise ValueError("the glue fixture changed its written conditional premise")
    renamed = check.folder / "other-glue.rs"
    alternate = check.folder / "other-glue.rgcrate"
    renamed.write_bytes(text.replace(attribute, b"").replace(clause, b"").replace(
        control, attribute + control.removesuffix(b" {}") + b"\n" + clause + b"{}"))
    check.run("other-glue", [*exporter, "--source-profile", "module-body-operations-v14",
              "--metadata-output", alternate, "-o", check.folder / "other-glue.lowir", renamed],
              timeout=30)
    other = alternate.read_bytes()
    offset = role_address(other, 255, "drop_glue")
    replacement = other[offset:offset + 4]
    if packet[role:role + 4] == replacement:
        raise ValueError("the provider reused the two actual glue declaration identities")
    return replacement


def opponents(check, exporter, importer, source, client, explicit, artifact, contract, baseline):
    old = check.folder / "old.rgcrate"
    check.run("old-profile", [*exporter, "--source-profile", "module-bodies-v13",
              "--metadata-output", old, source], 1, [contract["old_profile"]], timeout=30)
    if old.exists():
        raise ValueError("old profile published drop glue")
    if baseline is not None:
        check.run("old-reader", [baseline.resolve(strict=True), *importer[1:],
                  "--extern", f"base={artifact}", client], 1, [contract["old_reader"]], timeout=30)

    check.run("rustgm-explicit", [*importer, "--extern", f"base={artifact}", explicit],
              1, [contract["rustgm_explicit"]], timeout=30)
    packet = artifact.read_bytes()
    role = role_address(packet, 255, "drop_glue")
    for name, offset, replacement in (
        ("foreign", role - 4, (1).to_bytes(4, "little")),
        ("unknown", role, bytes([255]) * 4),
        ("wrong-condition", role, retargeted_role(check, exporter, source, packet, role)),
    ):
        forged = check.folder / f"{name}.rgcrate"
        forged.write_bytes(packet[:offset] + replacement + packet[offset + 4:])
        check.run(name, [*importer, "--extern", f"base={forged}", client], 1,
                  [contract[name]], timeout=30)


def candidate(check, source, client, observer, expected):
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
    if observed != expected:
        raise ValueError("the fresh client changed the ordinary or selected destructor trace")
    print(observed.decode(), end="")
    return exporter, importer, artifact


def private_wrapper(check, source, client, observer, contract, rustc, run_candidate):
    private = Check(check.app, check.folder / "private-wrapper")
    provider = private.folder / "provider.rs"
    consumer = private.folder / "client.rs"
    declaration = b"pub const unsafe fn drop_glue"
    text = source.read_bytes()
    if text.count(declaration) != 1:
        raise ValueError("the private-wrapper control requires the exact direct declaration")
    wrapper = (FIXTURE / contract["private_wrapper"]).read_bytes()
    provider.write_bytes(text.replace(declaration, b"pub(crate) const unsafe fn drop_glue")
                         + b"\n" + wrapper)
    consumer.write_bytes(client.read_bytes().replace(b"base::drop_glue", b"base::cleanup"))
    pinned(private, rustc, provider, consumer, observer, contract["stdout"].encode())
    if run_candidate:
        candidate(private, provider, consumer, observer, contract["stdout"].encode())


def provider_owned(check, source, contract, rustc, run_candidate):
    owned = Check(check.app, check.folder / contract["name"])
    provider = owned.folder / "provider.rs"
    features = ", ".join(contract.get("provider_features", ()))
    prefix = f"#![feature({features})]\n".encode() if features else b""
    provider.write_bytes(prefix + source.read_bytes() + b"\n"
                         + (FIXTURE / contract["provider"]).read_bytes())
    consumer = FIXTURE / contract["client"]
    observer = owned.folder / "observer.o"
    owned.run("cc-observer", ["cc", "-c", FIXTURE / contract["observer"], "-o", observer],
              timeout=30)
    expected = contract["stdout"].encode()
    pinned(owned, rustc, provider, consumer, observer, expected)
    if run_candidate:
        candidate(owned, provider, consumer, observer, expected)


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
    client = FIXTURE / contract["client"]
    explicit = FIXTURE / contract["explicit"]
    observer = check.folder / "observer.o"
    check.run("cc-observer", ["cc", "-c", FIXTURE / contract["observer"], "-o", observer], timeout=30)
    pinned(check, args.rustc, source, client, observer, contract["stdout"].encode())
    check.run("rustc-explicit", [args.rustc.resolve(strict=True), *COMMON, "--emit=metadata",
              "--extern", f"base={check.folder / 'libbase.rlib'}", explicit,
              "-o", check.folder / "explicit.rmeta"], 1, [contract["rustc_explicit"]],
              timeout=30, environment={"RUSTC_BOOTSTRAP": "1"})
    if args.app is not None:
        exporter, importer, artifact = candidate(
            check, source, client, observer, contract["stdout"].encode())
        opponents(check, exporter, importer, source, client, explicit, artifact, contract,
                  args.baseline)
    private_wrapper(check, source, client, observer, contract, args.rustc, args.app is not None)
    for owned in contract["owned"]:
        provider_owned(check, source, owned, args.rustc, args.app is not None)


if __name__ == "__main__":
    main()
