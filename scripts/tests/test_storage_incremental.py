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

    def test_idle_linker_scratch_preserves_hardlinked_incremental_objects(self):
        current = self.session("s-current-complete", 1)
        deps = self.crate.parent.parent / "deps"
        deps.mkdir()
        shared = deps / "test.unit.rcgu.o"
        shared.write_bytes(b"compiled")
        os.link(shared, current / "cached.o")
        loose = deps / "test.other.rcgu.o"
        loose.write_bytes(b"abandoned")
        metadata = deps / "libtest.rmeta"
        metadata.write_bytes(b"warm")
        with mock.patch.object(storage_incremental, "active_artifacts", return_value=[]):
            rows = storage_incremental.retire(self.root)
        self.assertEqual(len(rows), 2)
        self.assertFalse(shared.exists())
        self.assertFalse(loose.exists())
        self.assertEqual((current / "cached.o").read_bytes(), b"compiled")
        self.assertEqual(metadata.read_bytes(), b"warm")

    def test_pinned_linker_scratch_is_preserved(self):
        deps = self.crate.parent.parent / "deps"
        deps.mkdir()
        (deps / ".keep-artifacts").touch()
        scratch = deps / "test.unit.rcgu.o"
        scratch.write_bytes(b"pinned")
        with mock.patch.object(storage_incremental, "active_artifacts", return_value=[]):
            self.assertEqual(storage_incremental.retire(self.root), [])
        self.assertTrue(scratch.exists())

    def test_codegen_symlink_is_rejected_before_unlinking_scratch(self):
        deps = self.crate.parent.parent / "deps"
        deps.mkdir()
        scratch = deps / "a.rcgu.o"
        scratch.write_bytes(b"keep")
        (deps / "z.rcgu.o").symlink_to(scratch)
        with mock.patch.object(storage_incremental, "active_artifacts", return_value=[]):
            with self.assertRaisesRegex(ValueError, "symlinked codegen"):
                storage_incremental.retire(self.root)
        self.assertEqual(scratch.read_bytes(), b"keep")

    def test_generated_root_symlink_is_rejected_before_retirement(self):
        import shutil
        original = self.root / "obj"
        outside = self.root / "outside"
        shutil.move(original, outside)
        original.symlink_to(outside, target_is_directory=True)
        with mock.patch.object(storage_incremental, "active_artifacts", return_value=[]):
            with self.assertRaisesRegex(ValueError, "symlinked"):
                storage_incremental.retire(self.root)
        self.assertTrue((outside / "rust/dev-fast/incremental/example").is_dir())

    def test_working_session_needs_a_free_known_lock_before_retirement(self):
        import fcntl
        current = self.session("s-current-complete", 1)
        working = self.session("s-interrupted-working", 2)
        (current / "cached.o").write_bytes(b"warm")
        os.link(current / "cached.o", working / "cached.o")
        lock = self.crate / "s-interrupted.lock"
        with lock.open("wb") as descriptor:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            with mock.patch.object(storage_incremental, "active_artifacts", return_value=[]):
                self.assertEqual(storage_incremental.retire(self.root), [])
            self.assertTrue(working.is_dir())
        with mock.patch.object(storage_incremental, "active_artifacts", return_value=[]):
            rows = storage_incremental.retire(self.root)
        self.assertEqual([row["path"] for row in rows], [str(working.relative_to(self.root))])
        self.assertFalse(working.exists())
        self.assertEqual((current / "cached.o").read_bytes(), b"warm")
        self.assertTrue(lock.exists())

    def test_unknown_working_session_lock_and_explicit_pin_are_preserved(self):
        working = self.session("s-pinned-working", 1)
        (working / ".keep-artifacts").touch()
        (self.crate / "s-pinned.lock").touch()
        unknown = self.session("s-unknown-working", 2)
        with mock.patch.object(storage_incremental, "active_artifacts", return_value=[]):
            self.assertEqual(storage_incremental.retire(self.root), [])
        self.assertTrue(working.is_dir())
        self.assertTrue(unknown.is_dir())

    def test_linked_clippy_cache_retires_only_superseded_sessions_too(self):
        ordinary = self.session("s-ordinary-current", 1)
        self.crate = self.root / "obj/rust/private-lint/dev-fast/incremental/example"
        self.crate.mkdir(parents=True)
        old = self.session("s-old-complete", 1)
        current = self.session("s-current-complete", 2)
        with mock.patch.object(storage_incremental, "active_artifacts", return_value=[]):
            rows = storage_incremental.retire(self.root)
        self.assertEqual([row["path"] for row in rows], [str(old.relative_to(self.root))])
        self.assertTrue(ordinary.is_dir())
        self.assertTrue(current.is_dir())

    def test_private_profile_symlink_is_rejected_before_ordinary_retirement(self):
        old = self.session("s-ordinary-old", 1)
        self.session("s-ordinary-current", 2)
        outside = self.root / "outside"
        outside.mkdir()
        (self.root / "obj/rust/private-lint").symlink_to(outside, target_is_directory=True)
        with mock.patch.object(storage_incremental, "active_artifacts", return_value=[]):
            with self.assertRaisesRegex(ValueError, "symlinked cache"):
                storage_incremental.retire(self.root)
        self.assertTrue(old.is_dir())
