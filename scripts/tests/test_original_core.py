"""Reusing pinned sources must not weaken the whole-crate observer's integrity."""
import hashlib
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from observe_original_core import lint_profile_arguments, verify_sources


class OriginalCoreInputs(unittest.TestCase):
    def test_strict_observation_cannot_silently_reuse_a_capped_profile_or_producer(self):
        for profile, producer in (({}, {}), ({"lint_cap": "uncapped"}, {"lint_cap": "uncapped"})):
            self.assertEqual(lint_profile_arguments({"profile": profile}, producer, None), [])
        for profile, producer in (({"lint_cap": "allow"}, {}), ({}, {"lint_cap": "allow"}),
                                  ({"lint_cap": "allow"}, {"lint_cap": "allow"})):
            with self.subTest(profile=profile, producer=producer), self.assertRaises(ValueError):
                lint_profile_arguments({"profile": profile}, producer, None)

    def test_capped_observation_requires_both_explicit_owners(self):
        self.assertEqual(lint_profile_arguments(
            {"profile": {"lint_cap": "allow"}}, {"lint_cap": "allow"}, "allow"),
            ["--cap-lints=allow"])
        for profile, producer in (({}, {}), ({"lint_cap": "allow"}, {}),
                                  ({}, {"lint_cap": "allow"})):
            with self.subTest(profile=profile, producer=producer), self.assertRaises(ValueError):
                lint_profile_arguments({"profile": profile}, producer, "allow")
        with self.assertRaisesRegex(ValueError, "unsupported"):
            lint_profile_arguments({"profile": {}}, {}, "warn")

    def test_all_files_are_checked_even_when_the_compiler_never_opens_them(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = {"files": []}
            for name in ("lib.rs", "not_yet_opened.rs"):
                path = root / name
                data = b"pub fn f() {}"
                path.write_bytes(data)
                manifest["files"].append({"path": name, "bytes": len(data),
                                          "sha256": hashlib.sha256(data).hexdigest()})
            verify_sources(manifest, root)
            (root / "not_yet_opened.rs").write_bytes(b"pub fn g() {}")
            with self.assertRaisesRegex(ValueError, "changed pinned source"):
                verify_sources(manifest, root)
            (root / "not_yet_opened.rs").unlink()
            with self.assertRaisesRegex(ValueError, "missing or changed"):
                verify_sources(manifest, root)

    def test_paths_and_links_cannot_escape_the_verified_tree(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "source"
            root.mkdir()
            outside = root.parent / "outside.rs"
            outside.write_bytes(b"source")
            (root / "link.rs").symlink_to(outside)
            for path in ("../outside.rs", str(outside), "link.rs"):
                with self.subTest(path=path), self.assertRaises(ValueError):
                    verify_sources({"files": [{"path": path, "bytes": 6,
                                               "sha256": hashlib.sha256(b"source").hexdigest()}]}, root)
