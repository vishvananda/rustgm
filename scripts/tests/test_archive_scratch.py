"""Scratch compaction preserves evidence bytes, links, and active/pinned inputs."""
from pathlib import Path
import os
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from archive_scratch import archive


class ScratchArchive(unittest.TestCase):
    def test_round_trip_evidence_and_links_without_touching_managed_or_pinned(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            folder = root / "obj/probe-12345"
            folder.mkdir(parents=True)
            (folder / "input.rs").write_bytes(b"original input\x00\xff")
            os.link(folder / "input.rs", folder / "hardlink")
            (folder / "external").symlink_to(root)
            pinned = root / "obj/pinned-54321"
            pinned.mkdir()
            (pinned / ".keep-artifacts").touch()
            cache = root / "obj/rust"
            cache.mkdir()
            with patch("archive_scratch.check_space"), patch("archive_scratch.active_artifacts", return_value=[]):
                archive(root, 0)
            self.assertFalse(folder.exists())
            self.assertTrue(pinned.is_dir())
            self.assertTrue(cache.is_dir())
            saved_path, = (root / "obj/archives").glob("*.tar.gz")
            with tarfile.open(saved_path) as saved:
                self.assertEqual(saved.extractfile("obj/probe-12345/input.rs").read(), b"original input\x00\xff")
                self.assertEqual(saved.extractfile("obj/probe-12345/hardlink").read(), b"original input\x00\xff")
                self.assertTrue(saved.getmember("obj/probe-12345/external").issym())
