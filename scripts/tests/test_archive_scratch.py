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


class ClosedRootArchive(unittest.TestCase):
    @staticmethod
    def run_archive(root, paths, **options):
        with patch("archive_scratch.check_space"), patch("archive_scratch.active_artifacts", return_value=[]):
            archive(root, 0, paths, **options)

    @staticmethod
    def receipts(root):
        import json
        return [json.loads(path.read_text()) for path in (root / "obj/archives").glob("*.json")]

    def test_named_nested_root_chunks_restore_exact_bytes_and_cross_chunk_hardlinks(self):
        import hashlib
        import json
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            folder = root / "obj/parallel/closed/language-inputs"
            folder.mkdir(parents=True)
            content = b"original\x00\xff evidence"
            (folder / "input.rs").write_bytes(content)
            os.link(folder / "input.rs", folder / "hardlink")
            (folder / "external").symlink_to(root / "unselected")
            unselected = root / "obj/probe-9999"
            unselected.mkdir()
            self.run_archive(root, [folder], chunk_members=1, chunk_bytes=4)
            self.assertFalse(folder.exists())
            self.assertTrue(unselected.is_dir())
            receipt, = self.receipts(root)
            self.assertEqual(receipt["state"], "archived")
            parts_path = root / receipt["parts_index"]
            self.assertEqual(hashlib.sha256(parts_path.read_bytes()).hexdigest(),
                             receipt["parts_index_sha256"])
            parts = [json.loads(line) for line in parts_path.read_text().splitlines()]
            self.assertEqual(len(parts), 4)
            restore = root / "restore"
            restore.mkdir()
            for part in parts:
                path = root / part["archive"]
                self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), part["sha256"])
                with tarfile.open(path) as saved:
                    saved.extractall(restore, filter="fully_trusted")
            restored = restore / folder.relative_to(root)
            self.assertEqual((restored / "input.rs").read_bytes(), content)
            self.assertEqual((restored / "hardlink").read_bytes(), content)
            self.assertEqual((restored / "input.rs").stat().st_ino,
                             (restored / "hardlink").stat().st_ino)
            self.assertEqual(os.readlink(restored / "external"), str(root / "unselected"))
            self.assertFalse(list((root / "obj/archives").glob("*.sqlite")))

    def test_explicit_unsafe_cache_archive_ledger_and_symlink_roots_reject_before_writes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ["obj/closed", "obj/rust/dev-fast", "obj/audit-venv", "obj/cleanup",
                         "obj/archives/nested", "obj/parallel/resources/ledger", "outside"]:
                (root / name).mkdir(parents=True, exist_ok=True)
            (root / "obj/link").symlink_to(root / "outside", target_is_directory=True)
            for bad in ["obj", "obj/parallel", "obj/rust", "obj/rust/dev-fast",
                        "obj/audit-venv", "obj/cleanup", "obj/archives", "obj/archives/nested",
                        "obj/parallel/resources", "obj/parallel/resources/ledger", "outside",
                        "obj/closed/../rust", "obj/link"]:
                with self.subTest(path=bad), self.assertRaises(ValueError):
                    self.run_archive(root, ["obj/closed", bad])
                self.assertTrue((root / "obj/closed").is_dir())
                self.assertFalse(list((root / "obj/archives").glob("*.tar.gz")))

    def test_pinned_ancestor_descendant_and_nested_worktree_reject(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            folder = root / "obj/closed/nested"
            folder.mkdir(parents=True)
            for marker in [folder / ".keep-artifacts", folder.parent / ".keep-artifacts",
                           folder / "child/.keep-artifacts", folder / "child/.git"]:
                with self.subTest(marker=marker):
                    marker.parent.mkdir(exist_ok=True)
                    marker.touch()
                    with self.assertRaises(ValueError):
                        self.run_archive(root, [folder])
                    self.assertTrue(folder.exists())
                    marker.unlink()
            self.assertFalse((root / "obj/archives").exists())

    def test_corrupt_archived_bytes_preserve_all_originals_and_failed_receipt(self):
        import gzip
        import scratch_archive
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            folder = root / "obj/closed"
            folder.mkdir(parents=True)
            source = folder / "input.rs"
            source.write_bytes(b"distinct-evidence-payload")
            verify = scratch_archive.verify_chunk

            def corrupt(path, *args):
                raw = gzip.decompress(path.read_bytes())
                self.assertIn(b"distinct-evidence-payload", raw)
                path.write_bytes(gzip.compress(raw.replace(b"distinct-evidence-payload",
                                                           b"changed!-evidence-payload")))
                return verify(path, *args)

            with patch("scratch_archive.verify_chunk", side_effect=corrupt):
                with self.assertRaises(ValueError):
                    self.run_archive(root, [folder])
            self.assertEqual(source.read_bytes(), b"distinct-evidence-payload")
            receipt, = self.receipts(root)
            self.assertEqual(receipt["state"], "failed")
            self.assertTrue(list((root / "obj/archives").glob("*.sqlite")))

    def test_final_source_mutation_new_pin_and_symlink_swap_preserve_tree(self):
        import scratch_archive
        for mutation in ["same_size_bytes", "new_entry", "missing_entry", "pin", "symlink"]:
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                folder = root / "obj/closed"
                folder.mkdir(parents=True)
                source = folder / "input.rs"
                source.write_bytes(b"old")
                link = folder / "link"
                link.symlink_to("old-target")
                verify = scratch_archive.verify_source

                def mutate(*args):
                    if mutation == "same_size_bytes":
                        old = source.stat()
                        source.write_bytes(b"new")
                        os.utime(source, ns=(old.st_atime_ns, old.st_mtime_ns))
                    elif mutation == "new_entry":
                        (folder / "new").touch()
                    elif mutation == "missing_entry":
                        source.unlink()
                    elif mutation == "pin":
                        (folder / ".keep-artifacts").touch()
                    else:
                        link.unlink()
                        link.symlink_to("new-target")
                    return verify(*args)

                with patch("scratch_archive.verify_source", side_effect=mutate):
                    with self.assertRaises(ValueError):
                        self.run_archive(root, [folder])
                self.assertTrue(folder.is_dir())
                receipt, = self.receipts(root)
                self.assertEqual(receipt["state"], "failed")

    def test_tar_caches_and_chunk_indices_stay_bounded(self):
        import json
        real_add, real_next = tarfile.TarFile.addfile, tarfile.TarFile.next
        peak = {"write": 0, "read": 0}

        def add(saved, *args, **kwargs):
            self.assertEqual(len(saved.members), 0)
            self.assertEqual(len(saved.inodes), 0)
            result = real_add(saved, *args, **kwargs)
            peak["write"] = max(peak["write"], len(saved.members))
            return result

        def next_member(saved, *args, **kwargs):
            result = real_next(saved, *args, **kwargs)
            peak["read"] = max(peak["read"], len(saved.members))
            return result

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            folder = root / "obj/closed"
            folder.mkdir(parents=True)
            for number in range(257):
                (folder / str(number)).write_text(str(number))
            with patch.object(tarfile.TarFile, "addfile", add), patch.object(
                    tarfile.TarFile, "next", next_member):
                self.run_archive(root, [folder], chunk_members=17)
            receipt, = self.receipts(root)
            self.assertEqual(receipt["members"], 258)
            parts = [json.loads(line) for line in (root / receipt["parts_index"]).read_text().splitlines()]
            self.assertTrue(all(part["members"] <= 17 for part in parts))
            self.assertEqual(sum(part["members"] for part in parts), 258)
            self.assertEqual(peak, {"write": 1, "read": 1})

    def test_active_process_rejects_even_explicit_closed_roots(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            folder = root / "obj/closed"
            folder.mkdir(parents=True)
            with patch("archive_scratch.check_space"), patch(
                    "archive_scratch.active_artifacts", return_value=[123]):
                with self.assertRaises(ValueError):
                    archive(root, 0, [folder])
            self.assertTrue(folder.is_dir())
            self.assertFalse((root / "obj/archives").exists())
