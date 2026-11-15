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
import backend
from setup_backend import install

PREFIX = "cppgm-v9.9.9-linux-x86_64"
TOOLS = ("lowir", "lowir2native", "cppgm")
LIBRARY = ("libcppgm_lowir.a", "libcppgm_lowir.so", "lowir_interface.h", "lowir_records.h")


def sha256(data):
    return hashlib.sha256(data).hexdigest()


class BackendDistribution(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.payloads = {name: ("approved " + name).encode() for name in TOOLS + LIBRARY}
        hashes = {name: sha256(data) for name, data in self.payloads.items()}
        self.lock = {"target": "x86_64-linux", "commit": "release-commit", "sha256": {
            name: hashes[name] for name in ("lowir", "lowir2native")},
            "optional": {
                "object": {"target": "x86_64-linux", "object_format": "elf64-x86-64",
                           "capabilities": ["lowir-to-object"], "sha256": {"cppgm": hashes["cppgm"]}},
                "library": {"target": "x86_64-linux", "commit": "release-commit", "abi_version": 2,
                            "sha256": {name: hashes[name] for name in LIBRARY}}}}

    def members(self, payloads):
        """The release package's layout: tools, then library and headers, plus unlocked files."""
        named = {f"{PREFIX}/README.md": b"unlocked note", f"{PREFIX}/bin/preproc": b"unlocked tool"}
        for name, data in payloads.items():
            folder = "bin" if name in TOOLS else backend.library_member(name).split("/")[0]
            named[f"{PREFIX}/{folder}/{name}"] = data
        return named

    def bundle(self, members, prefix=PREFIX):
        path = self.root / "bundle.tar.gz"
        with tarfile.open(path, "w:gz") as archive:
            for name, data in members.items():
                member = tarfile.TarInfo(name)
                member.size = len(data)
                archive.addfile(member, io.BytesIO(data))
        (self.root / "backend.lock.json").write_text(json.dumps(self.lock))
        (self.root / "backend.distribution.json").write_text(json.dumps({
            "sha256": sha256(path.read_bytes()), "prefix": prefix,
            "backend_lock_sha256": sha256((self.root / "backend.lock.json").read_bytes())}))
        return path

    def test_installs_verified_private_copies_and_complete_configuration(self):
        path = self.bundle(self.members(self.payloads))
        install(self.root, path)
        configured = json.loads((self.root / "backend.local.json").read_text())
        self.assertEqual(set(configured), {"tools", "library"})
        self.assertEqual(set(configured["tools"]), set(TOOLS))
        for name, value in configured["tools"].items():
            pinned = Path(value)
            self.assertEqual(pinned.read_bytes(), self.payloads[name])
            self.assertTrue((pinned.parent / ".keep-artifacts").is_file())
        # The library and its headers sit together; nothing unlocked is installed.
        library = Path(configured["library"])
        self.assertEqual({path.name for path in library.iterdir()}, {*LIBRARY, ".keep-artifacts"})
        for name in LIBRARY:
            self.assertEqual((library / name).read_bytes(), self.payloads[name])
        self.assertEqual(list(self.root.rglob("preproc")), [])
        path.write_bytes(b"changed download")
        self.assertEqual(Path(configured["tools"]["lowir"]).read_bytes(), self.payloads["lowir"])

    def test_a_bad_later_file_cannot_publish_any_pin_or_replace_configuration(self):
        local = self.root / "backend.local.json"
        for name in ("lowir2native", "lowir_records.h"):
            path = self.bundle(self.members({**self.payloads, name: b"wrong"}))
            local.write_text("old configuration")
            with self.assertRaisesRegex(ValueError, f"{name}: distribution file differs from backend lock"):
                install(self.root, path)
            self.assertEqual(local.read_text(), "old configuration")
            self.assertFalse((self.root / "obj/backend/pins").exists())

    def test_archive_digest_and_missing_files_fail_before_publication(self):
        path = self.bundle(self.members(self.payloads))
        path.write_bytes(b"corrupt")
        with self.assertRaisesRegex(ValueError, "locked archive"):
            install(self.root, path)
        for absent in ("lowir", "libcppgm_lowir.so"):
            kept = {name: data for name, data in self.payloads.items() if name != absent}
            with self.assertRaisesRegex(ValueError, "missing a locked file"):
                install(self.root, self.bundle(self.members(kept)))
        self.assertFalse((self.root / "backend.local.json").exists())
        self.assertFalse((self.root / "obj/backend/pins").exists())

    def test_entries_outside_the_release_directory_fail(self):
        members = self.members(self.payloads)
        for name in ("../outside", "/absolute", "other-release/bin/lowir", f"{PREFIX}/../escape", PREFIX):
            with self.assertRaisesRegex(ValueError, "unexpected backend archive entry"):
                install(self.root, self.bundle({**members, name: b"bad"}))
        for prefix in ("", "../" + PREFIX, PREFIX + "/bin"):
            with self.assertRaisesRegex(ValueError, "names no release directory"):
                install(self.root, self.bundle(members, prefix=prefix))
        self.assertFalse((self.root / "backend.local.json").exists())

    def test_the_library_profile_must_belong_to_the_locked_release(self):
        for change in ({"commit": "another-commit"}, {"abi_version": 3}, {"target": "aarch64-linux"},
                       {"sha256": {"lowir_interface.h": "0" * 64}},
                       {"sha256": {"libcppgm_lowir.a": "0" * 64, "lowir_interface.h": "0" * 64,
                                   "../lowir_records.h": "0" * 64}}):
            lock = json.loads(json.dumps(self.lock))
            lock["optional"]["library"].update(change)
            with self.assertRaisesRegex(ValueError, "LowIR library capability is unavailable"):
                backend.library_profile(lock)
        self.assertEqual(backend.library_profile(self.lock)["abi_version"], 2)
        self.assertEqual(backend.library_member("lowir_interface.h"), "include/lowir_interface.h")
        self.assertEqual(backend.library_member("libcppgm_lowir.so"), "lib/libcppgm_lowir.so")


if __name__ == "__main__":
    unittest.main()
