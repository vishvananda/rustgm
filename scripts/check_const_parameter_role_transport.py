#!/usr/bin/env python3
"""Check the structural const-parameter marker roles in fresh source clients."""
import argparse
import json
from pathlib import Path

import backend
from check_source_constants import COMMON, independent, provider_source
from crate_qualification import Check

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "pa31/tests/spec/source-const-parameter-role"
IDENTITY = "base@const_parameter_role"
OPTIONS = [*COMMON, "--emit-lowir", "--cap-lints=allow", "--crate-disambiguator",
           "const_parameter_role"]


def role_address(packet, tag, spelling):
    """Find the declaration coordinate after an artifact-zero spelled role."""
    name = spelling.encode()
    marker = bytes((tag,)) + len(name).to_bytes(4, "little") + name + bytes(4)
    at = packet.find(marker)
    if at < 0 or packet.find(marker, at + 1) != -1:
        raise ValueError(f"expected one provider binding for {spelling}")
    return at + len(marker)


def opponents(check, command, artifact, contract, baseline):
    packet = artifact.read_bytes()
    eq = role_address(packet, 254, "Eq")
    structural = role_address(packet, 255, "structural_peq")
    unrelated = role_address(packet, 254, "Send")
    corruptions = {"foreign-eq": (eq - 4, (1).to_bytes(4, "little")),
                   "same-role": (eq, packet[structural:structural + 4]),
                   "unrelated-eq": (eq, packet[unrelated:unrelated + 4])}
    source = FIXTURE / "client.rs"
    for name, diagnostic in contract["forgeries"].items():
        offset, replacement = corruptions[name]
        forged = check.folder / f"{name}.rgcrate"
        forged.write_bytes(packet[:offset] + replacement + packet[offset + len(replacement):])
        check.run(name, [*command, "--extern", f"base={forged}", source],
                  1, [diagnostic], timeout=40)
    if baseline is not None:
        check.run("old-reader", [baseline.resolve(strict=True), *command[1:],
                  "--extern", f"base={artifact}", source], 1, [contract["old_reader_diagnostic"]],
                  timeout=40)


def candidate(check, source, contract, baseline):
    artifact = check.folder / "base.rgcrate"
    provider = check.folder / "provider.lowir"
    exporter = [check.app, *OPTIONS, "--crate-name", "base", "--source-core", IDENTITY]
    check.run("rustgm-provider", [*exporter, "--source-profile", "module-body-operations-v14",
              "--metadata-output", artifact, "-o", provider, source], timeout=40)
    old = check.folder / "old.rgcrate"
    check.run("old-profile", [*exporter, "--source-profile", "module-bodies-v13",
              "--metadata-output", old, source], 1, [contract["old_profile_diagnostic"]], timeout=40)
    if old.exists():
        raise ValueError("old profile published a marker contract it does not validate")

    importer = [check.app, *OPTIONS, "--crate-name", "client", "--source-client", IDENTITY]
    for case in contract["cases"]:
        test = FIXTURE / case["source"]
        check.run(f"rustgm-{test.stem}", [*importer, "--extern", f"base={artifact}",
                  "-o", check.folder / f"{test.stem}.lowir", test], case["status"],
                  [case["diagnostic"]] if "diagnostic" in case else (), timeout=40)
    opponents(check, importer, artifact, contract, baseline)

    driver = check.folder / "observer.o"
    check.run("cc-driver", ["cc", "-c", FIXTURE / contract["observer"], "-o", driver], timeout=40)
    executable = check.folder / "client"
    backend.build_many([provider, check.folder / "client.lowir"], executable, native_objects=[driver])
    result = check.run("native-client", [executable], timeout=40)
    if result.stdout != contract["stdout"].encode():
        raise ValueError("fresh client changed its aggregate const arguments")
    print(result.stdout.decode(), end="")


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
    source.write_bytes(b"#![feature(min_adt_const_params)]\n" + source.read_bytes())
    independent(check, args.rustc, source, FIXTURE, contract)
    candidate(check, source, contract, args.baseline)


if __name__ == "__main__":
    main()
