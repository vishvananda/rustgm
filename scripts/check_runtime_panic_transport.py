#!/usr/bin/env python3
"""Check the separately published formatting panic roles and call ABI."""
import argparse
import json
from pathlib import Path
import re

from check_location_role_transport import GLOBALS, initialized_bytes, location_site, role_address
from check_source_nominal_transport import FIXTURES, RUSTGM, provider, source_prefix
from crate_qualification import Check


FIXTURE = FIXTURES / "source-runtime-panic-transport"
IDENTITY = "base@runtime_panic_roles"
PROFILE = "module-body-operations-v14"
NOUNWIND = b'#[lang = "panic_nounwind"]\n#[rustc_nounwind]\n#[rustc_const_stable_indirect]\n'


def untracked_call(program, site):
    callee = site["callee"].encode()
    symbol = re.escape(f"$f${len(callee)}_{callee.hex()}$")
    _, marker, function = program.partition(f"function @{site['caller']}()")
    body, _, _ = function.partition("\n}\n")
    calls = re.findall(rf"^    call void @(\S*{symbol}\S*)\((%\w+)\)$", body, re.MULTILINE)
    if not marker or len(calls) != 1:
        raise ValueError("the non-unwinding call lost its single slice argument")
    target, pointer = calls[0]
    if f"function @{target}(%p0 : ptr) -> void [return=noreturn] {{" not in program:
        raise ValueError("the non-unwinding target changed its ABI")
    slot = re.search(rf"^    {re.escape(pointer)} = addr (\$s\d+)$", body, re.MULTILINE)
    if slot is None or f"slot {slot[1]} : obj<16x8>" not in body:
        raise ValueError("the non-unwinding target requires the real two-word slice")
    storage = re.search(r"^    (%\w+) = addr @(\S+)$", body, re.MULTILINE)
    message = site["message"].encode()
    if (storage is None or f"store ptr {storage[1]}, {slot[1]}" not in body
            or f"store i64 {len(message)}," not in body):
        raise ValueError("the non-unwinding call lost its string address or length")
    globals_ = {name: rows for name, _, _, rows in GLOBALS.findall(program)}
    if initialized_bytes(globals_[storage[2]].splitlines()) != message:
        raise ValueError("the non-unwinding call changed the message bytes")


def opponents(check, exporter, importer, source, client, artifact, contract, nominal, baseline):
    original = source.read_bytes()
    without_nounwind = check.folder / "without-nounwind.rs"
    without_nounwind.write_bytes(original.replace(b"#[rustc_nounwind]\n", b"", 1))
    check.run("source-policy", [*exporter, "--source-profile", PROFILE, "--metadata-output",
              check.folder / "without-nounwind.rgcrate", without_nounwind], 1,
              [contract["source_policy"]], timeout=30)
    alternate_source, alternate = (check.folder / name for name in
                                   ("alternate.rs", "alternate.rgcrate"))
    alternate_source.write_bytes(original.replace(NOUNWIND, b"", 1).replace(
        b"pub const fn same_signature", NOUNWIND + b"pub const fn same_signature", 1))
    check.run("alternate-role", [*exporter, "--source-profile", PROFILE, "--metadata-output",
              alternate, "-o", check.folder / "alternate.lowir", alternate_source], timeout=30)
    packet, other = artifact.read_bytes(), alternate.read_bytes()
    role = role_address(packet, 255, "panic_nounwind")
    other_role = role_address(other, 255, "panic_nounwind")
    plain = other[other_role:other_role + 4]
    if plain == packet[role:role + 4]:
        raise ValueError("the same-signature declarations unexpectedly shared one owner")
    for name, offset, replacement in [("foreign_binding", role - 4, b"\x01\0\0\0"),
                                      ("wrong_binding", role, plain)]:
        forged = check.folder / f"{name}.rgcrate"
        forged.write_bytes(packet[:offset] + replacement + packet[offset + 4:])
        check.run(name, [*importer, "--extern", f"base={forged}", client], 1,
                  [contract[name]], timeout=30)
    older_source = check.folder / "older.rs"
    older_artifact = check.folder / "older.rgcrate"
    prefix = source_prefix(nominal["foundation"], nominal["prefix_marker"])
    _, marker, remainder = original.partition(NOUNWIND)
    older_source.write_bytes(prefix + marker + remainder)
    check.run("old-profile", [*exporter, "--source-profile", "module-bodies-v13", "--metadata-output",
              older_artifact, older_source], 1, [contract["old_profile"]], timeout=30)
    if older_artifact.exists():
        raise ValueError("an older profile published the unsupported panic role")
    if baseline is not None:
        check.run("old-reader", [baseline.resolve(strict=True), *importer[1:],
                  "--extern", f"base={artifact}", client], 1, [contract["old_reader"]], timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    check = Check(args.app, args.out)
    prefix = FIXTURES / contract["nominal_fixture"]
    nominal = json.loads((prefix / "outcomes.json").read_text())
    source = provider(check, prefix, nominal)
    source.write_bytes(source.read_bytes() + b"\n" + (FIXTURE / contract["provider"]).read_bytes())
    client = FIXTURE / contract["client"]
    metadata = check.folder / "libbase.rmeta"
    rustc = [args.rustc.resolve(strict=True), "--edition=2024", "--emit=metadata", "--crate-type=lib",
             "--cfg", "rustc_oracle", "-C", "panic=abort"]
    bootstrap = {"RUSTC_BOOTSTRAP": "1"}
    check.run("rustc-provider", [*rustc, "--crate-name", "base", "-o", metadata, source],
              timeout=30, environment=bootstrap)
    client_rustc = [*rustc, "--crate-name", "client", "--extern", f"base={metadata}",
                    "-o", check.folder / "libclient.rmeta"]
    check.run("rustc-client", [*client_rustc, client], timeout=30, environment=bootstrap)
    check.run("rustc-wrong-argument", [*client_rustc, "--cfg", "wrong_argument", client], 1,
              [contract["rustc_wrong_argument"]], timeout=30, environment=bootstrap)
    artifact = check.folder / "base.rgcrate"
    exporter = [check.app, *RUSTGM, "--crate-type=lib", "--crate-name", "base",
                "--crate-disambiguator", "runtime_panic_roles", "--source-core", IDENTITY]
    check.run("rustgm-provider", [*exporter, "--source-profile", PROFILE, "--metadata-output",
              artifact, "-o", check.folder / "provider.lowir", source], timeout=30)
    importer = [check.app, *RUSTGM, "--crate-type=bin", "--crate-name", "client",
                "--crate-disambiguator", "runtime_panic_client", "--source-client", IDENTITY]
    lowered = check.folder / "client.lowir"
    check.run("rustgm-client", [*importer, "--extern", f"base={artifact}", "-o", lowered, client],
              timeout=30)
    program = lowered.read_text()
    location_site(program, client, contract, contract["tracked"])
    untracked_call(program, contract["untracked"])
    check.run("rustgm-wrong-argument", [*importer, "--extern", f"base={artifact}",
              "--cfg", "wrong_argument", client], 1, [contract["rustgm_wrong_argument"]], timeout=30)
    opponents(check, exporter, importer, source, client, artifact, contract, nominal, args.baseline)
    print("Fresh runtime panic calls preserve their argument types and caller ABI")


if __name__ == "__main__":
    main()
