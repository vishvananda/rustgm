#!/usr/bin/env python3
"""Pinned LowIR executable/object bridges; never a Rust frontend fallback."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

import native_links

ROOT = Path(__file__).resolve().parents[1]
# The one interface version of the in-process library that this bridge reviews.
LIBRARY_ABI_VERSION = 4
# What the library's text view of the handshake module must state: the
# boundary forms of RustGM's writer. Interface version 2 carried only the
# storage and the arity. The classes of rg_library_turn are the ones the
# library derives from record types (interface version 4).
LIBRARY_TEXT = (
    "[classes=integer_sse]) -> obj<16x8> [result_classes=sse_integer, ",
    "[classes=sse_integer]) -> obj<16x8> [result_classes=integer_sse",
    ") as (",
    "[classes=integer_sse]) -> obj<16x8> [result_classes=sse_integer]\n",
    "[classes=1:integer_sse]",
    "[arity=variadic",
    "[return=noreturn",
    "[storage=readonly",
    "[storage=thread_local",
    " intel readwrite 1 0 0 2 {",
    "    asm @",
    " = vector pshufd ",
)
# The same forms in the library's view of the module the Rust bridge states
# from RustGM's model (tests/backend/library-observer.c calls it).
BRIDGE_TEXT = (
    "[classes=integer_sse]) -> obj<16x8> [result_classes=sse_integer] {",
    "[classes=sse_integer]) -> obj<16x8> [result_classes=integer_sse] {",
    ") as (",
    "[classes=1:integer_sse]",
    "[arity=variadic]",
    "[return=noreturn]",
    "[storage=readonly]",
    "[storage=thread_local]",
    " intel readwrite 1 0 0 2 {",
    "    asm @",
)
# The bridge crate builds apart from the frontend's Cargo cache.
BRIDGE_TARGET = ROOT / "obj" / "rust" / "bridge"


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
    # Keep whatever else is configured, such as the object driver and library.
    configured = json.loads(local.read_text()) if local.is_file() else {}
    for name, path in tools.items():
        configured.setdefault("tools", {})[name] = str(pin_tool(Path(path), name, lock["sha256"][name]))
    local.write_text(json.dumps(configured, indent=2) + "\n")
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


def library_profile(lock):
    """The in-process LowIR library is an optional capability of the same release.

    Its headers and both library forms are pinned together: a client built
    from one header set must link the library those headers describe."""
    profile = lock.get("optional", {}).get("library", {})
    hashes = profile.get("sha256", {})
    if (profile.get("target") != "x86_64-linux" or profile.get("commit") != lock.get("commit") or
            profile.get("abi_version") != LIBRARY_ABI_VERSION or
            not {"libcppgm_lowir.a", "lowir_interface.h"} <= set(hashes) or
            any(Path(name).name != name or not name.endswith((".a", ".so", ".h")) for name in hashes)):
        raise ValueError("reviewed LowIR library capability is unavailable")
    return profile


def library_release(lock):
    """The release the pinned library must report: the lock's tag without its `v`."""
    release = lock.get("release", "")
    if not isinstance(release, str) or not re.fullmatch(r"v[0-9]+(\.[0-9]+){2}", release):
        raise ValueError("backend lock names no release for the LowIR library")
    return release[1:]


def library_member(name):
    """Where the release package keeps one library file."""
    return ("include/" if name.endswith(".h") else "lib/") + name


def pin_library(source, profile, *, root=None):
    """Keep the verified library and its headers in one immutable folder."""
    hashes = profile["sha256"]
    pin = (ROOT if root is None else root) / "obj" / "backend" / "pins" / hashes["libcppgm_lowir.a"]
    pin.mkdir(parents=True, exist_ok=True)
    for name, expected in hashes.items():
        target = pin / name
        if not target.exists():
            shutil.copyfile(source / name, target)
        if digest(target) != expected:
            raise ValueError(f"{name}: immutable backend pin differs from reviewed bytes")
        target.chmod(0o444)
    (pin / ".keep-artifacts").write_text("Current reviewed LowIR backend library\n")
    return pin


