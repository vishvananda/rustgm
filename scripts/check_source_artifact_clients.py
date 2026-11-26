#!/usr/bin/env python3
"""Run fresh native clients of separately published source artifacts."""
import argparse
import json
from hashlib import sha256
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_location_role_transport import role_address
from check_source_nominal_transport import FIXTURES, ROOT, RUSTGM, source_prefix
from crate_qualification import Check


FOUNDATION = "pa31/tests/spec/source-enum-role-transport/provider.rs"
ASSEMBLY_CASES = (
    "source-integer-assembly", "source-fixed-registers", "source-memory-assembly",
    "source-wide-registers", "source-byte-registers",
)


def assemble_provider(check, fixture, contract):
    features = ", ".join(contract.get("features", ()))
    attributes = "".join(f"#![{attribute}]\n" for attribute in contract.get("attributes", ()))
    source = check.folder / "provider.rs"
    boundary = contract.get("prefix_marker", '#[rustc_diagnostic_item = "Option"]')
    features = f"#![feature({features})]\n" if features else ""
    parts = [f"{features}{attributes}".encode(),
             source_prefix(contract.get("foundation", FOUNDATION), boundary)]
    tail = (fixture / "provider.rs").read_bytes()
    for fragment in contract.get("originals", ()):
        text = (ROOT / fragment["path"]).read_bytes()[fragment["start"]:fragment["end"]]
        if sha256(text).hexdigest() != fragment["sha256"]:
            raise ValueError(f"{fragment['path']}: checked original bytes changed")
        marker = fragment["marker"].encode()
        if tail.count(marker) != 1:
            raise ValueError(f"{fixture.name}: expected one original source marker")
        tail = tail.replace(marker, text)
    source.write_bytes(b"".join(parts) + tail)
    return source


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", required=True,
                        choices=("source-reference-cast-transport", "source-marker-trait-transport",
                                 "source-offset-intrinsic-transport", "source-slice-length-transport",
                                 "source-range-transport", "source-promotion-transport",
                                 "source-transmute-options-transport",
                                 "source-contract-helper-transport",
                                 "source-fn-pointer-transport", "source-receiver-transport",
                                 "source-poll-transport", "source-cstr-transport",
                                 "source-into-future-transport", "source-async-generator-helpers",
                                 "source-async-fn-transport", "source-trait-prefix-transport",
                                 "source-trait-lifetime-transport", "source-field-transport",
                                 "source-dynamic-transport",
                                 "source-enum-policy-and-dyn-inherent",
                                 "source-fn-lifetime-transport",
                                 "source-static-aggregate-transport",
                                 "source-early-lifetime-transport",
                                 "source-private-c-import-transport",
                                 "source-pointer-lint-transport",
                                 "source-const-trait-transport",
                                 "source-declared-intrinsic-transport",
                                 "source-async-drop-transport",
                                 "source-rpit-transport",
                                 "source-function-header-extensions",
                                 "source-inherent-const-and-tracked-traits",
                                 "source-free-and-inherent-const-originals",
                                 "source-nonnull-parameter-pattern",
                                 "source-generic-associated-definitions",
                                 "source-core-structural-inherents",
                                 "source-primitive-reexports",
                                 "source-internal-alias-reexports",
                                 "source-caller-site-transport",
                                 "source-extended-body-ops",
                                 "source-scoped-normalization-transport",
                                 "source-protocol-calls-transport",
                                 "source-closure-transport",
                                 "source-unsizing-and-literals-transport",
                                 "source-unchecked-const-body-transport",
                                 "source-inline-const-transport",
                                 "source-comptime-call-transport",
                                 "source-nested-item-arguments",
                                 "source-wide-scalar-transport",
                                 "source-initializer-numbering",
                                 "source-wide-cast-transport",
                                 "source-selected-projection-calls",
                                 "source-captured-callable-equalities",
                                 "source-override-late-binders",
                                 "source-shared-method-sites",
                                 "source-dispatched-call-signatures",
                                 "source-captured-protocol-requests", *ASSEMBLY_CASES))
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    check = Check(args.app, args.out)
    fixture_root = ROOT / "pa41/tests/spec" if args.case in ASSEMBLY_CASES else FIXTURES
    fixture = fixture_root / args.case
    contract = json.loads((fixture / "outcomes.json").read_text())
    disambiguator = args.case.replace("-", "_")
    identity = f"base@{disambiguator}"
    source = assemble_provider(check, fixture, contract)
    client = fixture / "client.rs"
    observer = check.folder / "observer.o"
    check.run("observer", ["cc", "-c", fixture / "observe.c", "-o", observer], timeout=15)
    expected = contract["stdout"].encode()
    pinned(check, args.rustc, source, client, observer, expected)
    common = [check.app, *RUSTGM, "--crate-type=lib"]
    exporter = [*common, "--crate-name", "base", "--crate-disambiguator", disambiguator,
                "--source-core", identity]
    artifact, provider, consumer = (check.folder / name for name in
                                    ("base.rgcrate", "provider.lowir", "client.lowir"))
    check.run("provider", [*exporter, "--source-profile", "module-body-operations-v14",
                           "--metadata-output", artifact, "-o", provider, source], timeout=30)
    importer = [*common, "--crate-name", "client", "--crate-disambiguator", f"{disambiguator}_client",
                "--source-client", identity]
    check.run("client", [*importer, "--extern", f"base={artifact}", "-o", consumer, client], timeout=30)
    if any(fragment not in consumer.read_text() for fragment in contract.get("required_lowir", ())):
        raise ValueError("required LowIR was not emitted")
    native = check.folder / "observe"
    backend.build_many([provider, consumer], native, native_objects=[observer])
    if check.run("execute", [native], timeout=15).stdout != expected:
        raise ValueError("the fresh native client differed from the declared stdout")
    for compiled in contract.get("compile_only_clients", ()):
        path = fixture / compiled["path"]
        label = path.stem.replace("-", "_")
        check.run(f"rustc-compile-only-{label}", [args.rustc.resolve(strict=True),
                  "--edition=2024", "-C", "panic=abort", "--crate-type=lib", "--crate-name", label,
                  "--extern", f"base={check.folder / 'libbase.rlib'}", "--emit=obj",
                  "-o", check.folder / f"pinned-{label}.o", path],
                  timeout=30, environment={"RUSTC_BOOTSTRAP": "1"})
        lowir = check.folder / f"{label}.lowir"
        check.run(f"compile-only-{label}", [*importer, "--extern", f"base={artifact}",
                  "-o", lowir, path], timeout=30)
        text = lowir.read_text()
        if any(fragment not in text for fragment in compiled["required_lowir"]):
            raise ValueError(f"{label}: required original/vector body was not emitted")
        output = check.folder / f"{label}.o"
        backend.emit_object(lowir, output, backend.load_tools(("cppgm",)))
        (check.folder / f"{label}.object.sha256").write_text(sha256(output.read_bytes()).hexdigest() + "\n")
    if refusal := contract.get("older_profile"):
        old = check.folder / "old.rgcrate"
        older_options = ["--cfg", contract["older_profile_cfg"]] if "older_profile_cfg" in contract else []
        check.run("older-profile", [*exporter, "--source-profile", "module-bodies-v13",
                                  *older_options, "--metadata-output", old, source],
                  1, [refusal], timeout=30)
        if old.exists():
            raise ValueError("older source profile published the new metadata")
    if fences := contract.get("profile_fences"):
        prefix, found, _ = source.read_bytes().partition(fences["prefix_marker"].encode())
        if not found:
            raise ValueError("profile fence has no declared source prefix")
        for name in fences["cases"]:
            stem = Path(name).stem
            fenced = check.folder / name
            fenced.write_bytes(prefix + (fixture / name).read_bytes())
            check.run(f"fence-rustc-{stem}", [args.rustc.resolve(strict=True), "--edition=2024",
                      "--crate-type=lib", "--crate-name=base", "--emit=metadata", "-o",
                      check.folder / f"{stem}.rmeta", fenced], fences["rustc_status"], fences.get("rustc", ()),
                      timeout=30, environment={"RUSTC_BOOTSTRAP": "1"})
            for profile in fences["profiles"]:
                rejected = check.folder / f"{stem}-{profile}.rgcrate"
                check.run(f"fence-{stem}-{profile}", [*exporter, "--source-profile", profile,
                          "--metadata-output", rejected, "-o", check.folder / f"{stem}.lowir",
                          fenced], fences["rustgm_status"], [fences["rustgm"]], timeout=30)
                if rejected.exists():
                    raise ValueError("rejected assembly control published metadata")
    if error := contract.get("unmarked_error"):
        marker = b"#[rustc_promotable]\n"
        text = source.read_bytes()
        if text.count(marker) != 1:
            raise ValueError("expected the source promotion declaration")
        unmarked = check.folder / "unmarked.rs"
        unmarked.write_bytes(text.replace(marker, b""))
        unmarked_artifact = check.folder / "unmarked.rgcrate"
        check.run("unmarked-provider", [*exporter, "--source-profile",
                  "module-body-operations-v14", "--metadata-output", unmarked_artifact,
                  "-o", check.folder / "unmarked.lowir", unmarked], timeout=30)
        check.run("unmarked-client", [*importer, "--extern", f"base={unmarked_artifact}",
                  client], 1, [error], timeout=30)
    if role := contract.get("foreign_role"):
        packet = bytearray(artifact.read_bytes())
        offset = role_address(packet, 255, role["spelling"]) - 4
        packet[offset:offset + 4] = (1).to_bytes(4, "little")
        forged = check.folder / "foreign-role.rgcrate"
        forged.write_bytes(packet)
        check.run("foreign-role", [*importer, "--extern", f"base={forged}", client], 1,
                  [role["diagnostic"]], timeout=30)
    if restricted := contract.get("restricted_client"):
        negative = fixture / restricted["path"]
        check.run("restricted-rustc", [args.rustc.resolve(strict=True), "--edition=2024",
                  "--crate-type=lib", "--crate-name=client", "--emit=metadata",
                  "--extern", f"base={check.folder / 'libbase.rlib'}",
                  "-o", check.folder / "restricted.rmeta", negative],
                  1, [restricted["rustc"]], timeout=30, environment={"RUSTC_BOOTSTRAP": "1"})
        check.run("restricted-client", [*importer, "--extern", f"base={artifact}", negative],
                  1, [restricted["rustgm"]], timeout=30)
    if args.baseline is not None and (refusal := contract.get("older_reader")):
        check.run("older-reader", [args.baseline.resolve(strict=True), *importer[1:],
                  "--extern", f"base={artifact}", client], 1, [refusal], timeout=30)
    print(expected.decode(), end="")


if __name__ == "__main__":
    main()
