"""Large diagnostic cleanup retains recoverable bytes and refuses unsafe replacement."""
from contextlib import redirect_stdout
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import patch

from storage import prune
from storage_archives import archive_diagnostic, large_diagnostic


class DiagnosticArchives(unittest.TestCase):
    CONTENT = b"Nested<Shared<Reference<u8>>, Other>\n" * 40000

    def source(self, root, name="run/type.long-type-123.txt"):
        path = root / "obj" / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(self.CONTENT)
        return path

    def test_cleanup_archives_exact_bytes_and_records_original_checksum(self):
        with tempfile.TemporaryDirectory() as folder, redirect_stdout(io.StringIO()):
            root = Path(folder)
            source = self.source(root)
            archive = source.with_name(source.name + ".gz")
            other = source.parent / "ordinary.stderr"
            other.write_bytes(self.CONTENT)
            self.assertEqual(prune(root, paths=["obj/run"]), 1)
            self.assertTrue(source.exists())
            self.assertFalse(archive.exists())
            self.assertEqual(prune(root, apply=True, paths=["obj/run"]), 1)
            self.assertFalse(source.exists())
            self.assertEqual(gzip.decompress(archive.read_bytes()), self.CONTENT)
            self.assertEqual(other.read_bytes(), self.CONTENT)
            self.assertLess(archive.stat().st_size, len(self.CONTENT) // 10)
            records = [json.loads(line) for p in (root / "obj/cleanup").glob("*.jsonl")
                       for line in p.read_text().splitlines()]
            applied = next(row for row in records if row.get("archived"))
            self.assertEqual(applied["sha256"], hashlib.sha256(self.CONTENT).hexdigest())
            self.assertEqual(applied["archive"], str(archive.relative_to(root)))
            self.assertEqual(prune(root, apply=True, paths=["obj/run"]), 0)

    def test_existing_archive_must_match_and_cannot_be_a_symlink(self):
        with tempfile.TemporaryDirectory() as folder, redirect_stdout(io.StringIO()):
            root = Path(folder)
            source = self.source(root)
            archive = source.with_name(source.name + ".gz")
            archive.write_bytes(gzip.compress(b"different diagnostic"))
            with self.assertRaisesRegex(ValueError, "differs"):
                prune(root, apply=True, paths=["obj/run"])
            self.assertEqual(source.read_bytes(), self.CONTENT)
            self.assertEqual(gzip.decompress(archive.read_bytes()), b"different diagnostic")
            archive.write_bytes(gzip.compress(self.CONTENT)[:20])
            with self.assertRaisesRegex(ValueError, "Cannot verify"):
                prune(root, apply=True, paths=["obj/run"])
            self.assertEqual(source.read_bytes(), self.CONTENT)
            archive.unlink()
            external = root / "outside.gz"
            external.write_bytes(gzip.compress(self.CONTENT))
            archive.symlink_to(external)
            with self.assertRaisesRegex(ValueError, "regular file"):
                prune(root, apply=True, paths=["obj/run"])
            self.assertTrue(source.exists())
            self.assertTrue(archive.is_symlink())
            archive.unlink()
            archive.write_bytes(external.read_bytes())
            self.assertEqual(prune(root, apply=True, paths=["obj/run"]), 1)
            self.assertEqual(gzip.decompress(archive.read_bytes()), self.CONTENT)

    def test_verification_failure_and_changed_source_preserve_input(self):
        with tempfile.TemporaryDirectory() as folder, redirect_stdout(io.StringIO()):
            root = Path(folder)
            source = self.source(root)
            with patch("storage_archives.digest", return_value=(0, "wrong")):
                with self.assertRaisesRegex(ValueError, "verification"):
                    prune(root, apply=True, paths=["obj/run"])
            self.assertEqual(source.read_bytes(), self.CONTENT)
            self.assertFalse(list(source.parent.glob("*.gz")))
            self.assertFalse(list(source.parent.glob(".diagnostic-*")))

            def changed(path, info):
                result = archive_diagnostic(path, info)
                path.write_bytes(b"new diagnostic")
                return result

            with patch("storage.archive_diagnostic", side_effect=changed):
                with self.assertRaisesRegex(ValueError, "before removal"):
                    prune(root, apply=True, paths=["obj/run"])
            self.assertEqual(source.read_bytes(), b"new diagnostic")
            archive = source.with_name(source.name + ".gz")
            self.assertEqual(gzip.decompress(archive.read_bytes()), self.CONTENT)

    def test_pins_age_scope_and_file_kind_are_preserved(self):
        with tempfile.TemporaryDirectory() as folder, redirect_stdout(io.StringIO()):
            root = Path(folder)
            source = self.source(root)
            pinned = self.source(root, "pinned/type.long-type-456.txt")
            (pinned.parent / ".keep-artifacts").touch()
            other = self.source(root, "other/type.long-type-789.txt")
            link = source.parent / "link.long-type-123.txt"
            link.symlink_to(other)
            self.assertFalse(large_diagnostic(link, link.lstat()))
            small = source.parent / "small.long-type-123.txt"
            small.write_text("u8")
            self.assertFalse(large_diagnostic(small, small.stat()))
            self.assertEqual(prune(root, apply=True, hours=1, paths=["obj/run"]), 0)
            os.utime(source, (time.time() - 7200,) * 2)
            self.assertEqual(prune(root, apply=True, hours=1,
                                   paths=["obj/run", "obj/pinned"]), 1)
            for path in (pinned, other, link, small):
                self.assertTrue(path.exists())
