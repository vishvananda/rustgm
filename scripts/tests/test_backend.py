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


if __name__ == "__main__":
    unittest.main()
