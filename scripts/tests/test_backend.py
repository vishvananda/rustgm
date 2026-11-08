"""Boundary controls for the optional object bridge and immutable tool pins."""
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

SPEC = importlib.util.spec_from_file_location(
    "backend", Path(__file__).resolve().parents[2] / "scripts" / "backend.py")
backend = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(backend)


class ObjectBridge(unittest.TestCase):
    def test_both_compilers_must_produce_relocatable_x86_64_objects(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "output.o"
            header = b"\x7fELF\x02\x01\x01" + b"\0" * 9 + b"\x01\x00\x3e\x00"
            path.write_bytes(header)
            backend.verify_object(path)
            for invalid in (b"", header[:19], b"other" + header[5:],
                            header[:16] + b"\x02\x00\x3e\x00",
                            header[:18] + b"\xb7\x00"):
                path.write_bytes(invalid)
                with self.assertRaises(ValueError):
                    backend.verify_object(path)

    def test_rust_input_never_reaches_the_multi_frontend_driver(self):
        with mock.patch.object(backend, "invoke") as invoke:
            with self.assertRaisesRegex(ValueError, "only serialized .lowir"):
                backend.build_object(Path("input.rs"), Path("output.o"))
            invoke.assert_not_called()

    def test_configuration_pins_bytes_and_preserves_executable_tools(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            checkout = root / "checkout"
            (checkout / "dev").mkdir(parents=True)
            tools = {}
            hashes = {}
            for name in ("lowir", "lowir2native", "cppgm++"):
                path = checkout / "dev" / name
                path.write_bytes((name + " reviewed bytes").encode())
                tools[name] = str(path)
                hashes[name] = backend.digest(path)
            original = {name: tools[name] for name in ("lowir", "lowir2native")}
            (root / "backend.local.json").write_text(json.dumps({"tools": original}))
            (root / "backend.lock.json").write_text(json.dumps({
                "commit": "original-executable-commit", "sha256": {
                    name: hashes[name] for name in original},
                "optional": {"object": {
                    "commit": "reviewed-commit", "target": "x86_64-linux",
                    "object_format": "elf64-x86-64", "capabilities": ["lowir-to-object"],
                    "sha256": {"cppgm++": hashes["cppgm++"]}}}}))
            with mock.patch.object(backend, "ROOT", root), mock.patch.object(
                    backend.subprocess, "check_output", return_value="reviewed-commit\n"):
                backend.configure_object(checkout)
                configured = backend.load_tools()
                self.assertEqual({name: configured[name] for name in original}, original)
                pin = Path(configured["cppgm++"])
                self.assertNotEqual(pin, Path(tools["cppgm++"]))
                self.assertEqual(backend.digest(pin), hashes["cppgm++"])
                Path(tools["cppgm++"]).write_bytes(b"later live rebuild")
                backend.load_tools(("cppgm++",))
                pin.chmod(0o755)
                pin.write_bytes(b"changed pin")
                with self.assertRaisesRegex(ValueError, "backend bytes changed"):
                    backend.load_tools(("cppgm++",))
                # An unused optional capability cannot change the earlier bridge.
                backend.load_tools()


class AssemblyCapability(unittest.TestCase):
    def lock(self, capabilities, object_commit="reviewed"):
        return {"commit": "reviewed", "sha256": {}, "optional": {"object": {
            "commit": object_commit, "target": "x86_64-linux", "object_format": "elf64-x86-64",
            "capabilities": capabilities, "sha256": {}}}}

    def test_only_a_same_commit_reviewed_object_profile_advertises_assembly(self):
        profile = backend.assembly_profile(self.lock(["lowir-to-object", "assembly"]))
        self.assertIn("assembly", profile["capabilities"])
        for lock in (self.lock(["lowir-to-object"]),
                     self.lock(["lowir-to-object", "assembly"], object_commit="other"),
                     self.lock(["assembly"])):
            with self.assertRaises(ValueError):
                backend.assembly_profile(lock)

    def test_each_named_function_needs_the_requested_mnemonic_and_opcode(self):
        listing = ("\n0000000000000000 <rg_stream_ps>:\n"
                   "   0:\t41 0f 2b 03          \tmovntps XMMWORD PTR [r11],xmm0\n"
                   "   4:\tc3                   \tret\n"
                   "\n0000000000000010 <rg_other>:\n"
                   "  10:\t0f 11 07             \tmovups XMMWORD PTR [rdi],xmm0\n")
        for text, functions, accepted in (
                (listing, ("rg_stream_ps",), True),
                (listing, ("rg_stream_ps", "rg_other"), False),
                (listing.replace("movntps", "movaps"), ("rg_stream_ps",), False),
                (listing.replace("0f 2b", "0f 29"), ("rg_stream_ps",), False)):
            completed = mock.Mock(returncode=0, stdout=text.encode(), stderr=b"")
            with mock.patch.object(backend.subprocess, "run", return_value=completed):
                if accepted:
                    backend.require_instruction(Path("stream.o"), "movntps", "0f 2b", functions)
                else:
                    with self.assertRaisesRegex(ValueError, "lacks the requested movntps"):
                        backend.require_instruction(Path("stream.o"), "movntps", "0f 2b", functions)


    def test_section_free_executables_are_inspected_through_executable_segments(self):
        code = b"\x45\x0f\x2b\x32"
        header = bytearray(64)
        header[:7] = b"\x7fELF\x02\x01\x01"
        header[32:40] = (64).to_bytes(8, "little")
        header[54:56] = (56).to_bytes(2, "little")
        header[56:58] = (2).to_bytes(2, "little")
        def segment(flags, offset, size):
            row = bytearray(56)
            row[0:4] = (1).to_bytes(4, "little")
            row[4:8] = flags.to_bytes(4, "little")
            row[8:16] = offset.to_bytes(8, "little")
            row[32:40] = size.to_bytes(8, "little")
            return bytes(row)
        image = bytes(header) + segment(4, 0, 64) + segment(5, 176, len(code)) + code
        listing = ("\n0000000000000000 <.data>:\n"
                   "   0:\t45 0f 2b 32          \tmovntps XMMWORD PTR [r10],xmm14\n")
        with tempfile.TemporaryDirectory() as temporary:
            binary = Path(temporary) / "program"
            for text, data, accepted in ((listing, image, True),
                                         (listing.replace("movntps", "movups"), image, False),
                                         (listing, image[:120] + bytes(56) + code, False)):
                binary.write_bytes(data)
                completed = mock.Mock(returncode=0, stdout=text.encode(), stderr=b"")
                with mock.patch.object(backend.subprocess, "run", return_value=completed) as run:
                    if accepted:
                        backend.require_executable_instruction(binary, "movntps", "0f 2b")
                        self.assertEqual(Path(temporary, "program.code").read_bytes(), code)
                        self.assertIn("binary", run.call_args.args[0])
                    else:
                        with self.assertRaises(ValueError):
                            backend.require_executable_instruction(binary, "movntps", "0f 2b")


class ExecutablePins(unittest.TestCase):
    def test_live_rebuild_does_not_change_configured_execution(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            checkout = root / "checkout"
            (checkout / "dev").mkdir(parents=True)
            validator = checkout / "dev" / "lowir"
            linker = checkout / "dev" / "lowir2native"
            validator.write_text("#!/usr/bin/env python3\nimport pathlib,shutil,sys\n"
                                 "shutil.copyfile(sys.argv[-1],sys.argv[2])\n")
            linker.write_text("#!/usr/bin/env python3\nimport pathlib,sys\n"
                              "p=pathlib.Path(sys.argv[3]);p.write_text('#!/bin/sh\\nexit 42\\n');p.chmod(0o755)\n")
            for tool in (validator, linker):
                tool.chmod(0o755)
            hashes = {tool.name: backend.digest(tool) for tool in (validator, linker)}
            (root / "backend.lock.json").write_text(json.dumps({"commit": "reviewed", "sha256": hashes}))
            # An independently installed optional profile must survive default setup.
            (root / "backend.local.json").write_text(json.dumps({"tools": {"cppgm++": "optional-pin"}}))
            source, output = root / "source.lowir", root / "program"
            source.write_text("frozen LowIR input")
            with mock.patch.object(backend, "ROOT", root), mock.patch.object(
                    backend.subprocess, "check_output", return_value="reviewed\n"):
                backend.configure(checkout)
                configured = backend.load_tools()
                self.assertEqual(configured["cppgm++"], "optional-pin")
                for tool in (validator, linker):
                    self.assertNotEqual(Path(configured[tool.name]), tool)
                    self.assertEqual(backend.digest(Path(configured[tool.name])), hashes[tool.name])
                    tool.write_text("#!/bin/sh\nexit 77\n")
                backend.build(source, output)
                executed = backend.subprocess.run([str(output)], capture_output=True, check=False)
                self.assertEqual((executed.returncode, executed.stdout, executed.stderr), (42, b"", b""))
                before = (root / "backend.local.json").read_bytes()
                with self.assertRaisesRegex(ValueError, "binary differs from reviewed backend"):
                    backend.configure(checkout)
                self.assertEqual((root / "backend.local.json").read_bytes(), before)
                pin = Path(configured["lowir2native"])
                pin.chmod(0o755)
                pin.write_text("#!/bin/sh\nexit 99\n")
                with self.assertRaisesRegex(ValueError, "backend bytes changed"):
                    backend.load_tools()

    def test_incorrect_commit_or_existing_pin_cannot_replace_configuration(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            checkout = root / "checkout"
            (checkout / "dev").mkdir(parents=True)
            hashes = {}
            for name in ("lowir", "lowir2native"):
                tool = checkout / "dev" / name
                tool.write_bytes(name.encode())
                hashes[name] = backend.digest(tool)
            (root / "backend.lock.json").write_text(json.dumps({"commit": "reviewed", "sha256": hashes}))
            local = root / "backend.local.json"
            local.write_text(json.dumps({"tools": {"old": "retained"}}))
            before = local.read_bytes()
            with mock.patch.object(backend, "ROOT", root), mock.patch.object(
                    backend.subprocess, "check_output", return_value="different\n"):
                with self.assertRaisesRegex(ValueError, "reviewed backend commit"):
                    backend.configure(checkout)
            pin = root / "obj" / "backend" / "pins" / hashes["lowir"] / "lowir"
            pin.parent.mkdir(parents=True)
            pin.write_bytes(b"corrupt existing pin")
            with mock.patch.object(backend, "ROOT", root), mock.patch.object(
                    backend.subprocess, "check_output", return_value="reviewed\n"):
                with self.assertRaisesRegex(ValueError, "immutable backend pin differs"):
                    backend.configure(checkout)
                self.assertEqual(local.read_bytes(), before)


class MultipleUnits(unittest.TestCase):
    def test_multiple_units_without_object_profile_fail_before_execution(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "source.lowir"
            source.write_text("serialized unit")
            with mock.patch.object(backend, "load_tools", side_effect=ValueError("missing object profile")), mock.patch.object(backend, "invoke") as invoke:
                with self.assertRaisesRegex(ValueError, "missing object profile"):
                    backend.build_many([source, source], Path(temporary) / "output")
                invoke.assert_not_called()

    def test_missing_or_empty_input_fails_before_tool_loading(self):
        with mock.patch.object(backend, "load_tools") as tools:
            for sources in ([], [Path("missing.lowir")]):
                with self.assertRaises(ValueError):
                    backend.build_many(sources, Path("output"))
            tools.assert_not_called()

    def test_single_input_preserves_the_existing_command_and_validated_path(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source, output = root / "source.lowir", root / "program"
            source.write_text("serialized unit")
            tools = {"lowir": "validator", "lowir2native": "linker"}
            with mock.patch.object(backend, "load_tools", return_value=tools), mock.patch.object(
                    backend, "invoke") as invoke:
                backend.build(source, output)
                validated = str(output.with_suffix(".validated.lowir"))
                self.assertEqual(invoke.call_args_list, [
                    mock.call(["validator", "-o", validated, str(source)]),
                    mock.call(["linker", "-O0", "-o", str(output), validated])])

    def test_multiple_units_use_the_reviewed_host_object_validation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            sources = [root / "one.lowir", root / "two.lowir"]
            for source in sources:
                source.write_text("serialized unit")
            output = root / "program"
            tools = {"lowir": "validator", "cppgm++": "object-compiler"}
            with mock.patch.object(backend, "load_tools", return_value=tools), mock.patch.object(
                    backend, "invoke") as invoke, mock.patch.object(backend, "emit_object") as emit:
                backend.build_many(sources, output)
                objects = [output.with_suffix(f".unit-{index}.o") for index in range(2)]
                self.assertEqual(emit.call_args_list, [mock.call(unit, target, tools)
                                                     for unit, target in zip(sources, objects)])
                self.assertEqual(invoke.call_args_list, [
                    mock.call(["cc", *map(str, objects), "-lm", "-o", str(output)])])

    def test_object_inputs_cannot_route_to_the_cpp_frontend(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source.cpp"
            source.write_text("int main() { return 0; }")
            with mock.patch.object(backend, "load_tools") as tools, mock.patch.object(
                    backend, "invoke") as invoke:
                with self.assertRaisesRegex(ValueError, "serialized .lowir"):
                    backend.build_objects([source], root / "output")
                tools.assert_not_called()
                invoke.assert_not_called()
                with self.assertRaisesRegex(ValueError, "serialized .lowir"):
                    backend.emit_object(source, root / "output.o", {})
                invoke.assert_not_called()


if __name__ == "__main__":
    unittest.main()