def load_library():
    local = ROOT / "backend.local.json"
    if not local.is_file():
        raise ValueError("install the backend first: python3 scripts/setup_backend.py")
    folder = json.loads(local.read_text()).get("library")
    profile = library_profile(json.loads((ROOT / "backend.lock.json").read_text()))
    if folder is None:
        raise ValueError("LowIR library: install the reviewed backend capability first")
    for name, expected in profile["sha256"].items():
        if digest(Path(folder) / name) != expected:
            raise ValueError(f"{name}: backend bytes changed")
    return Path(folder)


def library_config():
    """What the in-process bridge's build script links: the verified pin only.

    One `key=value` line each, so that the build script needs no JSON reader."""
    lock = json.loads((ROOT / "backend.lock.json").read_text())
    folder = load_library()
    return (f"folder={folder}\nrelease={library_release(lock)}\n"
            f"abi_version={library_profile(lock)['abi_version']}\n")


def load_tools(required=("lowir", "lowir2native")):
    local = ROOT / "backend.local.json"
    if not local.is_file():
        raise ValueError("install the backend first: python3 scripts/setup_backend.py")
    tools = json.loads(local.read_text())["tools"]
    lock = json.loads((ROOT / "backend.lock.json").read_text())
    for name in required:
        hashes = object_profile(lock)["sha256"] if name == "cppgm" else lock["sha256"]
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
    source = checkout / "dev" / "cppgm"
    expected = profile["sha256"]["cppgm"]
    if digest(source) != expected:
        raise ValueError("cppgm: binary differs from reviewed object backend")
    tools["cppgm"] = str(pin_tool(source, "cppgm", expected))
    local = ROOT / "backend.local.json"
    configured = {**json.loads(local.read_text()), "tools": tools}
    local.write_text(json.dumps(configured, indent=2) + "\n")
    print("Configured immutable pinned CPPGM LowIR object backend")


# A coarse runaway guard on one backend command, not a performance bound: a
# base for small inputs on a loaded host plus a share per MB that it reads.
# The pinned tools take under 0.2 s per MB of LowIR even at -O2.
COMMAND_SECONDS = 120
COMMAND_SECONDS_PER_MB = 1


def time_limit(command):
    """The seconds one command may run: the base plus a share per MB of its existing inputs."""
    size = 0
    arguments = iter(command[1:])
    for argument in arguments:
        if argument == "-o":
            next(arguments, None)
            continue
        path = Path(argument)
        if path.is_file():
            size += path.stat().st_size
    return COMMAND_SECONDS + COMMAND_SECONDS_PER_MB * size // 1_000_000


def invoke(command):
    limit = time_limit(command)
    try:
        result = subprocess.run(command, capture_output=True, timeout=limit)
    except subprocess.TimeoutExpired as expired:
        raise ValueError(f"backend command {Path(command[0]).name} exceeded its runaway limit of "
                         f"{limit} s for its inputs") from expired
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
    tools = load_tools(("cppgm",))
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
    tools = load_tools(("cppgm",))
    emit_object(source, output, tools)


def emit_object(source, output, tools, *, run=None):
    if source.suffix != ".lowir":
        raise ValueError("object bridge accepts only serialized .lowir input")
    output.parent.mkdir(parents=True, exist_ok=True)
    # This mode validates serialized LowIR with the host-object ABI facts enabled.
    # The suffix gate prevents the multi-input driver from selecting a frontend.
    (invoke if run is None else run)([tools["cppgm"], "-O0", "-c", str(source), "-o", str(output)])
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


