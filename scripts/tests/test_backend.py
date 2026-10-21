"""Boundary controls for the optional object bridge and immutable tool pins."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

SPEC = importlib.util.spec_from_file_location(
    "backend", Path(__file__).resolve().parents[2] / "scripts" / "backend.py")
backend = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(backend)


class ObjectBridge(unittest.TestCase):
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

    def test_multiple_units_require_the_reviewed_object_profile(self):
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
                validated = [str(output.with_suffix(f".unit-{index}.validated.lowir"))
                             for index in range(2)]
                objects = [output.with_suffix(f".unit-{index}.o") for index in range(2)]
                self.assertEqual(emit.call_args_list, [mock.call(Path(unit), target, tools)
                                                     for unit, target in zip(validated, objects)])
                self.assertEqual(invoke.call_args_list, [
                    mock.call(["validator", "-o", validated[0], str(sources[0])]),
                    mock.call(["validator", "-o", validated[1], str(sources[1])]),
                    mock.call(["cc", *map(str, objects), "-lm", "-o", str(output)])])


if __name__ == "__main__":
    unittest.main()
