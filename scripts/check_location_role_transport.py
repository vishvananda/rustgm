#!/usr/bin/env python3
"""Check an imported guard's actual caller-location layout and hidden pointer."""
import argparse
import json
from pathlib import Path
import re

from check_source_constants import COMMON, provider_source
from crate_qualification import Check

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "pa31/tests/spec/source-location-transport"
IDENTITY = "base@caller_location"
OPTIONS = [*COMMON, "--emit-lowir", "--cap-lints=allow", "--crate-disambiguator",
           "caller_location"]
GLOBALS = re.compile(r"^global @(\S+) : obj<(\d+)x(\d+)> \[storage=readonly\] = \{\n(.*?)^\}",
                     re.MULTILINE | re.DOTALL)


def initialized_bytes(rows):
    result = bytearray()
    for row in rows:
        field, value = row.split()
        number = int(value)
        if field == "zero":
            result.extend(bytes(number))
        elif field in ("u8", "i16", "i32", "i64"):
            result.extend(number.to_bytes(int(field[1:]) // 8, "little"))
        else:
            raise ValueError(f"unexpected caller filename storage field {field!r}")
    return bytes(result)


def location_site(program, source, contract, site):
    callee = site["callee"].encode()
    symbol = re.escape(f"$f${len(callee)}_{callee.hex()}$")
    _, marker, function = program.partition(f"function @{site['caller']}(")
    body, _, _ = function.partition("\n}\n")
    calls = re.findall(rf"^    (?:%\w+ = )?call {site['result']} @(\S*{symbol}\S*)\(([^\n]*)\)$",
                       body, re.MULTILINE)
    if not marker or len(calls) != 1:
        raise ValueError(f"{site['caller']} did not call its actual tracked target")
    target, arguments = calls[0]
    inputs = arguments.split(", ")
    parameters = ", ".join(f"%p{index} : {ty}" for index, ty in enumerate(site["parameters"]))
    suffix = " [return=noreturn]" if site.get("noreturn") else ""
    if len(inputs) != len(site["parameters"]) or (
            f"function @{target}({parameters}) -> {site['result']}{suffix} {{" not in program):
        raise ValueError(f"{site['callee']} lost its hidden pointer ABI")
    pointer = inputs[-1]
    address = re.search(rf"^    {re.escape(pointer)} = addr @(\S+)$", body, re.MULTILINE)
    if address is None:
        raise ValueError("guard did not receive the address of the caller's immutable record")
    globals_ = {name: (int(size), int(align), fields.splitlines())
                for name, size, align, fields in GLOBALS.findall(program)}
    size, align, fields = globals_[address[1]]
    if (size, align) != (contract["size"], contract["alignment"]):
        raise ValueError("caller record differs from the pinned source shape")
    filename = source.as_posix().encode()
    if len(fields) != 4 or not fields[0].startswith("  ptr addr @") or fields[1:] != [
            f"  i64 {len(filename)}", f"  u32 {site['line']}", f"  u32 {site['column']}"]:
        raise ValueError("caller file length, line or column differs from the consumer source")
    file_symbol = fields[0].removeprefix("  ptr addr @")
    size, _, pieces = globals_[file_symbol]
    if size != len(filename) + 1 or initialized_bytes(pieces) != filename + b"\0":
        raise ValueError("caller filename lacks its exact bytes and one extra NUL")


def role_address(packet, tag, spelling):
    word = spelling.encode()
    marker = bytes((tag,)) + len(word).to_bytes(4, "little") + word + bytes(4)
    offset = packet.find(marker)
    if offset < 0 or packet.find(marker, offset + 1) != -1:
        raise ValueError(f"expected one actual provider binding for {spelling}")
    return offset + len(marker)


def opponents(check, importer, artifact, client, contract, baseline):
    packet = artifact.read_bytes()
    location = role_address(packet, 255, "panic_location")
    filename = role_address(packet, 254, "NonNull")
    foreign = (1).to_bytes(4, "little")
    patches = {"foreign-location": (location - 4, foreign),
               "foreign-nonnull": (filename - 4, foreign),
               "wrong-nonnull": (filename, packet[location:location + 4])}
    for name, diagnostic in contract["forgeries"].items():
        offset, replacement = patches[name]
        altered = check.folder / f"{name}.rgcrate"
        altered.write_bytes(packet[:offset] + replacement + packet[offset + 4:])
        check.run(name, [*importer, "--extern", f"base={altered}", client],
                  1, [diagnostic], timeout=30)
    if baseline is not None:
        check.run("old-reader", [baseline.resolve(strict=True), *importer[1:],
                  "--extern", f"base={artifact}", client], 1, [contract["old_reader_diagnostic"]],
                  timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    check = Check(args.app, args.out)
    source = provider_source(check.folder, FIXTURE, contract)
    source.write_bytes(b"#![feature(pattern_types)]\n" + source.read_bytes())
    client = FIXTURE / contract["client"]
    metadata = check.folder / "libbase.rmeta"
    rustc = [args.rustc.resolve(strict=True), *COMMON, "--emit=metadata", "--cfg", "rustc_oracle",
             "-C", "overflow-checks=yes", "-C", "panic=abort"]
    bootstrap = {"RUSTC_BOOTSTRAP": "1"}
    check.run("rustc-provider", [*rustc, "--crate-name", "base", "-o", metadata, source],
              timeout=30, environment=bootstrap)
    check.run("rustc-client", [*rustc, "--crate-name", "client", "--extern", f"base={metadata}",
              "-o", check.folder / "client.rmeta", client], timeout=30, environment=bootstrap)

    artifact = check.folder / "base.rgcrate"
    exporter = [check.app, *OPTIONS, "--crate-name", "base", "--source-core", IDENTITY]
    check.run("rustgm-provider", [*exporter, "--source-profile", "module-body-operations-v14",
              "--metadata-output", artifact, "-o", check.folder / "provider.lowir", source], timeout=30)
    importer = [check.app, *OPTIONS, "--crate-name", "client", "--source-client", IDENTITY]
    lowered = check.folder / "client.lowir"
    check.run("rustgm-client", [*importer, "--extern", f"base={artifact}", "-o", lowered,
              client], timeout=30)
    program = lowered.read_text()
    for site in contract["sites"]:
        location_site(program, client, contract, site)
    old = check.folder / "old.rgcrate"
    check.run("old-profile", [*exporter, "--source-profile", "module-bodies-v13",
              "--metadata-output", old, source], 1, [contract["old_profile_diagnostic"]], timeout=30)
    if old.exists():
        raise ValueError("old profile published an unsupported caller record")
    opponents(check, importer, artifact, client, contract, args.baseline)
    print("Fresh overflow and direct calls pass their 24-byte caller sites with exact source coordinates")


if __name__ == "__main__":
    main()