def test_library():
    library = load_library()
    release = library_release(json.loads((ROOT / "backend.lock.json").read_text()))
    tools = load_tools(("cppgm",))
    folder = ROOT / "obj" / "backend" / "library-handshake"
    folder.mkdir(parents=True, exist_ok=True)
    client, producer = folder / "producer.o", folder / "producer"
    strict = ["cc", "-std=c11", "-Wall", "-Wextra", "-Werror"]
    invoke([*strict, "-I", str(library), "-c", str(ROOT / "tests" / "backend" / "library.c"),
            "-o", str(client)])
    # The library is C++ inside: its C client links with the C++ driver.
    invoke(["c++", str(client), str(library / "libcppgm_lowir.a"), "-pthread", "-o", str(producer)])
    # Fixed code is what the object bridge's own command makes; the host C
    # driver links position-independent code by default.
    for form, flags in (("fixed", []), ("pic", ["-fPIC"])):
        module, view, observer = (folder / (form + suffix) for suffix in (".o", ".lowir", "-observer"))
        from_text = folder / (form + "-text.o")
        for stale in (module, view, from_text):
            stale.unlink(missing_ok=True)
        result = subprocess.run([str(producer), str(module), str(view), release, form],
                                capture_output=True, timeout=30)
        if result.returncode or result.stdout or result.stderr:
            raise ValueError(f"LowIR library producer failed ({result.returncode}): "
                             + result.stderr.decode(errors="replace"))
        verify_object(module)
        stated = view.read_text()
        missing = [words for words in LIBRARY_TEXT if words not in stated]
        if missing:
            raise ValueError(f"LowIR library text view lacks {missing}")
        # The pinned driver must make the same object from that text: the
        # library is the reviewed backend, not a second implementation.
        invoke([tools["cppgm"], "-O0", *flags, "-c", str(view), "-o", str(from_text)])
        if module.read_bytes() != from_text.read_bytes():
            raise ValueError(f"object compiled in process differs from the driver's object ({form})")
        require_instruction(module, "movntps", "0f 2b", ("rg_library_stream",))
        invoke([*strict, str(ROOT / "tests" / "backend" / "library-observer.c"), str(module),
                "-o", str(observer)])
        result = subprocess.run([str(observer)], capture_output=True, timeout=5)
        if result.returncode or result.stdout or result.stderr:
            raise ValueError(f"object compiled in process did not link and compute its declared results ({form})")
    print("PASS: the LowIR library builds a module with boundary classes, call boundaries, noreturn, "
          "assembly and record types in process; its objects equal the driver's and link")


def build_bridge():
    """The optional bridge crate's binaries, linked against the verified pin."""
    environment = dict(os.environ, CARGO_TARGET_DIR=str(BRIDGE_TARGET))
    result = subprocess.run(["cargo", "build", "--manifest-path", str(ROOT / "dev/backend-bridge/Cargo.toml"),
                             "--locked", "--profile", "dev-fast", "--bins"],
                            capture_output=True, timeout=1800, env=environment)
    if result.returncode:
        raise ValueError("the in-process bridge did not build: " + result.stderr.decode(errors="replace")[-4000:])
    return BRIDGE_TARGET / "dev-fast"


def require_bridge_weak_bindings(module, stated):
    """Check the bridge's text and host ELF bindings in both code modes."""
    data, function = "rg_library_weak_data", "rg_library_weak_function"
    cell = "rg_library_weak_cells"
    for kind, name in (("global", data), ("function", function)):
        if not re.search(rf"^declare {kind} @{name}(?:\s|\()[^\n]*\bbinding=weak\b",
                         stated, re.M):
            raise ValueError(f"the library view lacks the weak {kind} declaration {name}")
    if not re.search(rf"^global @{cell} : [^\n]*\bbinding=internal\b", stated, re.M):
        raise ValueError("the library view lacks its internal weak-address cells")
    elf = subprocess.check_output(["readelf", "--wide", "--symbols", "--relocs", str(module)],
                                  timeout=5, text=True)
    def symbols(name):
        return re.findall(r"^\s*\d+:\s+\w+\s+\d+\s+\w+\s+(\w+)\s+\w+\s+(\w+)\s+"
                          + re.escape(name) + r"\s*$", elf, re.M)
    for name in (data, function):
        if symbols(name) != [("WEAK", "UND")]:
            raise ValueError(f"{module.name} does not bind {name} as weak undefined")
        if not re.search(rf"^.*\bR_X86_64_64\b[^\n]*\b{name}\b", elf, re.M):
            raise ValueError(f"{module.name} lacks a pointer relocation for {name}")
    # The pinned host writer prefixes a local symbol without object= by '@'.
    local_rows = symbols("@" + cell)
    if (len(local_rows) != 1 or local_rows[0][0] != "LOCAL"
            or not local_rows[0][1].isdigit() or symbols(cell)):
        raise ValueError(f"{module.name} does not keep its cell private and defined")


