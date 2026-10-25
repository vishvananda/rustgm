"""CI must install only the approved complete backend without executing downloads."""
import hashlib
import io
import json
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from setup_backend import install


class BackendDistribution(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.payloads = {name: ("approved " + name).encode() for name in ("lowir", "lowir2native", "cppgm++")}
        self.lock = {"target": "x86_64-linux", "sha256": {
            name: hashlib.sha256(data).hexdigest() for name, data in self.payloads.items() if name != "cppgm++"},
            "optional": {"object": {"target": "x86_64-linux", "object_format": "elf64-x86-64",
                "capabilities": ["lowir-to-object"], "sha256": {
                    "cppgm++": hashlib.sha256(self.payloads["cppgm++"]).hexdigest()}}}}

    def bundle(self, payloads):
        path = self.root / "bundle.tar.gz"
        with tarfile.open(path, "w:gz") as archive:
            for name, data in payloads.items():
                member = tarfile.TarInfo(name)
                member.size = len(data)
                archive.addfile(member, io.BytesIO(data))
        self.lock["distribution"] = {"sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
        (self.root / "backend.lock.json").write_text(json.dumps(self.lock))
        return path

    def test_installs_verified_private_copies_and_complete_configuration(self):
        path = self.bundle(self.payloads)
        install(self.root, path)
        tools = json.loads((self.root / "backend.local.json").read_text())["tools"]
        self.assertEqual(set(tools), set(self.payloads))
        for name, value in tools.items():
            pinned = Path(value)
            self.assertEqual(pinned.read_bytes(), self.payloads[name])
            self.assertTrue((pinned.parent / ".keep-artifacts").is_file())
        path.write_bytes(b"changed download")
        self.assertEqual(Path(tools["lowir"]).read_bytes(), self.payloads["lowir"])

    def test_bad_later_tool_cannot_publish_any_pin_or_replace_configuration(self):
        payloads = {**self.payloads, "lowir2native": b"wrong"}
        path = self.bundle(payloads)
        local = self.root / "backend.local.json"
        local.write_text("old configuration")
        with self.assertRaisesRegex(ValueError, "differs from backend lock"):
            install(self.root, path)
        self.assertEqual(local.read_text(), "old configuration")
        self.assertFalse((self.root / "obj/backend/pins").exists())

    def test_archive_digest_and_missing_tools_fail_before_publication(self):
        path = self.bundle(self.payloads)
        path.write_bytes(b"corrupt")
        with self.assertRaisesRegex(ValueError, "locked archive"):
            install(self.root, path)
        path = self.bundle({"lowir": self.payloads["lowir"]})
        with self.assertRaisesRegex(ValueError, "missing a locked tool"):
            install(self.root, path)
        self.assertFalse((self.root / "backend.local.json").exists())

    def test_unsafe_member_paths_fail(self):
        path = self.bundle({**self.payloads, "../outside": b"bad"})
        with self.assertRaisesRegex(ValueError, "unexpected backend archive entry"):
            install(self.root, path)
        self.assertFalse((self.root / "backend.local.json").exists())
