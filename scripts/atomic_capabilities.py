#!/usr/bin/env python3
"""Independent PA28 backend/Rust observations; never invokes the RustGM frontend."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

import backend

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = Path("pa28/tests/backend/atomics/contract.json")


def checked_input(root, item, key, hash_key):
    path = Path(item[key])
    if path.is_absolute() or ".." in path.parts or path.as_posix() != item[key]:
        raise ValueError("noncanonical fixture path")
    if not path.is_relative_to("pa28/tests/backend"):
        raise ValueError("fixture outside owning backend lane")
    if backend.digest(root / path) != item[hash_key]:
        raise ValueError(f"fixture bytes changed: {path}")


def load_contract(root):
    contract = json.loads((root / MANIFEST).read_text())
    if contract["schema"] != 1 or contract["kind"] != "external-backend-capabilities":
        raise ValueError("unsupported capability contract")
    lock = json.loads((root / "backend.lock.json").read_text())
    if contract["backend_lock"] != lock:
        raise ValueError("capability contract differs from pinned backend")
    names = set()
    for case in contract["cases"]:
        name = case["name"]
        if not re.fullmatch(r"[a-z][a-z0-9-]*", name) or name in names:
            raise ValueError("invalid or duplicate case name")
        names.add(name)
        checked_input(root, case, "lowir", "lowir_sha256")
        if case["validator_expected"] not in (0, 1) or type(case["native_valid"]) is not bool:
            raise ValueError("invalid backend outcome")
        if case["native_valid"]:
            if case["validator_expected"] != 0 or case["native_expected"] != 42:
                raise ValueError("native execution requires a valid declared contract")
            machine = case["machine"]
            if machine["bits"] not in (0, 8, 16, 32, 64):
                raise ValueError("unsupported machine operand width")
            if any(type(value) is not int or value < 0 for value in machine.values()):
                raise ValueError("invalid machine observation")
            if machine["bits"] == 0 and any(machine[key] for key in
                                             ("lock_xadd", "lock_cmpxchg", "memory_xchg")):
                raise ValueError("RMW observation requires an operand width")
        elif case["native_expected"] is not None or "machine" in case:
            raise ValueError("validator-only case must never execute")
        if case["category"] in ("Rust-invalid-order", "backend-invalid-range"):
            if case["native_valid"]:
                raise ValueError("invalid-order/range cases must never execute")
        if "rust" in case:
            checked_rust(root, case)
            if case["category"] == "Rust-invalid-order" and case["rustc_expected"] != 1:
                raise ValueError("Rust-invalid-order case must reject independently")
    for case in contract["additional_independent"]:
        checked_rust(root, case)
        if case["name"] in names or case["equivalent_backend_case"] not in names:
            raise ValueError("invalid additional independent case")
        names.add(case["name"])
    checked_input(root, contract["abi"], "source", "sha256")
    return contract


def checked_rust(root, case):
    checked_input(root, case, "rust", "rust_sha256")
    status = case["rustc_expected"]
    if status not in (0, 1) or case["rustc_native_expected"] != (42 if status == 0 else None):
        raise ValueError("invalid independent Rust outcome")
    if status and case.get("rustc_diagnostic") != "invalid_atomic_ordering":
        raise ValueError("Rust rejection needs its declared ordering diagnostic")


def invoke(command, output, label, records, expected=0, stdout=b"", diagnostic=None):
    result = subprocess.run([str(x) for x in command], capture_output=True, timeout=60)
    for suffix, data in (("stdout", result.stdout), ("stderr", result.stderr)):
        (output / f"{label}.{suffix}").write_bytes(data)
    records.append({"command": [str(x) for x in command], "status": result.returncode,
                    "stdout_sha256": hashlib.sha256(result.stdout).hexdigest(),
                    "stderr_sha256": hashlib.sha256(result.stderr).hexdigest()})
    if result.returncode != expected or result.stdout != stdout:
        raise ValueError(f"{label}: unexpected status/stdout; inspect retained logs")
    if diagnostic is not None:
        if diagnostic.encode() not in result.stderr:
            raise ValueError(f"{label}: missing declared diagnostic")
    elif result.stderr:
        raise ValueError(f"{label}: unexpected stderr; inspect retained logs")
    return result


def backend_cases(contract, root, output, tools, records):
    artifacts = []
    for case in contract["cases"]:
        name = case["name"]
        validated = output / f"{name}.validated.lowir"
        invoke([tools["lowir"], "-o", validated, root / case["lowir"]], output,
               f"{name}.validate", records, case["validator_expected"],
               diagnostic=case.get("validator_diagnostic"))
        if not case["native_valid"]:
            continue
        text = validated.read_text()
        for operation, count in case.get("expected_operations", {}).items():
            if text.count(f"{operation} ") != count:
                raise ValueError(f"{name}: validated operation count changed: {operation}")
        executable = output / name
        invoke([tools["lowir2native"], "-O0", "-o", executable, validated], output,
               f"{name}.build", records)
        invoke([executable], output, f"{name}.native", records, case["native_expected"])
        machine = case["machine"]
        columns = [name, *[str(machine[key]) for key in
                          ("bits", "lock_xadd", "lock_cmpxchg", "memory_xchg", "mfence")]]
        artifacts.append("\t".join(columns))
    (output / "executables.tsv").write_text("\n".join(artifacts) + "\n")


def rust_cases(contract, root, output, records):
    version = subprocess.check_output(["rustc", "-Vv"], text=True)
    (output / "rustc-version.txt").write_text(version)
    if f"commit-hash: {contract['rustc_commit']}\n" not in version:
        raise ValueError("independent rustc differs from declared pin")
    target = contract["abi"]["target"]
    flags = ["--edition=2024", "--target", target, "-Cpanic=abort", "-Ccodegen-units=1"]
    cfg = subprocess.check_output(["rustc", "--print", "cfg", *flags], text=True)
    (output / "rustc-cfg.txt").write_text(cfg)
    atomics = {line for line in cfg.splitlines() if line.startswith("target_has_atomic=")}
    if atomics != {f'target_has_atomic="{x}"' for x in (8, 16, 32, 64, "ptr")}:
        raise ValueError("independent target atomic cfg changed")
    cases = [case for case in contract["cases"] if "rust" in case]
    for case in [*cases, *contract["additional_independent"]]:
        name = case["name"]
        executable = output / name
        command = ["rustc", *flags, "--crate-name", name.replace("-", "_"), root / case["rust"]]
        if case["rustc_expected"]:
            command.append("--emit=obj")
        invoke([*command, "-o", executable], output, f"{name}.rustc", records,
               case["rustc_expected"], diagnostic=case.get("rustc_diagnostic"))
        if case["rustc_native_expected"] is not None:
            invoke([executable], output, f"{name}.native", records, case["rustc_native_expected"])
    abi = contract["abi"]
    executable = output / "abi-observations"
    invoke(["rustc", *flags, root / abi["source"], "-o", executable], output,
           "abi.rustc", records, abi["rustc_expected"])
    invoke([executable], output, "abi.native", records, abi["native_expected"],
           stdout=abi["stdout_expected"].encode())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=("backend", "rust"))
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    contract = load_contract(ROOT)  # Validate every input/outcome before invoking any tool.
    output = args.output.resolve()
    if not output.is_relative_to(ROOT / "obj/parallel/pa28") or output.exists():
        raise ValueError("requires a fresh owning PA28 evidence directory")
    output.mkdir(parents=True)
    receipt = {"manifest_sha256": backend.digest(ROOT / MANIFEST), "phase": args.phase,
               "backend_lock": contract["backend_lock"], "commands": [], "complete": False}
    try:
        if args.phase == "backend":
            backend_cases(contract, ROOT, output, backend.load_tools(), receipt["commands"])
        else:
            rust_cases(contract, ROOT, output, receipt["commands"])
        receipt["complete"] = True
    finally:
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
