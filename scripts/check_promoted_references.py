#!/usr/bin/env python3
"""Qualify evaluated references with typed native relocations in fresh clients."""
import argparse
import json
from pathlib import Path

import backend
from check_source_constants import COMMON, independent, provider_source
from crate_qualification import Check

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "pa31/tests/spec/source-promoted-references"
IDENTITY = "base@promoted_functions"
OPTIONS = [*COMMON, "--emit-lowir", "--cap-lints=allow", "--crate-disambiguator",
           "promoted_functions"]


def opponents(check, command, source, artifact, contract, baseline):
    packet = artifact.read_bytes()
    # Promoted body tag, allocation owner, target count and two typed targets.
    candidates = []
    for offset, tag in enumerate(packet[:-25]):
        if tag != 3:
            continue
        owner, count, first, first_type, second, second_type = (
            int.from_bytes(packet[index:index + 4], "little")
            for index in range(offset + 1, offset + 25, 4)
        )
        if owner != 0 and count == 2 and first == 0 and second == 1 and first_type != second_type:
            candidates.append((offset, first_type))
    if len(candidates) != 1:
        raise ValueError("expected one promoted allocation with two distinct handlers")
    offset, first_type = candidates[0]
    corruptions = {"foreign-owner": (offset + 1, 0),
                   "wrong-signature": (offset + 21, first_type)}
    for name, diagnostic in contract["corruptions"].items():
        at, wrong = corruptions[name]
        corrupted = check.folder / (name + ".rgcrate")
        corrupted.write_bytes(packet[:at] + wrong.to_bytes(4, "little") + packet[at + 4:])
        check.run(name, [*command, "--extern", f"base={corrupted}", source],
                  1, [diagnostic], timeout=40)
    if slots := contract.get("binder_slots"):
        # No outer frame, CallableType, and the total bound region slot count.
        marker = bytes((0, 4)) + slots.to_bytes(4, "little")
        at = packet.find(marker)
        if at < 0 or packet.find(marker, at + 1) != -1:
            raise ValueError("expected one closed higher-ranked frame")
        count = at + 2
        wrong = check.folder / "missing-bound-slot.rgcrate"
        wrong.write_bytes(packet[:count] + (slots - 1).to_bytes(4, "little") + packet[count + 4:])
        check.run("missing-bound-slot", [*command, "--extern", f"base={wrong}", source],
                  1, [contract["binder_diagnostic"]], timeout=40)
    if diagnostic := contract.get("relocation_diagnostic"):
        owner = packet[offset + 1:offset + 5]
        marker = bytes((255,)) * 4 + owner
        at = packet.find(marker)
        if at < 0 or packet.find(marker, at + 1) != -1:
            raise ValueError("expected one static relocation to the actual allocation owner")
        owner_at = at + 4
        wrong = check.folder / "wrong-relocation.rgcrate"
        wrong.write_bytes(packet[:owner_at] + bytes(4) + packet[owner_at + 4:])
        check.run("wrong-relocation", [*command, "--extern", f"base={wrong}", source],
                  1, [diagnostic], timeout=40)
    if baseline is not None:
        check.run("legacy-reader", [baseline.resolve(strict=True), *command[1:],
                  "--extern", f"base={artifact}", source], 1, [contract["legacy_diagnostic"]],
                  timeout=40)


def candidate(check, source, contract, baseline):
    artifact = check.folder / "base.rgcrate"
    provider = check.folder / "provider.lowir"
    check.run("rustgm-provider", [check.app, *OPTIONS, "--crate-name", "base",
              "--source-core", IDENTITY, "--source-profile", "module-body-operations-v14",
              "--metadata-output", artifact, "-o", provider, source], timeout=40)
    client = [check.app, *OPTIONS, "--crate-name", "client", "--source-client", IDENTITY]
    for case in contract["cases"]:
        test = FIXTURE / case["source"]
        check.run(f"rustgm-{test.stem}", [*client, "--extern", f"base={artifact}", test],
                  case["status"], [case["diagnostic"]] if "diagnostic" in case else (), timeout=40)
    native = contract["native"]
    driver = check.folder / "driver.o"
    consumer = check.folder / f"rustgm-{Path(native['client']).stem}.stdout"
    provider_text = provider.read_text()
    count = contract["artifact"]["handler_count"]
    relocations = contract["artifact"].get("native_relocations", count)
    if provider_text.count("ptr addr @") != relocations or any(
        f"promotion_fn${handler}" not in provider_text for handler in range(count)
    ):
        raise ValueError("provider lost its distinct native handler relocations")
    if storage := native.get("storage"):
        lines = provider_text.splitlines()
        initializers = [line for index, line in enumerate(lines[:-1])
                        if "static$" in line and "$promotion$" in lines[index + 1]]
        if len(initializers) != 1 or (
            ("[storage=" not in initializers[0]) != (storage == "writable")
        ):
            raise ValueError("provider lost the actual Rust static storage policy")
    opponents(check, client, FIXTURE / native["client"], artifact, contract["artifact"], baseline)
    check.run("cc-driver", ["cc", "-c", FIXTURE / native["driver"], "-o", driver], timeout=40)
    executable = check.folder / "client"
    native_lowir = check.folder / "client.lowir"
    native_lowir.write_bytes(consumer.read_bytes())
    backend.build_many([provider, native_lowir], executable, native_objects=[driver])
    check.run("native-client", [executable], native["exit"], timeout=40)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    variant = parser.add_mutually_exclusive_group()
    variant.add_argument("--borrowed", action="store_true")
    variant.add_argument("--relocated", action="store_true")
    variant.add_argument("--interior", action="store_true")
    variant.add_argument("--projected", action="store_true")
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    manifest = "outcomes.json"
    if args.borrowed:
        manifest = "borrowed.json"
    elif args.relocated:
        manifest = "relocated.json"
    elif args.interior:
        manifest = "interior.json"
    elif args.projected:
        manifest = "projected.json"
    contract = json.loads((FIXTURE / manifest).read_text())
    check = Check(args.app, args.out)
    source = provider_source(check.folder, FIXTURE, contract)
    if features := contract.get("features"):
        prefix = "#![feature(" + ", ".join(features) + ")]\n"
        source.write_bytes(prefix.encode() + source.read_bytes())
    independent(check, args.rustc, source, FIXTURE, contract)
    candidate(check, source, contract, args.baseline)
    print("Fresh compiled client and typed promoted handler relocations returned 34")


if __name__ == "__main__":
    main()
