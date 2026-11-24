#!/usr/bin/env python3
"""Check exact runtime imports across the source artifact boundary."""
import argparse
import json
from pathlib import Path

import backend
from check_source_constants import COMMON, independent, provider_source
from crate_qualification import Check

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "pa31/tests/spec/source-runtime-imports"
IDENTITY = "base@runtime_imports"
OPTIONS = [*COMMON, "--emit-lowir", "--cap-lints=allow", "--crate-disambiguator",
           "runtime_imports"]


def native_rows(packet, symbols):
    rows = {}
    for name in symbols:
        # The native import marker replaces the otherwise required body owner.
        text = name.encode()
        marker = b"\xff" * 4 + len(text).to_bytes(4, "little") + text
        at = packet.find(marker)
        if at < 4 or packet.find(marker, at + 1) != -1:
            raise ValueError(f"{name}: expected one actual runtime import")
        rows[name] = at
    return rows


def opponents(check, options, artifact, contract):
    packet = artifact.read_bytes()
    rows = native_rows(packet, contract["symbols"])
    at = rows[contract["symbols"][0]]
    owner = packet[at - 4:at]
    spelling = (contract["symbols"][0] + "_fn").encode()
    role = bytes((255,)) + len(spelling).to_bytes(4, "little") + spelling + bytes(4) + owner
    binding = packet.find(role)
    if binding < 0 or packet.find(role, binding + 1) != -1:
        raise ValueError("expected one spelled runtime role and its actual declaration")
    corruptions = {"wrong-owner": (at - 4, bytes(4)),
                   "invented-body": (at, bytes(4)),
                   "invalid-symbol": (at + 8, bytes(1)),
                   "wrong-role-owner": (binding + len(role) - 4, bytes(4))}
    for name, diagnostic in contract["forgeries"].items():
        offset, replacement = corruptions[name]
        altered = check.folder / f"{name}.rgcrate"
        altered.write_bytes(packet[:offset] + replacement + packet[offset + len(replacement):])
        check.run(name, [*options, "--extern", f"base={altered}", FIXTURE / "client.rs"],
                  1, [diagnostic], timeout=40)


def candidate(check, source, contract):
    artifact = check.folder / "base.rgcrate"
    provider = check.folder / "provider.lowir"
    check.run("rustgm-provider", [check.app, *OPTIONS, "--crate-name", "base",
              "--source-core", IDENTITY, "--source-profile", "module-body-operations-v14",
              "--metadata-output", artifact, "-o", provider, source], timeout=40)
    options = [check.app, *OPTIONS, "--crate-name", "client", "--source-client", IDENTITY]
    for case in contract["cases"]:
        test = FIXTURE / case["source"]
        check.run(f"rustgm-{test.stem}", [*options, "--extern", f"base={artifact}",
                  "-o", check.folder / f"{test.stem}.lowir", test], case["status"],
                  [case["diagnostic"]] if "diagnostic" in case else (), timeout=40)
    consumer = check.folder / "client.lowir"
    program = consumer.read_text()
    for name in contract["symbols"]:
        if f"@{name}(" not in program:
            raise ValueError(f"{name}: fresh client did not use its actual external symbol")
    opponents(check, options, artifact, contract)
    driver = check.folder / "observer.o"
    check.run("cc-driver", ["cc", "-c", ROOT / contract["observer"], "-o", driver], timeout=40)
    executable = check.folder / "client"
    backend.build_many([provider, consumer], executable, native_objects=[driver])
    result = check.run("native-client", [executable], timeout=40)
    if result.stdout != contract["stdout"].encode():
        raise ValueError("native observer changed its libc results")
    print(result.stdout.decode(), end="")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rustc", required=True, type=Path)
    parser.add_argument("--app", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    check = Check(args.app, args.out)
    source = provider_source(check.folder, FIXTURE, contract)
    independent(check, args.rustc, source, FIXTURE, contract)
    candidate(check, source, contract)


if __name__ == "__main__":
    main()
