#!/usr/bin/env python3
"""Pinned LowIR executable/object bridges; never a Rust frontend fallback."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def configure(checkout):
    checkout = checkout.resolve()
    commit = subprocess.check_output(["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True).strip()
    tools = {name: str(checkout / "dev" / name) for name in ("lowir", "lowir2native")}
    lock = json.loads((ROOT / "backend.lock.json").read_text())
    if commit != lock["commit"]:
        raise ValueError("checkout differs from the reviewed backend commit")
    for name, path in tools.items():
        if digest(Path(path)) != lock["sha256"][name]:
            raise ValueError(f"{name}: binary differs from reviewed backend; qualify a new lock explicitly")
    # Verify every live input before installing pins or replacing configuration.
    local = ROOT / "backend.local.json"
    configured = json.loads(local.read_text())["tools"] if local.is_file() else {}
    for name, path in tools.items():
        configured[name] = str(pin_tool(Path(path), name, lock["sha256"][name]))
    local.write_text(json.dumps({"tools": configured}, indent=2) + "\n")
    print("Configured immutable pinned CPPGM backend")


def pin_tool(source, name, expected):
    pin = ROOT / "obj" / "backend" / "pins" / expected / name
    pin.parent.mkdir(parents=True, exist_ok=True)
    if not pin.exists():
        shutil.copyfile(source, pin)
    if digest(pin) != expected:
        raise ValueError(f"{name}: immutable backend pin differs from reviewed bytes")
    pin.chmod(0o555)
    (pin.parent / ".keep-artifacts").write_text("Current reviewed LowIR backend pin\n")
    return pin


def object_profile(lock):
    profile = lock.get("optional", {}).get("object", {})
    if (profile.get("target") != "x86_64-linux" or
            profile.get("object_format") != "elf64-x86-64" or
            "lowir-to-object" not in profile.get("capabilities", [])):
        raise ValueError("reviewed x86-64 ELF object capability is unavailable")
    return profile


def load_tools(required=("lowir", "lowir2native")):
    local = ROOT / "backend.local.json"
    if not local.is_file():
        raise ValueError("configure backend first: python3 scripts/backend.py --configure CPPGM_CHECKOUT")
    tools = json.loads(local.read_text())["tools"]
    lock = json.loads((ROOT / "backend.lock.json").read_text())
    for name in required:
        hashes = object_profile(lock)["sha256"] if name == "cppgm++" else lock["sha256"]
        if name not in tools or name not in hashes:
            raise ValueError(f"{name}: configure the reviewed backend capability first")
        if digest(Path(tools[name])) != hashes[name]:
            raise ValueError(f"{name}: backend bytes changed")
    return tools


def configure_object(checkout):
    tools = load_tools()
    checkout = checkout.resolve()
    lock = json.loads((ROOT / "backend.lock.json").read_text())
    profile = object_profile(lock)
    commit = subprocess.check_output(["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True).strip()
    if commit != profile["commit"]:
        raise ValueError("checkout differs from the reviewed object backend commit")
    source = checkout / "dev" / "cppgm++"
    expected = profile["sha256"]["cppgm++"]
    if digest(source) != expected:
        raise ValueError("cppgm++: binary differs from reviewed object backend")
    tools["cppgm++"] = str(pin_tool(source, "cppgm++", expected))
    (ROOT / "backend.local.json").write_text(json.dumps({"tools": tools}, indent=2) + "\n")
    print("Configured immutable pinned CPPGM LowIR object backend")


def invoke(command):
    result = subprocess.run(command, capture_output=True, timeout=60)
    if result.returncode:
        raise ValueError(f"backend command failed ({result.returncode}): {result.stderr.decode(errors='replace')}")


def build(source, output):
    build_many([source], output)


def build_many(sources, output):
    if not sources:
        raise ValueError("executable bridge requires at least one LowIR input")
    if any(not source.is_file() for source in sources):
        raise ValueError("LowIR input does not exist")
    if len(sources) > 1025:
        raise ValueError("LowIR input count exceeds crate profile budget")
    if len(sources) > 1:
        build_objects(sources, output)
        return
    tools = load_tools()
    output.parent.mkdir(parents=True, exist_ok=True)
    units = []
    for index, source in enumerate(sources):
        suffix = ".validated.lowir" if len(sources) == 1 else f".unit-{index}.validated.lowir"
        validated = output.with_suffix(suffix)
        invoke([tools["lowir"], "-o", str(validated), str(source)])
        units.append(str(validated))
    invoke([tools["lowir2native"], "-O0", "-o", str(output), *units])


def build_objects(sources, output):
    # The executable reader concatenates symbols instead of merging declarations.
    # Separate units therefore require the explicit reviewed ELF object profile.
    tools = load_tools(("lowir", "cppgm++"))
    output.parent.mkdir(parents=True, exist_ok=True)
    objects = []
    for index, source in enumerate(sources):
        validated = output.with_suffix(f".unit-{index}.validated.lowir")
        invoke([tools["lowir"], "-o", str(validated), str(source)])
        object_file = output.with_suffix(f".unit-{index}.o")
        emit_object(validated, object_file, tools)
        objects.append(str(object_file))
    invoke(["cc", *objects, "-lm", "-o", str(output)])


def build_object(source, output):
    if source.suffix != ".lowir":
        raise ValueError("object bridge accepts only serialized .lowir input")
    tools = load_tools(("cppgm++",))
    emit_object(source, output, tools)


def emit_object(source, output, tools):
    output.parent.mkdir(parents=True, exist_ok=True)
    # This mode validates serialized LowIR with the host-object ABI facts enabled.
    # The suffix gate prevents the multi-input driver from selecting a frontend.
    invoke([tools["cppgm++"], "-O0", "-c", str(source), "-o", str(output)])
    with output.open("rb") as stream:
        header = stream.read(20)
    # ELF64, little-endian, current ELF version; ET_REL and EM_X86_64.
    if (header[:7] != b"\x7fELF\x02\x01\x01" or
            header[16:20] != b"\x01\x00\x3e\x00"):
        raise ValueError("object backend did not produce an x86-64 ELF64 relocatable object")


def test():
    source = ROOT / "tests/backend/return42.lowir"
    output = ROOT / "obj/backend/return42"
    build(source, output)
    result = subprocess.run([str(output)], capture_output=True, timeout=5)
    if result.returncode != 42 or result.stdout or result.stderr:
        raise ValueError("LowIR smoke program did not return exactly 42 with empty output")
    print("PASS: CPPGM parses, validates and executes LowIR (exit 42)")


def test_object():
    folder = ROOT / "obj" / "backend" / "object-handshake"
    object_file = folder / "interop.o"
    binary = folder / "interop"
    build_object(ROOT / "tests" / "backend" / "interop.lowir", object_file)
    invoke(["cc", "-std=c11", "-Wall", "-Wextra", "-Werror",
            str(ROOT / "tests" / "backend" / "interop.c"), str(object_file), "-o", str(binary)])
    result = subprocess.run([str(binary)], capture_output=True, timeout=5)
    if result.returncode or result.stdout or result.stderr:
        raise ValueError("LowIR/C object handshake did not succeed with empty output")
    print("PASS: LowIR object interoperates with independent C calls, callbacks and globals")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--configure", type=Path)
    parser.add_argument("--configure-object", type=Path)
    parser.add_argument("--test", action="store_true")
    parser.add_argument("--test-object", action="store_true")
    parser.add_argument("--emit-object", action="store_true")
    parser.add_argument("source", nargs="*", type=Path)
    parser.add_argument("-o", type=Path)
    args = parser.parse_args()
    try:
        if args.configure:
            configure(args.configure)
        elif args.configure_object:
            configure_object(args.configure_object)
        elif args.test:
            test()
        elif args.test_object:
            test_object()
        elif args.source and args.o:
            if args.emit_object:
                if len(args.source) != 1:
                    raise ValueError("object bridge requires exactly one LowIR input")
                build_object(args.source[0].resolve(), args.o.resolve())
            else:
                build_many([source.resolve() for source in args.source], args.o.resolve())
        else:
            parser.error("use --configure, --configure-object, --test, --test-object, or SOURCE -o OUTPUT")
        return 0
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"backend error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
