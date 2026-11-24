#!/usr/bin/env python3
"""Check staged alias predicates with a pinned Rust oracle and fresh clients."""
import argparse
import json
import os
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "pa31/tests/spec/source-bounded-alias-transport"
IDENTITY = "base@bounded_alias"


def declared_provider(contract, source):
    foundation = (ROOT / contract["foundation"]).read_bytes()
    prefix, marker, _ = foundation.partition(contract["prefix_marker"].encode())
    if not marker:
        raise ValueError("source provider foundation is missing its declared prefix marker")
    return prefix + (FIXTURE / source).read_bytes()


def check(command, stderr, status, diagnostic=None, *, env=None):
    result = subprocess.run(command, capture_output=True, text=True, check=False, timeout=60, env=env)
    stderr.write_text(result.stderr)
    if result.returncode != status or diagnostic is not None and diagnostic not in result.stderr:
        raise ValueError(f"expected exit {status} and {diagnostic!r} for {command[-1]}: {result.stderr}")


def independent(rustc, out, source, consumers):
    environment = {**os.environ, "RUSTC_BOOTSTRAP": "1"}
    options = [rustc, "--edition=2024", "--emit=metadata", "--crate-type=lib", "--cfg", "rustc_oracle"]
    artifact = out / "libbase.rmeta"
    check([*options, "--crate-name", "base", "-o", artifact, source],
          out / "rustc-provider.stderr", 0, env=environment)
    for case in consumers:
        name = Path(case["source"]).stem
        test = FIXTURE / case["source"]
        check([*options, "--extern", f"base={artifact}", "-o", out / f"{name}.rmeta", test],
              out / f"rustc-{name}.stderr", case["status"], case.get("rustc_diagnostic"), env=environment)


PROFILE = ["--emit-lowir", "--edition=2024", "--crate-type=lib", "--cap-lints=allow",
           "--crate-disambiguator", "bounded_alias"]


def publish(app, out, name, source):
    artifact = out / f"{name}.rgcrate"
    check([app, *PROFILE, "--crate-name", "base", "--source-core", IDENTITY,
           "--source-profile", "module-body-operations-v14", "--metadata-output",
           artifact, "-o", out / f"{name}.lowir", source], out / f"{name}.stderr", 0)
    if not artifact.is_file() or artifact.stat().st_size == 0:
        raise ValueError("provider emitted no crate metadata")
    return artifact


def candidate(app, out, source, consumers):
    artifact = publish(app, out, "base", source)
    for case in consumers:
        name = Path(case["source"]).stem
        lowir = out / f"{name}.lowir"
        test = FIXTURE / case["source"]
        check([app, *PROFILE, "--crate-name", name, "--source-client", IDENTITY,
               "--extern", f"base={artifact}", "-o", lowir, test],
              out / f"rustgm-{name}.stderr", case["status"], case.get("diagnostic"))
        if "lowir" in case and case["lowir"] not in lowir.read_text():
            raise ValueError(f"{name}: the imported alias did not produce the declared LowIR")
    return artifact


def compatibility(app, baseline, out, contract, artifact):
    source = out / "legacy.rs"
    source.write_bytes(declared_provider(contract, contract["legacy"]))
    before = publish(baseline, out, "legacy-before", source)
    after = publish(app, out, "legacy-after", source)
    if before.read_bytes() != after.read_bytes():
        raise ValueError("the v14 closed alias control changed metadata bytes")
    check([baseline, *PROFILE, "--crate-name", "consumer", "--source-client", IDENTITY,
           "--extern", f"base={artifact}", "-o", out / "legacy-reader.lowir", FIXTURE / "proved.rs"],
          out / "legacy-reader.stderr", 1, "invalid compiler-operation const-body trailer presence")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    options = parser.parse_args()
    contract = json.loads((FIXTURE / "outcomes.json").read_text())
    options.out.mkdir(parents=True, exist_ok=False)
    source = options.out / "provider.rs"
    source.write_bytes(declared_provider(contract["provider"], contract["provider"]["source"]))
    independent(options.rustc.resolve(strict=True), options.out, source, contract["consumers"])
    if options.app is not None:
        app = options.app.resolve(strict=True)
        artifact = candidate(app, options.out, source, contract["consumers"])
        if options.baseline is not None:
            compatibility(app, options.baseline.resolve(strict=True), options.out, contract["provider"], artifact)
    print(f"Checked {len(contract['consumers'])} independently compiled alias consumers")


if __name__ == "__main__":
    main()