def test_bridge():
    """The Rust side of the library handshake: the module of library.c, built
    through RustGM's LowIR model and stated by the bridge crate."""
    local = ROOT / "backend.local.json"
    if not local.is_file() or json.loads(local.read_text()).get("library") is None:
        print("SKIP: in-process LowIR bridge: the pinned LowIR library is not installed "
              "(python3 scripts/setup_backend.py)")
        return
    # An installed library whose bytes changed is a failure, never a skip.
    load_library()
    tools = load_tools(("cppgm",))
    producer = build_bridge() / "rustgm-bridge-handshake"
    folder = ROOT / "obj" / "backend" / "bridge-handshake"
    folder.mkdir(parents=True, exist_ok=True)
    for stale in folder.iterdir():
        if stale.is_file():
            stale.unlink()
    result = subprocess.run([str(producer), str(folder)], capture_output=True, timeout=60)
    if result.returncode or result.stdout or result.stderr:
        raise ValueError(f"in-process bridge producer failed ({result.returncode}): "
                         + result.stderr.decode(errors="replace"))
    strict = ["cc", "-std=c11", "-Wall", "-Wextra", "-Werror"]
    for form, flags in (("fixed", []), ("pic", ["-fPIC"])):
        module, view = folder / (form + ".o"), folder / (form + ".lowir")
        verify_object(module)
        stated = view.read_text()
        missing = [words for words in BRIDGE_TEXT if words not in stated]
        if missing:
            raise ValueError(f"the bridge's library view lacks {missing}")
        # The library's view and RustGM's own text must both make this object:
        # the bridge states the module the reader builds from that text.
        for source, name in ((view, "view"), (folder / "text.lowir", "text")):
            compiled = folder / f"{form}-{name}.o"
            invoke([tools["cppgm"], "-O0", *flags, "-c", str(source), "-o", str(compiled)])
            if module.read_bytes() != compiled.read_bytes():
                raise ValueError(f"object compiled in process differs from the driver's object of the {name} ({form})")
        require_bridge_weak_bindings(module, stated)
        require_instruction(module, "movntps", "0f 2b", ("rg_library_stream",))
        observer = folder / (form + "-observer")
        invoke([*strict, str(ROOT / "tests" / "backend" / "library-observer.c"), str(module), "-o", str(observer)])
        result = subprocess.run([str(observer)], capture_output=True, timeout=5)
        if result.returncode or result.stdout or result.stderr:
            raise ValueError(f"object the bridge compiled did not link and compute its declared results ({form})")
    print("PASS: the in-process bridge states RustGM's LowIR to the library; its objects equal the driver's "
          "objects of the library view and of RustGM's text, and link")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--configure", type=Path)
    parser.add_argument("--configure-object", type=Path)
    parser.add_argument("--test", action="store_true")
    parser.add_argument("--test-object", action="store_true")
    parser.add_argument("--test-assembly", action="store_true")
    parser.add_argument("--test-library", action="store_true")
    parser.add_argument("--test-bridge", action="store_true")
    parser.add_argument("--library-config", action="store_true")
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
        elif args.test_library:
            test_library()
        elif args.test_bridge:
            test_bridge()
        elif args.library_config:
            sys.stdout.write(library_config())
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
                         "--test-library, --test-bridge, --library-config, or SOURCE -o OUTPUT")
        return 0
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"backend error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
