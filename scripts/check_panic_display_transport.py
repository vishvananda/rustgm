#!/usr/bin/env python3
"""Check the fresh runtime panic_display bound, body, and caller ABI."""
import argparse
import json
from pathlib import Path

from check_location_role_transport import location_site, role_address
from check_runtime_panic_transport import PROFILE
from check_source_nominal_transport import FIXTURES, RUSTGM, provider, source_prefix
from crate_qualification import Check


FIXTURE = FIXTURES / "source-panic-display-transport"
IDENTITY = "base@panic_display_roles"
DISPLAY = b'#[rustc_diagnostic_item = "Display"]\n'
POLICIES = (b'#[track_caller]\n#[lang = "panic_display"]\n'
            b'#[rustc_do_not_const_check]\n#[rustc_const_stable_indirect]\n')


def opponents(check, exporter, importer, source, client, artifact, contract, nominal, baseline):
    original = source.read_bytes()
    wrong_source = check.folder / "wrong-source.rs"
    wrong_source.write_bytes(original.replace(b"panic_display<T: WriteDisplay>",
                                              b"panic_display<T: OtherDisplay>"))
    check.run("source-bound", [*exporter, "--source-profile", PROFILE, "--metadata-output",
              check.folder / "wrong-source.rgcrate", wrong_source], 1,
              [contract["source_bound"]], timeout=30)
    alternate_bound = original.replace(DISPLAY + b"pub trait WriteDisplay", b"pub trait WriteDisplay")
    alternate_bound = alternate_bound.replace(b"pub trait OtherDisplay", DISPLAY + b"pub trait OtherDisplay")
    alternate_bound = alternate_bound.replace(b"panic_display<T: WriteDisplay>",
                                              b"panic_display<T: OtherDisplay>")
    alternate_callee = original.replace(POLICIES, b"", 1).replace(
        b"pub const fn ordinary_display", POLICIES + b"pub const fn ordinary_display", 1
    ).replace(b"    stop_display_at_runtime()", b"    loop {}", 1)
    roles = [
        ("wrong_bound", alternate_bound, 254, "Display"),
        ("wrong_binding", alternate_callee, 255, "panic_display"),
    ]
    packet = artifact.read_bytes()
    display = role_address(packet, 254, "Display")
    foreign = check.folder / "foreign-display.rgcrate"
    foreign.write_bytes(packet[:display - 4] + bytes([1, 0, 0, 0]) + packet[display:])
    check.run("foreign-display", [*importer, "--extern", f"base={foreign}", client], 1,
              [contract["foreign_display"]], timeout=30)
    for name, body, tag, spelling in roles:
        alternate_source = check.folder / f"{name}-alternate.rs"
        alternate = check.folder / f"{name}-alternate.rgcrate"
        alternate_source.write_bytes(body)
        check.run(f"{name}-alternate", [*exporter, "--source-profile", PROFILE, "--metadata-output",
                  alternate, "-o", check.folder / f"{name}.lowir", alternate_source], timeout=30)
        other = alternate.read_bytes()
        position = role_address(packet, tag, spelling)
        swapped = role_address(other, tag, spelling)
        different = other[swapped:swapped + 4]
        if different == packet[position:position + 4]:
            raise ValueError(f"{name}: alternate kept the same declaration")
        forged = check.folder / f"{name}.rgcrate"
        forged.write_bytes(packet[:position] + different + packet[position + 4:])
        check.run(name, [*importer, "--extern", f"base={forged}", client], 1,
                  [contract[name]], timeout=30)
    older = check.folder / "older.rs"
    prefix = source_prefix(nominal["foundation"], nominal["prefix_marker"])
    fragment, boundary, _ = (FIXTURE / contract["provider"]).read_bytes().partition(b'#[lang = "const_panic_fmt"]')
    if not boundary:
        raise ValueError("the runtime-only provider has no declared fixture boundary")
    older.write_bytes(prefix + fragment)
    older_artifact = check.folder / "older.rgcrate"
    check.run("old-profile", [*exporter, "--source-profile", "module-bodies-v13", "--metadata-output",
              older_artifact, older], 1, [contract["old_profile"]], timeout=30)
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
    check.run("rustc-missing-display", [*client_rustc, "--cfg", "missing_display", client], 1,
              [contract["rustc_missing_display"]], timeout=30, environment=bootstrap)
    artifact = check.folder / "base.rgcrate"
    exporter = [check.app, *RUSTGM, "--crate-type=lib", "--crate-name", "base",
                "--crate-disambiguator", "panic_display_roles", "--source-core", IDENTITY]
    check.run("rustgm-provider", [*exporter, "--source-profile", PROFILE, "--metadata-output",
              artifact, "-o", check.folder / "provider.lowir", source], timeout=30)
    importer = [check.app, *RUSTGM, "--crate-type=bin", "--crate-name", "client",
                "--crate-disambiguator", "display_client", "--source-client", IDENTITY]
    lowered = check.folder / "client.lowir"
    check.run("rustgm-client", [*importer, "--extern", f"base={artifact}", "-o", lowered, client],
              timeout=30)
    program = lowered.read_text()
    location_site(program, client, contract, contract["tracked"])
    if "stop_display_at_runtime".encode().hex() not in program:
        raise ValueError("the imported const panic hook lost its original runtime call")
    _, marker, helper_caller = program.partition("function @fresh_const_format(")
    helper_body, _, _ = helper_caller.partition("\n}\n")
    if not marker or "const_format_stop".encode().hex() not in helper_body:
        raise ValueError("the fresh const function lost its actual const formatting call")
    check.run("rustgm-missing-display", [*importer, "--extern", f"base={artifact}",
              "--cfg", "missing_display", client], 1, [contract["rustgm_missing_display"]], timeout=30)
    opponents(check, exporter, importer, source, client, artifact, contract, nominal, args.baseline)
    print("Fresh panic_display calls preserve the checked Display bound and caller ABI")


if __name__ == "__main__":
    main()
