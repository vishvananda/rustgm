#!/usr/bin/env python3
"""Check that a fresh client routes overflow to its provider's real guard."""
import argparse
import json
from pathlib import Path
import re

from crate_qualification import Check


ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "pa31/tests/spec/source-guard-transport"
IDENTITY = "base@guard_role_fixture"
RUSTC_FLAGS = ["-C", "panic=abort", "-C", "overflow-checks=yes"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", required=True, type=Path)
    parser.add_argument("--rustc", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    check = Check(args.app, args.out)
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    prefix, marker, _ = (ROOT / contract["foundation"]).read_bytes().partition(
        contract["prefix_marker"].encode())
    if not marker:
        raise ValueError("provider foundation has no declared boundary")
    source = check.folder / "provider.rs"
    source.write_bytes(prefix + (FIXTURE / contract["provider"]).read_bytes())
    consumer = FIXTURE / contract["client"]
    rustc_provider = check.folder / "libbase.rmeta"
    rustc = [args.rustc, "--edition=2024", *RUSTC_FLAGS, "--emit=metadata", "--cfg", "rustc_oracle"]
    check.run("rustc-provider", [*rustc, "--crate-type=lib", "--crate-name", "base",
                                "-o", rustc_provider, source],
              environment={"RUSTC_BOOTSTRAP": "1"})
    check.run("rustc-consumer", [*rustc, "--crate-type=lib", "--crate-name", "consumer",
                                "--extern", f"base={rustc_provider}",
                                "-o", check.folder / "consumer.rmeta", consumer],
              contract["rustc"], environment={"RUSTC_BOOTSTRAP": "1"})
    artifact = check.folder / "base.rgcrate"
    rustgm = [check.app, "--edition=2024", "--emit-lowir", "--cap-lints=allow",
              "--crate-disambiguator", "guard_role_fixture"]
    check.run("rustgm-provider", [*rustgm, "--crate-type=lib", "--crate-name", "base",
                                 "--source-core", IDENTITY, "--source-profile", contract["profile"],
                                 "--metadata-output", artifact,
                                 "-o", check.folder / "provider.lowir", source])
    lowered = check.folder / "consumer.lowir"
    check.run("rustgm-consumer", [*rustgm, "--crate-type=bin", "--crate-name", "consumer",
                                 "--source-client", IDENTITY, "--extern", f"base={artifact}",
                                 "-o", lowered, consumer], contract["rustgm"])
    expected = contract["lowir"]
    item = expected["provider_symbol"].encode()
    symbol = f"$f${len(item)}_{item.hex()}$"
    calls = re.findall(r"^\s+call [^\n]*" + re.escape(symbol), lowered.read_text(), re.MULTILINE)
    if len(calls) != expected["linked_guard_role_calls"]:
        raise ValueError("fresh client did not call the checked provider guard")
    print("Fresh arithmetic guard calls the actual provider function")


if __name__ == "__main__":
    main()
