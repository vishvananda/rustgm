#!/usr/bin/env python3
"""Pinned LowIR executable/object bridges; never a Rust frontend fallback."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

import native_links

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


def pin_tool(source, name, expected, *, root=None):
    pin = (ROOT if root is None else root) / "obj" / "backend" / "pins" / expected / name
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


def assembly_profile(lock):
    """Assembly is an optional capability of the reviewed object profile.

    The standalone reader and executable linker share that compiler's assembly
    boundary, so a lock mixing tool commits cannot advertise it."""
    profile = object_profile(lock)
    if "assembly" not in profile["capabilities"] or profile.get("commit") != lock.get("commit"):
        raise ValueError("reviewed x86-64 assembly capability is unavailable")
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


def build_many(sources, output, *, native_plan=None, native_search=(), native_objects=()):
    if not sources:
        raise ValueError("executable bridge requires at least one LowIR input")
    if any(not source.is_file() for source in sources):
        raise ValueError("LowIR input does not exist")
    if len(sources) > 1025:
        raise ValueError("LowIR input count exceeds crate profile budget")
    if len(native_search) > 1024 or len(native_objects) > 1024:
        raise ValueError("native search/object count exceeds profile budget")
    if len(sources) > 1 or native_plan is not None or native_search or native_objects:
        libraries = native_links.load(native_plan) if native_plan is not None else []
        build_objects(sources, output, libraries=libraries, search=native_search, native_objects=native_objects)
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


def build_objects(sources, output, *, libraries=(), search=(), native_objects=()):
    # The executable reader concatenates symbols instead of merging declarations.
    # The object driver validates with the host ABI facts, including SSEUP.
    # The standalone reader deliberately has a narrower executable profile.
    if not sources or any(source.suffix != ".lowir" or not source.is_file() for source in sources):
        raise ValueError("object bridge accepts only existing serialized .lowir inputs")
    tools = load_tools(("cppgm++",))
    output.parent.mkdir(parents=True, exist_ok=True)
    objects = []
    for index, source in enumerate(sources):
        object_file = output.with_suffix(f".unit-{index}.o")
        emit_object(source, object_file, tools)
        objects.append(str(object_file))
    for path in native_objects:
        verify_object(path)
        objects.append(str(path))
    invoke(["cc", *objects, *native_links.arguments(libraries, search), "-lm", "-o", str(output)])


def build_object(source, output):
    if source.suffix != ".lowir":
        raise ValueError("object bridge accepts only serialized .lowir input")
    tools = load_tools(("cppgm++",))
    emit_object(source, output, tools)


def emit_object(source, output, tools, *, run=None):
    if source.suffix != ".lowir":
        raise ValueError("object bridge accepts only serialized .lowir input")
    output.parent.mkdir(parents=True, exist_ok=True)
    # This mode validates serialized LowIR with the host-object ABI facts enabled.
    # The suffix gate prevents the multi-input driver from selecting a frontend.
    (invoke if run is None else run)([tools["cppgm++"], "-O0", "-c", str(source), "-o", str(output)])
    verify_object(output)


def verify_object(output):
    """Validate the shared object contract independently of the producing compiler."""
    with output.open("rb") as stream:
        header = stream.read(20)
    # ELF64, little-endian, current ELF version; ET_REL and EM_X86_64.
    if (header[:7] != b"\x7fELF\x02\x01\x01" or
            header[16:20] != b"\x01\x00\x3e\x00"):
        raise ValueError("compiler did not produce an x86-64 ELF64 relocatable object")


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


def listed_functions(listing, mnemonic, opcode):
    """Name each disassembled function (None for raw code) containing the instruction."""
    found = set()
    current = None
    for line in listing.splitlines():
        if line.endswith(">:") and "<" in line:
            current = line[line.index("<") + 1:-2]
            continue
        fields = line.split("\t")
        if len(fields) >= 3 and fields[2].split()[:1] == [mnemonic] and opcode in fields[1]:
            found.add(current)
    return found


def objdump(arguments):
    result = subprocess.run(["objdump", "-M", "intel", *arguments], capture_output=True, timeout=30)
    if result.returncode or result.stderr:
        raise ValueError(f"objdump failed ({result.returncode}): {result.stderr.decode(errors='replace')}")
    return result.stdout.decode(errors="replace")


def require_instruction(binary, mnemonic, opcode, functions):
    """Each named object function must contain the requested instruction itself.

    The source names this instruction; an equivalent ordinary store is not accepted."""
    found = listed_functions(objdump(["-d", str(binary)]), mnemonic, opcode)
    missing = [name for name in functions if name not in found]
    if missing:
        raise ValueError(f"{binary.name}: {', '.join(missing)} lacks the requested {mnemonic} instruction")


def require_executable_instruction(binary, mnemonic, opcode):
    """The standalone linker writes no section headers; inspect its executable segments."""
    data = binary.read_bytes()
    if data[:7] != b"\x7fELF\x02\x01\x01":
        raise ValueError(f"{binary.name}: not an x86-64 ELF64 executable")
    offset = int.from_bytes(data[32:40], "little")
    size = int.from_bytes(data[54:56], "little")
    count = int.from_bytes(data[56:58], "little")
    code = bytearray()
    for index in range(count):
        header = data[offset + index * size:offset + (index + 1) * size]
        # PT_LOAD with PF_X: p_type, p_flags, p_offset and p_filesz.
        if len(header) < 56:
            raise ValueError(f"{binary.name}: truncated ELF program header")
        if int.from_bytes(header[0:4], "little") == 1 and int.from_bytes(header[4:8], "little") & 1:
            start = int.from_bytes(header[8:16], "little")
            code += data[start:start + int.from_bytes(header[32:40], "little")]
    if not code:
        raise ValueError(f"{binary.name}: no executable segment")
    segment = binary.with_name(binary.name + ".code")
    segment.write_bytes(code)
    listing = objdump(["-D", "-b", "binary", "-m", "i386:x86-64", str(segment)])
    if not listed_functions(listing, mnemonic, opcode):
        raise ValueError(f"{binary.name}: executable code lacks the requested {mnemonic} instruction")


def test_assembly():
    assembly_profile(json.loads((ROOT / "backend.lock.json").read_text()))
    folder = ROOT / "obj" / "backend" / "assembly-handshake"
    object_file = folder / "stream.o"
    binary = folder / "stream"
    build_object(ROOT / "tests" / "backend" / "assembly.lowir", object_file)
    require_instruction(object_file, "movntps", "0f 2b", ("rg_stream_ps", "rg_stream_ps_stored"))
    invoke(["cc", "-std=c11", "-Wall", "-Wextra", "-Werror",
            str(ROOT / "tests" / "backend" / "assembly.c"), str(object_file), "-o", str(binary)])
    result = subprocess.run([str(binary)], capture_output=True, timeout=5)
    if result.returncode or result.stdout or result.stderr:
        raise ValueError("LowIR assembly object did not stream every bit with intact guards")
    program = folder / "program"
    build(ROOT / "tests" / "backend" / "assembly-program.lowir", program)
    require_executable_instruction(program, "movntps", "0f 2b")
    result = subprocess.run([str(program)], capture_output=True, timeout=5)
    if result.returncode != 42 or result.stdout or result.stderr:
        raise ValueError("standalone LowIR assembly program did not stream every bit with intact guards")
    print("PASS: LowIR assembly emits movntps in objects and executables with full bits and guards")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--configure", type=Path)
    parser.add_argument("--configure-object", type=Path)
    parser.add_argument("--test", action="store_true")
    parser.add_argument("--test-object", action="store_true")
    parser.add_argument("--test-assembly", action="store_true")
    parser.add_argument("--emit-object", action="store_true")
    parser.add_argument("--native-links", type=Path)
    parser.add_argument("--native-search", type=Path, action="append", default=[])
    parser.add_argument("--native-object", type=Path, action="append", default=[])
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
        elif args.test_assembly:
            test_assembly()
        elif args.source and args.o:
            if args.emit_object:
                if args.native_links or args.native_search or args.native_object:
                    raise ValueError("native dependencies require final executable linking")
                if len(args.source) != 1:
                    raise ValueError("object bridge requires exactly one LowIR input")
                build_object(args.source[0].resolve(), args.o.resolve())
            else:
                build_many([source.resolve() for source in args.source], args.o.resolve(),
                           native_plan=args.native_links, native_search=args.native_search,
                           native_objects=[path.resolve() for path in args.native_object])
        else:
            parser.error("use --configure, --configure-object, --test, --test-object, --test-assembly, "
                         "or SOURCE -o OUTPUT")
        return 0
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"backend error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
