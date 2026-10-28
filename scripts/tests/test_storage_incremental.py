"""Idle cache retirement preserves current artifacts and respects active work."""
from pathlib import Path
import os
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import storage_incremental


class IncrementalRetirement(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.crate = self.root / "obj/rust/dev-fast/incremental/example"
        self.crate.mkdir(parents=True)

    def session(self, name, stamp):
        path = self.crate / name
        path.mkdir()
        (path / "work-products.bin").write_bytes(b"cache")
        os.utime(path, ns=(stamp, stamp))
        return path

    def test_preserves_latest_working_pins_and_hardlinked_current_data(self):
        old = self.session("s-old-complete", 1)
        current = self.session("s-current-complete", 2)
        (old / "shared.bin").write_bytes(b"shared")
        os.link(old / "shared.bin", current / "shared.bin")
        working = self.session("s-later-working", 3)
        lock = self.crate / "s-old.lock"
        lock.touch()
        pin = self.root / "obj/parallel/pin"
        pin.mkdir(parents=True)
        (pin / ".keep-artifacts").touch()
        (pin / "baseline").write_bytes(b"baseline")
        with mock.patch.object(storage_incremental, "active_artifacts", return_value=[]):
            rows = storage_incremental.retire(self.root)
        self.assertEqual([row["path"] for row in rows], [str(old.relative_to(self.root))])
        self.assertFalse(old.exists())
        self.assertEqual((current / "shared.bin").read_bytes(), b"shared")
        self.assertTrue(working.is_dir())
        self.assertTrue(lock.exists())
        self.assertEqual((pin / "baseline").read_bytes(), b"baseline")
        self.assertEqual(len(list((self.root / "obj/cleanup").glob("incremental-*.json"))), 1)

    def test_active_compiler_blocks_retirement(self):
        old = self.session("s-old-complete", 1)
        self.session("s-current-complete", 2)
        with mock.patch.object(storage_incremental, "active_artifacts", return_value=[123]):
            with self.assertRaisesRegex(ValueError, "active"):
                storage_incremental.retire(self.root)
        self.assertTrue(old.exists())

    def test_external_symlink_is_never_followed(self):
        elsewhere = self.root / "outside"
        elsewhere.mkdir()
        (elsewhere / "work-products.bin").write_bytes(b"keep")
        (self.crate / "s-linked").symlink_to(elsewhere, target_is_directory=True)
        with mock.patch.object(storage_incremental, "active_artifacts", return_value=[]):
            with self.assertRaisesRegex(ValueError, "symlinked session"):
                storage_incremental.retire(self.root)
        self.assertEqual((elsewhere / "work-products.bin").read_bytes(), b"keep")

    def test_single_warm_cache_needs_no_retirement_record(self):
        current = self.session("s-current-complete", 1)
        with mock.patch.object(storage_incremental, "active_artifacts", return_value=[]):
            self.assertEqual(storage_incremental.retire(self.root), [])
        self.assertTrue(current.is_dir())
        self.assertFalse((self.root / "obj/cleanup").exists())

    def test_nested_cache_symlink_is_rejected_before_traversal(self):
        import shutil
        original = self.root / "obj/rust"
        outside = self.root / "outside"
        shutil.move(original, outside)
        original.symlink_to(outside, target_is_directory=True)
        with mock.patch.object(storage_incremental, "active_artifacts", return_value=[]):
            with self.assertRaisesRegex(ValueError, "symlinked cache"):
                storage_incremental.retire(self.root)
        self.assertTrue((outside / "dev-fast/incremental/example").is_dir())

    def test_explicitly_pinned_old_session_is_preserved(self):
        old = self.session("s-pinned-complete", 1)
        self.session("s-current-complete", 2)
        (old / ".keep-artifacts").touch()
        with mock.patch.object(storage_incremental, "active_artifacts", return_value=[]):
            self.assertEqual(storage_incremental.retire(self.root), [])
        self.assertTrue(old.is_dir())
